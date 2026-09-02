use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AnchorRef, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle, CloseReason,
    CommitDisposition, ConnectionRevision, FocusOperationResult, FocusRegistry, InputKind,
    InputSource, LayerRole, LogicalParentRef, NativeStyle, OverlayEvent, OverlayEventEnvelope,
    OverlayHost, OverlayLease, OverlayRect, OverlayRequest, OverlaySurfaceRef, PlacementPolicy,
    PlacementSnapshot, ProtoAdapter, PrototypeKey, PrototypeProfile, Result, SessionId,
    SessionSnapshot, ShadcnTheme, Side, SideAlign, SlotProjection, StartRequest, ViewEpoch,
};

const SELECT_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSelectRoot,
    exposed_states: &["open", "value", "textValue"],
    signals: &["openChange", "valueChange"],
};

const SELECT_TRIGGER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSelectTrigger,
    exposed_states: &[
        "disabled",
        "hovered",
        "focused",
        "focusVisible",
        "pressed",
        "placeholder",
    ],
    signals: &[],
};

const SELECT_VALUE_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSelectValue,
    exposed_states: &["displayValue"],
    signals: &[],
};

const SELECT_CONTENT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSelectContent,
    exposed_states: &["open"],
    signals: &[],
};

const SELECT_ITEM_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnSelectItem,
    exposed_states: &[
        "disabled",
        "hovered",
        "focused",
        "focusVisible",
        "pressed",
        "active",
        "selected",
    ],
    signals: &["select"],
};

/// Placement mode used by Shadcn Select Content.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectPosition {
    #[default]
    ItemAligned,
    Popper,
}

impl SelectPosition {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ItemAligned => "item-aligned",
            Self::Popper => "popper",
        }
    }
}

/// Alias matching the long-form Content terminology used by callers.
pub type SelectContentPosition = SelectPosition;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectRootProps {
    pub open: Option<bool>,
    pub default_open: bool,
    pub value: Option<String>,
    pub default_value: String,
    pub disabled: bool,
    pub close_on_select: bool,
}

impl SelectRootProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(open) = self.open {
            props.insert("open".to_owned(), Value::Bool(open));
        }
        props.insert("defaultOpen".to_owned(), Value::Bool(self.default_open));
        if let Some(value) = &self.value {
            props.insert("value".to_owned(), Value::String(value.clone()));
        }
        props.insert(
            "defaultValue".to_owned(),
            Value::String(self.default_value.clone()),
        );
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props.insert(
            "closeOnSelect".to_owned(),
            Value::Bool(self.close_on_select),
        );
        props
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectTriggerProps {
    pub disabled: bool,
}

impl SelectTriggerProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectValueProps {
    pub placeholder: String,
}

impl SelectValueProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert(
            "placeholder".to_owned(),
            Value::String(self.placeholder.clone()),
        );
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectContentProps {
    pub position: SelectPosition,
    pub side: Side,
    pub align: SideAlign,
    pub side_offset: f32,
    pub align_offset: f32,
    pub avoid_collisions: bool,
    pub collision_padding: f32,
}

impl Default for SelectContentProps {
    fn default() -> Self {
        Self {
            position: SelectPosition::ItemAligned,
            side: Side::Bottom,
            align: SideAlign::Center,
            side_offset: 4.0,
            align_offset: 0.0,
            avoid_collisions: true,
            collision_padding: 10.0,
        }
    }
}

impl SelectContentProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert(
            "position".to_owned(),
            Value::String(self.position.as_str().to_owned()),
        );
        props.insert(
            "side".to_owned(),
            Value::String(side_name(self.side).to_owned()),
        );
        props.insert(
            "align".to_owned(),
            Value::String(align_name(self.align).to_owned()),
        );
        props.insert("sideOffset".to_owned(), Value::from(self.side_offset));
        props.insert("alignOffset".to_owned(), Value::from(self.align_offset));
        props.insert(
            "avoidCollisions".to_owned(),
            Value::Bool(self.avoid_collisions),
        );
        props.insert(
            "collisionPadding".to_owned(),
            Value::from(self.collision_padding),
        );
        props
    }

    fn placement_policy(&self) -> PlacementPolicy {
        match self.position {
            SelectPosition::ItemAligned => PlacementPolicy::ItemAligned {
                align: self.align,
                align_offset: self.align_offset,
            },
            SelectPosition::Popper => PlacementPolicy::Popper {
                side: self.side,
                side_offset: self.side_offset,
                align: self.align,
                align_offset: self.align_offset,
                avoid_collisions: self.avoid_collisions,
                collision_padding: self.collision_padding,
            },
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectItemProps {
    pub value: String,
    pub text_value: String,
    pub disabled: bool,
    pub close_on_select: Option<bool>,
}

impl SelectItemProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert("value".to_owned(), Value::String(self.value.clone()));
        props.insert(
            "textValue".to_owned(),
            Value::String(self.text_value.clone()),
        );
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        if let Some(close_on_select) = self.close_on_select {
            props.insert("closeOnSelect".to_owned(), Value::Bool(close_on_select));
        }
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectRootSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub value: String,
    pub open: bool,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectTriggerSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub disabled: bool,
    pub placeholder: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectValueSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub display_value: String,
    pub placeholder: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectContentSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub present: bool,
    pub placement: Option<PlacementSnapshot>,
    pub portal_lease_id: Option<u64>,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectItemSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub value: String,
    pub text_value: String,
    pub disabled: bool,
    pub active: bool,
    pub selected: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub selected_indicator: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoSelectSnapshot {
    pub root: SelectRootSnapshot,
    pub trigger: Option<SelectTriggerSnapshot>,
    pub value: Option<SelectValueSnapshot>,
    pub content: Option<SelectContentSnapshot>,
    pub items: Vec<SelectItemSnapshot>,
}

/// Alias for callers that use the family name without the host prefix.
pub type SelectSnapshot = ProtoSelectSnapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectDispatchOutcome {
    pub open_change_count: usize,
    pub value_change_count: usize,
    pub item_select_count: usize,
    pub selected_values: Vec<String>,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl SelectDispatchOutcome {
    #[must_use]
    pub fn signal_count(&self, key: &str) -> usize {
        match key {
            "openChange" => self.open_change_count,
            "valueChange" => self.value_change_count,
            "select" => self.item_select_count,
            _ => 0,
        }
    }

    #[must_use]
    pub fn selection_count(&self) -> usize {
        self.item_select_count
    }
}

struct SelectRootRecord {
    id: String,
    label: String,
    props: SelectRootProps,
    snapshot: Option<SelectRootSnapshot>,
}

struct SelectTriggerRecord {
    id: String,
    props: SelectTriggerProps,
    snapshot: Option<SelectTriggerSnapshot>,
}

struct SelectValueRecord {
    id: String,
    props: SelectValueProps,
    snapshot: Option<SelectValueSnapshot>,
}

struct SelectContentRecord {
    id: String,
    props: SelectContentProps,
    snapshot: Option<SelectContentSnapshot>,
}

struct SelectItemRecord {
    id: String,
    label: String,
    props: SelectItemProps,
    snapshot: Option<SelectItemSnapshot>,
}

/// Host-side adapter for a Shadcn Select Root/Trigger/Value/Content/Item family.
///
/// Registration records the complete logical graph before any Runtime sessions
/// are started. Once [`Self::setup`] runs, each member remains an independent
/// Proto session linked by opaque parent and family route references. Proto owns
/// open/value/selection/typeahead semantics; this host only projects snapshots,
/// native focus targets, and the Rust-owned overlay lease.
pub struct ProtoSelectHost {
    adapter: ProtoAdapter,
    root: Option<SelectRootRecord>,
    trigger: Option<SelectTriggerRecord>,
    value: Option<SelectValueRecord>,
    content: Option<SelectContentRecord>,
    items: BTreeMap<String, SelectItemRecord>,
    item_order: Vec<String>,
    focus: FocusRegistry,
    overlay: OverlayHost,
    overlay_lease: Option<OverlayLease>,
    placement: Option<PlacementSnapshot>,
    family_route: String,
    setup_done: bool,
}

impl ProtoSelectHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        static NEXT_FAMILY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let family = NEXT_FAMILY.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            root: None,
            trigger: None,
            value: None,
            content: None,
            items: BTreeMap::new(),
            item_order: Vec::new(),
            focus: FocusRegistry::new(),
            overlay: OverlayHost::new(64),
            overlay_lease: None,
            placement: None,
            family_route: format!("select:{family}"),
            setup_done: false,
        })
    }

    /// Register the Select Root. No Runtime session is started until setup.
    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: SelectRootProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "select root")?;
        validate_identity(&label, "select root accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.root.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate select root".to_owned(),
            });
        }
        self.root = Some(SelectRootRecord {
            id,
            label,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_trigger(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: SelectTriggerProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "select trigger")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown select root: {root_id}"),
            });
        }
        if self.trigger.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate select trigger".to_owned(),
            });
        }
        self.trigger = Some(SelectTriggerRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_value(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: SelectValueProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "select value")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown select root: {root_id}"),
            });
        }
        if self.value.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate select value".to_owned(),
            });
        }
        self.value = Some(SelectValueRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_content(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: SelectContentProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "select content")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown select root: {root_id}"),
            });
        }
        if self.content.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate select content".to_owned(),
            });
        }
        self.content = Some(SelectContentRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_item(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        content_id: &str,
        props: SelectItemProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "select item")?;
        validate_identity(&label, "select item accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown select content: {content_id}"),
            });
        }
        if self.items.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate select item: {id}"),
            });
        }
        self.item_order.push(id.clone());
        self.items.insert(
            id.clone(),
            SelectItemRecord {
                id,
                label,
                props,
                snapshot: None,
            },
        );
        Ok(())
    }

    /// Start Root then all registered children using current opaque parent refs.
    pub fn setup(&mut self) -> Result<()> {
        if self.setup_done {
            return Ok(());
        }
        if self.trigger.is_none() || self.content.is_none() || self.items.is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "select graph requires trigger, content, and items".to_owned(),
            });
        }

        let (root_id, root_label, root_props) = {
            let root = self.require_root_record()?;
            (root.id.clone(), root.label.clone(), root.props.clone())
        };
        let (trigger_id, trigger_props) = {
            let trigger = self.require_trigger_record()?;
            (trigger.id.clone(), trigger.props.clone())
        };
        let value = self
            .value
            .as_ref()
            .map(|record| (record.id.clone(), record.props.clone()));
        let (content_id, content_props) = {
            let content = self.require_content_record()?;
            (content.id.clone(), content.props.clone())
        };
        let items: Vec<_> = self
            .item_order
            .iter()
            .map(|id| {
                let item = self.items.get(id).expect("registered select item");
                (item.id.clone(), item.label.clone(), item.props.clone())
            })
            .collect();

        let started = (|| -> Result<()> {
            self.start_root(&root_id, &root_label, &root_props)?;
            let root_parent = self.parent_ref(&root_id)?;
            self.start_trigger(&trigger_id, &root_parent, &trigger_props)?;
            if let Some((value_id, value_props)) = &value {
                self.start_value(value_id, &root_parent, value_props)?;
            }
            self.start_content(&content_id, &root_parent, &content_props)?;
            let content_parent = self.parent_ref(&content_id)?;
            for (id, label, props) in &items {
                self.start_item(id, label, &content_parent, props)?;
            }
            Ok(())
        })();
        if let Err(error) = started {
            self.dispose_started_sessions();
            return Err(error);
        }

        self.setup_done = true;
        if let Err(error) = self.snapshot() {
            self.setup_done = false;
            self.dispose_started_sessions();
            return Err(error);
        }
        Ok(())
    }

    /// Read the complete Select family snapshot and reconcile the Rust portal.
    pub fn snapshot(&mut self) -> Result<ProtoSelectSnapshot> {
        if !self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "select graph is not set up".to_owned(),
            });
        }

        let root_record = self.require_root_record()?.id.clone();
        let root_props = self.require_root_record()?.props.clone();
        let root_adapter = self.adapter.snapshot(&root_record)?;
        let root_session = root_adapter.session.clone();
        let root_value = state_string(&root_session, "value").unwrap_or_else(|| {
            root_props
                .value
                .clone()
                .unwrap_or(root_props.default_value.clone())
        });
        let root_open = state_bool(&root_session, "open")
            .unwrap_or_else(|| root_props.open.unwrap_or(root_props.default_open));
        let root_disabled = state_bool(&root_session, "disabled").unwrap_or(root_props.disabled);

        let root_snapshot = SelectRootSnapshot {
            id: root_record.clone(),
            label: self.require_root_record()?.label.clone(),
            session: root_session,
            native_style: root_adapter.native_style,
            value: root_value.clone(),
            open: root_open,
            disabled: root_disabled,
        };
        self.root.as_mut().expect("registered root").snapshot = Some(root_snapshot.clone());

        let trigger_snapshot = if let Some(record) = self.trigger.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let disabled =
                state_bool(&session, "disabled").unwrap_or(record.props.disabled || root_disabled);
            let placeholder = state_bool(&session, "placeholder").unwrap_or(root_value.is_empty());
            let focused = state_bool(&session, "focused").unwrap_or(false);
            let focus_visible = state_bool(&session, "focusVisible").unwrap_or(false);
            let snapshot = SelectTriggerSnapshot {
                id: record.id.clone(),
                label: self.require_root_record()?.label.clone(),
                resolved_style: ButtonStyle::from_projection(&session.style, self.adapter.theme()),
                native_style: adapter.native_style,
                open: root_open,
                disabled,
                placeholder,
                focused,
                focus_visible,
                a11y: session.a11y.clone(),
                session,
            };
            self.trigger.as_mut().expect("registered trigger").snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let value_snapshot = if let Some(record) = self.value.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let display_value = state_string(&session, "displayValue")
                .unwrap_or_else(|| value_placeholder(record, &root_value));
            let snapshot = SelectValueSnapshot {
                id: record.id.clone(),
                session,
                native_style: adapter.native_style,
                display_value,
                placeholder: record.props.placeholder.clone(),
            };
            self.value.as_mut().expect("registered value").snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let (content_snapshot, content_open, content_epoch) = if let Some(record) =
            self.content.as_ref()
        {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let epoch = session.projection.view_epoch;
            let snapshot = SelectContentSnapshot {
                id: record.id.clone(),
                resolved_style: ButtonStyle::from_projection(&session.style, self.adapter.theme()),
                native_style: adapter.native_style,
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.overlay_lease.as_ref().map(OverlayLease::id),
                a11y: session.a11y.clone(),
                session,
            };
            self.content.as_mut().expect("registered content").snapshot = Some(snapshot.clone());
            (Some(snapshot), open, Some(epoch))
        } else {
            (None, false, None)
        };

        if let Some(epoch) = content_epoch {
            self.sync_overlay(content_open, epoch)?;
        }

        let mut items = Vec::with_capacity(self.item_order.len());
        for id in self.item_order.clone() {
            let record = self.items.get(&id).expect("registered select item");
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let value = record.props.value.clone();
            let text_value = if record.props.text_value.is_empty() {
                value.clone()
            } else {
                record.props.text_value.clone()
            };
            let disabled =
                state_bool(&session, "disabled").unwrap_or(record.props.disabled || root_disabled);
            let selected = state_bool(&session, "selected")
                .unwrap_or(!value.is_empty() && value == root_value);
            let active =
                state_bool(&session, "active").unwrap_or(root_open && selected && !disabled);
            let focused = state_bool(&session, "focused").unwrap_or(false);
            let focus_visible = state_bool(&session, "focusVisible").unwrap_or(false);
            let a11y = session.a11y.clone();
            let resolved_style = ButtonStyle::from_projection(&session.style, self.adapter.theme());
            let selected_indicator = has_selected_indicator(&session.projection.template);
            let snapshot = SelectItemSnapshot {
                id: id.clone(),
                label: record.label.clone(),
                session,
                native_style: adapter.native_style,
                resolved_style,
                value,
                text_value,
                disabled,
                active,
                selected,
                focused,
                focus_visible,
                selected_indicator,
                a11y,
            };
            self.items
                .get_mut(&id)
                .expect("registered select item")
                .snapshot = Some(snapshot.clone());
            items.push(snapshot);
        }

        // Re-read Content after lease reconciliation so the same snapshot
        // exposes the current portal id and placement without JS layout work.
        let content = if content_snapshot.is_some() {
            let record = self.require_content_record()?.id.clone();
            let adapter = self.adapter.snapshot(&record)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let refreshed = SelectContentSnapshot {
                id: record,
                resolved_style: ButtonStyle::from_projection(&session.style, self.adapter.theme()),
                native_style: adapter.native_style,
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.overlay_lease.as_ref().map(OverlayLease::id),
                a11y: session.a11y.clone(),
                session,
            };
            self.content.as_mut().expect("registered content").snapshot = Some(refreshed.clone());
            Some(refreshed)
        } else {
            None
        };

        Ok(ProtoSelectSnapshot {
            root: root_snapshot,
            trigger: trigger_snapshot,
            value: value_snapshot,
            content,
            items,
        })
    }

    pub fn root(&self) -> Result<&SelectRootSnapshot> {
        self.root
            .as_ref()
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_select_root("root"))
    }

    pub fn trigger(&self, id: &str) -> Result<&SelectTriggerSnapshot> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_select_trigger(id))
    }

    pub fn value(&self, id: &str) -> Result<&SelectValueSnapshot> {
        self.value
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_select_value(id))
    }

    pub fn content(&self, id: &str) -> Result<&SelectContentSnapshot> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_select_content(id))
    }

    pub fn item(&self, id: &str) -> Result<&SelectItemSnapshot> {
        self.items
            .get(id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_select_item(id))
    }

    pub fn selected_value(&mut self) -> Result<String> {
        Ok(self.snapshot()?.root.value)
    }

    pub fn active_value(&mut self) -> Result<String> {
        Ok(self
            .snapshot()?
            .items
            .iter()
            .find(|item| item.active)
            .map(|item| item.value.clone())
            .unwrap_or_default())
    }

    pub fn display_value(&mut self) -> Result<String> {
        Ok(self
            .snapshot()?
            .value
            .map(|value| value.display_value)
            .unwrap_or_default())
    }

    pub fn is_open(&mut self) -> Result<bool> {
        Ok(self.snapshot()?.root.open)
    }

    /// Dispatch an input to any registered member. Selection/open snapshots
    /// are refreshed before the outcome is returned.
    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<SelectDispatchOutcome> {
        if !self.is_registered_id(id) {
            return Err(unknown_select_member(id));
        }
        self.dispatch_inner(id, kind, source, detail, None)
    }
    pub fn dispatch_trigger(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<SelectDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch_inner(id, kind, source, detail, None)
    }

    pub fn dispatch_content(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<SelectDispatchOutcome> {
        self.require_content(id)?;
        self.dispatch_inner(id, kind, source, detail, None)
    }

    pub fn dispatch_item(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<SelectDispatchOutcome> {
        self.require_item(id)?;
        self.dispatch_inner(id, kind, source, detail, Some(id))
    }

    /// Dispatch the normalized keyboard key to Trigger while closed and to
    /// Content while open. Proto owns navigation and typeahead semantics.
    pub fn dispatch_key(&mut self, key: &str) -> Result<SelectDispatchOutcome> {
        let open = self.is_open()?;
        let detail = Some(serde_json::json!({ "key": key }));
        if !open && matches!(key, "ArrowDown" | "ArrowUp") {
            let id = self.require_trigger_record()?.id.clone();
            self.dispatch_inner(&id, InputKind::KeyDown, InputSource::Keyboard, detail, None)
        } else {
            let id = self.require_content_record()?.id.clone();
            self.dispatch_inner(&id, InputKind::KeyDown, InputSource::Keyboard, detail, None)
        }
    }

    pub fn press_commit(&mut self, id: &str, source: InputSource) -> Result<SelectDispatchOutcome> {
        if self.items.contains_key(id) {
            self.dispatch_item(id, InputKind::PressCommit, source, None)
        } else {
            self.dispatch_trigger(id, InputKind::PressCommit, source, None)
        }
    }

    pub fn press_item(&mut self, id: &str, source: InputSource) -> Result<SelectDispatchOutcome> {
        self.dispatch_item(id, InputKind::PressCommit, source, None)
    }

    pub fn press_trigger(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<SelectDispatchOutcome> {
        self.dispatch_trigger(id, InputKind::PressCommit, source, None)
    }

    /// Request open through the trigger's Proto event route.
    pub fn open(&mut self) -> Result<SelectDispatchOutcome> {
        if self.is_open()? {
            return Ok(SelectDispatchOutcome::default());
        }
        let id = self.require_trigger_record()?.id.clone();
        self.dispatch_inner(
            &id,
            InputKind::PressCommit,
            InputSource::Programmatic,
            Some(serde_json::json!({ "key": "Enter" })),
            None,
        )
    }

    /// Close through the semantic trigger route and close the Rust lease with
    /// the supplied dismissal reason. Controlled roots remain open until the
    /// caller supplies `open: Some(false)` through [`Self::set_root_props`].
    pub fn close(&mut self, reason: CloseReason) -> Result<SelectDispatchOutcome> {
        let was_open = self.is_open()?;
        if !was_open {
            return Ok(SelectDispatchOutcome::default());
        }
        if self.require_root_record()?.props.open.is_none()
            && let Some(lease) = self.overlay_lease.as_ref()
        {
            lease.close(reason)?;
        }
        let id = self.require_trigger_record()?.id.clone();
        let outcome = self.dispatch_inner(
            &id,
            InputKind::PressCommit,
            InputSource::Programmatic,
            None,
            None,
        )?;
        if matches!(
            reason,
            CloseReason::OutsidePress | CloseReason::FocusOutside
        ) {
            let _ = self.focus_with_source(&id, InputSource::Programmatic);
        }
        Ok(outcome)
    }

    pub fn dismiss_escape(&mut self) -> Result<SelectDispatchOutcome> {
        self.close(CloseReason::Escape)
    }

    pub fn dismiss_outside(&mut self) -> Result<SelectDispatchOutcome> {
        self.close(CloseReason::OutsidePress)
    }

    pub fn set_root_props(&mut self, props: SelectRootProps) -> Result<CommitDisposition> {
        let id = self.require_root_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.root.as_mut().expect("registered root").props = props;
        Ok(disposition)
    }

    pub fn set_trigger_props(&mut self, props: SelectTriggerProps) -> Result<CommitDisposition> {
        let id = self.require_trigger_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.trigger.as_mut().expect("registered trigger").props = props;
        Ok(disposition)
    }

    pub fn set_value_props(&mut self, props: SelectValueProps) -> Result<CommitDisposition> {
        let id = self.require_value_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.value.as_mut().expect("registered value").props = props;
        Ok(disposition)
    }

    pub fn set_content_props(&mut self, props: SelectContentProps) -> Result<CommitDisposition> {
        let id = self.require_content_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.content.as_mut().expect("registered content").props = props;
        if let Some(lease) = self.overlay_lease.take() {
            lease.dispose();
        }
        self.placement = None;
        Ok(disposition)
    }

    pub fn set_item_props(
        &mut self,
        id: &str,
        props: SelectItemProps,
    ) -> Result<CommitDisposition> {
        self.require_item(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.items
            .get_mut(id)
            .expect("registered select item")
            .props = props;
        Ok(disposition)
    }

    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        if !self.is_registered_id(id) {
            return Err(unknown_select_member(id));
        }
        self.adapter.parent_ref(id)
    }

    pub fn set_focus_ready(&mut self, id: &str, ready: bool) -> Result<()> {
        self.require_trigger(id)?;
        if ready {
            let target = self.focus_target(id)?;
            self.focus.register(id, target)?;
        } else {
            self.focus.remove(id);
        }
        Ok(())
    }

    pub fn focus(&mut self, id: &str) -> Result<FocusOperationResult> {
        self.focus_with_source(id, InputSource::Keyboard)
    }

    pub fn focus_with_source(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<FocusOperationResult> {
        if self.trigger.as_ref().is_none_or(|record| record.id != id) {
            return Ok(FocusOperationResult::Rejected);
        }
        let Some(target) = self.focus.get(id) else {
            return Ok(FocusOperationResult::NotReady);
        };
        let current = self.focus_target(id)?;
        if current != *target {
            return Ok(FocusOperationResult::Rejected);
        }
        let outcome = self.adapter.dispatch(id, InputKind::Focus, source, None)?;
        if outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.fatal)
        {
            return Ok(FocusOperationResult::Rejected);
        }
        self.snapshot()?;
        Ok(FocusOperationResult::Accepted)
    }

    pub fn blur(&mut self, id: &str, source: InputSource) -> Result<SelectDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch_inner(id, InputKind::Blur, source, None, None)
    }

    pub fn focus_target(&self, id: &str) -> Result<crate::FocusTarget> {
        self.require_trigger(id)?;
        let snapshot = self.adapter.snapshot_current(id)?;
        Ok(crate::FocusTarget {
            session_id: snapshot.session.session_id.as_str().to_owned(),
            instance_id: snapshot.session.instance_id.as_str().to_owned(),
            view_epoch: snapshot.session.projection.view_epoch,
            route_ref: self.family_route.clone(),
            role: "select-trigger".to_owned(),
        })
    }

    pub fn focus_with_target(
        &mut self,
        target: crate::FocusTarget,
    ) -> Result<FocusOperationResult> {
        let Some(id) = self.trigger.as_ref().map(|record| record.id.clone()) else {
            return Ok(FocusOperationResult::Rejected);
        };
        if self.focus_target(&id)? != target {
            return Ok(FocusOperationResult::Rejected);
        }
        self.focus(&id)
    }

    pub fn remount_trigger(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_trigger(id)?;
        let epoch = self.adapter.remount(id)?;
        self.focus.remove(id);
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_content(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_content(id)?;
        let epoch = self.adapter.remount(id)?;
        if let Some(lease) = self.overlay_lease.take() {
            lease.dispose();
        }
        self.placement = None;
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_item(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_item(id)?;
        self.adapter.remount(id)
    }

    /// Compute placement entirely from host-provided geometry facts.
    pub fn compute_placement(
        &self,
        anchor_rect: OverlayRect,
        floating_size: (f32, f32),
        viewport: OverlayRect,
    ) -> Result<PlacementSnapshot> {
        self.require_content_record()?
            .props
            .placement_policy()
            .compute_placement(anchor_rect, floating_size, viewport)
    }

    pub fn update_placement(&mut self, placement: PlacementSnapshot) -> Result<()> {
        let lease = self
            .overlay_lease
            .as_ref()
            .ok_or(BridgeError::StaleOverlayLease { lease_id: 0 })?;
        lease.update(placement.clone())?;
        self.placement = Some(placement.clone());
        self.refresh_cached_content_placement(placement);
        Ok(())
    }

    pub fn update_placement_with_revision(
        &mut self,
        revision: ConnectionRevision,
        placement: PlacementSnapshot,
    ) -> Result<()> {
        let lease = self
            .overlay_lease
            .as_ref()
            .ok_or(BridgeError::StaleOverlayLease { lease_id: 0 })?;
        lease.update_with_revision(revision, placement.clone())?;
        self.placement = Some(placement.clone());
        self.refresh_cached_content_placement(placement);
        Ok(())
    }

    pub fn update_placement_from_geometry(
        &mut self,
        anchor_rect: OverlayRect,
        floating_size: (f32, f32),
        viewport: OverlayRect,
    ) -> Result<PlacementSnapshot> {
        let content_id = self.require_content_record()?.id.clone();
        let epoch = self
            .adapter
            .snapshot_current(&content_id)?
            .session
            .projection
            .view_epoch;
        let placement = self
            .compute_placement(anchor_rect, floating_size, viewport)?
            .with_view_epoch(epoch);
        self.update_placement(placement.clone())?;
        Ok(placement)
    }

    pub fn set_anchor_geometry(
        &mut self,
        anchor_rect: OverlayRect,
        floating_size: (f32, f32),
        viewport: OverlayRect,
    ) -> Result<PlacementSnapshot> {
        self.update_placement_from_geometry(anchor_rect, floating_size, viewport)
    }

    fn refresh_cached_content_placement(&mut self, placement: PlacementSnapshot) {
        if let Some(record) = self.content.as_mut()
            && let Some(snapshot) = record.snapshot.as_mut()
        {
            snapshot.placement = Some(placement);
        }
    }

    #[must_use]
    pub fn overlay_lease_id(&self) -> Option<u64> {
        self.overlay_lease.as_ref().map(OverlayLease::id)
    }

    #[must_use]
    pub fn overlay_revision(&self) -> Option<ConnectionRevision> {
        self.overlay_lease
            .as_ref()
            .and_then(|lease| self.overlay.current_revision(lease.id()))
    }

    #[must_use]
    pub fn family_route(&self) -> &str {
        &self.family_route
    }

    pub fn drain_overlay_events(&mut self) -> Vec<OverlayEvent> {
        self.overlay.drain_events()
    }

    pub fn drain_tagged_overlay_events(&mut self) -> Vec<OverlayEventEnvelope> {
        self.overlay.drain_tagged_events()
    }

    pub fn dispose(&mut self) -> Result<()> {
        if let Some(lease) = self.overlay_lease.take() {
            lease.dispose();
        }
        self.placement = None;
        if self.setup_done {
            for id in self.registered_ids() {
                let _ = self.adapter.dispose(&id);
            }
        }
        self.items.clear();
        self.item_order.clear();
        self.content = None;
        self.value = None;
        self.trigger = None;
        self.root = None;
        self.focus = FocusRegistry::new();
        self.setup_done = false;
        Ok(())
    }

    fn dispatch_inner(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
        item_id: Option<&str>,
    ) -> Result<SelectDispatchOutcome> {
        let mut outcome = self.adapter.dispatch(id, kind, source, detail)?;
        for sibling in self.registered_ids() {
            if sibling != id {
                outcome.absorb(self.adapter.drain(&sibling)?);
            }
        }
        let value_change_count = outcome.signal_count("valueChange");
        let selected_values = if value_change_count == 0 {
            Vec::new()
        } else if let Some(item_id) = item_id {
            vec![self.require_item(item_id)?.props.value.clone()]
        } else {
            vec![self.selected_value()?]
        };
        let result = SelectDispatchOutcome {
            open_change_count: outcome.signal_count("openChange"),
            value_change_count,
            item_select_count: outcome.signal_count("select"),
            selected_values,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        };
        // A signal can update sibling sessions through context. Refreshing the
        // family here also advances the portal lease in lockstep with open state.
        self.snapshot()?;
        Ok(result)
    }

    fn sync_overlay(&mut self, open: bool, epoch: ViewEpoch) -> Result<()> {
        let content = self.require_content_record()?.clone_for_overlay();
        let trigger_id = self.require_trigger_record()?.id.clone();
        if !open {
            if let Some(lease) = self.overlay_lease.as_ref() {
                lease.close(CloseReason::Programmatic)?;
            }
            return Ok(());
        }

        let existing_id = self.overlay_lease.as_ref().map(OverlayLease::id);
        let replace = existing_id.is_some_and(|id| self.overlay.view_epoch_of(id) != Some(epoch));
        if replace {
            if let Some(lease) = self.overlay_lease.take() {
                lease.dispose();
            }
            self.placement = None;
        }
        if let Some(id) = self.overlay_lease.as_ref().map(OverlayLease::id) {
            self.overlay.reopen(id)?;
            return Ok(());
        }

        let request = OverlayRequest::new(
            AnchorRef::new(format!("{}:anchor:{}", self.family_route, trigger_id))?,
            OverlaySurfaceRef::new(format!("{}:surface:{}", self.family_route, content.id))?,
            epoch,
            LayerRole::SelectContent,
            content.props.placement_policy(),
            Default::default(),
        )?
        .with_focus_restore_target(trigger_id);
        self.overlay_lease = Some(self.overlay.attach(request)?);
        Ok(())
    }

    fn start_root(&mut self, id: &str, label: &str, props: &SelectRootProps) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:select:{id}:root"))?,
            crate::InstanceId::new(format!("sailbreak:select:{id}:root-instance"))?,
            PrototypeKey::ShadcnSelectRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route);
        self.adapter.start(id, label, SELECT_ROOT_PROFILE, request)
    }

    fn start_trigger(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &SelectTriggerProps,
    ) -> Result<()> {
        let label = self.require_root_record()?.label.clone();
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:select:{id}:trigger"))?,
            crate::InstanceId::new(format!("sailbreak:select:{id}:trigger-instance"))?,
            PrototypeKey::ShadcnSelectTrigger,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Select trigger", SELECT_TRIGGER_PROFILE, request)
    }

    fn start_value(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &SelectValueProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:select:{id}:value"))?,
            crate::InstanceId::new(format!("sailbreak:select:{id}:value-instance"))?,
            PrototypeKey::ShadcnSelectValue,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Select value"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Select value", SELECT_VALUE_PROFILE, request)
    }

    fn start_content(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &SelectContentProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:select:{id}:content"))?,
            crate::InstanceId::new(format!("sailbreak:select:{id}:content-instance"))?,
            PrototypeKey::ShadcnSelectContent,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Select content"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Select content", SELECT_CONTENT_PROFILE, request)
    }

    fn start_item(
        &mut self,
        id: &str,
        label: &str,
        parent: &LogicalParentRef,
        props: &SelectItemProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:select:{id}:item"))?,
            crate::InstanceId::new(format!("sailbreak:select:{id}:item-instance"))?,
            PrototypeKey::ShadcnSelectItem,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter.start(id, label, SELECT_ITEM_PROFILE, request)
    }

    fn ensure_graph_open(&self) -> Result<()> {
        if self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "select graph is already set up".to_owned(),
            });
        }
        Ok(())
    }

    fn ensure_unique_id(&self, id: &str) -> Result<()> {
        if self.is_registered_id(id) {
            return Err(BridgeError::DuplicateSemanticId { id: id.to_owned() });
        }
        Ok(())
    }

    fn is_registered_id(&self, id: &str) -> bool {
        self.root.as_ref().is_some_and(|record| record.id == id)
            || self.trigger.as_ref().is_some_and(|record| record.id == id)
            || self.value.as_ref().is_some_and(|record| record.id == id)
            || self.content.as_ref().is_some_and(|record| record.id == id)
            || self.items.contains_key(id)
    }

    fn registered_ids(&self) -> Vec<String> {
        let mut ids = Vec::with_capacity(self.items.len() + 4);
        ids.extend(self.item_order.iter().cloned());
        if let Some(content) = &self.content {
            ids.push(content.id.clone());
        }
        if let Some(value) = &self.value {
            ids.push(value.id.clone());
        }
        if let Some(trigger) = &self.trigger {
            ids.push(trigger.id.clone());
        }
        if let Some(root) = &self.root {
            ids.push(root.id.clone());
        }
        ids
    }

    fn dispose_started_sessions(&mut self) {
        for id in self.registered_ids() {
            let _ = self.adapter.dispose(&id);
        }
        self.focus = FocusRegistry::new();
        self.overlay_lease = None;
        self.placement = None;
    }

    fn require_root_record(&self) -> Result<&SelectRootRecord> {
        self.root
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "select root is not registered".to_owned(),
            })
    }

    fn require_trigger_record(&self) -> Result<&SelectTriggerRecord> {
        self.trigger
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "select trigger is not registered".to_owned(),
            })
    }

    fn require_value_record(&self) -> Result<&SelectValueRecord> {
        self.value
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "select value is not registered".to_owned(),
            })
    }

    fn require_content_record(&self) -> Result<&SelectContentRecord> {
        self.content
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "select content is not registered".to_owned(),
            })
    }

    fn require_trigger(&self, id: &str) -> Result<&SelectTriggerRecord> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_select_trigger(id))
    }

    fn require_content(&self, id: &str) -> Result<&SelectContentRecord> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_select_content(id))
    }

    fn require_item(&self, id: &str) -> Result<&SelectItemRecord> {
        self.items.get(id).ok_or_else(|| unknown_select_item(id))
    }
}

impl SelectContentRecord {
    fn clone_for_overlay(&self) -> Self {
        Self {
            id: self.id.clone(),
            props: self.props.clone(),
            snapshot: None,
        }
    }
}

fn state_string(snapshot: &SessionSnapshot, key: &str) -> Option<String> {
    snapshot
        .state_values
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn state_bool(snapshot: &SessionSnapshot, key: &str) -> Option<bool> {
    snapshot.state_values.get(key).and_then(Value::as_bool)
}

fn value_placeholder(record: &SelectValueRecord, root_value: &str) -> String {
    if root_value.is_empty() {
        record.props.placeholder.clone()
    } else {
        root_value.to_owned()
    }
}

fn has_selected_indicator(nodes: &[crate::TemplateNode]) -> bool {
    nodes.iter().any(|node| match node {
        crate::TemplateNode::Svg {
            tag,
            attributes,
            children,
        } => {
            (tag == "path"
                && attributes
                    .get("d")
                    .is_some_and(|value| value == "m20 6-11 11-5-5"))
                || has_selected_indicator(children)
        }
        crate::TemplateNode::Container { children, .. } => has_selected_indicator(children),
        crate::TemplateNode::Text { .. } | crate::TemplateNode::Slot { .. } => false,
    })
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Top => "top",
        Side::Right => "right",
        Side::Bottom => "bottom",
        Side::Left => "left",
    }
}

fn align_name(align: SideAlign) -> &'static str {
    match align {
        SideAlign::Start => "start",
        SideAlign::Center => "center",
        SideAlign::End => "end",
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

fn unknown_select_member(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown select member: {id}"),
    }
}

fn unknown_select_root(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown select root: {id}"),
    }
}

fn unknown_select_trigger(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown select trigger: {id}"),
    }
}

fn unknown_select_value(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown select value: {id}"),
    }
}

fn unknown_select_content(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown select content: {id}"),
    }
}

fn unknown_select_item(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown select item: {id}"),
    }
}
