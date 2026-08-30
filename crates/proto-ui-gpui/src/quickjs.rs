use rquickjs::{CatchResultExt, Context, Function, Object, Runtime};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::protocol::{BridgeCommand, BridgeError, BridgeEvent, PrototypeKey, Result};

const BRIDGE_BUNDLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/proto-ui-bridge.js"
));

fn bundle_digest() -> String {
    let digest = Sha256::digest(BRIDGE_BUNDLE.as_bytes());
    format!("sha256:{digest:x}")
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
        Ok(Self {
            _runtime: runtime,
            context,
            bundle_digest: bundle_digest(),
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
        let value: Value =
            serde_json::from_str(serialized).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;
        validate_command(&value)?;
        let command: BridgeCommand =
            serde_json::from_value(value).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;
        let serialized =
            serde_json::to_string(&command).map_err(|error| BridgeError::Serialization {
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

        let mut events: Vec<BridgeEvent> =
            serde_json::from_str(&output).map_err(|error| BridgeError::Decode {
                detail: error.to_string(),
            })?;
        for event in &mut events {
            if let BridgeEvent::Ready { handshake } = event {
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
