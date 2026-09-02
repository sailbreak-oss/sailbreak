use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AdapterDispatchOutcome, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle,
    CommitDisposition, FocusOperationResult, FocusRegistry, InputKind, InputSource,
    LogicalParentRef, NativeStyle, ProtoAdapter, PrototypeKey, PrototypeProfile, Result, SessionId,
    SessionSnapshot, ShadcnTheme, SlotProjection, StartRequest, ViewEpoch,
};

// ── Profiles ───────────────────────────────────────────────────────────────────

const TABS_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnTabsRoot,
    exposed_states: &["value"],
    signals: &["valueChange"],
};

const TABS_LIST_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnTabsList,
    exposed_states: &[],
    signals: &[],
};

const TABS_TRIGGER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnTabsTrigger,
    exposed_states: &[
        "disabled",
        "hovered",
        "focused",
        "focusVisible",
        "pressed",
        "selected",
    ],
    signals: &["click"],
};

const TABS_CONTENT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnTabsContent,
    exposed_states: &["current", "hidden"],
    signals: &[],
};

// ── Props ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl TabsOrientation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsActivationMode {
    #[default]
    Automatic,
    Manual,
}

impl TabsActivationMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabsRootProps {
    pub value: Option<String>,
    pub default_value: String,
    pub orientation: TabsOrientation,
    pub activation_mode: TabsActivationMode,
}

impl Default for TabsRootProps {
    fn default() -> Self {
        Self {
            value: None,
            default_value: String::new(),
            orientation: TabsOrientation::Horizontal,
            activation_mode: TabsActivationMode::Automatic,
        }
    }
}

impl TabsRootProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(value) = &self.value {
            props.insert("value".to_owned(), Value::String(value.clone()));
        }
        props.insert(
            "defaultValue".to_owned(),
            Value::String(self.default_value.clone()),
        );
        props.insert(
            "orientation".to_owned(),
            Value::String(self.orientation.as_str().to_owned()),
        );
        props.insert(
            "activationMode".to_owned(),
            Value::String(self.activation_mode.as_str().to_owned()),
        );
        props
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TabsListProps {
    pub orientation: Option<TabsOrientation>,
    pub loop_navigation: bool,
    pub a11y_label: String,
}

impl TabsListProps {
    #[must_use]
    pub fn with_loop(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(orientation) = &self.orientation {
            props.insert(
                "orientation".to_owned(),
                Value::String(orientation.as_str().to_owned()),
            );
        }
        props.insert("loop".to_owned(), Value::Bool(self.loop_navigation));
        if !self.a11y_label.is_empty() {
            props.insert(
                "a11yLabel".to_owned(),
                Value::String(self.a11y_label.clone()),
            );
        }
        props
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TabsTriggerProps {
    pub value: String,
    pub disabled: bool,
}

impl TabsTriggerProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert("value".to_owned(), Value::String(self.value.clone()));
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TabsContentProps {
    pub value: String,
    pub keep_mounted: bool,
}

impl TabsContentProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        props.insert("value".to_owned(), Value::String(self.value.clone()));
        props.insert("keepMounted".to_owned(), Value::Bool(self.keep_mounted));
        props
    }
}

// ── Snapshots ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct TabsRootSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabsListSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub a11y: Option<A11ySnapshot>,
    pub loop_navigation: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabsTriggerSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub value: String,
    pub disabled: bool,
    pub selected: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
    /// Opaque tabpanel id this trigger controls (host-side stable key).
    pub controls: String,
    /// Opaque tab id for the tabpanel `labelledBy` relation (host-side stable key).
    pub tab_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabsContentSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub value: String,
    /// True when this panel is mounted (selected and not hidden).
    pub present: bool,
    pub a11y: Option<A11ySnapshot>,
    pub labelled_by: String,
    pub tabpanel_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabsSnapshot {
    pub root: Option<TabsRootSnapshot>,
    pub list: TabsListSnapshot,
    pub triggers: Vec<TabsTriggerSnapshot>,
    pub contents: Vec<TabsContentSnapshot>,
}

// ── Dispatch outcome ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TabsDispatchOutcome {
    pub click_count: usize,
    pub value_change_count: usize,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

// ── Host ───────────────────────────────────────────────────────────────────────

/// Host-side adapter for a Shadcn Tabs Root/List/Trigger/Content family.
///
/// All four members are independent Proto sessions linked through opaque
/// parent/route refs. The complete logical graph is established during
/// registration (before any native focus is claimed); `setup()` then publishes
/// the initial Tabs context.
///
/// Focus operations return [`FocusOperationResult`]. Allocating a native
/// `FocusHandle` is not treated as readiness: a target is only `Accepted` after
/// [`ProtoTabsHost::set_focus_ready`] has been called for it.
pub struct ProtoTabsHost {
    adapter: ProtoAdapter,
    root: Option<TabsRootRecord>,
    list: Option<TabsListRecord>,
    triggers: BTreeMap<String, TabsTriggerRecord>,
    trigger_order: Vec<String>,
    contents: BTreeMap<String, TabsContentRecord>,
    content_order: Vec<String>,
    focus: FocusRegistry,
    family_route: String,
    setup_done: bool,
}

struct TabsRootRecord {
    id: String,
    label: String,
    props: TabsRootProps,
}

struct TabsListRecord {
    id: String,
    props: TabsListProps,
}

struct TabsTriggerRecord {
    label: String,
    props: TabsTriggerProps,
    snapshot: Option<TabsTriggerSnapshot>,
}

struct TabsContentRecord {
    props: TabsContentProps,
    snapshot: Option<TabsContentSnapshot>,
}

impl ProtoTabsHost {
    pub fn new() -> Result<Self> {
        Self::with_theme(ShadcnTheme::default())
    }

    pub fn with_theme(theme: ShadcnTheme) -> Result<Self> {
        static NEXT_FAMILY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let family = NEXT_FAMILY.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let family_route = format!("tabs:{family}");
        Ok(Self {
            adapter: ProtoAdapter::with_theme(theme)?,
            root: None,
            list: None,
            triggers: BTreeMap::new(),
            trigger_order: Vec::new(),
            contents: BTreeMap::new(),
            content_order: Vec::new(),
            focus: FocusRegistry::new(),
            family_route,
            setup_done: false,
        })
    }

    /// Register the Tabs Root in the logical graph. Runtime sessions are
    /// created only by [`Self::setup`] after every part is known.
    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: TabsRootProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "tabs root")?;
        validate_identity(&label, "tabs root accessible name")?;
        if self.root.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate tabs root".to_owned(),
            });
        }
        self.root = Some(TabsRootRecord { id, label, props });
        Ok(())
    }

    pub fn register_list(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: TabsListProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "tabs list")?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown tabs root: {root_id}"),
            });
        }
        if self.list.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate tabs list".to_owned(),
            });
        }
        self.list = Some(TabsListRecord { id, props });
        Ok(())
    }

    pub fn register_trigger(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        list_id: &str,
        props: TabsTriggerProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "tabs trigger")?;
        validate_identity(&label, "tabs trigger accessible name")?;
        if self.require_list_record()?.id != list_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown tabs list: {list_id}"),
            });
        }
        if self.triggers.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate tabs trigger: {id}"),
            });
        }
        self.trigger_order.push(id.clone());
        self.triggers.insert(
            id,
            TabsTriggerRecord {
                label,
                props,
                snapshot: None,
            },
        );
        Ok(())
    }

    pub fn register_content(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: TabsContentProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "tabs content")?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown tabs root: {root_id}"),
            });
        }
        if self.contents.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate tabs content: {id}"),
            });
        }
        self.content_order.push(id.clone());
        self.contents.insert(
            id,
            TabsContentRecord {
                props,
                snapshot: None,
            },
        );
        Ok(())
    }

    /// Create Runtime sessions after the complete logical graph exists.
    pub fn setup(&mut self) -> Result<()> {
        if self.setup_done {
            return Ok(());
        }
        if self.triggers.is_empty() || self.contents.is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "tabs graph requires triggers and contents".to_owned(),
            });
        }
        let (root_id, root_label, root_props) = {
            let root = self.require_root_record()?;
            (root.id.clone(), root.label.clone(), root.props.clone())
        };
        let (list_id, list_props) = {
            let list = self.require_list_record()?;
            (list.id.clone(), list.props.clone())
        };
        let triggers: Vec<_> = self
            .trigger_order
            .iter()
            .map(|id| {
                let record = self.triggers.get(id).expect("registered trigger");
                (id.clone(), record.label.clone(), record.props.clone())
            })
            .collect();
        let contents: Vec<_> = self
            .content_order
            .iter()
            .map(|id| {
                let record = self.contents.get(id).expect("registered content");
                (id.clone(), record.props.clone())
            })
            .collect();

        let started = (|| -> Result<()> {
            self.start_root(&root_id, &root_label, &root_props)?;
            let root_parent = self.parent_ref(&root_id)?;
            self.start_list(&list_id, &root_parent, &list_props)?;
            let list_parent = self.parent_ref(&list_id)?;
            for (id, label, props) in &triggers {
                self.start_trigger(id, label, &list_parent, props)?;
            }
            for (id, props) in &contents {
                self.start_content(id, &root_parent, props)?;
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

    /// Read the full family snapshot.
    pub fn snapshot(&mut self) -> Result<TabsSnapshot> {
        if !self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "tabs graph is not set up".to_owned(),
            });
        }
        let (root_id, root_props) = {
            let root = self.require_root_record()?;
            (root.id.clone(), root.props.clone())
        };
        let (list_id, loop_navigation) = {
            let list = self.require_list_record()?;
            (list.id.clone(), list.props.loop_navigation)
        };

        let root_adapter = self.adapter.snapshot(&root_id)?;
        let root_session = root_adapter.session;
        let root_value = root_session
            .state_values
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| {
                root_props
                    .value
                    .clone()
                    .unwrap_or_else(|| root_props.default_value.clone())
            });
        let root_style = ButtonStyle::from_projection(&root_session.style, self.adapter.theme());

        let list_adapter = self.adapter.snapshot(&list_id)?;
        let list_a11y = list_adapter.session.a11y.clone();

        let trigger_ids = self.trigger_order.clone();
        let mut triggers = Vec::with_capacity(trigger_ids.len());
        for id in &trigger_ids {
            triggers.push(self.snapshot_trigger(id, &root_value)?);
        }

        let content_ids = self.content_order.clone();
        let mut contents = Vec::with_capacity(content_ids.len());
        for id in &content_ids {
            contents.push(self.snapshot_content(id, &root_value)?);
        }

        Ok(TabsSnapshot {
            root: Some(TabsRootSnapshot {
                id: root_id,
                session: root_session,
                native_style: root_adapter.native_style,
                resolved_style: root_style,
                value: root_value,
            }),
            list: TabsListSnapshot {
                id: list_id,
                session: list_adapter.session,
                native_style: list_adapter.native_style,
                a11y: list_a11y,
                loop_navigation,
            },
            triggers,
            contents,
        })
    }

    fn snapshot_trigger(&mut self, id: &str, root_value: &str) -> Result<TabsTriggerSnapshot> {
        let adapter = self.adapter.snapshot(id)?;
        let session = adapter.session;
        let style = ButtonStyle::from_projection(&session.style, self.adapter.theme());
        let (label, value, props_disabled) = {
            let trigger = self.require_trigger(id)?;
            (
                trigger.label.clone(),
                trigger.props.value.clone(),
                trigger.props.disabled,
            )
        };
        let disabled = session
            .state_values
            .get("disabled")
            .and_then(Value::as_bool)
            .unwrap_or(props_disabled);
        let selected = session
            .state_values
            .get("selected")
            .and_then(Value::as_bool)
            .unwrap_or(value == root_value);
        let focused = session
            .state_values
            .get("focused")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let focus_visible = session
            .state_values
            .get("focusVisible")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let a11y = session.a11y.clone();
        let tab_id = format!("{}:tab:{value}", self.family_route);
        let controls = format!("{}:panel:{value}", self.family_route);
        let snapshot = TabsTriggerSnapshot {
            id: id.to_owned(),
            label,
            session,
            native_style: adapter.native_style,
            resolved_style: style,
            value,
            disabled,
            selected,
            focused,
            focus_visible,
            a11y,
            controls,
            tab_id,
        };
        self.triggers
            .get_mut(id)
            .ok_or_else(|| unknown_trigger(id))?
            .snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    fn snapshot_content(&mut self, id: &str, root_value: &str) -> Result<TabsContentSnapshot> {
        let adapter = self.adapter.snapshot(id)?;
        let session = adapter.session;
        let value = self
            .contents
            .get(id)
            .ok_or_else(|| unknown_content(id))?
            .props
            .value
            .clone();
        let current = session
            .state_values
            .get("current")
            .and_then(Value::as_bool)
            .unwrap_or(value == root_value);
        let hidden = session
            .state_values
            .get("hidden")
            .and_then(Value::as_bool)
            .unwrap_or(value != root_value);
        let a11y = session.a11y.clone();
        let labelled_by = format!("{}:tab:{value}", self.family_route);
        let tabpanel_id = format!("{}:panel:{value}", self.family_route);
        let snapshot = TabsContentSnapshot {
            id: id.to_owned(),
            session,
            native_style: adapter.native_style,
            value,
            present: current && !hidden,
            a11y,
            labelled_by,
            tabpanel_id,
        };
        self.contents
            .get_mut(id)
            .ok_or_else(|| unknown_content(id))?
            .snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    /// Read a cached trigger snapshot.
    pub fn trigger(&self, id: &str) -> Result<&TabsTriggerSnapshot> {
        self.triggers
            .get(id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_trigger(id))
    }

    /// Read a cached content snapshot.
    pub fn content(&self, id: &str) -> Result<&TabsContentSnapshot> {
        self.contents
            .get(id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_content(id))
    }

    /// The currently selected value (Tabs Root `value`).
    pub fn selected_value(&mut self) -> Result<String> {
        Ok(self
            .snapshot()?
            .root
            .map(|root| root.value)
            .unwrap_or_default())
    }

    /// The currently active (focused, not necessarily selected) value.
    pub fn active_value(&mut self) -> Result<String> {
        let snapshot = self.snapshot()?;
        Ok(snapshot
            .triggers
            .iter()
            .find(|trigger| trigger.focused)
            .map(|trigger| trigger.value.clone())
            .unwrap_or_default())
    }

    /// Dispatch a keyboard navigation key to the Tabs List session.
    pub fn dispatch_key(&mut self, key: &str) -> Result<TabsDispatchOutcome> {
        let list_id = self.require_list_record()?.id.clone();
        let detail = serde_json::json!({ "key": key });
        let outcome = self.adapter.dispatch(
            &list_id,
            InputKind::KeyDown,
            InputSource::Keyboard,
            Some(detail),
        )?;
        Ok(outcome.into())
    }

    /// Dispatch a `PressCommit` activation to a trigger.
    pub fn press_commit(&mut self, id: &str, source: InputSource) -> Result<TabsDispatchOutcome> {
        self.require_trigger(id)?;
        Ok(self
            .adapter
            .dispatch(id, InputKind::PressCommit, source, None)?
            .into())
    }

    pub fn dispatch_trigger(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<TabsDispatchOutcome> {
        self.require_trigger(id)?;
        Ok(self.adapter.dispatch(id, kind, source, detail)?.into())
    }

    /// Mark a trigger's native surface as ready to receive focus.
    ///
    /// This is not the same as granting focus: readiness tracks the native
    /// surface (for example a `FocusHandle`) and is cleared by a remount.
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

    /// Request native focus for a trigger using keyboard modality.
    pub fn focus(&mut self, id: &str) -> Result<FocusOperationResult> {
        self.focus_with_source(id, InputSource::Keyboard)
    }

    pub fn focus_with_source(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<FocusOperationResult> {
        if self.require_trigger(id).is_err() {
            return Ok(FocusOperationResult::Rejected);
        }
        let Some(target) = self.focus.get(id) else {
            return Ok(FocusOperationResult::NotReady);
        };
        let current = self.focus_target(id)?;
        if current.view_epoch != target.view_epoch || current.instance_id != target.instance_id {
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
        Ok(FocusOperationResult::Accepted)
    }

    pub fn blur(&mut self, id: &str, source: InputSource) -> Result<TabsDispatchOutcome> {
        self.require_trigger(id)?;
        Ok(self
            .adapter
            .dispatch(id, InputKind::Blur, source, None)?
            .into())
    }

    /// The current focus target identity for a trigger.
    pub fn focus_target(&self, id: &str) -> Result<crate::FocusTarget> {
        self.require_trigger(id)?;
        let snapshot = self.adapter.snapshot_current(id)?;
        Ok(crate::FocusTarget {
            session_id: snapshot.session.session_id.as_str().to_owned(),
            instance_id: snapshot.session.instance_id.as_str().to_owned(),
            view_epoch: snapshot.session.projection.view_epoch,
            route_ref: self.family_route.clone(),
            role: "tab".to_owned(),
        })
    }

    /// Focus against an explicit target identity (stale-target testing).
    pub fn focus_with_target(
        &mut self,
        target: crate::FocusTarget,
    ) -> Result<FocusOperationResult> {
        let trigger_id = self.triggers.keys().find_map(|id| {
            self.adapter
                .snapshot_current(id)
                .ok()
                .filter(|snapshot| snapshot.session.session_id.as_str() == target.session_id)
                .map(|_| id.clone())
        });
        let Some(trigger_id) = trigger_id else {
            return Ok(FocusOperationResult::Rejected);
        };
        let current = self.focus_target(&trigger_id)?;
        if current != target {
            return Ok(FocusOperationResult::Rejected);
        }
        self.focus(&trigger_id)
    }

    /// Remount a trigger session, advancing its view epoch and clearing readiness.
    pub fn remount_trigger(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_trigger(id)?;
        let epoch = self.adapter.remount(id)?;
        self.focus.remove(id);
        Ok(epoch)
    }

    /// Update root props after setup.
    pub fn set_root_props(&mut self, props: TabsRootProps) -> Result<CommitDisposition> {
        let root_id = self.require_root_record()?.id.clone();
        let disposition = self.adapter.set_props(&root_id, props.to_map())?;
        self.root.as_mut().expect("registered root").props = props;
        Ok(disposition)
    }

    /// Update list props after setup.
    pub fn set_list_props(&mut self, props: TabsListProps) -> Result<CommitDisposition> {
        let list_id = self.require_list_record()?.id.clone();
        let disposition = self.adapter.set_props(&list_id, props.to_map())?;
        self.list.as_mut().expect("registered list").props = props;
        Ok(disposition)
    }

    /// The current opaque parent reference for a family member.
    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        self.adapter.parent_ref(id)
    }

    /// Dispose the whole family (children first).
    pub fn dispose(&mut self) -> Result<()> {
        if self.setup_done {
            for id in self.registered_ids() {
                self.adapter.dispose(&id)?;
            }
        }
        self.triggers.clear();
        self.contents.clear();
        self.trigger_order.clear();
        self.content_order.clear();
        self.list = None;
        self.root = None;
        self.focus = FocusRegistry::new();
        self.setup_done = false;
        Ok(())
    }

    // ── session start helpers ──────────────────────────────────────────────────

    fn start_root(&mut self, id: &str, label: &str, props: &TabsRootProps) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:tabs:{id}:root"))?,
            crate::InstanceId::new(format!("sailbreak:tabs:{id}:root-instance"))?,
            PrototypeKey::ShadcnTabsRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route);
        self.adapter.start(id, label, TABS_ROOT_PROFILE, request)
    }

    fn start_list(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &TabsListProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:tabs:{id}:list"))?,
            crate::InstanceId::new(format!("sailbreak:tabs:{id}:list-instance"))?,
            PrototypeKey::ShadcnTabsList,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Tabs list"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Tabs list", TABS_LIST_PROFILE, request)
    }

    fn start_trigger(
        &mut self,
        id: &str,
        label: &str,
        parent: &LogicalParentRef,
        props: &TabsTriggerProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:tabs:{id}:trigger"))?,
            crate::InstanceId::new(format!("sailbreak:tabs:{id}:trigger-instance"))?,
            PrototypeKey::ShadcnTabsTrigger,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter.start(id, label, TABS_TRIGGER_PROFILE, request)
    }

    fn start_content(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &TabsContentProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:tabs:{id}:content"))?,
            crate::InstanceId::new(format!("sailbreak:tabs:{id}:content-instance"))?,
            PrototypeKey::ShadcnTabsContent,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Tabs content"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Tabs content", TABS_CONTENT_PROFILE, request)
    }

    fn ensure_graph_open(&self) -> Result<()> {
        if self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "tabs graph is already set up".to_owned(),
            });
        }
        Ok(())
    }

    fn registered_ids(&self) -> Vec<String> {
        let mut ids = Vec::with_capacity(self.contents.len() + self.triggers.len() + 2);
        ids.extend(self.content_order.iter().cloned());
        ids.extend(self.trigger_order.iter().cloned());
        if let Some(list) = &self.list {
            ids.push(list.id.clone());
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
    }

    fn require_root_record(&self) -> Result<&TabsRootRecord> {
        self.root
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "tabs root is not registered".to_owned(),
            })
    }

    fn require_list_record(&self) -> Result<&TabsListRecord> {
        self.list
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "tabs list is not registered".to_owned(),
            })
    }

    fn require_trigger(&self, id: &str) -> Result<&TabsTriggerRecord> {
        self.triggers.get(id).ok_or_else(|| unknown_trigger(id))
    }
}

impl From<AdapterDispatchOutcome> for TabsDispatchOutcome {
    fn from(outcome: crate::AdapterDispatchOutcome) -> Self {
        Self {
            click_count: outcome.signal_count("click"),
            value_change_count: outcome.signal_count("valueChange"),
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        }
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

fn unknown_trigger(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown tabs trigger: {id}"),
    }
}

fn unknown_content(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown tabs content: {id}"),
    }
}
