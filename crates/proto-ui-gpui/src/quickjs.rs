use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use rquickjs::{CatchResultExt, Context, Function, Object, Runtime};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::protocol::{
    BridgeCommand, BridgeError, BridgeEvent, PROTOCOL_MAJOR, PROTOCOL_MINOR, PrototypeKey, Result,
};
const MAX_BRIDGE_MESSAGE_BYTES: usize = 256 * 1024;
const MAX_SHARED_PENDING_PER_SESSION: usize = 256;
const MAX_SHARED_PENDING_TOTAL: usize = 1024;
const MAX_JSON_DEPTH: usize = 16;

const BRIDGE_BUNDLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/proto-ui-bridge.js"
));
const BUNDLE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/proto-ui-bridge/upstream.json"
));

fn bundle_digest() -> String {
    let digest = Sha256::digest(BRIDGE_BUNDLE.as_bytes());
    format!("sha256:{digest:x}")
}

fn recorded_bundle_digest() -> Result<String> {
    let manifest: Value =
        serde_json::from_str(BUNDLE_MANIFEST).map_err(|error| BridgeError::Decode {
            detail: format!("invalid embedded Proto UI manifest: {error}"),
        })?;
    let digest = manifest
        .get("bundle_sha256")
        .and_then(Value::as_str)
        .filter(|value| value.starts_with("sha256:") && value.len() == 71)
        .ok_or_else(|| BridgeError::Decode {
            detail: "embedded Proto UI manifest has no valid bundle digest".to_owned(),
        })?;
    Ok(digest.to_owned())
}

fn host_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows-gpui"
    } else if cfg!(target_os = "linux") {
        "linux-gpui"
    } else {
        "unsupported-gpui"
    }
}

/// Embedded Proto UI Runtime host.
///
/// QuickJS stays on the owning thread. GPUI receives only decoded protocol
/// values; JavaScript objects and callbacks never cross this boundary.
pub struct QuickJsBridge {
    _runtime: Runtime,
    context: Context,
    bundle_digest: String,
}

impl QuickJsBridge {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new().map_err(runtime_error)?;
        let context = Context::full(&runtime).map_err(runtime_error)?;
        context
            .with(|ctx| ctx.eval::<(), _>(BRIDGE_BUNDLE))
            .map_err(runtime_error)?;
        let digest = bundle_digest();
        let recorded_digest = recorded_bundle_digest()?;
        if digest != recorded_digest {
            return Err(BridgeError::Runtime {
                detail: format!(
                    "embedded Proto UI bundle digest {digest} does not match recorded {recorded_digest}"
                ),
            });
        }
        Ok(Self {
            _runtime: runtime,
            context,
            bundle_digest: digest,
        })
    }

    pub fn dispatch(&mut self, command: &BridgeCommand) -> Result<Vec<BridgeEvent>> {
        let serialized =
            serde_json::to_string(command).map_err(|error| BridgeError::Serialization {
                detail: error.to_string(),
            })?;
        self.dispatch_json(&serialized)
    }

    pub fn dispatch_json(&mut self, serialized: &str) -> Result<Vec<BridgeEvent>> {
        if serialized.len() > MAX_BRIDGE_MESSAGE_BYTES {
            return Err(BridgeError::Decode {
                detail: format!("bridge message exceeds {} bytes", MAX_BRIDGE_MESSAGE_BYTES),
            });
        }
        let value: Value =
            serde_json::from_str(serialized).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;
        validate_json(&value, 0)?;
        validate_command(&value)?;
        let _command: BridgeCommand =
            serde_json::from_value(value).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;

        let output = self.context.with(|ctx| {
            let globals = ctx.globals();
            let bridge: Object = globals
                .get("__sailbreak_proto_ui_bridge_v1")
                .map_err(runtime_error)?;
            let dispatch: Function = bridge.get("dispatch").map_err(runtime_error)?;
            match dispatch.call::<_, String>((serialized,)).catch(&ctx) {
                Ok(output) => Ok(output),
                Err(error) => Err(BridgeError::Runtime {
                    detail: error.to_string(),
                }),
            }
        })?;
        if output.len() > MAX_BRIDGE_MESSAGE_BYTES {
            return Err(BridgeError::Decode {
                detail: format!("bridge response exceeds {} bytes", MAX_BRIDGE_MESSAGE_BYTES),
            });
        }
        let output_value: Value =
            serde_json::from_str(&output).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;
        validate_json(&output_value, 0)?;
        let mut events: Vec<BridgeEvent> =
            serde_json::from_value(output_value).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;
        for event in &mut events {
            if let BridgeEvent::Ready { handshake } = event {
                if handshake.protocol.major != PROTOCOL_MAJOR
                    || handshake.protocol.minor != PROTOCOL_MINOR
                {
                    return Err(BridgeError::Runtime {
                        detail: format!(
                            "unsupported Proto UI bridge protocol {}.{}, expected {}.{}",
                            handshake.protocol.major,
                            handshake.protocol.minor,
                            PROTOCOL_MAJOR,
                            PROTOCOL_MINOR
                        ),
                    });
                }
                handshake.registry_digest = self.bundle_digest.clone();
                handshake.host.platform = host_platform().to_owned();
            }
        }
        if let Some(diagnostic) = events.iter().find_map(|event| match event {
            BridgeEvent::Diagnostic { diagnostic } if diagnostic.fatal => Some(diagnostic),
            _ => None,
        }) {
            return Err(BridgeError::Runtime {
                detail: diagnostic.detail.clone(),
            });
        }
        Ok(events)
    }
}
/// A single embedded QuickJS context that can be shared by multiple semantic
/// sessions. The bridge demultiplexes session-tagged events so independently
/// owned Rust session hosts never observe another session's projection.
#[derive(Clone)]
pub(crate) struct SharedQuickJsBridge {
    state: Rc<RefCell<SharedQuickJsBridgeState>>,
}

struct SharedQuickJsBridgeState {
    bridge: QuickJsBridge,
    pending: BTreeMap<String, Vec<BridgeEvent>>,
    failed: Option<String>,
}

impl SharedQuickJsBridge {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            state: Rc::new(RefCell::new(SharedQuickJsBridgeState {
                bridge: QuickJsBridge::new()?,
                pending: BTreeMap::new(),
                failed: None,
            })),
        })
    }

    pub(crate) fn dispatch(
        &self,
        command: &BridgeCommand,
        target_session: Option<&str>,
    ) -> Result<Vec<BridgeEvent>> {
        let mut state = self.state.borrow_mut();
        if let Some(detail) = &state.failed {
            return Err(BridgeError::Runtime {
                detail: detail.clone(),
            });
        }
        let fresh = state.bridge.dispatch(command)?;
        let mut events = target_session
            .and_then(|session| state.pending.remove(session))
            .unwrap_or_default();
        for event in fresh {
            if let Some(session) = event_session_id(&event) {
                if target_session == Some(session) {
                    events.push(event);
                } else {
                    let per_session = state.pending.get(session).map_or(0, Vec::len);
                    let total = state.pending.values().map(Vec::len).sum::<usize>();
                    if per_session >= MAX_SHARED_PENDING_PER_SESSION
                        || total >= MAX_SHARED_PENDING_TOTAL
                    {
                        let detail =
                            format!("shared bridge pending event overflow for session {session}");
                        state.failed = Some(detail.clone());
                        return Err(BridgeError::Runtime { detail });
                    }
                    state
                        .pending
                        .entry(session.to_owned())
                        .or_default()
                        .push(event);
                }
            } else {
                events.push(event);
            }
        }
        Ok(events)
    }

    pub(crate) fn drain(&self, target_session: &str) -> Result<Vec<BridgeEvent>> {
        let mut state = self.state.borrow_mut();
        if let Some(detail) = &state.failed {
            return Err(BridgeError::Runtime {
                detail: detail.clone(),
            });
        }
        Ok(state.pending.remove(target_session).unwrap_or_default())
    }
}

fn event_session_id(event: &BridgeEvent) -> Option<&str> {
    match event {
        BridgeEvent::Projection { projection } => Some(projection.session_id.as_str()),
        BridgeEvent::Style { session_id, .. }
        | BridgeEvent::A11y { session_id, .. }
        | BridgeEvent::State { session_id, .. }
        | BridgeEvent::Signal { session_id, .. } => Some(session_id.as_str()),
        BridgeEvent::Registry { .. }
        | BridgeEvent::Ready { .. }
        | BridgeEvent::Diagnostic { .. } => None,
    }
}

fn validate_json(value: &Value, depth: usize) -> Result<()> {
    if depth > MAX_JSON_DEPTH {
        return Err(BridgeError::Decode {
            detail: format!("JSON nesting exceeds {MAX_JSON_DEPTH} levels"),
        });
    }
    match value {
        Value::Array(values) => {
            for value in values {
                validate_json(value, depth + 1)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_json(value, depth + 1)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}

fn validate_command(value: &Value) -> Result<()> {
    let object = value.as_object().ok_or_else(|| BridgeError::Decode {
        detail: "bridge command must be a JSON object".to_owned(),
    })?;
    if let Some(prototype) = object.get("prototype").and_then(Value::as_str) {
        PrototypeKey::parse(prototype)?;
    }
    Ok(())
}

fn runtime_error(error: rquickjs::Error) -> BridgeError {
    BridgeError::Runtime {
        detail: error.to_string(),
    }
}
