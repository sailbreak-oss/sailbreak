use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AdapterDispatchOutcome, BridgeDiagnostic, BridgeError, BridgeEvent,
    CommitDisposition, InputKind, InputSource, NativeStyle, ProtoAdapter, PrototypeKey,
    PrototypeProfile, Result, SessionId, SessionSnapshot, ShadcnTheme, SlotProjection,
    StartRequest, TextControlEvent, TextControlEventEnvelope, TextControlEventType,
    TextControlPatch, TextControlRef, TextControlSelection, TextControlValueMode, TextControlWrap,
    ViewEpoch,
};

const TEXTAREA_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnTextareaRoot,
    exposed_states: &[
        "value",
        "disabled",
        "readOnly",
        "focused",
        "focusVisible",
        "composing",
    ],
    signals: &[
        "valueChange",
        "change",
        "compositionStart",
        "compositionUpdate",
        "compositionEnd",
    ],
};

/// The shadcn Textarea API exposed by this host adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextareaProps {
    pub value: Option<String>,
    pub default_value: String,
    pub disabled: bool,
    pub read_only: bool,
    pub placeholder: String,
    pub rows: u32,
    pub required: bool,
    pub name: String,
    pub auto_complete: String,
    pub min_length: i32,
    pub max_length: i32,
    pub wrap: TextControlWrap,
    pub aria_label: String,
    pub labelled_by: String,
    pub described_by: String,
}

impl Default for TextareaProps {
    fn default() -> Self {
        Self {
            value: None,
            default_value: String::new(),
            disabled: false,
            read_only: false,
            placeholder: String::new(),
            rows: 2,
            required: false,
            name: String::new(),
            auto_complete: String::new(),
            min_length: -1,
            max_length: -1,
            wrap: TextControlWrap::Soft,
            aria_label: String::new(),
            labelled_by: String::new(),
            described_by: String::new(),
        }
    }
}

impl TextareaProps {
    #[must_use]
    pub fn to_patch(&self) -> TextControlPatch {
        TextControlPatch {
            value_mode: Some(if self.value.is_some() {
                TextControlValueMode::Controlled
            } else {
                TextControlValueMode::Uncontrolled
            }),
            value: self.value.as_deref().map(canonicalize),
            default_value: Some(canonicalize(&self.default_value)),
            disabled: Some(self.disabled),
            read_only: Some(self.read_only),
            placeholder: Some(self.placeholder.clone()),
            rows: Some(self.rows.max(1)),
            required: Some(self.required),
            name: Some(self.name.clone()),
            auto_complete: Some(self.auto_complete.clone()),
            min_length: Some(self.min_length),
            max_length: Some(self.max_length),
            wrap: Some(self.wrap),
        }
    }

    fn to_map(&self) -> Map<String, Value> {
        let patch = self.to_patch();
        let mut props = Map::new();
        if let Some(value) = patch.value {
            props.insert("value".to_owned(), Value::String(value));
        }
        if let Some(value) = patch.default_value {
            props.insert("defaultValue".to_owned(), Value::String(value));
        }
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props.insert("readOnly".to_owned(), Value::Bool(self.read_only));
        props.insert(
            "placeholder".to_owned(),
            Value::String(self.placeholder.clone()),
        );
        props.insert("rows".to_owned(), Value::from(self.rows.max(1)));
        props.insert("required".to_owned(), Value::Bool(self.required));
        props.insert("name".to_owned(), Value::String(self.name.clone()));
        props.insert(
            "autoComplete".to_owned(),
            Value::String(self.auto_complete.clone()),
        );
        props.insert("minLength".to_owned(), Value::from(self.min_length));
        props.insert("maxLength".to_owned(), Value::from(self.max_length));
        props.insert(
            "wrap".to_owned(),
            Value::String(
                match self.wrap {
                    TextControlWrap::Soft => "soft",
                    TextControlWrap::Hard => "hard",
                }
                .to_owned(),
            ),
        );
        props.insert(
            "ariaLabel".to_owned(),
            Value::String(self.aria_label.clone()),
        );
        props.insert(
            "labelledBy".to_owned(),
            Value::String(self.labelled_by.clone()),
        );
        props.insert(
            "describedBy".to_owned(),
            Value::String(self.described_by.clone()),
        );
        props
    }
}

/// Alias matching the public textarea terminology while retaining the module's
/// canonical wrap type.
pub type TextareaWrap = TextControlWrap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextareaDispatchOutcome {
    pub input_event_count: usize,
    pub change_event_count: usize,
    pub composition_start_count: usize,
    pub composition_update_count: usize,
    pub composition_end_count: usize,
    pub events: Vec<BridgeEvent>,
    pub text_events: Vec<TextControlEventEnvelope>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl TextareaDispatchOutcome {
    #[must_use]
    pub fn text_event_count(&self, event_type: TextControlEventType) -> usize {
        self.text_events
            .iter()
            .filter(|event| event.event.event_type == event_type)
            .count()
    }

    fn absorb(&mut self, mut other: Self) {
        self.input_event_count += other.input_event_count;
        self.change_event_count += other.change_event_count;
        self.composition_start_count += other.composition_start_count;
        self.composition_update_count += other.composition_update_count;
        self.composition_end_count += other.composition_end_count;
        self.events.append(&mut other.events);
        self.text_events.append(&mut other.text_events);
        self.diagnostics.append(&mut other.diagnostics);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct NativeTextControl {
    control_ref: TextControlRef,
    epoch: ViewEpoch,
    value: String,
    selection: TextControlSelection,
    composing: bool,
    controlled: bool,
    focused: bool,
    deferred_value: Option<String>,
    input_event_count: usize,
    change_event_count: usize,
    composition_start_count: usize,
    composition_update_count: usize,
    composition_end_count: usize,
}

impl NativeTextControl {
    fn new(control_ref: TextControlRef, epoch: ViewEpoch, value: String, controlled: bool) -> Self {
        Self {
            control_ref,
            epoch,
            value: canonicalize(&value),
            selection: TextControlSelection::caret(0),
            composing: false,
            focused: false,
            deferred_value: None,
            controlled,
            input_event_count: 0,
            change_event_count: 0,
            composition_start_count: 0,
            composition_update_count: 0,
            composition_end_count: 0,
        }
    }

    fn apply_patch(&mut self, patch: &TextControlPatch, allow_value_projection: bool) {
        if let Some(value_mode) = patch.value_mode {
            self.controlled = value_mode == TextControlValueMode::Controlled;
        }
        let next_value = self.controlled.then(|| patch.value.clone()).flatten();
        if let Some(next_value) = next_value.map(|value| canonicalize(&value))
            && next_value != self.value
        {
            if allow_value_projection {
                self.replace_value(next_value);
                self.deferred_value = None;
            } else {
                self.deferred_value = Some(next_value);
            }
        }
    }

    fn replace_value(&mut self, value: String) {
        self.value = value;
        self.selection = self.selection.clamp(utf16_len(&self.value));
    }

    fn finish_composition(&mut self) {
        self.composing = false;
        if let Some(value) = self.deferred_value.take() {
            self.replace_value(value);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoTextareaSnapshot {
    pub id: String,
    pub label: String,
    pub profile: PrototypeProfile,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub value: String,
    pub native_value: String,
    pub selection: TextControlSelection,
    pub disabled: bool,
    pub read_only: bool,
    pub placeholder: String,
    pub rows: u32,
    pub required: bool,
    pub name: String,
    pub auto_complete: String,
    pub min_length: i32,
    pub max_length: i32,
    pub wrap: TextControlWrap,
    pub composing: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
    pub input_event_count: usize,
    pub change_event_count: usize,
    pub composition_start_count: usize,
    pub composition_update_count: usize,
    pub composition_end_count: usize,
}

pub struct ProtoTextareaHost {
    adapter: ProtoAdapter,
    props: BTreeMap<String, TextareaProps>,
    native: BTreeMap<String, NativeTextControl>,
}

impl ProtoTextareaHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            props: BTreeMap::new(),
            native: BTreeMap::new(),
        })
    }

    pub fn register(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: TextareaProps,
    ) -> Result<()> {
        let id = id.into();
        let label = label.into();
        let mut props = props;
        if props.aria_label.trim().is_empty() {
            props.aria_label = label.clone();
        }
        validate_identity(&id, "textarea")?;
        validate_identity(&label, "textarea accessible name")?;
        let session_id = SessionId::new(format!("sailbreak:textarea:{id}"))?;
        let instance_id = crate::InstanceId::new(format!("sailbreak:textarea-instance:{id}"))?;
        let request = StartRequest::new(
            session_id,
            instance_id,
            PrototypeKey::ShadcnTextareaRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        );
        self.adapter
            .start(id.clone(), label, TEXTAREA_PROFILE, request)?;
        let snapshot = self.adapter.snapshot_current(&id)?;
        let control_ref = TextControlRef::new(format!("sailbreak:textarea:{id}:text-control"))?;
        let initial_value = props
            .value
            .as_deref()
            .unwrap_or(&props.default_value)
            .to_owned();
        self.native.insert(
            id.clone(),
            NativeTextControl::new(
                control_ref,
                snapshot.session.projection.view_epoch,
                initial_value,
                props.value.is_some(),
            ),
        );
        self.props.insert(id, props);
        Ok(())
    }

    pub fn register_textarea(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: TextareaProps,
    ) -> Result<()> {
        self.register(id, label, props)
    }

    pub fn set_props(&mut self, id: &str, props: TextareaProps) -> Result<CommitDisposition> {
        self.require_props(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        let epoch = self
            .adapter
            .snapshot_current(id)?
            .session
            .projection
            .view_epoch;
        let native = self
            .native
            .get_mut(id)
            .ok_or_else(|| unknown_textarea(id))?;
        native.epoch = epoch;
        native.apply_patch(&props.to_patch(), !native.composing);
        self.props.insert(id.to_owned(), props);
        Ok(disposition)
    }

    pub fn snapshot(&mut self, id: &str) -> Result<ProtoTextareaSnapshot> {
        let props = self.require_props(id)?.clone();
        let snapshot = self.adapter.snapshot(id)?;
        let native = self
            .native
            .get_mut(id)
            .ok_or_else(|| unknown_textarea(id))?;
        native.epoch = snapshot.session.projection.view_epoch;
        let state_string = |key: &str| {
            snapshot
                .session
                .state_values
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_owned)
        };
        let state_bool = |key: &str, fallback: bool| {
            snapshot
                .session
                .state_values
                .get(key)
                .and_then(Value::as_bool)
                .unwrap_or(fallback)
        };
        let disabled = state_bool("disabled", props.disabled);
        let read_only = state_bool("readOnly", props.read_only);
        let value = state_string("value").unwrap_or_else(|| native.value.clone());
        Ok(ProtoTextareaSnapshot {
            id: snapshot.id,
            label: snapshot.label,
            profile: snapshot.profile,
            native_style: snapshot.native_style,
            session: snapshot.session.clone(),
            value,
            native_value: native.value.clone(),
            selection: native.selection,
            disabled,
            read_only,
            placeholder: props.placeholder,
            rows: props.rows.max(1),
            required: props.required,
            name: props.name,
            auto_complete: props.auto_complete,
            min_length: props.min_length,
            max_length: props.max_length,
            wrap: props.wrap,
            composing: state_bool("composing", native.composing),
            focused: state_bool("focused", native.focused),
            focus_visible: state_bool("focusVisible", false),
            a11y: snapshot.session.a11y,
            input_event_count: native.input_event_count,
            change_event_count: native.change_event_count,
            composition_start_count: native.composition_start_count,
            composition_update_count: native.composition_update_count,
            composition_end_count: native.composition_end_count,
        })
    }

    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<TextareaDispatchOutcome> {
        self.require_props(id)?;
        if matches!(kind, InputKind::Focus) && self.snapshot(id)?.disabled {
            return Ok(TextareaDispatchOutcome::default());
        }
        let outcome = self.adapter.dispatch(id, kind, source, detail)?;
        let native = self
            .native
            .get_mut(id)
            .ok_or_else(|| unknown_textarea(id))?;
        if kind == InputKind::Focus {
            native.focused = true;
        } else if kind == InputKind::Blur {
            native.focused = false;
        }
        Ok(adapter_outcome(outcome))
    }

    pub fn focus(&mut self, id: &str) -> Result<bool> {
        let outcome = self.dispatch(id, InputKind::Focus, InputSource::Programmatic, None)?;
        Ok(!outcome.events.is_empty() && self.snapshot(id)?.focused)
    }

    pub fn blur(&mut self, id: &str) -> Result<()> {
        self.dispatch(id, InputKind::Blur, InputSource::Programmatic, None)?;
        Ok(())
    }

    pub fn set_selection(&mut self, id: &str, selection: TextControlSelection) -> Result<()> {
        let native = self
            .native
            .get_mut(id)
            .ok_or_else(|| unknown_textarea(id))?;
        native.selection = selection.clamp(utf16_len(&native.value));
        Ok(())
    }

    #[must_use]
    pub fn text_control_ref(&self, id: &str) -> Option<&TextControlRef> {
        self.native.get(id).map(|native| &native.control_ref)
    }

    pub fn dispatch_text(
        &mut self,
        id: &str,
        event: TextControlEvent,
    ) -> Result<TextareaDispatchOutcome> {
        let epoch = self
            .adapter
            .snapshot_current(id)?
            .session
            .projection
            .view_epoch;
        self.dispatch_text_at_epoch(id, epoch, event)
    }

    pub fn dispatch_text_at_epoch(
        &mut self,
        id: &str,
        epoch: ViewEpoch,
        event: TextControlEvent,
    ) -> Result<TextareaDispatchOutcome> {
        self.dispatch_text_event_at_epoch(id, epoch, event, None)
    }

    pub fn dispatch_text_with_selection_at_epoch(
        &mut self,
        id: &str,
        epoch: ViewEpoch,
        event: TextControlEvent,
        selection: TextControlSelection,
    ) -> Result<TextareaDispatchOutcome> {
        self.dispatch_text_event_at_epoch(id, epoch, event, Some(selection))
    }

    fn dispatch_text_event_at_epoch(
        &mut self,
        id: &str,
        epoch: ViewEpoch,
        mut event: TextControlEvent,
        selection: Option<TextControlSelection>,
    ) -> Result<TextareaDispatchOutcome> {
        let props = self.require_props(id)?.clone();
        let native = self.native.get(id).ok_or_else(|| unknown_textarea(id))?;
        if native.epoch != epoch {
            return Err(BridgeError::StaleEpoch {
                expected: native.epoch,
                received: epoch,
            });
        }
        if props.disabled
            || (props.read_only
                && matches!(
                    event.event_type,
                    TextControlEventType::Input
                        | TextControlEventType::CompositionStart
                        | TextControlEventType::CompositionUpdate
                        | TextControlEventType::CompositionEnd
                ))
        {
            return Ok(TextareaDispatchOutcome::default());
        }
        event.value = canonicalize(&event.value);
        let selection = {
            let native = self.native.get(id).ok_or_else(|| unknown_textarea(id))?;
            Some(
                selection
                    .unwrap_or(native.selection)
                    .clamp(utf16_len(&event.value)),
            )
        };
        let outcome = self
            .adapter
            .dispatch_text_at_epoch(id, epoch, event.clone(), selection)?;
        let native = self
            .native
            .get_mut(id)
            .ok_or_else(|| unknown_textarea(id))?;
        native.selection = selection.unwrap_or(native.selection);
        match event.event_type {
            TextControlEventType::Input => {
                native.input_event_count += 1;
                native.composing = event.composing;
                if props.value.is_some() && !event.composing {
                    native.replace_value(canonicalize(props.value.as_deref().unwrap_or_default()));
                } else {
                    native.replace_value(event.value);
                }
            }
            TextControlEventType::Change => {
                native.change_event_count += 1;
                if let Some(value) = props.value.as_deref() {
                    native.replace_value(canonicalize(value));
                }
            }
            TextControlEventType::CompositionStart => {
                native.composition_start_count += 1;
                native.composing = true;
                native.replace_value(event.value);
            }
            TextControlEventType::CompositionUpdate => {
                native.composition_update_count += 1;
                native.composing = true;
                native.replace_value(event.value);
            }
            TextControlEventType::CompositionEnd => {
                native.composition_end_count += 1;
                native.replace_value(event.value);
                native.finish_composition();
                if let Some(value) = props.value.as_deref() {
                    native.replace_value(canonicalize(value));
                }
            }
        }
        Ok(adapter_outcome(outcome))
    }

    pub fn input(
        &mut self,
        id: &str,
        value: impl Into<String>,
        data: Option<String>,
        input_type: Option<String>,
        composing: bool,
    ) -> Result<TextareaDispatchOutcome> {
        let event = TextControlEvent {
            event_type: TextControlEventType::Input,
            value: value.into(),
            composing,
            data,
            input_type,
        };
        self.dispatch_text(id, event)
    }

    pub fn change(&mut self, id: &str) -> Result<TextareaDispatchOutcome> {
        let props = self.require_props(id)?;
        let value = props.value.clone().unwrap_or_else(|| {
            self.native
                .get(id)
                .map(|native| native.value.clone())
                .unwrap_or_default()
        });
        self.dispatch_text(id, TextControlEvent::change(value))
    }

    pub fn composition_start(
        &mut self,
        id: &str,
        data: Option<String>,
    ) -> Result<TextareaDispatchOutcome> {
        let value = self
            .native
            .get(id)
            .ok_or_else(|| unknown_textarea(id))?
            .value
            .clone();
        self.dispatch_text(id, TextControlEvent::composition_start(value, data))
    }

    pub fn composition_update(
        &mut self,
        id: &str,
        data: Option<String>,
    ) -> Result<TextareaDispatchOutcome> {
        let value = self
            .native
            .get(id)
            .ok_or_else(|| unknown_textarea(id))?
            .value
            .clone();
        self.dispatch_text(id, TextControlEvent::composition_update(value, data))
    }

    pub fn composition_end(
        &mut self,
        id: &str,
        data: Option<String>,
    ) -> Result<TextareaDispatchOutcome> {
        let value = self
            .native
            .get(id)
            .ok_or_else(|| unknown_textarea(id))?
            .value
            .clone();
        let mut outcome = self.dispatch_text(
            id,
            TextControlEvent::composition_end(value.clone(), data.clone()),
        )?;
        let trailing_input = TextControlEvent {
            event_type: TextControlEventType::Input,
            value,
            composing: false,
            data,
            input_type: Some("insertCompositionText".to_owned()),
        };
        outcome.absorb(self.dispatch_text(id, trailing_input)?);
        Ok(outcome)
    }

    pub fn remount(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_props(id)?;
        let epoch = self.adapter.remount(id)?;
        let native = self
            .native
            .get_mut(id)
            .ok_or_else(|| unknown_textarea(id))?;
        native.epoch = epoch;
        native.composing = false;
        native.deferred_value = None;
        native.focused = false;
        Ok(epoch)
    }

    pub fn dispose(&mut self, id: &str) -> Result<()> {
        self.require_props(id)?;
        self.adapter.dispose(id)?;
        self.props.remove(id);
        self.native.remove(id);
        Ok(())
    }

    fn require_props(&self, id: &str) -> Result<&TextareaProps> {
        self.props.get(id).ok_or_else(|| unknown_textarea(id))
    }
}

fn adapter_outcome(outcome: AdapterDispatchOutcome) -> TextareaDispatchOutcome {
    let mut result = TextareaDispatchOutcome {
        text_events: outcome.text_events,
        events: outcome.events,
        diagnostics: outcome.diagnostics,
        ..TextareaDispatchOutcome::default()
    };
    for event in &result.text_events {
        match event.event.event_type {
            TextControlEventType::Input => result.input_event_count += 1,
            TextControlEventType::Change => result.change_event_count += 1,
            TextControlEventType::CompositionStart => result.composition_start_count += 1,
            TextControlEventType::CompositionUpdate => result.composition_update_count += 1,
            TextControlEventType::CompositionEnd => result.composition_end_count += 1,
        }
    }
    result
}

fn canonicalize(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn validate_identity(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: kind.to_owned(),
        });
    }
    Ok(())
}
fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

fn unknown_textarea(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown textarea: {id}"),
    }
}
