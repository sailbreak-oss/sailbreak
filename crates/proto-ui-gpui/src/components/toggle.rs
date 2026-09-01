use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle, CommitDisposition,
    InputEnvelope, InputKind, InputPayload, InputRequest, InputSource, InstanceId, NativeStyle,
    ProjectionAck, PropsRequest, ProtoSessionHost, PrototypeKey, Result, SessionId,
    SessionSnapshot, ShadcnTheme, SlotProjection, StartRequest, translate_projection,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

impl ToggleVariant {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ToggleProps {
    pub variant: ToggleVariant,
    pub size: ToggleSize,
    pub active: Option<bool>,
    pub default_active: bool,
    pub disabled: bool,
}

impl ToggleProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert(
            "variant".to_owned(),
            Value::String(self.variant.as_str().to_owned()),
        );
        props.insert(
            "size".to_owned(),
            Value::String(self.size.as_str().to_owned()),
        );
        if let Some(active) = self.active {
            props.insert("active".to_owned(), Value::Bool(active));
        }
        props.insert("defaultActive".to_owned(), Value::Bool(self.default_active));
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoToggleSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub active: bool,
    pub disabled: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToggleDispatchOutcome {
    pub active_change_count: usize,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

struct ToggleRecord {
    id: String,
    label: String,
    props: ToggleProps,
    session: ProtoSessionHost,
    next_sequence: u64,
}

pub struct ProtoToggleHost {
    toggles: BTreeMap<String, ToggleRecord>,
    theme: ShadcnTheme,
}

impl ProtoToggleHost {
    pub fn new() -> Result<Self> {
        Ok(Self {
            toggles: BTreeMap::new(),
            theme: ShadcnTheme::default(),
        })
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            toggles: BTreeMap::new(),
            theme,
        })
    }

    pub fn register(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: ToggleProps,
    ) -> Result<()> {
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "toggle")?;
        validate_identity(&label, "toggle accessible name")?;
        if self.toggles.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate toggle: {id}"),
            });
        }

        let session_id = SessionId::new(format!("sailbreak:toggle:{id}"))?;
        let instance_id = InstanceId::new(format!("sailbreak:toggle-instance:{id}"))?;
        let mut session = ProtoSessionHost::new()?;
        session.start(StartRequest::new(
            session_id,
            instance_id,
            PrototypeKey::ShadcnToggle,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        ))?;
        acknowledge_pending(&mut session)?;

        self.toggles.insert(
            id.clone(),
            ToggleRecord {
                id,
                label,
                props,
                session,
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
    ) -> Result<ToggleDispatchOutcome> {
        let record = self.record_mut(id)?;
        record.next_sequence =
            record
                .next_sequence
                .checked_add(1)
                .ok_or(BridgeError::NonMonotonicSequence {
                    last: u64::MAX,
                    received: u64::MAX,
                })?;
        let current = record.session.snapshot()?;
        let input = InputEnvelope::new(
            current.session_id.clone(),
            current.instance_id.clone(),
            current.projection.view_epoch,
            record.next_sequence,
            InputPayload::new(
                format!("{id}:sample:{}", record.next_sequence),
                id,
                source,
                kind,
            ),
        );
        let outcome = record.session.input(InputRequest::new(input, detail))?;
        let active_change_count = outcome
            .events
            .iter()
            .filter(
                |event| matches!(event, BridgeEvent::Signal { key, .. } if key == "activeChange"),
            )
            .count();
        Ok(ToggleDispatchOutcome {
            active_change_count,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        })
    }

    pub fn set_props(&mut self, id: &str, props: ToggleProps) -> Result<CommitDisposition> {
        let record = self.record_mut(id)?;
        let snapshot = record.session.snapshot()?;
        let disposition = record.session.set_props(PropsRequest::new(
            snapshot.session_id,
            snapshot.instance_id,
            props.to_map(),
        ))?;
        acknowledge_pending(&mut record.session)?;
        record.props = props;
        Ok(disposition)
    }

    pub fn snapshot(&self, id: &str) -> Result<ProtoToggleSnapshot> {
        let record = self.record(id)?;
        let session = record.session.snapshot()?;
        let active = state_bool(&session, "active")
            .unwrap_or_else(|| record.props.active.unwrap_or(record.props.default_active));
        let disabled = state_bool(&session, "disabled").unwrap_or(record.props.disabled);
        let native_style = translate_projection(&session.style, self.theme);
        let resolved_style = ButtonStyle::from_projection(&session.style, self.theme);
        let a11y = session.a11y.clone();
        Ok(ProtoToggleSnapshot {
            id: record.id.clone(),
            label: record.label.clone(),
            session,
            native_style,
            resolved_style,
            active,
            disabled,
            a11y,
        })
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        let mut record = self.toggles.remove(id).ok_or_else(|| unknown_toggle(id))?;
        record.session.dispose()
    }

    fn record(&self, id: &str) -> Result<&ToggleRecord> {
        self.toggles.get(id).ok_or_else(|| unknown_toggle(id))
    }

    fn record_mut(&mut self, id: &str) -> Result<&mut ToggleRecord> {
        self.toggles.get_mut(id).ok_or_else(|| unknown_toggle(id))
    }
}

fn acknowledge_pending(session: &mut ProtoSessionHost) -> Result<()> {
    let snapshot = session.snapshot()?;
    if snapshot.pending_commit {
        session.acknowledge(ProjectionAck::applied(
            snapshot.session_id,
            snapshot.instance_id,
            snapshot.projection.view_epoch,
            snapshot.projection.commit_id,
        ))?;
    }
    Ok(())
}

fn state_bool(snapshot: &SessionSnapshot, key: &str) -> Option<bool> {
    snapshot.state_values.get(key).and_then(Value::as_bool)
}

fn validate_identity(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: kind.to_owned(),
        });
    }
    Ok(())
}

fn unknown_toggle(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown toggle: {id}"),
    }
}
