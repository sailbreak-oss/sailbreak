use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle, CloseReason,
    CommitDisposition, ConnectionRevision, FocusOperationResult, FocusRegistry, InputKind,
    InputSource, LayerRole, LogicalParentRef, NativeStyle, OverlayEvent, OverlayEventEnvelope,
    OverlayHost, OverlayLease, OverlayRect, OverlayRequest, OverlaySurfaceRef, PlacementPolicy,
    PlacementSnapshot, ProtoAdapter, PrototypeKey, PrototypeProfile, Result, SessionId,
    SessionSnapshot, ShadcnTheme, Side, SideAlign, SlotProjection, StartRequest, ViewEpoch,
};

const DROPDOWN_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDropdownRoot,
    exposed_states: &["open"],
    signals: &["openChange"],
};

const DROPDOWN_TRIGGER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDropdownTrigger,
    exposed_states: &["disabled", "hovered", "focused", "focusVisible", "pressed"],
    signals: &[],
};

const DROPDOWN_CONTENT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDropdownContent,
    exposed_states: &["open"],
    signals: &[],
};

const DROPDOWN_ITEM_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDropdownItem,
    exposed_states: &[
        "disabled",
        "hovered",
        "focused",
        "focusVisible",
        "pressed",
        "active",
    ],
    signals: &["select"],
};

/// Entry policy used when a Dropdown opens through keyboard or programmatic
/// interaction. The policy is interpreted by the Proto Dropdown Content
/// prototype; Rust only transports the value as bounded props.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DropdownOpenEntry {
    #[default]
    ActiveOrFirst,
    First,
    Last,
    ValueOrFirst,
}

impl DropdownOpenEntry {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActiveOrFirst => "active-or-first",
            Self::First => "first",
            Self::Last => "last",
            Self::ValueOrFirst => "value-or-first",
        }
    }
}

/// Indicator icon rendered by the Shadcn Dropdown Trigger prototype.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DropdownTriggerIndicatorIcon {
    #[default]
    ChevronDown,
    ChevronsUpDown,
}

impl DropdownTriggerIndicatorIcon {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChevronDown => "chevron-down",
            Self::ChevronsUpDown => "chevrons-up-down",
        }
    }
}

/// Visual variant used by a Dropdown Item.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DropdownItemVariant {
    #[default]
    Default,
    Destructive,
}

impl DropdownItemVariant {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropdownRootProps {
    pub open: Option<bool>,
    pub default_open: bool,
    pub disabled: bool,
    pub close_on_item_commit: bool,
    pub open_entry: DropdownOpenEntry,
    pub open_entry_value: String,
}

impl Default for DropdownRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            disabled: false,
            close_on_item_commit: true,
            open_entry: DropdownOpenEntry::ActiveOrFirst,
            open_entry_value: String::new(),
        }
    }
}

impl DropdownRootProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(open) = self.open {
            props.insert("open".to_owned(), Value::Bool(open));
        }
        props.insert("defaultOpen".to_owned(), Value::Bool(self.default_open));
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props.insert(
            "closeOnItemCommit".to_owned(),
            Value::Bool(self.close_on_item_commit),
        );
        props.insert(
            "openEntry".to_owned(),
            Value::String(self.open_entry.as_str().to_owned()),
        );
        props.insert(
            "openEntryValue".to_owned(),
            Value::String(self.open_entry_value.clone()),
        );
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownTriggerProps {
    pub disabled: bool,
    pub indicator: bool,
    pub indicator_icon: DropdownTriggerIndicatorIcon,
    pub indicator_size: f32,
    pub indicator_stroke_width: f32,
}

impl Default for DropdownTriggerProps {
    fn default() -> Self {
        Self {
            disabled: false,
            indicator: false,
            indicator_icon: DropdownTriggerIndicatorIcon::ChevronDown,
            indicator_size: 16.0,
            indicator_stroke_width: 2.0,
        }
    }
}

impl DropdownTriggerProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props.insert("indicator".to_owned(), Value::Bool(self.indicator));
        props.insert(
            "indicatorIcon".to_owned(),
            Value::String(self.indicator_icon.as_str().to_owned()),
        );
        props.insert("indicatorSize".to_owned(), Value::from(self.indicator_size));
        props.insert(
            "indicatorStrokeWidth".to_owned(),
            Value::from(self.indicator_stroke_width),
        );
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownContentProps {
    pub side: Side,
    pub align: SideAlign,
    pub side_offset: f32,
    pub align_offset: f32,
    pub avoid_collisions: bool,
    pub collision_padding: f32,
}

impl Default for DropdownContentProps {
    fn default() -> Self {
        Self {
            side: Side::Bottom,
            align: SideAlign::Center,
            side_offset: 4.0,
            align_offset: 0.0,
            avoid_collisions: true,
            collision_padding: 0.0,
        }
    }
}

impl DropdownContentProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
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
        PlacementPolicy::Popper {
            side: self.side,
            side_offset: self.side_offset,
            align: self.align,
            align_offset: self.align_offset,
            avoid_collisions: self.avoid_collisions,
            collision_padding: self.collision_padding,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropdownItemProps {
    pub value: String,
    pub text_value: String,
    pub disabled: bool,
    pub close_on_commit: Option<bool>,
    pub inset: bool,
    pub variant: DropdownItemVariant,
}

impl Default for DropdownItemProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            text_value: String::new(),
            disabled: false,
            close_on_commit: None,
            inset: false,
            variant: DropdownItemVariant::Default,
        }
    }
}

impl DropdownItemProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert("value".to_owned(), Value::String(self.value.clone()));
        props.insert(
            "textValue".to_owned(),
            Value::String(self.text_value.clone()),
        );
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        if let Some(close_on_commit) = self.close_on_commit {
            props.insert("closeOnCommit".to_owned(), Value::Bool(close_on_commit));
        }
        props.insert("inset".to_owned(), Value::Bool(self.inset));
        props.insert(
            "variant".to_owned(),
            Value::String(self.variant.as_str().to_owned()),
        );
        props
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownRootSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub open: bool,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownTriggerSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub disabled: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownContentSnapshot {
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
pub struct DropdownItemSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub value: String,
    pub text_value: String,
    pub disabled: bool,
    pub active: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoDropdownSnapshot {
    pub root: DropdownRootSnapshot,
    pub trigger: Option<DropdownTriggerSnapshot>,
    pub content: Option<DropdownContentSnapshot>,
    pub items: Vec<DropdownItemSnapshot>,
}

/// Alias for callers that use the family name without the host prefix.
pub type DropdownSnapshot = ProtoDropdownSnapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DropdownDispatchOutcome {
    pub open_change_count: usize,
    pub item_select_count: usize,
    pub selected_values: Vec<String>,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl DropdownDispatchOutcome {
    #[must_use]
    pub fn signal_count(&self, key: &str) -> usize {
        match key {
            "openChange" => self.open_change_count,
            "select" => self.item_select_count,
            _ => 0,
        }
    }

    #[must_use]
    pub fn selection_count(&self) -> usize {
        self.item_select_count
    }
}

struct DropdownRootRecord {
    id: String,
    label: String,
    props: DropdownRootProps,
    snapshot: Option<DropdownRootSnapshot>,
}

struct DropdownTriggerRecord {
    id: String,
    props: DropdownTriggerProps,
    snapshot: Option<DropdownTriggerSnapshot>,
}

struct DropdownContentRecord {
    id: String,
    props: DropdownContentProps,
    snapshot: Option<DropdownContentSnapshot>,
}

struct DropdownItemRecord {
    id: String,
    label: String,
    props: DropdownItemProps,
    snapshot: Option<DropdownItemSnapshot>,
}

/// Host-side adapter for a Shadcn Dropdown Root/Trigger/Content/Item family.
///
/// Registration records the complete logical graph before Runtime sessions are
/// started. Once [`Self::setup`] runs, each member remains an independent Proto
/// session linked through opaque parent and family route references. Proto owns
/// open, selection, and keyboard navigation semantics; this host only projects
/// snapshots, native focus targets, and the Rust-owned overlay lease.
pub struct ProtoDropdownHost {
    adapter: ProtoAdapter,
    root: Option<DropdownRootRecord>,
    trigger: Option<DropdownTriggerRecord>,
    content: Option<DropdownContentRecord>,
    items: BTreeMap<String, DropdownItemRecord>,
    item_order: Vec<String>,
    focus: FocusRegistry,
    overlay: OverlayHost,
    overlay_lease: Option<OverlayLease>,
    placement: Option<PlacementSnapshot>,
    family_route: String,
    setup_done: bool,
}

impl ProtoDropdownHost {
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
            content: None,
            items: BTreeMap::new(),
            item_order: Vec::new(),
            focus: FocusRegistry::new(),
            overlay: OverlayHost::new(64),
            overlay_lease: None,
            placement: None,
            family_route: format!("dropdown:{family}"),
            setup_done: false,
        })
    }

    /// Register the Dropdown Root. Runtime setup is deferred until every part
    /// of the family has been registered.
    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: DropdownRootProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "dropdown root")?;
        validate_identity(&label, "dropdown root accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.root.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dropdown root".to_owned(),
            });
        }
        self.root = Some(DropdownRootRecord {
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
        props: DropdownTriggerProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dropdown trigger")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dropdown root: {root_id}"),
            });
        }
        if self.trigger.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dropdown trigger".to_owned(),
            });
        }
        self.trigger = Some(DropdownTriggerRecord {
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
        props: DropdownContentProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dropdown content")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dropdown root: {root_id}"),
            });
        }
        if self.content.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dropdown content".to_owned(),
            });
        }
        self.content = Some(DropdownContentRecord {
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
        props: DropdownItemProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "dropdown item")?;
        validate_identity(&label, "dropdown item accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dropdown content: {content_id}"),
            });
        }
        if self.items.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate dropdown item: {id}"),
            });
        }
        self.item_order.push(id.clone());
        self.items.insert(
            id.clone(),
            DropdownItemRecord {
                id,
                label,
                props,
                snapshot: None,
            },
        );
        Ok(())
    }

    /// Start Root first, then all registered children using current opaque
    /// parent references.
    pub fn setup(&mut self) -> Result<()> {
        if self.setup_done {
            return Ok(());
        }
        if self.trigger.is_none() || self.content.is_none() || self.items.is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "dropdown graph requires trigger, content, and items".to_owned(),
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
        let (content_id, content_props) = {
            let content = self.require_content_record()?;
            (content.id.clone(), content.props.clone())
        };
        let items: Vec<_> = self
            .item_order
            .iter()
            .map(|id| {
                let item = self.items.get(id).expect("registered dropdown item");
                (item.id.clone(), item.label.clone(), item.props.clone())
            })
            .collect();

        let started = (|| -> Result<()> {
            self.start_root(&root_id, &root_label, &root_props)?;
            let root_parent = self.parent_ref(&root_id)?;
            self.start_trigger(&trigger_id, &root_parent, &trigger_props)?;
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

    /// Read the complete Dropdown family snapshot and reconcile the Rust
    /// overlay lease with the Proto-owned open state.
    pub fn snapshot(&mut self) -> Result<ProtoDropdownSnapshot> {
        if !self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "dropdown graph is not set up".to_owned(),
            });
        }
        let theme = self.adapter.theme();
        let (root_id, root_label, root_props) = {
            let root = self.require_root_record()?;
            (root.id.clone(), root.label.clone(), root.props.clone())
        };
        let root_adapter = self.adapter.snapshot(&root_id)?;
        let root_session = root_adapter.session;
        let root_disabled = state_bool(&root_session, "disabled").unwrap_or(root_props.disabled);

        let trigger_snapshot = if let Some(record) = self.trigger.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let disabled =
                state_bool(&session, "disabled").unwrap_or(record.props.disabled || root_disabled);
            let focused = state_bool(&session, "focused").unwrap_or(false);
            let focus_visible = state_bool(&session, "focusVisible").unwrap_or(false);
            let snapshot = DropdownTriggerSnapshot {
                id: record.id.clone(),
                label: root_label.clone(),
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                native_style: adapter.native_style,
                open: false,
                disabled,
                focused,
                focus_visible,
                a11y: session.a11y.clone(),
                session,
            };
            Some(snapshot)
        } else {
            None
        };

        let (content_snapshot, content_open, content_epoch) =
            if let Some(record) = self.content.as_ref() {
                let adapter = self.adapter.snapshot(&record.id)?;
                let session = adapter.session;
                let open = state_bool(&session, "open")
                    .unwrap_or_else(|| root_props.open.unwrap_or(root_props.default_open));
                let epoch = session.projection.view_epoch;
                let snapshot = DropdownContentSnapshot {
                    id: record.id.clone(),
                    resolved_style: ButtonStyle::from_projection(&session.style, theme),
                    native_style: adapter.native_style,
                    open,
                    present: open,
                    placement: self.placement.clone(),
                    portal_lease_id: self.overlay_lease.as_ref().map(OverlayLease::id),
                    a11y: session.a11y.clone(),
                    session,
                };
                (Some(snapshot), open, Some(epoch))
            } else {
                (None, false, None)
            };

        if let Some(epoch) = content_epoch {
            self.sync_overlay(content_open, epoch)?;
        }

        let root_open = content_snapshot.as_ref().map_or_else(
            || {
                state_bool(&root_session, "open")
                    .unwrap_or(root_props.open.unwrap_or(root_props.default_open))
            },
            |_| content_open,
        );
        let root_snapshot = DropdownRootSnapshot {
            id: root_id,
            label: root_label,
            session: root_session,
            native_style: root_adapter.native_style,
            open: root_open,
            disabled: root_disabled,
        };
        self.root
            .as_mut()
            .expect("registered dropdown root")
            .snapshot = Some(root_snapshot.clone());

        let trigger = trigger_snapshot.map(|mut snapshot| {
            snapshot.open = root_open;
            self.trigger
                .as_mut()
                .expect("registered dropdown trigger")
                .snapshot = Some(snapshot.clone());
            snapshot
        });

        let mut items = Vec::with_capacity(self.item_order.len());
        for id in self.item_order.clone() {
            let record = self.items.get(&id).expect("registered dropdown item");
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let disabled =
                state_bool(&session, "disabled").unwrap_or(record.props.disabled || root_disabled);
            let active = state_bool(&session, "active").unwrap_or(false);
            let focused = state_bool(&session, "focused").unwrap_or(false);
            let focus_visible = state_bool(&session, "focusVisible").unwrap_or(false);
            let text_value = if record.props.text_value.is_empty() {
                record.props.value.clone()
            } else {
                record.props.text_value.clone()
            };
            let snapshot = DropdownItemSnapshot {
                id: id.clone(),
                label: record.label.clone(),
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                native_style: adapter.native_style,
                value: record.props.value.clone(),
                text_value,
                disabled,
                active,
                focused,
                focus_visible,
                a11y: session.a11y.clone(),
                session,
            };
            self.items
                .get_mut(&id)
                .expect("registered dropdown item")
                .snapshot = Some(snapshot.clone());
            items.push(snapshot);
        }

        // Re-read Content after lease reconciliation so its portal id and
        // placement always describe the current Rust lease.
        let content = if content_snapshot.is_some() {
            let id = self.require_content_record()?.id.clone();
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let refreshed = DropdownContentSnapshot {
                id,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                native_style: adapter.native_style,
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.overlay_lease.as_ref().map(OverlayLease::id),
                a11y: session.a11y.clone(),
                session,
            };
            self.content
                .as_mut()
                .expect("registered dropdown content")
                .snapshot = Some(refreshed.clone());
            Some(refreshed)
        } else {
            None
        };

        Ok(ProtoDropdownSnapshot {
            root: root_snapshot,
            trigger,
            content,
            items,
        })
    }

    pub fn root(&self) -> Result<&DropdownRootSnapshot> {
        self.root
            .as_ref()
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dropdown_root("root"))
    }

    pub fn trigger(&self, id: &str) -> Result<&DropdownTriggerSnapshot> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dropdown_trigger(id))
    }

    pub fn content(&self, id: &str) -> Result<&DropdownContentSnapshot> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dropdown_content(id))
    }

    pub fn item(&self, id: &str) -> Result<&DropdownItemSnapshot> {
        self.items
            .get(id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dropdown_item(id))
    }

    pub fn is_open(&mut self) -> Result<bool> {
        Ok(self.snapshot()?.root.open)
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

    /// Dispatch an input to any registered Dropdown member. Proto owns all
    /// open, focus, and item-selection semantics.
    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DropdownDispatchOutcome> {
        if !self.is_registered_id(id) {
            return Err(unknown_dropdown_member(id));
        }
        if self.items.contains_key(id) && kind == InputKind::PressCommit && !self.is_open()? {
            return Ok(DropdownDispatchOutcome::default());
        }
        self.dispatch_inner(
            id,
            kind,
            source,
            detail,
            self.items.contains_key(id).then_some(id),
        )
    }

    pub fn dispatch_trigger(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DropdownDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch_inner(id, kind, source, detail, None)
    }

    pub fn dispatch_content(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DropdownDispatchOutcome> {
        self.require_content(id)?;
        self.dispatch_inner(id, kind, source, detail, None)
    }

    pub fn dispatch_item(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DropdownDispatchOutcome> {
        self.require_item(id)?;
        if kind == InputKind::PressCommit && !self.is_open()? {
            return Ok(DropdownDispatchOutcome::default());
        }
        self.dispatch_inner(id, kind, source, detail, Some(id))
    }

    /// Dispatch a normalized keyboard key to Trigger while closed and Content
    /// while open. Navigation remains entirely in the Proto runtime.
    pub fn dispatch_key(&mut self, key: &str) -> Result<DropdownDispatchOutcome> {
        if key == "Escape" && self.is_open()? {
            return self.dismiss_escape();
        }
        let open = self.is_open()?;
        let id = if !open && matches!(key, "ArrowDown" | "ArrowUp") {
            self.require_trigger_record()?.id.clone()
        } else {
            self.require_content_record()?.id.clone()
        };
        self.dispatch_inner(
            &id,
            InputKind::KeyDown,
            InputSource::Keyboard,
            Some(serde_json::json!({ "key": key })),
            None,
        )
    }

    pub fn press_commit(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<DropdownDispatchOutcome> {
        if self.items.contains_key(id) {
            self.press_item(id, source)
        } else {
            self.press_trigger(id, source)
        }
    }

    pub fn press_item(&mut self, id: &str, source: InputSource) -> Result<DropdownDispatchOutcome> {
        self.require_item(id)?;
        if !self.is_open()? {
            return Ok(DropdownDispatchOutcome::default());
        }
        self.dispatch_inner(id, InputKind::PressCommit, source, None, Some(id))
    }

    pub fn press_trigger(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<DropdownDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch_inner(id, InputKind::PressCommit, source, None, None)
    }

    /// Request opening through the Trigger's Proto event route.
    pub fn open(&mut self) -> Result<DropdownDispatchOutcome> {
        if self.is_open()? {
            return Ok(DropdownDispatchOutcome::default());
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

    /// Close through the semantic Trigger route and close the Rust lease with
    /// the supplied dismissal reason.
    pub fn close(&mut self, reason: CloseReason) -> Result<DropdownDispatchOutcome> {
        if !self.is_open()? {
            return Ok(DropdownDispatchOutcome::default());
        }
        if self.require_root_record()?.props.open.is_none()
            && let Some(lease) = self.overlay_lease.as_ref()
        {
            lease.close(reason)?;
        }
        let trigger_id = self.require_trigger_record()?.id.clone();
        let outcome = self.dispatch_inner(
            &trigger_id,
            InputKind::PressCommit,
            InputSource::Programmatic,
            None,
            None,
        )?;
        if matches!(
            reason,
            CloseReason::OutsidePress | CloseReason::FocusOutside | CloseReason::Escape
        ) {
            let _ = self.focus_with_source(&trigger_id, InputSource::Programmatic);
        }
        Ok(outcome)
    }

    pub fn dismiss_escape(&mut self) -> Result<DropdownDispatchOutcome> {
        self.close(CloseReason::Escape)
    }

    pub fn dismiss_outside(&mut self) -> Result<DropdownDispatchOutcome> {
        self.close(CloseReason::OutsidePress)
    }

    pub fn set_root_props(&mut self, props: DropdownRootProps) -> Result<CommitDisposition> {
        let id = self.require_root_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.root.as_mut().expect("registered dropdown root").props = props;
        Ok(disposition)
    }

    pub fn set_trigger_props(&mut self, props: DropdownTriggerProps) -> Result<CommitDisposition> {
        let id = self.require_trigger_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.trigger
            .as_mut()
            .expect("registered dropdown trigger")
            .props = props;
        Ok(disposition)
    }

    pub fn set_content_props(&mut self, props: DropdownContentProps) -> Result<CommitDisposition> {
        let id = self.require_content_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.content
            .as_mut()
            .expect("registered dropdown content")
            .props = props;
        if let Some(lease) = self.overlay_lease.take() {
            lease.dispose();
        }
        self.placement = None;
        Ok(disposition)
    }

    pub fn set_item_props(
        &mut self,
        id: &str,
        props: DropdownItemProps,
    ) -> Result<CommitDisposition> {
        self.require_item(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.items
            .get_mut(id)
            .expect("registered dropdown item")
            .props = props;
        Ok(disposition)
    }

    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        if !self.is_registered_id(id) {
            return Err(unknown_dropdown_member(id));
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

    pub fn blur(&mut self, id: &str, source: InputSource) -> Result<DropdownDispatchOutcome> {
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
            role: "dropdown-trigger".to_owned(),
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

    /// Dispose every Runtime session and the overlay lease. Repeated disposal
    /// is intentionally a no-op so a dropped native subtree cannot resurrect a
    /// stale portal or focus target.
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
    ) -> Result<DropdownDispatchOutcome> {
        let mut outcome = self.adapter.dispatch(id, kind, source, detail)?;
        for sibling in self.registered_ids() {
            if sibling != id {
                outcome.absorb(self.adapter.drain(&sibling)?);
            }
        }
        let selected_values = if outcome.signal_count("select") == 0 {
            Vec::new()
        } else if let Some(item_id) = item_id {
            vec![self.require_item(item_id)?.props.value.clone()]
        } else {
            Vec::new()
        };
        let result = DropdownDispatchOutcome {
            open_change_count: outcome.signal_count("openChange"),
            item_select_count: outcome.signal_count("select"),
            selected_values,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        };
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
            crate::AnchorRef::new(format!("{}:anchor:{}", self.family_route, trigger_id))?,
            OverlaySurfaceRef::new(format!("{}:surface:{}", self.family_route, content.id))?,
            epoch,
            LayerRole::DropdownContent,
            content.props.placement_policy(),
            Default::default(),
        )?
        .with_focus_restore_target(trigger_id);
        self.overlay_lease = Some(self.overlay.attach(request)?);
        Ok(())
    }

    fn start_root(&mut self, id: &str, label: &str, props: &DropdownRootProps) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dropdown:{id}:root"))?,
            crate::InstanceId::new(format!("sailbreak:dropdown:{id}:root-instance"))?,
            PrototypeKey::ShadcnDropdownRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route);
        self.adapter
            .start(id, label, DROPDOWN_ROOT_PROFILE, request)
    }

    fn start_trigger(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DropdownTriggerProps,
    ) -> Result<()> {
        let label = self.require_root_record()?.label.clone();
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dropdown:{id}:trigger"))?,
            crate::InstanceId::new(format!("sailbreak:dropdown:{id}:trigger-instance"))?,
            PrototypeKey::ShadcnDropdownTrigger,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.clone()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dropdown trigger", DROPDOWN_TRIGGER_PROFILE, request)
    }

    fn start_content(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DropdownContentProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dropdown:{id}:content"))?,
            crate::InstanceId::new(format!("sailbreak:dropdown:{id}:content-instance"))?,
            PrototypeKey::ShadcnDropdownContent,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Dropdown content"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dropdown content", DROPDOWN_CONTENT_PROFILE, request)
    }

    fn start_item(
        &mut self,
        id: &str,
        label: &str,
        parent: &LogicalParentRef,
        props: &DropdownItemProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dropdown:{id}:item"))?,
            crate::InstanceId::new(format!("sailbreak:dropdown:{id}:item-instance"))?,
            PrototypeKey::ShadcnDropdownItem,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, label, DROPDOWN_ITEM_PROFILE, request)
    }

    fn refresh_cached_content_placement(&mut self, placement: PlacementSnapshot) {
        if let Some(record) = self.content.as_mut()
            && let Some(snapshot) = record.snapshot.as_mut()
        {
            snapshot.placement = Some(placement);
        }
    }

    fn ensure_graph_open(&self) -> Result<()> {
        if self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "dropdown graph is already set up".to_owned(),
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
            || self.content.as_ref().is_some_and(|record| record.id == id)
            || self.items.contains_key(id)
    }

    fn registered_ids(&self) -> Vec<String> {
        let mut ids = Vec::with_capacity(self.items.len() + 3);
        ids.extend(self.item_order.iter().cloned());
        if let Some(content) = &self.content {
            ids.push(content.id.clone());
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

    fn require_root_record(&self) -> Result<&DropdownRootRecord> {
        self.root
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dropdown root is not registered".to_owned(),
            })
    }

    fn require_trigger_record(&self) -> Result<&DropdownTriggerRecord> {
        self.trigger
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dropdown trigger is not registered".to_owned(),
            })
    }

    fn require_content_record(&self) -> Result<&DropdownContentRecord> {
        self.content
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dropdown content is not registered".to_owned(),
            })
    }

    fn require_trigger(&self, id: &str) -> Result<&DropdownTriggerRecord> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dropdown_trigger(id))
    }

    fn require_content(&self, id: &str) -> Result<&DropdownContentRecord> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dropdown_content(id))
    }

    fn require_item(&self, id: &str) -> Result<&DropdownItemRecord> {
        self.items.get(id).ok_or_else(|| unknown_dropdown_item(id))
    }
}

impl DropdownContentRecord {
    fn clone_for_overlay(&self) -> Self {
        Self {
            id: self.id.clone(),
            props: self.props.clone(),
            snapshot: None,
        }
    }
}

fn state_bool(snapshot: &SessionSnapshot, key: &str) -> Option<bool> {
    snapshot.state_values.get(key).and_then(Value::as_bool)
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

fn unknown_dropdown_member(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dropdown member: {id}"),
    }
}

fn unknown_dropdown_root(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dropdown root: {id}"),
    }
}

fn unknown_dropdown_trigger(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dropdown trigger: {id}"),
    }
}

fn unknown_dropdown_content(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dropdown content: {id}"),
    }
}

fn unknown_dropdown_item(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dropdown item: {id}"),
    }
}
