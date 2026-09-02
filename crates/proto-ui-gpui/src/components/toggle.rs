use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle, CommitDisposition,
    InputKind, InputSource, NativeStyle, ProtoAdapter, PrototypeKey, PrototypeProfile, Result,
    SessionId, SessionSnapshot, ShadcnTheme, SlotProjection, StartRequest,
};

const TOGGLE_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnToggle,
    exposed_states: &[
        "active",
        "disabled",
        "hovered",
        "pressed",
        "focused",
        "focusVisible",
    ],
    signals: &["activeChange"],
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

pub struct ProtoToggleHost {
    adapter: ProtoAdapter,
    props: BTreeMap<String, ToggleProps>,
}

impl ProtoToggleHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            props: BTreeMap::new(),
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
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:toggle:{id}"))?,
            crate::InstanceId::new(format!("sailbreak:toggle-instance:{id}"))?,
            PrototypeKey::ShadcnToggle,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        );
        self.adapter
            .start(id.clone(), label, TOGGLE_PROFILE, request)?;
        self.props.insert(id, props);
        Ok(())
    }

    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<ToggleDispatchOutcome> {
        self.require_props(id)?;
        let outcome = self.adapter.dispatch(id, kind, source, detail)?;
        Ok(ToggleDispatchOutcome {
            active_change_count: outcome.signal_count("activeChange"),
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        })
    }

    pub fn set_props(&mut self, id: &str, props: ToggleProps) -> Result<CommitDisposition> {
        self.require_props(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.props.insert(id.to_owned(), props);
        Ok(disposition)
    }

    pub fn snapshot(&self, id: &str) -> Result<ProtoToggleSnapshot> {
        let props = self.require_props(id)?.clone();
        let snapshot = self.adapter.snapshot_current(id)?;
        let active = snapshot
            .state_bool("active")
            .unwrap_or_else(|| props.active.unwrap_or(props.default_active));
        let disabled = snapshot.state_bool("disabled").unwrap_or(props.disabled);
        let a11y = snapshot.session.a11y.clone();
        Ok(ProtoToggleSnapshot {
            id: snapshot.id,
            label: snapshot.label,
            session: snapshot.session,
            native_style: snapshot.native_style,
            resolved_style: snapshot.resolved_style,
            active,
            disabled,
            a11y,
        })
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        self.require_props(id)?;
        self.adapter.dispose(id)?;
        self.props.remove(id);
        Ok(())
    }

    fn require_props(&self, id: &str) -> Result<&ToggleProps> {
        self.props.get(id).ok_or_else(|| unknown_toggle(id))
    }
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
