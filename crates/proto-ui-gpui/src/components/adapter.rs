use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::quickjs::SharedQuickJsBridge;
use crate::{
    BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle, CommitDisposition, InputEnvelope,
    InputKind, InputPayload, InputRequest, InputSource, LogicalParentRef, NativeStyle,
    ProjectionAck, PropsRequest, ProtoSessionHost, PrototypeKey, Result, SessionSnapshot,
    ShadcnTheme, StartRequest, ViewEpoch, translate_projection,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrototypeProfile {
    pub prototype: PrototypeKey,
    pub exposed_states: &'static [&'static str],
    pub signals: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdapterSnapshot {
    pub id: String,
    pub label: String,
    pub profile: PrototypeProfile,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
}

impl AdapterSnapshot {
    #[must_use]
    pub fn state_bool(&self, key: &str) -> Option<bool> {
        self.session.state_values.get(key).and_then(Value::as_bool)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdapterDispatchOutcome {
    pub signal_counts: BTreeMap<String, usize>,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl AdapterDispatchOutcome {
    #[must_use]
    pub fn signal_count(&self, key: &str) -> usize {
        self.signal_counts.get(key).copied().unwrap_or(0)
    }
}

struct ManagedSession {
    id: String,
    label: String,
    profile: PrototypeProfile,
    host: ProtoSessionHost,
    next_sequence: u64,
}

pub struct ProtoAdapter {
    bridge: SharedQuickJsBridge,
    sessions: BTreeMap<String, ManagedSession>,
    theme: ShadcnTheme,
}

impl ProtoAdapter {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            bridge: SharedQuickJsBridge::new()?,
            sessions: BTreeMap::new(),
            theme,
        })
    }

    pub fn start(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        profile: PrototypeProfile,
        mut request: StartRequest,
    ) -> Result<()> {
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "adapter session")?;
        if self.sessions.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate adapter session: {id}"),
            });
        }
        if request.prototype != profile.prototype {
            return Err(BridgeError::InvalidIdentity {
                kind: format!(
                    "profile prototype {} does not match request {}",
                    profile.prototype, request.prototype
                ),
            });
        }
        request.meta.insert(
            "colorScheme".to_owned(),
            Value::String(self.theme.color_scheme_name().to_owned()),
        );
        if request.route_ref.is_none() {
            request.route_ref = Some(id.clone());
        }

        let mut host = ProtoSessionHost::with_shared_bridge(self.bridge.clone())?;
        host.start(request)?;
        acknowledge_pending(&mut host)?;
        self.sessions.insert(
            id.clone(),
            ManagedSession {
                id,
                label,
                profile,
                host,
                next_sequence: 0,
            },
        );
        Ok(())
    }

    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<AdapterDispatchOutcome> {
        let session = self.session_mut(id)?;
        session.next_sequence =
            session
                .next_sequence
                .checked_add(1)
                .ok_or_else(|| BridgeError::Runtime {
                    detail: format!("adapter input sequence overflow: {id}"),
                })?;
        let snapshot = session.host.snapshot()?;
        let input = InputEnvelope::new(
            snapshot.session_id,
            snapshot.instance_id,
            snapshot.projection.view_epoch,
            session.next_sequence,
            InputPayload::new(
                format!("{id}:sample:{}", session.next_sequence),
                id,
                source,
                kind,
            ),
        );
        let outcome = session.host.input(InputRequest::new(input, detail))?;
        let mut signal_counts = BTreeMap::new();
        for event in &outcome.events {
            if let BridgeEvent::Signal { key, .. } = event {
                *signal_counts.entry(key.clone()).or_insert(0) += 1;
            }
        }
        Ok(AdapterDispatchOutcome {
            signal_counts,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        })
    }

    pub fn set_props(&mut self, id: &str, props: Map<String, Value>) -> Result<CommitDisposition> {
        let session = self.session_mut(id)?;
        let snapshot = session.host.snapshot()?;
        let disposition = session.host.set_props(PropsRequest::new(
            snapshot.session_id,
            snapshot.instance_id,
            props,
        ))?;
        acknowledge_pending(&mut session.host)?;
        Ok(disposition)
    }

    pub fn snapshot(&mut self, id: &str) -> Result<AdapterSnapshot> {
        let theme = self.theme;
        let session = self.session_mut(id)?;
        session.host.drain_events()?;
        acknowledge_pending(&mut session.host)?;
        build_snapshot(session, theme)
    }

    /// Read the last applied Rust snapshot without draining sibling-session events.
    pub fn snapshot_current(&self, id: &str) -> Result<AdapterSnapshot> {
        build_snapshot(self.session(id)?, self.theme)
    }

    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        let snapshot = self.snapshot(id)?;
        LogicalParentRef::new(
            snapshot.session.session_id,
            snapshot.session.instance_id,
            snapshot.session.projection.view_epoch,
            id,
        )
    }

    pub fn remount(&mut self, id: &str) -> Result<ViewEpoch> {
        self.session_mut(id)?.host.remount()
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        let mut session = self
            .sessions
            .remove(id)
            .ok_or_else(|| unknown_session(id))?;
        session.host.dispose()
    }

    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.sessions.contains_key(id)
    }

    fn session(&self, id: &str) -> Result<&ManagedSession> {
        self.sessions.get(id).ok_or_else(|| unknown_session(id))
    }

    fn session_mut(&mut self, id: &str) -> Result<&mut ManagedSession> {
        self.sessions.get_mut(id).ok_or_else(|| unknown_session(id))
    }
}

fn build_snapshot(session: &ManagedSession, theme: ShadcnTheme) -> Result<AdapterSnapshot> {
    let snapshot = session.host.snapshot()?;
    Ok(AdapterSnapshot {
        id: session.id.clone(),
        label: session.label.clone(),
        profile: session.profile,
        native_style: translate_projection(&snapshot.style, theme),
        resolved_style: ButtonStyle::from_projection(&snapshot.style, theme),
        session: snapshot,
    })
}

fn acknowledge_pending(host: &mut ProtoSessionHost) -> Result<()> {
    let snapshot = host.snapshot()?;
    if snapshot.pending_commit {
        host.acknowledge(ProjectionAck::applied(
            snapshot.session_id,
            snapshot.instance_id,
            snapshot.projection.view_epoch,
            snapshot.projection.commit_id,
        ))?;
    }
    Ok(())
}

fn validate_identity(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: kind.to_owned(),
        });
    }
    Ok(())
}

fn unknown_session(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown adapter session: {id}"),
    }
}
