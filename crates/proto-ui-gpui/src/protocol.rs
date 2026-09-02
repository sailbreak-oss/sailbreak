use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

pub const PROTO_UI_VERSION: &str = "main-snapshot";
pub const GPUI_VERSION: &str = "0.2.2";
pub const PROTOCOL_MAJOR: u16 = 1;
pub const PROTOCOL_MINOR: u16 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostIdentity {
    pub name: String,
    pub gpui: String,
    pub platform: String,
}

impl HostIdentity {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        gpui: impl Into<String>,
        platform: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            gpui: gpui.into(),
            platform: platform.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BridgeHandshake {
    pub protocol: ProtocolVersion,
    pub proto_ui: String,
    pub host: HostIdentity,
    pub registry_digest: String,
}

impl BridgeHandshake {
    #[must_use]
    pub fn new(
        protocol: ProtocolVersion,
        host: HostIdentity,
        registry_digest: impl Into<String>,
    ) -> Self {
        Self {
            protocol,
            proto_ui: PROTO_UI_VERSION.to_owned(),
            host,
            registry_digest: registry_digest.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "session".to_owned(),
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId(String);

impl InstanceId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "instance".to_owned(),
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LogicalParentRef {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub view_epoch: ViewEpoch,
    pub route_ref: String,
}

impl LogicalParentRef {
    pub fn new(
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        route_ref: impl Into<String>,
    ) -> Result<Self> {
        let route_ref = route_ref.into();
        if route_ref.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "parent route".to_owned(),
            });
        }
        Ok(Self {
            session_id,
            instance_id,
            view_epoch,
            route_ref,
        })
    }
}

/// Opaque logical parent identity used to connect separately mounted parts.
pub type ParentRef = LogicalParentRef;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewEpoch(u64);

impl ViewEpoch {
    pub fn new(value: u64) -> Result<Self> {
        if value == 0 {
            return Err(BridgeError::InvalidEpoch);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
impl fmt::Display for ViewEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TemplateNode {
    Container {
        tag: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        style: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Self>,
    },
    Text {
        text: String,
    },
    Slot {
        slot_id: String,
    },
    Svg {
        tag: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        attributes: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Self>,
    },
}

impl TemplateNode {
    #[must_use]
    pub fn slot(slot_id: impl Into<String>) -> Self {
        Self::Slot {
            slot_id: slot_id.into(),
        }
    }

    #[must_use]
    pub fn svg(
        tag: impl Into<String>,
        attributes: BTreeMap<String, String>,
        children: Vec<Self>,
    ) -> Self {
        Self::Svg {
            tag: tag.into(),
            attributes,
            children,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotProjection {
    pub slot_id: String,
    pub accessible_name: String,
}

impl SlotProjection {
    #[must_use]
    pub fn new(slot_id: impl Into<String>, accessible_name: impl Into<String>) -> Self {
        Self {
            slot_id: slot_id.into(),
            accessible_name: accessible_name.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StyleProjection {
    pub tokens: Vec<String>,
}

impl StyleProjection {
    #[must_use]
    pub fn new(tokens: impl IntoIterator<Item = String>) -> Self {
        Self {
            tokens: tokens.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct A11ySnapshot {
    pub role: String,
    pub name: String,
    pub disabled: bool,
    pub focused: bool,
    pub focus_visible: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toggled: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
}

impl A11ySnapshot {
    #[must_use]
    pub fn button(name: impl Into<String>, disabled: bool) -> Self {
        Self {
            role: "button".to_owned(),
            name: name.into(),
            disabled,
            focused: false,
            focus_visible: false,
            hidden: false,
            orientation: None,
            selected: None,
            toggled: None,
            actions: vec!["activate".to_owned()],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionPayload {
    pub template: Vec<TemplateNode>,
    pub slot: SlotProjection,
    pub style: StyleProjection,
    pub a11y: Option<A11ySnapshot>,
}

impl ProjectionPayload {
    #[must_use]
    pub fn new(
        template: Vec<TemplateNode>,
        slot: SlotProjection,
        style: StyleProjection,
        a11y: Option<A11ySnapshot>,
    ) -> Self {
        Self {
            template,
            slot,
            style,
            a11y,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionTransaction {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub view_epoch: ViewEpoch,
    pub commit_id: u64,
    pub template: Vec<TemplateNode>,
    pub slot: SlotProjection,
    pub style: StyleProjection,
    pub a11y: Option<A11ySnapshot>,
}

impl ProjectionTransaction {
    pub fn new(
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        commit_id: u64,
        payload: ProjectionPayload,
    ) -> Result<Self> {
        if commit_id == 0 {
            return Err(BridgeError::InvalidCommit);
        }
        if payload.slot.slot_id.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "slot".to_owned(),
            });
        }
        if !contains_slot(&payload.template, &payload.slot.slot_id) {
            return Err(BridgeError::MissingSlot {
                slot_id: payload.slot.slot_id,
            });
        }
        Ok(Self {
            session_id,
            instance_id,
            view_epoch,
            commit_id,
            template: payload.template,
            slot: payload.slot,
            style: payload.style,
            a11y: payload.a11y,
        })
    }
}

fn contains_slot(nodes: &[TemplateNode], slot_id: &str) -> bool {
    nodes.iter().any(|node| match node {
        TemplateNode::Slot { slot_id: candidate } => candidate == slot_id,
        TemplateNode::Container { children, .. } => contains_slot(children, slot_id),
        TemplateNode::Svg { children, .. } => contains_slot(children, slot_id),
        TemplateNode::Text { .. } => false,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionStatus {
    Applied,
    Superseded,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionAck {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub view_epoch: ViewEpoch,
    pub commit_id: u64,
    pub status: ProjectionStatus,
}

impl ProjectionAck {
    #[must_use]
    pub fn applied(
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        commit_id: u64,
    ) -> Self {
        Self::with_status(
            session_id,
            instance_id,
            view_epoch,
            commit_id,
            ProjectionStatus::Applied,
        )
    }

    #[must_use]
    pub fn with_status(
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        commit_id: u64,
        status: ProjectionStatus,
    ) -> Self {
        Self {
            session_id,
            instance_id,
            view_epoch,
            commit_id,
            status,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    Mouse,
    Keyboard,
    Accessibility,
    Touch,
    Programmatic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    PointerDown,
    PointerUp,
    PointerMove,
    PointerEnter,
    PointerLeave,
    PointerCancel,
    KeyDown,
    KeyUp,
    Focus,
    Blur,
    PressStart,
    PressEnd,
    PressCancel,
    PressCommit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputPayload {
    pub sample_id: String,
    pub route_ref: String,
    pub source: InputSource,
    pub kind: InputKind,
}

impl InputPayload {
    #[must_use]
    pub fn new(
        sample_id: impl Into<String>,
        route_ref: impl Into<String>,
        source: InputSource,
        kind: InputKind,
    ) -> Self {
        Self {
            sample_id: sample_id.into(),
            route_ref: route_ref.into(),
            source,
            kind,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InputEnvelope {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub view_epoch: ViewEpoch,
    pub sequence: u64,
    pub sample_id: String,
    pub route_ref: String,
    pub source: InputSource,
    pub kind: InputKind,
}

impl InputEnvelope {
    #[must_use]
    pub fn new(
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        sequence: u64,
        payload: InputPayload,
    ) -> Self {
        Self {
            session_id,
            instance_id,
            view_epoch,
            sequence,
            sample_id: payload.sample_id,
            route_ref: payload.route_ref,
            source: payload.source,
            kind: payload.kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextControlValueMode {
    Controlled,
    Uncontrolled,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextControlWrap {
    #[default]
    Soft,
    Hard,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextControlRef(String);

impl TextControlRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "text control".to_owned(),
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TextControlRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextControlSelectionDirection {
    #[default]
    None,
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextControlSelection {
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub direction: TextControlSelectionDirection,
}

impl TextControlSelection {
    #[must_use]
    pub const fn caret(position: usize) -> Self {
        Self {
            start: position,
            end: position,
            direction: TextControlSelectionDirection::None,
        }
    }

    #[must_use]
    pub const fn range(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            direction: TextControlSelectionDirection::None,
        }
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: TextControlSelectionDirection) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub fn clamp(self, length: usize) -> Self {
        Self {
            start: self.start.min(length),
            end: self.end.min(length),
            direction: self.direction,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextControlPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_mode: Option<TextControlValueMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_complete: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap: Option<TextControlWrap>,
}
impl TextControlPatch {
    #[must_use]
    pub fn controlled(value: impl Into<String>) -> Self {
        Self {
            value_mode: Some(TextControlValueMode::Controlled),
            value: Some(value.into()),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn uncontrolled(default_value: impl Into<String>) -> Self {
        Self {
            value_mode: Some(TextControlValueMode::Uncontrolled),
            default_value: Some(default_value.into()),
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextControlEventType {
    Input,
    Change,
    #[serde(rename = "compositionstart")]
    CompositionStart,
    #[serde(rename = "compositionupdate")]
    CompositionUpdate,
    #[serde(rename = "compositionend")]
    CompositionEnd,
}

impl TextControlEventType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Change => "change",
            Self::CompositionStart => "compositionstart",
            Self::CompositionUpdate => "compositionupdate",
            Self::CompositionEnd => "compositionend",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "input" => Ok(Self::Input),
            "change" => Ok(Self::Change),
            "compositionstart" => Ok(Self::CompositionStart),
            "compositionupdate" => Ok(Self::CompositionUpdate),
            "compositionend" => Ok(Self::CompositionEnd),
            _ => Err(BridgeError::InvalidIdentity {
                kind: format!("text-control event: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextControlEvent {
    #[serde(rename = "type")]
    pub event_type: TextControlEventType,
    pub value: String,
    pub composing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
}

impl TextControlEvent {
    #[must_use]
    pub fn new(event_type: TextControlEventType, value: impl Into<String>) -> Self {
        Self {
            event_type,
            value: value.into(),
            composing: false,
            data: None,
            input_type: None,
        }
    }

    #[must_use]
    pub fn input(value: impl Into<String>) -> Self {
        Self::new(TextControlEventType::Input, value)
    }

    #[must_use]
    pub fn change(value: impl Into<String>) -> Self {
        Self::new(TextControlEventType::Change, value)
    }

    pub fn composition_start(value: impl Into<String>, data: Option<String>) -> Self {
        Self {
            event_type: TextControlEventType::CompositionStart,
            value: value.into(),
            composing: true,
            data,
            input_type: None,
        }
    }

    #[must_use]
    pub fn composition_update(value: impl Into<String>, data: Option<String>) -> Self {
        Self {
            event_type: TextControlEventType::CompositionUpdate,
            value: value.into(),
            composing: true,
            data,
            input_type: None,
        }
    }

    #[must_use]
    pub fn composition_end(value: impl Into<String>, data: Option<String>) -> Self {
        Self {
            event_type: TextControlEventType::CompositionEnd,
            value: value.into(),
            composing: false,
            data,
            input_type: None,
        }
    }

    #[must_use]
    pub fn with_composing(mut self, composing: bool) -> Self {
        self.composing = composing;
        self
    }

    #[must_use]
    pub fn with_input_type(mut self, input_type: impl Into<String>) -> Self {
        self.input_type = Some(input_type.into());
        self
    }
}
impl Default for TextControlEvent {
    fn default() -> Self {
        Self::input(String::new())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TextControlCommand {
    Attach {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        control_ref: TextControlRef,
        patch: TextControlPatch,
    },
    Update {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        control_ref: TextControlRef,
        patch: TextControlPatch,
    },
    Event {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        sequence: u64,
        control_ref: TextControlRef,
        event: TextControlEvent,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selection: Option<TextControlSelection>,
    },
    Snapshot {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        control_ref: TextControlRef,
    },
    Dispose {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        control_ref: TextControlRef,
    },
}

impl TextControlCommand {
    #[must_use]
    pub fn event(
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        sequence: u64,
        control_ref: TextControlRef,
        event: TextControlEvent,
        selection: Option<TextControlSelection>,
    ) -> Self {
        Self::Event {
            session_id,
            instance_id,
            view_epoch,
            sequence,
            control_ref,
            event,
            selection,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextControlSnapshot {
    pub value: String,
    pub composing: bool,
    #[serde(default)]
    pub selection: TextControlSelection,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextControlEventEnvelope {
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub view_epoch: ViewEpoch,
    pub sequence: u64,
    pub control_ref: TextControlRef,
    pub event: TextControlEvent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AckDisposition {
    Applied,
    Superseded,
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum BridgeError {
    #[error("invalid {kind} identity")]
    InvalidIdentity { kind: String },
    #[error("duplicate semantic id: {id}")]
    DuplicateSemanticId { id: String },
    #[error("view epoch must be greater than zero")]
    InvalidEpoch,
    #[error("commit id must be greater than zero")]
    InvalidCommit,
    #[error("projection is missing slot {slot_id}")]
    MissingSlot { slot_id: String },
    #[error("unknown Proto UI prototype: {prototype}")]
    UnknownPrototype { prototype: String },
    #[error("embedded Proto UI runtime failed: {detail}")]
    Runtime { detail: String },
    #[error("bridge message serialization failed: {detail}")]
    Serialization { detail: String },
    #[error("bridge message decoding failed: {detail}")]
    Decode { detail: String },
    #[error("bridge is disposed")]
    Disposed,
    #[error("session mismatch: expected {expected}, received {received}")]
    SessionMismatch {
        expected: SessionId,
        received: SessionId,
    },
    #[error("instance mismatch: expected {expected}, received {received}")]
    InstanceMismatch {
        expected: InstanceId,
        received: InstanceId,
    },
    #[error("stale view epoch: expected {expected}, received {received}")]
    StaleEpoch {
        expected: ViewEpoch,
        received: ViewEpoch,
    },
    #[error("stale parent epoch: expected {expected}, received {received}")]
    StaleParent {
        expected: ViewEpoch,
        received: ViewEpoch,
    },
    #[error("parent route mismatch: expected {expected}, received {received}")]
    ParentRouteMismatch { expected: String, received: String },
    #[error("stale commit: last {last}, received {received}")]
    StaleCommit { last: u64, received: u64 },
    #[error("non-monotonic sequence: last {last}, received {received}")]
    NonMonotonicSequence { last: u64, received: u64 },
    #[error("unsupported or failed projection status: {status:?}")]
    ProjectionRejected { status: ProjectionStatus },
    #[error("input route mismatch: expected {expected}, received {received}")]
    RouteMismatch { expected: String, received: String },
    #[error("no projection is pending")]
    NoPendingProjection,
    #[error("projection commit {commit_id} is awaiting acknowledgment")]
    ProjectionPending { commit_id: u64 },
    #[error("session is not mounted")]
    Unmounted,
}

pub type Result<T, E = BridgeError> = std::result::Result<T, E>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DispatchOutcome {
    pub events: Vec<BridgeEvent>,
    pub click_emitted: bool,
    pub text_events: Vec<TextControlEventEnvelope>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrototypeKey {
    ShadcnButton,
    ShadcnToggle,
    ShadcnCheckboxRoot,
    ShadcnCheckboxIndicator,
    ShadcnSeparatorRoot,
    ShadcnSwitchRoot,
    ShadcnSwitchThumb,
    ShadcnTextareaRoot,
    ShadcnTabsRoot,
    ShadcnTabsList,
    ShadcnTabsTrigger,
    ShadcnTabsContent,
    ShadcnHoverCardRoot,
    ShadcnHoverCardTrigger,
    ShadcnHoverCardContent,
    ShadcnDropdownRoot,
    ShadcnDropdownTrigger,
    ShadcnDropdownContent,
    ShadcnDropdownItem,
    ShadcnSelectRoot,
    ShadcnSelectTrigger,
    ShadcnSelectValue,
    ShadcnSelectContent,
    ShadcnSelectItem,
    ShadcnDialogRoot,
    ShadcnDialogTrigger,
    ShadcnDialogMask,
    ShadcnDialogContent,
    ShadcnDialogTitle,
    ShadcnDialogDescription,
    ShadcnDialogClose,
    ShadcnDialogCloseIcon,
    ShadcnDialogHeader,
    ShadcnDialogFooter,
}

impl PrototypeKey {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ShadcnButton => "shadcn-button",
            Self::ShadcnToggle => "shadcn-toggle",
            Self::ShadcnCheckboxRoot => "shadcn-checkbox-root",
            Self::ShadcnCheckboxIndicator => "shadcn-checkbox-indicator",
            Self::ShadcnSeparatorRoot => "shadcn-separator-root",
            Self::ShadcnSwitchRoot => "shadcn-switch-root",
            Self::ShadcnSwitchThumb => "shadcn-switch-thumb",
            Self::ShadcnTextareaRoot => "shadcn-textarea-root",
            Self::ShadcnTabsRoot => "shadcn-tabs-root",
            Self::ShadcnTabsList => "shadcn-tabs-list",
            Self::ShadcnTabsTrigger => "shadcn-tabs-trigger",
            Self::ShadcnTabsContent => "shadcn-tabs-content",
            Self::ShadcnHoverCardRoot => "shadcn-hover-card-root",
            Self::ShadcnHoverCardTrigger => "shadcn-hover-card-trigger",
            Self::ShadcnHoverCardContent => "shadcn-hover-card-content",
            Self::ShadcnDropdownRoot => "shadcn-dropdown-root",
            Self::ShadcnDropdownTrigger => "shadcn-dropdown-trigger",
            Self::ShadcnDropdownContent => "shadcn-dropdown-content",
            Self::ShadcnDropdownItem => "shadcn-dropdown-item",
            Self::ShadcnSelectRoot => "shadcn-select-root",
            Self::ShadcnSelectTrigger => "shadcn-select-trigger",
            Self::ShadcnSelectValue => "shadcn-select-value",
            Self::ShadcnSelectContent => "shadcn-select-content",
            Self::ShadcnSelectItem => "shadcn-select-item",
            Self::ShadcnDialogRoot => "shadcn-dialog-root",
            Self::ShadcnDialogTrigger => "shadcn-dialog-trigger",
            Self::ShadcnDialogMask => "shadcn-dialog-mask",
            Self::ShadcnDialogContent => "shadcn-dialog-content",
            Self::ShadcnDialogTitle => "shadcn-dialog-title",
            Self::ShadcnDialogDescription => "shadcn-dialog-description",
            Self::ShadcnDialogClose => "shadcn-dialog-close",
            Self::ShadcnDialogCloseIcon => "shadcn-dialog-close-icon",
            Self::ShadcnDialogHeader => "shadcn-dialog-header",
            Self::ShadcnDialogFooter => "shadcn-dialog-footer",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "shadcn-button" => Ok(Self::ShadcnButton),
            "shadcn-toggle" => Ok(Self::ShadcnToggle),
            "shadcn-checkbox-root" => Ok(Self::ShadcnCheckboxRoot),
            "shadcn-checkbox-indicator" => Ok(Self::ShadcnCheckboxIndicator),
            "shadcn-separator-root" => Ok(Self::ShadcnSeparatorRoot),
            "shadcn-switch-root" => Ok(Self::ShadcnSwitchRoot),
            "shadcn-switch-thumb" => Ok(Self::ShadcnSwitchThumb),
            "shadcn-textarea-root" => Ok(Self::ShadcnTextareaRoot),
            "shadcn-tabs-root" => Ok(Self::ShadcnTabsRoot),
            "shadcn-tabs-list" => Ok(Self::ShadcnTabsList),
            "shadcn-tabs-trigger" => Ok(Self::ShadcnTabsTrigger),
            "shadcn-tabs-content" => Ok(Self::ShadcnTabsContent),
            "shadcn-hover-card-root" => Ok(Self::ShadcnHoverCardRoot),
            "shadcn-hover-card-trigger" => Ok(Self::ShadcnHoverCardTrigger),
            "shadcn-hover-card-content" => Ok(Self::ShadcnHoverCardContent),
            "shadcn-dropdown-root" => Ok(Self::ShadcnDropdownRoot),
            "shadcn-dropdown-trigger" => Ok(Self::ShadcnDropdownTrigger),
            "shadcn-dropdown-content" => Ok(Self::ShadcnDropdownContent),
            "shadcn-dropdown-item" => Ok(Self::ShadcnDropdownItem),
            "shadcn-select-root" => Ok(Self::ShadcnSelectRoot),
            "shadcn-select-trigger" => Ok(Self::ShadcnSelectTrigger),
            "shadcn-select-value" => Ok(Self::ShadcnSelectValue),
            "shadcn-select-content" => Ok(Self::ShadcnSelectContent),
            "shadcn-select-item" => Ok(Self::ShadcnSelectItem),
            "shadcn-dialog-root" => Ok(Self::ShadcnDialogRoot),
            "shadcn-dialog-trigger" => Ok(Self::ShadcnDialogTrigger),
            "shadcn-dialog-mask" => Ok(Self::ShadcnDialogMask),
            "shadcn-dialog-content" => Ok(Self::ShadcnDialogContent),
            "shadcn-dialog-title" => Ok(Self::ShadcnDialogTitle),
            "shadcn-dialog-description" => Ok(Self::ShadcnDialogDescription),
            "shadcn-dialog-close" => Ok(Self::ShadcnDialogClose),
            "shadcn-dialog-close-icon" => Ok(Self::ShadcnDialogCloseIcon),
            "shadcn-dialog-header" => Ok(Self::ShadcnDialogHeader),
            "shadcn-dialog-footer" => Ok(Self::ShadcnDialogFooter),
            _ => Err(BridgeError::UnknownPrototype {
                prototype: value.to_owned(),
            }),
        }
    }

    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ShadcnButton,
            Self::ShadcnToggle,
            Self::ShadcnCheckboxRoot,
            Self::ShadcnCheckboxIndicator,
            Self::ShadcnSeparatorRoot,
            Self::ShadcnSwitchRoot,
            Self::ShadcnSwitchThumb,
            Self::ShadcnTextareaRoot,
            Self::ShadcnTabsRoot,
            Self::ShadcnTabsList,
            Self::ShadcnTabsTrigger,
            Self::ShadcnTabsContent,
            Self::ShadcnHoverCardRoot,
            Self::ShadcnHoverCardTrigger,
            Self::ShadcnHoverCardContent,
            Self::ShadcnDropdownRoot,
            Self::ShadcnDropdownTrigger,
            Self::ShadcnDropdownContent,
            Self::ShadcnDropdownItem,
            Self::ShadcnSelectRoot,
            Self::ShadcnSelectTrigger,
            Self::ShadcnSelectValue,
            Self::ShadcnSelectContent,
            Self::ShadcnSelectItem,
            Self::ShadcnDialogRoot,
            Self::ShadcnDialogTrigger,
            Self::ShadcnDialogMask,
            Self::ShadcnDialogContent,
            Self::ShadcnDialogTitle,
            Self::ShadcnDialogDescription,
            Self::ShadcnDialogClose,
            Self::ShadcnDialogCloseIcon,
            Self::ShadcnDialogHeader,
            Self::ShadcnDialogFooter,
        ]
    }
}

impl fmt::Display for PrototypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeCommand {
    Registry,
    Start {
        session_id: SessionId,
        instance_id: InstanceId,
        prototype: PrototypeKey,
        #[serde(default)]
        props: serde_json::Map<String, serde_json::Value>,
        #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
        meta: serde_json::Map<String, serde_json::Value>,
        slot: SlotProjection,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        route_ref: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent: Option<LogicalParentRef>,
    },
    ProjectionAck {
        ack: ProjectionAck,
    },
    Input {
        input: InputEnvelope,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<serde_json::Value>,
    },
    TextControl {
        command: TextControlCommand,
    },
    SetProps {
        session_id: SessionId,
        instance_id: InstanceId,
        props: serde_json::Map<String, serde_json::Value>,
    },
    Remount {
        session_id: SessionId,
        instance_id: InstanceId,
    },
    AdvanceTime {
        session_id: SessionId,
        instance_id: InstanceId,
        milliseconds: u64,
    },
    Unmount {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
    },
    Dispose {
        session_id: SessionId,
        instance_id: InstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeEvent {
    Registry {
        proto_ui: String,
        keys: Vec<String>,
    },
    Ready {
        handshake: BridgeHandshake,
    },
    Projection {
        projection: ProjectionTransaction,
    },
    Style {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        style: StyleProjection,
    },
    A11y {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        a11y: A11ySnapshot,
    },
    State {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        values: std::collections::BTreeMap<String, serde_json::Value>,
    },
    Signal {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        sequence: u64,
        key: String,
    },
    TextControl {
        session_id: SessionId,
        instance_id: InstanceId,
        view_epoch: ViewEpoch,
        sequence: u64,
        control_ref: TextControlRef,
        event: TextControlEvent,
    },
    Diagnostic {
        diagnostic: BridgeDiagnostic,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BridgeDiagnostic {
    pub code: String,
    pub detail: String,
    pub fatal: bool,
}

impl BridgeDiagnostic {
    #[must_use]
    pub fn new(code: impl Into<String>, detail: impl Into<String>, fatal: bool) -> Self {
        Self {
            code: code.into(),
            detail: detail.into(),
            fatal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeState {
    session_id: SessionId,
    instance_id: InstanceId,
    view_epoch: Option<ViewEpoch>,
    last_commit_id: u64,
    last_sequence: u64,
    disposed: bool,
}

impl BridgeState {
    #[must_use]
    pub fn new(session_id: SessionId, instance_id: InstanceId) -> Self {
        Self {
            session_id,
            instance_id,
            view_epoch: None,
            last_commit_id: 0,
            last_sequence: 0,
            disposed: false,
        }
    }

    pub fn install_view(&mut self, epoch: ViewEpoch) -> Result<()> {
        self.ensure_alive()?;
        if let Some(current) = self.view_epoch
            && epoch <= current
        {
            return Err(BridgeError::StaleEpoch {
                expected: current,
                received: epoch,
            });
        }
        self.view_epoch = Some(epoch);
        self.last_commit_id = 0;
        self.last_sequence = 0;
        Ok(())
    }

    pub fn accept_ack(&mut self, ack: ProjectionAck) -> Result<AckDisposition> {
        self.ensure_alive()?;
        self.check_identity(&ack.session_id, &ack.instance_id)?;
        self.check_epoch(ack.view_epoch)?;
        if ack.commit_id == 0 {
            return Err(BridgeError::InvalidCommit);
        }
        if ack.commit_id < self.last_commit_id {
            return Err(BridgeError::StaleCommit {
                last: self.last_commit_id,
                received: ack.commit_id,
            });
        }
        let disposition = match ack.status {
            ProjectionStatus::Applied => AckDisposition::Applied,
            ProjectionStatus::Superseded => AckDisposition::Superseded,
            status => return Err(BridgeError::ProjectionRejected { status }),
        };
        self.last_commit_id = self.last_commit_id.max(ack.commit_id);
        Ok(disposition)
    }

    pub fn accept_input(&mut self, input: InputEnvelope) -> Result<()> {
        self.ensure_alive()?;
        self.check_identity(&input.session_id, &input.instance_id)?;
        self.check_epoch(input.view_epoch)?;
        if input.sequence <= self.last_sequence {
            return Err(BridgeError::NonMonotonicSequence {
                last: self.last_sequence,
                received: input.sequence,
            });
        }
        self.last_sequence = input.sequence;
        Ok(())
    }

    pub fn dispose(&mut self) {
        self.disposed = true;
        self.view_epoch = None;
    }

    #[must_use]
    pub fn current_epoch(&self) -> Option<ViewEpoch> {
        self.view_epoch
    }

    fn ensure_alive(&self) -> Result<()> {
        if self.disposed {
            Err(BridgeError::Disposed)
        } else {
            Ok(())
        }
    }

    fn check_identity(&self, session_id: &SessionId, instance_id: &InstanceId) -> Result<()> {
        if session_id != &self.session_id {
            return Err(BridgeError::SessionMismatch {
                expected: self.session_id.clone(),
                received: session_id.clone(),
            });
        }
        if instance_id != &self.instance_id {
            return Err(BridgeError::InstanceMismatch {
                expected: self.instance_id.clone(),
                received: instance_id.clone(),
            });
        }
        Ok(())
    }

    fn check_epoch(&self, epoch: ViewEpoch) -> Result<()> {
        match self.view_epoch {
            Some(expected) if expected == epoch => Ok(()),
            Some(expected) => Err(BridgeError::StaleEpoch {
                expected,
                received: epoch,
            }),
            None => Err(BridgeError::StaleEpoch {
                expected: ViewEpoch(0),
                received: epoch,
            }),
        }
    }
}
