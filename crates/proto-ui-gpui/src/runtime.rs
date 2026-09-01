use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::protocol::{
    A11ySnapshot, AckDisposition, BridgeCommand, BridgeDiagnostic, BridgeError, BridgeEvent,
    BridgeState, DispatchOutcome, InstanceId, ProjectionAck, ProjectionTransaction, PrototypeKey,
    Result, SessionId, SlotProjection, StyleProjection, ViewEpoch,
};
use crate::quickjs::QuickJsBridge;
#[derive(Clone, Debug)]
pub struct StartRequest {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub prototype: PrototypeKey,
    pub props: Map<String, Value>,
    pub slot: SlotProjection,
    pub accessible_content: Option<String>,
}

impl StartRequest {
    #[must_use]
    pub fn new(
        session_id: SessionId,
        instance_id: InstanceId,
        prototype: PrototypeKey,
        props: Map<String, Value>,
        slot: SlotProjection,
    ) -> Self {
        Self {
            session_id,
            instance_id,
            prototype,
            props,
            slot,
            accessible_content: None,
        }
    }

    #[must_use]
    pub fn button(
        session_id: SessionId,
        instance_id: InstanceId,
        label: impl Into<String>,
    ) -> Self {
        let mut props = Map::new();
        props.insert("variant".to_owned(), Value::String("default".to_owned()));
        props.insert("size".to_owned(), Value::String("default".to_owned()));
        props.insert("disabled".to_owned(), Value::Bool(false));
        Self::new(
            session_id,
            instance_id,
            PrototypeKey::ShadcnButton,
            props,
            SlotProjection::new("button-slot", label),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputRequest {
    pub input: crate::protocol::InputEnvelope,
    pub detail: Option<Value>,
}

impl InputRequest {
    #[must_use]
    pub fn new(input: crate::protocol::InputEnvelope, detail: Option<Value>) -> Self {
        Self { input, detail }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropsRequest {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub props: Map<String, Value>,
}

impl PropsRequest {
    #[must_use]
    pub fn new(session_id: SessionId, instance_id: InstanceId, props: Map<String, Value>) -> Self {
        Self {
            session_id,
            instance_id,
            props,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitDisposition {
    Applied,
    Superseded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionSnapshot {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub prototype: PrototypeKey,
    pub projection: ProjectionTransaction,
    pub style: StyleProjection,
    pub a11y: Option<A11ySnapshot>,
    pub state_values: BTreeMap<String, Value>,
    pub diagnostics: Vec<BridgeDiagnostic>,
    pub pending_commit: bool,
}

struct SessionRecord {
    request: StartRequest,
    protocol: BridgeState,
    projection: ProjectionTransaction,
    style: StyleProjection,
    a11y: Option<A11ySnapshot>,
    state_values: BTreeMap<String, Value>,
    diagnostics: Vec<BridgeDiagnostic>,
    pending_commit: Option<u64>,
    route_ref: Option<String>,
    last_output_sequence: u64,
    mounted: bool,
}

/// Host-owned lifecycle and identity barrier around one embedded Proto UI session.
///
/// The QuickJS bridge only evaluates the pinned bundle. This type owns the durable
/// session identity, projection ACK barrier, native input ordering, and terminal
/// disposal state that are independent of any renderer.
pub struct ProtoSessionHost {
    bridge: QuickJsBridge,
    session: Option<SessionRecord>,
    disposed: bool,
}

impl ProtoSessionHost {
    pub fn new() -> Result<Self> {
        Ok(Self {
            bridge: QuickJsBridge::new()?,
            session: None,
            disposed: false,
        })
    }

    pub fn start(&mut self, mut request: StartRequest) -> Result<SessionSnapshot> {
        self.ensure_alive()?;
        if self.session.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("active session: {}", request.session_id),
            });
        }
        request.slot = effective_slot(&request)?;
        let command = BridgeCommand::Start {
            session_id: request.session_id.clone(),
            instance_id: request.instance_id.clone(),
            prototype: request.prototype,
            props: request.props.clone(),
            slot: request.slot.clone(),
        };
        let events = self.dispatch(&command)?;
        let projection = events
            .iter()
            .find_map(|event| match event {
                BridgeEvent::Projection { projection } => Some(projection.clone()),
                _ => None,
            })
            .ok_or_else(|| BridgeError::Runtime {
                detail: format!("Proto UI did not project {}", request.prototype),
            })?;
        if projection.session_id != request.session_id {
            return Err(BridgeError::SessionMismatch {
                expected: request.session_id.clone(),
                received: projection.session_id,
            });
        }
        if projection.instance_id != request.instance_id {
            return Err(BridgeError::InstanceMismatch {
                expected: request.instance_id.clone(),
                received: projection.instance_id,
            });
        }
        let mut protocol =
            BridgeState::new(request.session_id.clone(), request.instance_id.clone());
        protocol.install_view(projection.view_epoch)?;
        let mut record = SessionRecord {
            request,
            protocol,
            style: projection.style.clone(),
            a11y: projection.a11y.clone(),
            state_values: BTreeMap::new(),
            diagnostics: Vec::new(),
            pending_commit: Some(projection.commit_id),
            route_ref: None,
            last_output_sequence: 0,
            mounted: true,
            projection,
        };
        absorb_events(&mut record, &events)?;
        self.session = Some(record);
        self.snapshot()
    }

    pub fn acknowledge(&mut self, ack: ProjectionAck) -> Result<CommitDisposition> {
        self.ensure_alive()?;
        let (pending_commit, mut next_protocol) = {
            let record = self.record()?;
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                &ack.session_id,
                &ack.instance_id,
            )?;
            let expected_epoch = record
                .protocol
                .current_epoch()
                .ok_or(BridgeError::Unmounted)?;
            if ack.view_epoch != expected_epoch {
                return Err(BridgeError::StaleEpoch {
                    expected: expected_epoch,
                    received: ack.view_epoch,
                });
            }
            let pending_commit = record
                .pending_commit
                .ok_or(BridgeError::NoPendingProjection)?;
            if ack.commit_id != pending_commit {
                return Err(BridgeError::StaleCommit {
                    last: pending_commit,
                    received: ack.commit_id,
                });
            }
            (pending_commit, record.protocol.clone())
        };
        let disposition = next_protocol.accept_ack(ack.clone())?;
        let events = self.dispatch(&BridgeCommand::ProjectionAck { ack })?;
        let record = self.record_mut()?;
        record.protocol = next_protocol;
        absorb_events(record, &events)?;
        if matches!(disposition, AckDisposition::Applied)
            && record.pending_commit == Some(pending_commit)
        {
            record.pending_commit = None;
        }
        Ok(match disposition {
            AckDisposition::Applied => CommitDisposition::Applied,
            AckDisposition::Superseded => CommitDisposition::Superseded,
        })
    }

    pub fn input(&mut self, request: InputRequest) -> Result<DispatchOutcome> {
        self.ensure_alive()?;
        let next_protocol = {
            let record = self.record()?;
            if !record.mounted {
                return Err(BridgeError::Unmounted);
            }
            if let Some(commit_id) = record.pending_commit {
                return Err(BridgeError::ProjectionPending { commit_id });
            }
            check_input_route(record, &request.input.route_ref)?;
            let mut protocol = record.protocol.clone();
            protocol.accept_input(request.input.clone())?;
            protocol
        };
        let events = self.dispatch(&BridgeCommand::Input {
            input: request.input.clone(),
            detail: request.detail.clone(),
        })?;
        let record = self.record_mut()?;
        record.protocol = next_protocol;
        if record.route_ref.is_none() {
            record.route_ref = Some(request.input.route_ref.clone());
        }
        absorb_events(record, &events)
    }

    pub fn set_props(&mut self, request: PropsRequest) -> Result<CommitDisposition> {
        self.ensure_alive()?;
        let previous_pending = {
            let record = self.record()?;
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                &request.session_id,
                &request.instance_id,
            )?;
            record.pending_commit
        };
        let events = self.dispatch(&BridgeCommand::SetProps {
            session_id: request.session_id.clone(),
            instance_id: request.instance_id.clone(),
            props: request.props.clone(),
        })?;
        let record = self.record_mut()?;
        record.request.props = request.props;
        let outcome = absorb_events(record, &events)?;
        let has_new_projection = outcome
            .events
            .iter()
            .any(|event| matches!(event, BridgeEvent::Projection { .. }));
        Ok(if has_new_projection && previous_pending.is_some() {
            CommitDisposition::Superseded
        } else {
            CommitDisposition::Applied
        })
    }

    /// Advance the host-controlled virtual clock used by delayed Runtime work.
    pub fn advance_time(&mut self, milliseconds: u64) -> Result<DispatchOutcome> {
        self.ensure_alive()?;
        if milliseconds == 0 {
            return Ok(DispatchOutcome::default());
        }
        let (session_id, instance_id) = {
            let record = self.record()?;
            (
                record.request.session_id.clone(),
                record.request.instance_id.clone(),
            )
        };
        let events = self.dispatch(&BridgeCommand::AdvanceTime {
            session_id,
            instance_id,
            milliseconds,
        })?;
        self.record_mut()
            .and_then(|record| absorb_events(record, &events))
    }

    pub fn remount(&mut self) -> Result<ViewEpoch> {
        self.ensure_alive()?;
        let (session_id, instance_id, previous_epoch) = {
            let record = self.record()?;
            (
                record.request.session_id.clone(),
                record.request.instance_id.clone(),
                record.projection.view_epoch,
            )
        };
        let events = self.dispatch(&BridgeCommand::Remount {
            session_id,
            instance_id,
        })?;
        let record = self.record_mut()?;
        record.pending_commit = None;
        record.route_ref = None;
        absorb_events(record, &events)?;
        let epoch = record.projection.view_epoch;
        if epoch <= previous_epoch {
            return Err(BridgeError::Runtime {
                detail: format!("remount did not advance view epoch beyond {previous_epoch}"),
            });
        }
        record.mounted = true;
        Ok(epoch)
    }

    pub fn unmount(&mut self) -> Result<()> {
        self.ensure_alive()?;
        let (session_id, instance_id, view_epoch) = {
            let record = self.record()?;
            (
                record.request.session_id.clone(),
                record.request.instance_id.clone(),
                record.projection.view_epoch,
            )
        };
        if self.record()?.mounted {
            self.dispatch(&BridgeCommand::Unmount {
                session_id,
                instance_id,
                view_epoch,
            })?;
        }
        let record = self.record_mut()?;
        record.mounted = false;
        record.pending_commit = None;
        record.route_ref = None;
        Ok(())
    }

    pub fn dispose(&mut self) -> Result<()> {
        if self.disposed {
            return Ok(());
        }
        let command = self.session.as_ref().map(|record| BridgeCommand::Dispose {
            session_id: record.request.session_id.clone(),
            instance_id: record.request.instance_id.clone(),
        });
        let result = match command {
            Some(command) => self.dispatch(&command).map(|_| ()),
            None => Ok(()),
        };
        self.disposed = true;
        self.session = None;
        result
    }

    pub fn snapshot(&self) -> Result<SessionSnapshot> {
        self.ensure_alive()?;
        let record = self.record()?;
        Ok(SessionSnapshot {
            session_id: record.request.session_id.clone(),
            instance_id: record.request.instance_id.clone(),
            prototype: record.request.prototype,
            projection: record.projection.clone(),
            style: record.style.clone(),
            a11y: record.a11y.clone(),
            state_values: record.state_values.clone(),
            diagnostics: record.diagnostics.clone(),
            pending_commit: record.pending_commit.is_some(),
        })
    }

    fn ensure_alive(&self) -> Result<()> {
        if self.disposed {
            Err(BridgeError::Disposed)
        } else {
            Ok(())
        }
    }

    fn dispatch(&mut self, command: &BridgeCommand) -> Result<Vec<BridgeEvent>> {
        match self.bridge.dispatch(command) {
            Ok(events) => Ok(events),
            Err(error @ BridgeError::Runtime { .. }) => {
                self.disposed = true;
                self.session = None;
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    fn record(&self) -> Result<&SessionRecord> {
        self.session
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "session host has not started a session".to_owned(),
            })
    }

    fn record_mut(&mut self) -> Result<&mut SessionRecord> {
        self.session
            .as_mut()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "session host has not started a session".to_owned(),
            })
    }
}

fn effective_slot(request: &StartRequest) -> Result<SlotProjection> {
    let mut slot = request.slot.clone();
    if let Some(accessible_content) = request.accessible_content.as_deref() {
        if accessible_content.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "accessible content".to_owned(),
            });
        }
        slot.accessible_name = accessible_content.to_owned();
    }
    Ok(slot)
}

fn check_identity(
    expected_session: &SessionId,
    expected_instance: &InstanceId,
    received_session: &SessionId,
    received_instance: &InstanceId,
) -> Result<()> {
    if expected_session != received_session {
        return Err(BridgeError::SessionMismatch {
            expected: expected_session.clone(),
            received: received_session.clone(),
        });
    }
    if expected_instance != received_instance {
        return Err(BridgeError::InstanceMismatch {
            expected: expected_instance.clone(),
            received: received_instance.clone(),
        });
    }
    Ok(())
}

fn check_input_route(record: &SessionRecord, route_ref: &str) -> Result<()> {
    if route_ref.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: "route".to_owned(),
        });
    }
    if let Some(expected) = &record.route_ref {
        if expected != route_ref {
            return Err(BridgeError::RouteMismatch {
                expected: expected.clone(),
                received: route_ref.to_owned(),
            });
        }
    }
    Ok(())
}

fn absorb_events(record: &mut SessionRecord, events: &[BridgeEvent]) -> Result<DispatchOutcome> {
    let max_projection_epoch = events.iter().filter_map(event_projection_epoch).max();
    let mut deferred = Vec::new();
    let mut applied = Vec::new();
    let mut outcome = DispatchOutcome::default();

    for event in events {
        if let Some(epoch) = event_epoch(event) {
            if !matches!(event, BridgeEvent::Projection { .. }) {
                let current_epoch = record
                    .protocol
                    .current_epoch()
                    .ok_or(BridgeError::Unmounted)?;
                if max_projection_epoch.is_some_and(|max| epoch < max) {
                    continue;
                }
                if epoch < current_epoch {
                    return Err(BridgeError::StaleEpoch {
                        expected: current_epoch,
                        received: epoch,
                    });
                }
                if epoch > current_epoch {
                    deferred.push(event);
                    continue;
                }
            }
        }
        absorb_event(record, event, &mut outcome)?;
        applied.push(event.clone());
    }

    for event in deferred {
        let Some(epoch) = event_epoch(event) else {
            continue;
        };
        if record.protocol.current_epoch() != Some(epoch) {
            continue;
        }
        absorb_event(record, event, &mut outcome)?;
        applied.push(event.clone());
    }
    outcome.events = applied;
    Ok(outcome)
}

fn absorb_event(
    record: &mut SessionRecord,
    event: &BridgeEvent,
    outcome: &mut DispatchOutcome,
) -> Result<()> {
    match event {
        BridgeEvent::Projection { projection } => {
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                &projection.session_id,
                &projection.instance_id,
            )?;
            let current_epoch = record
                .protocol
                .current_epoch()
                .ok_or(BridgeError::Unmounted)?;
            if projection.view_epoch < current_epoch {
                return Err(BridgeError::StaleEpoch {
                    expected: current_epoch,
                    received: projection.view_epoch,
                });
            }
            if projection.view_epoch > current_epoch {
                record.protocol.install_view(projection.view_epoch)?;
            }
            if projection.commit_id == 0 {
                return Err(BridgeError::InvalidCommit);
            }
            if projection.commit_id < record.projection.commit_id {
                return Err(BridgeError::StaleCommit {
                    last: record.projection.commit_id,
                    received: projection.commit_id,
                });
            }
            record.projection = projection.clone();
            record.style = projection.style.clone();
            record.a11y = projection.a11y.clone();
            record.pending_commit = Some(projection.commit_id);
            record.mounted = true;
        }
        BridgeEvent::Style {
            session_id,
            instance_id,
            view_epoch,
            style,
        } => {
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                session_id,
                instance_id,
            )?;
            check_epoch(record, *view_epoch)?;
            record.style = style.clone();
        }
        BridgeEvent::A11y {
            session_id,
            instance_id,
            view_epoch,
            a11y,
        } => {
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                session_id,
                instance_id,
            )?;
            check_epoch(record, *view_epoch)?;
            record.a11y = Some(a11y.clone());
        }
        BridgeEvent::State {
            session_id,
            instance_id,
            view_epoch,
            values,
        } => {
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                session_id,
                instance_id,
            )?;
            check_epoch(record, *view_epoch)?;
            record.state_values = values.clone().into_iter().collect();
        }
        BridgeEvent::Signal {
            session_id,
            instance_id,
            view_epoch,
            sequence,
            key,
        } => {
            check_identity(
                &record.request.session_id,
                &record.request.instance_id,
                session_id,
                instance_id,
            )?;
            check_epoch(record, *view_epoch)?;
            if *sequence <= record.last_output_sequence {
                return Err(BridgeError::NonMonotonicSequence {
                    last: record.last_output_sequence,
                    received: *sequence,
                });
            }
            record.last_output_sequence = *sequence;
            if key == "click" {
                outcome.click_emitted = true;
            }
        }
        BridgeEvent::Diagnostic { diagnostic } => {
            if diagnostic.fatal {
                return Err(BridgeError::Runtime {
                    detail: diagnostic.detail.clone(),
                });
            }
            record.diagnostics.push(diagnostic.clone());
            outcome.diagnostics.push(diagnostic.clone());
        }
        BridgeEvent::Registry { .. } | BridgeEvent::Ready { .. } => {}
    }
    Ok(())
}

fn event_epoch(event: &BridgeEvent) -> Option<ViewEpoch> {
    match event {
        BridgeEvent::Projection { projection } => Some(projection.view_epoch),
        BridgeEvent::Style { view_epoch, .. }
        | BridgeEvent::A11y { view_epoch, .. }
        | BridgeEvent::State { view_epoch, .. }
        | BridgeEvent::Signal { view_epoch, .. } => Some(*view_epoch),
        BridgeEvent::Registry { .. }
        | BridgeEvent::Ready { .. }
        | BridgeEvent::Diagnostic { .. } => None,
    }
}

fn event_projection_epoch(event: &BridgeEvent) -> Option<ViewEpoch> {
    match event {
        BridgeEvent::Projection { projection } => Some(projection.view_epoch),
        _ => None,
    }
}

fn check_epoch(record: &SessionRecord, received: ViewEpoch) -> Result<()> {
    let expected = record
        .protocol
        .current_epoch()
        .ok_or(BridgeError::Unmounted)?;
    if expected != received {
        return Err(BridgeError::StaleEpoch { expected, received });
    }
    Ok(())
}
