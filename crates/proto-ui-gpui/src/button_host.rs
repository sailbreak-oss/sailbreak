use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::protocol::{
    BridgeCommand, BridgeDiagnostic, BridgeError, BridgeEvent, BridgeState, DispatchOutcome,
    InputEnvelope, InputKind, InputSource, InstanceId, ProjectionAck, ProjectionTransaction,
    Result, SessionId, SlotProjection,
};
use crate::quickjs::QuickJsBridge;
use crate::theme::{ButtonStyle, ShadcnTheme};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonVariant {
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ShadcnButtonVariant {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
            Self::Outline => "outline",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Link => "link",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonSize {
    Default,
    Sm,
    Lg,
    Icon,
}

impl ShadcnButtonSize {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Lg => "lg",
            Self::Icon => "icon",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProtoButtonState {
    pub id: String,
    pub label: String,
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub projection: ProjectionTransaction,
    pub style: ButtonStyle,
    pub a11y: Option<crate::protocol::A11ySnapshot>,
    pub state_values: BTreeMap<String, Value>,
    pub click_count: u64,
    variant: ShadcnButtonVariant,
    size: ShadcnButtonSize,
    disabled: bool,
    protocol: BridgeState,
    diagnostics: Vec<BridgeDiagnostic>,
}

impl ProtoButtonState {
    #[must_use]
    pub fn state(&self, key: &str) -> Option<bool> {
        self.state_values.get(key).and_then(Value::as_bool)
    }

    #[must_use]
    pub fn variant(&self) -> ShadcnButtonVariant {
        self.variant
    }

    #[must_use]
    pub fn size(&self) -> ShadcnButtonSize {
        self.size
    }

    #[must_use]
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[BridgeDiagnostic] {
        &self.diagnostics
    }

    fn apply_event(&mut self, event: &BridgeEvent, theme: ShadcnTheme) -> bool {
        match event {
            BridgeEvent::Projection { projection } => {
                self.projection = projection.clone();
                self.style = ButtonStyle::from_projection(&projection.style, theme);
                self.a11y = projection.a11y.clone();
                false
            }
            BridgeEvent::Style { style, .. } => {
                self.projection.style = style.clone();
                self.style = ButtonStyle::from_projection(style, theme);
                false
            }
            BridgeEvent::A11y { a11y, .. } => {
                self.a11y = Some(a11y.clone());
                false
            }
            BridgeEvent::State { values, .. } => {
                self.state_values = values.clone();
                self.disabled = values
                    .get("disabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(self.disabled);
                false
            }
            BridgeEvent::Signal { key, .. } if key == "click" => {
                self.click_count += 1;
                true
            }
            BridgeEvent::Diagnostic { diagnostic } => {
                self.diagnostics.push(diagnostic.clone());
                false
            }
            BridgeEvent::Registry { .. }
            | BridgeEvent::Ready { .. }
            | BridgeEvent::Signal { .. } => false,
        }
    }
}

/// Host-side manager for Proto UI Shadcn Button instances.
///
/// It owns no Button semantics. It only moves typed data into the embedded
/// Runtime and retains the latest projection for GPUI rendering.
pub struct ProtoButtonHost {
    bridge: QuickJsBridge,
    theme: ShadcnTheme,
    buttons: BTreeMap<String, ProtoButtonState>,
    next_sequence: u64,
}

impl ProtoButtonHost {
    pub fn new() -> Result<Self> {
        Ok(Self {
            bridge: QuickJsBridge::new()?,
            theme: ShadcnTheme::default(),
            buttons: BTreeMap::new(),
            next_sequence: 0,
        })
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        let mut host = Self::new()?;
        host.theme = theme;
        Ok(host)
    }

    pub fn register_button(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        variant: ShadcnButtonVariant,
        size: ShadcnButtonSize,
    ) -> Result<()> {
        let id = id.into();
        let label = label.into();
        if id.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "button".to_owned(),
            });
        }
        if label.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "button accessible name".to_owned(),
            });
        }
        if self.buttons.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate button: {id}"),
            });
        }

        let session_id = SessionId::new(format!("sailbreak:button:{id}"))?;
        let instance_id = InstanceId::new(format!("sailbreak:button-instance:{id}"))?;
        let command = BridgeCommand::Start {
            session_id: session_id.clone(),
            instance_id: instance_id.clone(),
            prototype: crate::protocol::PrototypeKey::ShadcnButton,
            props: button_props(variant, size, false),
            slot: SlotProjection::new(format!("{id}:slot"), label.clone()),
        };
        let start_events = self.bridge.dispatch(&command)?;
        let projection = start_events
            .iter()
            .find_map(|event| match event {
                BridgeEvent::Projection { projection } => Some(projection.clone()),
                _ => None,
            })
            .ok_or_else(|| BridgeError::Runtime {
                detail: format!("Proto UI did not project button {id}"),
            })?;

        let mut protocol = BridgeState::new(session_id.clone(), instance_id.clone());
        protocol.install_view(projection.view_epoch)?;
        let ack = ProjectionAck::applied(
            session_id.clone(),
            instance_id.clone(),
            projection.view_epoch,
            projection.commit_id,
        );
        protocol.accept_ack(ack.clone())?;
        let ack_events = self
            .bridge
            .dispatch(&BridgeCommand::ProjectionAck { ack })?;

        let mut state = ProtoButtonState {
            id: id.clone(),
            label,
            session_id,
            instance_id,
            style: ButtonStyle::from_projection(&projection.style, self.theme),
            a11y: projection.a11y.clone(),
            projection,
            state_values: BTreeMap::new(),
            click_count: 0,
            variant,
            size,
            disabled: false,
            protocol,
            diagnostics: Vec::new(),
        };
        for event in start_events.iter().chain(ack_events.iter()) {
            state.apply_event(event, self.theme);
        }
        self.buttons.insert(id, state);
        Ok(())
    }

    pub fn set_disabled(&mut self, id: &str, disabled: bool) -> Result<DispatchOutcome> {
        let (session_id, instance_id, variant, size) = {
            let button = self.button(id).ok_or_else(|| unknown_button(id))?;
            (
                button.session_id.clone(),
                button.instance_id.clone(),
                button.variant,
                button.size,
            )
        };
        let events = self.bridge.dispatch(&BridgeCommand::SetProps {
            session_id,
            instance_id,
            props: button_props(variant, size, disabled),
        })?;
        self.apply_projection_events(id, events)
    }

    pub fn set_variant(
        &mut self,
        id: &str,
        variant: ShadcnButtonVariant,
    ) -> Result<DispatchOutcome> {
        let (session_id, instance_id, size, disabled) = {
            let button = self.button(id).ok_or_else(|| unknown_button(id))?;
            (
                button.session_id.clone(),
                button.instance_id.clone(),
                button.size,
                button.disabled,
            )
        };
        let events = self.bridge.dispatch(&BridgeCommand::SetProps {
            session_id,
            instance_id,
            props: button_props(variant, size, disabled),
        })?;
        let outcome = self.apply_projection_events(id, events)?;
        if let Some(button) = self.buttons.get_mut(id) {
            button.variant = variant;
        }
        Ok(outcome)
    }

    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DispatchOutcome> {
        self.next_sequence =
            self.next_sequence
                .checked_add(1)
                .ok_or(BridgeError::NonMonotonicSequence {
                    last: u64::MAX,
                    received: u64::MAX,
                })?;
        let sequence = self.next_sequence;
        let button = self.buttons.get_mut(id).ok_or_else(|| unknown_button(id))?;
        let input = InputEnvelope::new(
            button.session_id.clone(),
            button.instance_id.clone(),
            button.projection.view_epoch,
            sequence,
            crate::protocol::InputPayload::new(format!("{id}:sample:{sequence}"), id, source, kind),
        );
        button.protocol.accept_input(input.clone())?;
        let events = self
            .bridge
            .dispatch(&BridgeCommand::Input { input, detail })?;
        let mut outcome = DispatchOutcome::default();
        for event in events {
            if event_belongs_to(button, &event) && button.apply_event(&event, self.theme) {
                outcome.click_emitted = true;
            }
            if let BridgeEvent::Diagnostic { diagnostic } = &event {
                outcome.diagnostics.push(diagnostic.clone());
            }
        }
        Ok(outcome)
    }

    #[must_use]
    pub fn button(&self, id: &str) -> Option<&ProtoButtonState> {
        self.buttons.get(id)
    }

    #[must_use]
    pub fn theme(&self) -> ShadcnTheme {
        self.theme
    }

    fn apply_projection_events(
        &mut self,
        id: &str,
        events: Vec<BridgeEvent>,
    ) -> Result<DispatchOutcome> {
        let theme = self.theme;
        let projection = events.iter().find_map(|event| match event {
            BridgeEvent::Projection { projection } => Some(projection.clone()),
            _ => None,
        });
        let Some(projection) = projection else {
            return Ok(self.apply_events(id, events));
        };

        let ack = {
            let button = self.buttons.get_mut(id).ok_or_else(|| unknown_button(id))?;
            if projection.view_epoch > button.projection.view_epoch {
                button.protocol.install_view(projection.view_epoch)?;
            }
            button.apply_event(
                &BridgeEvent::Projection {
                    projection: projection.clone(),
                },
                theme,
            );
            let ack = ProjectionAck::applied(
                button.session_id.clone(),
                button.instance_id.clone(),
                projection.view_epoch,
                projection.commit_id,
            );
            button.protocol.accept_ack(ack.clone())?;
            ack
        };
        let ack_events = self
            .bridge
            .dispatch(&BridgeCommand::ProjectionAck { ack })?;
        let button = self.buttons.get_mut(id).ok_or_else(|| unknown_button(id))?;
        let mut outcome = Self::apply_events_to_button(button, events, theme);
        let ack_outcome = Self::apply_events_to_button(button, ack_events, theme);
        outcome.click_emitted |= ack_outcome.click_emitted;
        outcome.diagnostics.extend(ack_outcome.diagnostics);
        Ok(outcome)
    }
    fn apply_events(&mut self, id: &str, events: Vec<BridgeEvent>) -> DispatchOutcome {
        let theme = self.theme;
        let Some(button) = self.buttons.get_mut(id) else {
            return DispatchOutcome {
                diagnostics: vec![BridgeDiagnostic::new(
                    "unknown-button",
                    format!("button is not registered: {id}"),
                    true,
                )],
                ..DispatchOutcome::default()
            };
        };
        Self::apply_events_to_button(button, events, theme)
    }
    fn apply_events_to_button(
        button: &mut ProtoButtonState,
        events: Vec<BridgeEvent>,
        theme: ShadcnTheme,
    ) -> DispatchOutcome {
        let mut outcome = DispatchOutcome::default();
        for event in events {
            if event_belongs_to(button, &event) && button.apply_event(&event, theme) {
                outcome.click_emitted = true;
            }
            if let BridgeEvent::Diagnostic { diagnostic } = &event {
                outcome.diagnostics.push(diagnostic.clone());
            }
        }
        outcome
    }
}

fn button_props(
    variant: ShadcnButtonVariant,
    size: ShadcnButtonSize,
    disabled: bool,
) -> Map<String, Value> {
    let mut props = Map::new();
    props.insert(
        "variant".to_owned(),
        Value::String(variant.as_str().to_owned()),
    );
    props.insert("size".to_owned(), Value::String(size.as_str().to_owned()));
    props.insert("disabled".to_owned(), Value::Bool(disabled));
    props
}

fn event_belongs_to(button: &ProtoButtonState, event: &BridgeEvent) -> bool {
    match event {
        BridgeEvent::Registry { .. }
        | BridgeEvent::Ready { .. }
        | BridgeEvent::Diagnostic { .. } => true,
        BridgeEvent::Projection { projection } => {
            projection.session_id == button.session_id
                && projection.instance_id == button.instance_id
        }
        BridgeEvent::Style {
            session_id,
            instance_id,
            ..
        }
        | BridgeEvent::A11y {
            session_id,
            instance_id,
            ..
        }
        | BridgeEvent::State {
            session_id,
            instance_id,
            ..
        }
        | BridgeEvent::Signal {
            session_id,
            instance_id,
            ..
        } => session_id == &button.session_id && instance_id == &button.instance_id,
    }
}

fn unknown_button(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown button: {id}"),
    }
}
