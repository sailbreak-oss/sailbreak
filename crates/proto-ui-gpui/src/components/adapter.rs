use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::quickjs::SharedQuickJsBridge;
use crate::{
    BridgeDiagnostic, BridgeError, BridgeEvent, CommitDisposition, InputEnvelope, InputKind,
    InputPayload, InputRequest, InputSource, LogicalParentRef, NativeStyle, ProjectionAck,
    PropsRequest, ProtoSessionHost, PrototypeKey, Result, SessionSnapshot, ShadcnTheme,
    StartRequest, TextControlCommand, TextControlEvent, TextControlEventEnvelope, TextControlRef,
    TextControlSelection, ViewEpoch, translate_projection,
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
    pub text_events: Vec<TextControlEventEnvelope>,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl AdapterDispatchOutcome {
    #[must_use]
    pub fn signal_count(&self, key: &str) -> usize {
        self.signal_counts.get(key).copied().unwrap_or(0)
    }
    #[must_use]
    pub fn text_event_count(&self, event_type: crate::TextControlEventType) -> usize {
        self.text_events
            .iter()
            .filter(|event| event.event.event_type == event_type)
            .count()
    }

    pub fn absorb(&mut self, mut other: Self) {
        for (key, count) in other.signal_counts {
            *self.signal_counts.entry(key).or_insert(0) += count;
        }
        self.text_events.append(&mut other.text_events);
        self.events.append(&mut other.events);
        self.diagnostics.append(&mut other.diagnostics);
    }
}

struct ManagedSession {
    id: String,
    label: String,
    profile: PrototypeProfile,
    host: ProtoSessionHost,
    next_sequence: u64,
    next_text_sequence: u64,
    route_ref: String,
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
        let route_ref = request.route_ref.clone().unwrap_or_else(|| id.clone());

        let mut host = ProtoSessionHost::with_shared_bridge(self.bridge.clone())?;
        host.start(request)?;
        if let Err(error) = acknowledge_pending(&mut host) {
            let _ = host.dispose();
            return Err(error);
        }
        self.sessions.insert(
            id.clone(),
            ManagedSession {
                id,
                label,
                profile,
                host,
                next_sequence: 0,
                next_text_sequence: 0,
                route_ref,
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
        prepare_session(&mut session.host)?;
        session.next_sequence =
            session
                .next_sequence
                .checked_add(1)
                .ok_or_else(|| BridgeError::Runtime {
                    detail: format!("adapter input sequence overflow: {id}"),
                })?;
        let snapshot = session.host.snapshot()?;
        let route_ref = session.route_ref.clone();
        let input = InputEnvelope::new(
            snapshot.session_id,
            snapshot.instance_id,
            snapshot.projection.view_epoch,
            session.next_sequence,
            InputPayload::new(
                format!("{id}:sample:{}", session.next_sequence),
                route_ref,
                source,
                kind,
            ),
        );
        let outcome = session.host.input(InputRequest::new(input, detail))?;
        Ok(adapter_dispatch_outcome(outcome))
    }
    /// Advance the Runtime's virtual scheduler for one managed session.
    ///
    /// The adapter deliberately exposes no native timer; callers choose the
    /// session whose Proto scheduler should advance and reconcile any emitted
    /// bridge events through the normal session host.
    pub fn advance_time(&mut self, id: &str, milliseconds: u64) -> Result<AdapterDispatchOutcome> {
        let session = self.session_mut(id)?;
        prepare_session(&mut session.host)?;
        let outcome = session.host.advance_time(milliseconds)?;
        Ok(adapter_dispatch_outcome(outcome))
    }

    pub fn drain(&mut self, id: &str) -> Result<AdapterDispatchOutcome> {
        let session = self.session_mut(id)?;
        let outcome = session.host.drain_events()?;
        acknowledge_pending(&mut session.host)?;
        Ok(adapter_dispatch_outcome(outcome))
    }
    pub fn dispatch_text(
        &mut self,
        id: &str,
        event: TextControlEvent,
        selection: Option<TextControlSelection>,
    ) -> Result<AdapterDispatchOutcome> {
        let epoch = {
            let session = self.session_mut(id)?;
            prepare_session(&mut session.host)?;
            session.host.snapshot()?.projection.view_epoch
        };
        self.dispatch_text_at_epoch(id, epoch, event, selection)
    }
    pub fn dispatch_text_event(
        &mut self,
        id: &str,
        event: TextControlEvent,
    ) -> Result<AdapterDispatchOutcome> {
        self.dispatch_text(id, event, None)
    }

    pub fn dispatch_text_at_epoch(
        &mut self,
        id: &str,
        epoch: ViewEpoch,
        event: TextControlEvent,
        selection: Option<TextControlSelection>,
    ) -> Result<AdapterDispatchOutcome> {
        let session = self.session_mut(id)?;
        prepare_session(&mut session.host)?;
        session.next_text_sequence =
            session
                .next_text_sequence
                .checked_add(1)
                .ok_or_else(|| BridgeError::Runtime {
                    detail: format!("adapter text input sequence overflow: {id}"),
                })?;
        let snapshot = session.host.snapshot()?;
        let control_ref = TextControlRef::new(format!("{}:text-control", snapshot.session_id))?;
        let command = TextControlCommand::Event {
            session_id: snapshot.session_id,
            instance_id: snapshot.instance_id,
            view_epoch: epoch,
            sequence: session.next_text_sequence,
            control_ref,
            event,
            selection,
        };
        let outcome = session.host.text_control(command)?;
        let mut signal_counts = BTreeMap::new();
        for event in &outcome.events {
            if let BridgeEvent::Signal { key, .. } = event {
                *signal_counts.entry(key.clone()).or_insert(0) += 1;
            }
        }
        Ok(AdapterDispatchOutcome {
            signal_counts,
            text_events: outcome.text_events,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        })
    }

    pub fn set_props(&mut self, id: &str, props: Map<String, Value>) -> Result<CommitDisposition> {
        let session = self.session_mut(id)?;
        prepare_session(&mut session.host)?;
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
        let route_ref = self.session(id)?.route_ref.clone();
        let snapshot = self.snapshot(id)?;
        LogicalParentRef::new(
            snapshot.session.session_id,
            snapshot.session.instance_id,
            snapshot.session.projection.view_epoch,
            route_ref,
        )
    }

    pub fn remount(&mut self, id: &str) -> Result<ViewEpoch> {
        let session = self.session_mut(id)?;
        prepare_session(&mut session.host)?;
        session.host.remount()
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

    #[must_use]
    pub fn theme(&self) -> ShadcnTheme {
        self.theme
    }

    fn session(&self, id: &str) -> Result<&ManagedSession> {
        self.sessions.get(id).ok_or_else(|| unknown_session(id))
    }

    fn session_mut(&mut self, id: &str) -> Result<&mut ManagedSession> {
        self.sessions.get_mut(id).ok_or_else(|| unknown_session(id))
    }
}

fn adapter_dispatch_outcome(outcome: crate::DispatchOutcome) -> AdapterDispatchOutcome {
    let mut signal_counts = BTreeMap::new();
    for event in &outcome.events {
        if let BridgeEvent::Signal { key, .. } = event {
            *signal_counts.entry(key.clone()).or_insert(0) += 1;
        }
    }
    AdapterDispatchOutcome {
        signal_counts,
        text_events: outcome.text_events,
        events: outcome.events,
        diagnostics: outcome.diagnostics,
    }
}

fn build_snapshot(session: &ManagedSession, theme: ShadcnTheme) -> Result<AdapterSnapshot> {
    let snapshot = session.host.snapshot()?;
    Ok(AdapterSnapshot {
        id: session.id.clone(),
        label: session.label.clone(),
        profile: session.profile,
        native_style: translate_projection(&snapshot.style, theme),
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

fn prepare_session(host: &mut ProtoSessionHost) -> Result<()> {
    host.drain_events()?;
    acknowledge_pending(host)
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
