use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AnchorRef, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle, CloseReason,
    CommitDisposition, ConnectionRevision, FocusOperationResult, FocusRegistry, InputKind,
    InputSource, LayerRole, LogicalParentRef, NativeStyle, OverlayEvent, OverlayEventEnvelope,
    OverlayHost, OverlayLease, OverlayRect, OverlayRequest, OverlaySurfaceRef, PlacementPolicy,
    PlacementSnapshot, ProtoAdapter, PrototypeKey, PrototypeProfile, Result, SessionId,
    SessionSnapshot, ShadcnTheme, SlotProjection, StartRequest, ViewEpoch,
};

const DIALOG_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogRoot,
    exposed_states: &["open", "disabled", "alert"],
    signals: &["openChange"],
};

const DIALOG_TRIGGER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogTrigger,
    exposed_states: &[
        "disabled",
        "hovered",
        "focused",
        "focusVisible",
        "pressed",
        "dialogExpanded",
        "dialogHasPopup",
        "dialogContentId",
    ],
    signals: &[],
};

const DIALOG_MASK_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogMask,
    exposed_states: &["open"],
    signals: &[],
};

const DIALOG_CONTENT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogContent,
    exposed_states: &[
        "open",
        "alert",
        "dialogRole",
        "dialogModal",
        "dialogContentId",
        "dialogAccessibleLabel",
        "dialogLabelledBy",
        "dialogDescribedBy",
        "focused",
    ],
    signals: &[],
};

const DIALOG_TITLE_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogTitle,
    exposed_states: &["dialogTitleId", "focused", "focusVisible"],
    signals: &[],
};

const DIALOG_DESCRIPTION_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogDescription,
    exposed_states: &["dialogDescriptionId", "focused", "focusVisible"],
    signals: &[],
};

const DIALOG_CLOSE_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogClose,
    exposed_states: &["disabled", "hovered", "focused", "focusVisible", "pressed"],
    signals: &[],
};

const DIALOG_HEADER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogHeader,
    exposed_states: &[],
    signals: &[],
};

const DIALOG_FOOTER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnDialogFooter,
    exposed_states: &[],
    signals: &[],
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogRootProps {
    pub open: Option<bool>,
    pub default_open: bool,
    pub disabled: bool,
    pub alert: bool,
    pub a11y_label: String,
}

impl DialogRootProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(open) = self.open {
            props.insert("open".to_owned(), Value::Bool(open));
        }
        props.insert("defaultOpen".to_owned(), Value::Bool(self.default_open));
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props.insert("alert".to_owned(), Value::Bool(self.alert));
        props.insert(
            "a11yLabel".to_owned(),
            Value::String(self.a11y_label.clone()),
        );
        props
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogTriggerProps {
    pub disabled: bool,
}

impl DialogTriggerProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::from_iter([(String::from("disabled"), Value::Bool(self.disabled))])
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogMaskProps {
    pub passthrough: bool,
}

impl DialogMaskProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::from_iter([(String::from("passthrough"), Value::Bool(self.passthrough))])
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogContentProps;

impl DialogContentProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::new()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogTitleProps;

impl DialogTitleProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::new()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogDescriptionProps;

impl DialogDescriptionProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::new()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogCloseProps {
    pub disabled: bool,
}

impl DialogCloseProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::from_iter([(String::from("disabled"), Value::Bool(self.disabled))])
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogHeaderProps;

impl DialogHeaderProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::new()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogFooterProps;

impl DialogFooterProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogRootSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub open: bool,
    pub disabled: bool,
    pub alert: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogTriggerSnapshot {
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
pub struct DialogMaskSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub present: bool,
    pub portal_lease_id: Option<u64>,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogContentSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub present: bool,
    pub placement: Option<PlacementSnapshot>,
    pub portal_lease_id: Option<u64>,
    pub modal: bool,
    pub role: String,
    pub focused: bool,
    pub labelled_by: Option<String>,
    pub described_by: Option<String>,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogTitleSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub dialog_id: Option<String>,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogDescriptionSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub dialog_id: Option<String>,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogCloseSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub disabled: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogHeaderSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogFooterSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoDialogSnapshot {
    pub root: DialogRootSnapshot,
    pub trigger: Option<DialogTriggerSnapshot>,
    pub mask: Option<DialogMaskSnapshot>,
    pub content: Option<DialogContentSnapshot>,
    pub title: Option<DialogTitleSnapshot>,
    pub description: Option<DialogDescriptionSnapshot>,
    pub closes: Vec<DialogCloseSnapshot>,
    pub header: Option<DialogHeaderSnapshot>,
    pub footer: Option<DialogFooterSnapshot>,
}

pub type DialogSnapshot = ProtoDialogSnapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogDispatchOutcome {
    pub open_change_count: usize,
    pub trigger_press_count: usize,
    pub close_press_count: usize,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl DialogDispatchOutcome {
    #[must_use]
    pub fn signal_count(&self, key: &str) -> usize {
        match key {
            "openChange" => self.open_change_count,
            "trigger" | "triggerPress" => self.trigger_press_count,
            "close" | "closePress" => self.close_press_count,
            _ => 0,
        }
    }

    #[must_use]
    pub const fn action_count(&self) -> usize {
        self.trigger_press_count + self.close_press_count
    }
}

struct DialogRootRecord {
    id: String,
    label: String,
    props: DialogRootProps,
    snapshot: Option<DialogRootSnapshot>,
}

struct DialogTriggerRecord {
    id: String,
    props: DialogTriggerProps,
    snapshot: Option<DialogTriggerSnapshot>,
}

struct DialogMaskRecord {
    id: String,
    props: DialogMaskProps,
    snapshot: Option<DialogMaskSnapshot>,
}

struct DialogContentRecord {
    id: String,
    props: DialogContentProps,
    snapshot: Option<DialogContentSnapshot>,
}

struct DialogTitleRecord {
    id: String,
    label: String,
    props: DialogTitleProps,
    snapshot: Option<DialogTitleSnapshot>,
}

struct DialogDescriptionRecord {
    id: String,
    label: String,
    props: DialogDescriptionProps,
    snapshot: Option<DialogDescriptionSnapshot>,
}

struct DialogCloseRecord {
    id: String,
    label: String,
    props: DialogCloseProps,
    snapshot: Option<DialogCloseSnapshot>,
}

struct DialogHeaderRecord {
    id: String,
    props: DialogHeaderProps,
    snapshot: Option<DialogHeaderSnapshot>,
}

struct DialogFooterRecord {
    id: String,
    props: DialogFooterProps,
    snapshot: Option<DialogFooterSnapshot>,
}

/// Host-side facade for a Shadcn Dialog Root/Trigger/Mask/Content family.
///
/// Registration builds the complete logical graph before Runtime sessions are
/// started. Every part remains an independent Proto session connected through
/// opaque parent and family route references. Proto owns open, modal, focus
/// scope, dismissal, presence, and relation semantics; this host only projects
/// those facts, keeps native focus targets ready, and leases the two dialog
/// layers from the shared OverlayHost.
pub struct ProtoDialogHost {
    adapter: ProtoAdapter,
    root: Option<DialogRootRecord>,
    trigger: Option<DialogTriggerRecord>,
    mask: Option<DialogMaskRecord>,
    content: Option<DialogContentRecord>,
    title: Option<DialogTitleRecord>,
    description: Option<DialogDescriptionRecord>,
    closes: BTreeMap<String, DialogCloseRecord>,
    close_order: Vec<String>,
    header: Option<DialogHeaderRecord>,
    footer: Option<DialogFooterRecord>,
    focus: FocusRegistry,
    overlay: OverlayHost,
    mask_lease: Option<OverlayLease>,
    content_lease: Option<OverlayLease>,
    placement: Option<PlacementSnapshot>,
    family_route: String,
    setup_done: bool,
}

impl ProtoDialogHost {
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
            mask: None,
            content: None,
            title: None,
            description: None,
            closes: BTreeMap::new(),
            close_order: Vec::new(),
            header: None,
            footer: None,
            focus: FocusRegistry::new(),
            overlay: OverlayHost::new(128),
            mask_lease: None,
            content_lease: None,
            placement: None,
            family_route: format!("dialog:{family}"),
            setup_done: false,
        })
    }

    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: DialogRootProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "dialog root")?;
        validate_identity(&label, "dialog root accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.root.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog root".to_owned(),
            });
        }
        self.root = Some(DialogRootRecord {
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
        props: DialogTriggerProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dialog trigger")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog root: {root_id}"),
            });
        }
        if self.trigger.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog trigger".to_owned(),
            });
        }
        self.trigger = Some(DialogTriggerRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_mask(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: DialogMaskProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dialog mask")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog root: {root_id}"),
            });
        }
        if self.mask.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog mask".to_owned(),
            });
        }
        self.mask = Some(DialogMaskRecord {
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
        props: DialogContentProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dialog content")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog root: {root_id}"),
            });
        }
        if self.content.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog content".to_owned(),
            });
        }
        self.content = Some(DialogContentRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_title(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        content_id: &str,
        props: DialogTitleProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "dialog title")?;
        validate_identity(&label, "dialog title accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog content: {content_id}"),
            });
        }
        if self.title.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog title".to_owned(),
            });
        }
        self.title = Some(DialogTitleRecord {
            id,
            label,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_description(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        content_id: &str,
        props: DialogDescriptionProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "dialog description")?;
        validate_identity(&label, "dialog description accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog content: {content_id}"),
            });
        }
        if self.description.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog description".to_owned(),
            });
        }
        self.description = Some(DialogDescriptionRecord {
            id,
            label,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_close(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        content_id: &str,
        props: DialogCloseProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "dialog close")?;
        validate_identity(&label, "dialog close accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog content: {content_id}"),
            });
        }
        if self.closes.contains_key(&id) {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("duplicate dialog close: {id}"),
            });
        }
        self.close_order.push(id.clone());
        self.closes.insert(
            id.clone(),
            DialogCloseRecord {
                id,
                label,
                props,
                snapshot: None,
            },
        );
        Ok(())
    }

    pub fn register_header(
        &mut self,
        id: impl Into<String>,
        content_id: &str,
        props: DialogHeaderProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dialog header")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog content: {content_id}"),
            });
        }
        if self.header.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog header".to_owned(),
            });
        }
        self.header = Some(DialogHeaderRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    pub fn register_footer(
        &mut self,
        id: impl Into<String>,
        content_id: &str,
        props: DialogFooterProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "dialog footer")?;
        self.ensure_unique_id(&id)?;
        if self.require_content_record()?.id != content_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown dialog content: {content_id}"),
            });
        }
        if self.footer.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate dialog footer".to_owned(),
            });
        }
        self.footer = Some(DialogFooterRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    /// Start Root first and then every registered child through fresh opaque
    /// parent references. Optional title, description, close, header, and
    /// footer parts are omitted from the graph when not registered.
    pub fn setup(&mut self) -> Result<()> {
        if self.setup_done {
            return Ok(());
        }
        if self.trigger.is_none() || self.mask.is_none() || self.content.is_none() {
            return Err(BridgeError::InvalidIdentity {
                kind: "dialog graph requires trigger, mask, and content".to_owned(),
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
        let (mask_id, mask_props) = {
            let mask = self.require_mask_record()?;
            (mask.id.clone(), mask.props.clone())
        };
        let (content_id, content_props) = {
            let content = self.require_content_record()?;
            (content.id.clone(), content.props.clone())
        };
        let title = self.title.as_ref().map(|record| {
            (
                record.id.clone(),
                record.label.clone(),
                record.props.clone(),
            )
        });
        let description = self.description.as_ref().map(|record| {
            (
                record.id.clone(),
                record.label.clone(),
                record.props.clone(),
            )
        });
        let closes: Vec<_> = self
            .close_order
            .iter()
            .map(|id| {
                let record = self.closes.get(id).expect("registered dialog close");
                (
                    record.id.clone(),
                    record.label.clone(),
                    record.props.clone(),
                )
            })
            .collect();
        let header = self
            .header
            .as_ref()
            .map(|record| (record.id.clone(), record.props.clone()));
        let footer = self
            .footer
            .as_ref()
            .map(|record| (record.id.clone(), record.props.clone()));

        let started = (|| -> Result<()> {
            self.start_root(&root_id, &root_label, &root_props)?;
            let root_parent = self.parent_ref(&root_id)?;
            self.start_trigger(&trigger_id, &root_parent, &trigger_props)?;
            self.start_mask(&mask_id, &root_parent, &mask_props)?;
            self.start_content(&content_id, &root_parent, &content_props)?;
            let content_parent = self.parent_ref(&content_id)?;
            if let Some((id, label, props)) = &title {
                self.start_title(id, label, &content_parent, props)?;
            }
            if let Some((id, label, props)) = &description {
                self.start_description(id, label, &content_parent, props)?;
            }
            for (id, label, props) in &closes {
                self.start_close(id, label, &content_parent, props)?;
            }
            if let Some((id, props)) = &header {
                self.start_header(id, &content_parent, props)?;
            }
            if let Some((id, props)) = &footer {
                self.start_footer(id, &content_parent, props)?;
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

    /// Read all dialog parts and reconcile the shared native overlay leases.
    pub fn snapshot(&mut self) -> Result<ProtoDialogSnapshot> {
        if !self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "dialog graph is not set up".to_owned(),
            });
        }
        let theme = self.adapter.theme();
        let (root_id, root_label, root_props) = {
            let root = self.require_root_record()?;
            (root.id.clone(), root.label.clone(), root.props.clone())
        };
        let root_adapter = self.adapter.snapshot(&root_id)?;
        let root_session = root_adapter.session;
        let root_open = state_bool(&root_session, "open")
            .unwrap_or(root_props.open.unwrap_or(root_props.default_open));
        let root_disabled = state_bool(&root_session, "disabled").unwrap_or(root_props.disabled);
        let root_alert = state_bool(&root_session, "alert").unwrap_or(root_props.alert);
        let root_snapshot = DialogRootSnapshot {
            id: root_id.clone(),
            label: root_label.clone(),
            native_style: root_adapter.native_style,
            open: root_open,
            disabled: root_disabled,
            alert: root_alert,
            a11y: root_session.a11y.clone(),
            session: root_session,
        };
        self.root.as_mut().expect("registered dialog root").snapshot = Some(root_snapshot.clone());

        let trigger = if let Some(record) = self.trigger.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let disabled =
                state_bool(&session, "disabled").unwrap_or(record.props.disabled || root_disabled);
            let focused = state_bool(&session, "focused").unwrap_or(false);
            let focus_visible = state_bool(&session, "focusVisible").unwrap_or(false);
            let snapshot = DialogTriggerSnapshot {
                id: record.id.clone(),
                label: root_label.clone(),
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open: root_open,
                disabled,
                focused,
                focus_visible,
                a11y: session.a11y.clone(),
                session,
            };
            self.trigger
                .as_mut()
                .expect("registered dialog trigger")
                .snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let (_mask, mask_epoch) = if self.mask.is_some() {
            let id = self.require_mask_record()?.id.clone();
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let epoch = session.projection.view_epoch;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let snapshot = DialogMaskSnapshot {
                id,
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open,
                present: open,
                portal_lease_id: self.mask_lease.as_ref().map(OverlayLease::id),
                a11y: session.a11y.clone(),
                session,
            };
            if let Some(record) = self.mask.as_mut() {
                record.snapshot = Some(snapshot.clone());
            }
            (Some(snapshot), Some(epoch))
        } else {
            (None, None)
        };

        let title_relation = self
            .title
            .as_ref()
            .map(|_| format!("{}:title", self.family_route));
        let description_relation = self
            .description
            .as_ref()
            .map(|_| format!("{}:description", self.family_route));
        let (_content, content_open, content_epoch) = if self.content.is_some() {
            let id = self.require_content_record()?.id.clone();
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let epoch = session.projection.view_epoch;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let role = state_string(&session, "dialogRole").unwrap_or_else(|| {
                if root_alert {
                    "alertdialog".to_owned()
                } else {
                    "dialog".to_owned()
                }
            });
            let modal = state_bool(&session, "dialogModal").unwrap_or(true);
            let labelled_by = title_relation.clone();
            let described_by = description_relation.clone();
            let focused = false;
            let snapshot = DialogContentSnapshot {
                id,
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.content_lease.as_ref().map(OverlayLease::id),
                modal,
                role,
                focused,
                labelled_by,
                described_by,
                a11y: session.a11y.clone(),
                session,
            };
            if let Some(record) = self.content.as_mut() {
                record.snapshot = Some(snapshot.clone());
            }
            (Some(snapshot), open, Some(epoch))
        } else {
            (None, false, None)
        };

        let epoch = content_epoch.or(mask_epoch);
        if let Some(epoch) = epoch {
            self.sync_overlay(content_open, epoch)?;
        }

        let title = if let Some(record) = self.title.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let snapshot = DialogTitleSnapshot {
                id: record.id.clone(),
                label: record.label.clone(),
                native_style: adapter.native_style,
                dialog_id: title_relation.clone(),
                a11y: session.a11y.clone(),
                session,
            };
            self.title
                .as_mut()
                .expect("registered dialog title")
                .snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let description = if let Some(record) = self.description.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let snapshot = DialogDescriptionSnapshot {
                id: record.id.clone(),
                label: record.label.clone(),
                native_style: adapter.native_style,
                dialog_id: description_relation.clone(),
                a11y: session.a11y.clone(),
                session,
            };
            self.description
                .as_mut()
                .expect("registered dialog description")
                .snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let mut closes = Vec::with_capacity(self.close_order.len());
        for id in self.close_order.clone() {
            let record = self.closes.get(&id).expect("registered dialog close");
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let disabled =
                state_bool(&session, "disabled").unwrap_or(record.props.disabled || root_disabled);
            let focused = state_bool(&session, "focused").unwrap_or(false);
            let focus_visible = state_bool(&session, "focusVisible").unwrap_or(false);
            let snapshot = DialogCloseSnapshot {
                id: id.clone(),
                label: record.label.clone(),
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                disabled,
                focused,
                focus_visible,
                a11y: session.a11y.clone(),
                session,
            };
            self.closes
                .get_mut(&id)
                .expect("registered dialog close")
                .snapshot = Some(snapshot.clone());
            closes.push(snapshot);
        }

        let header = if let Some(record) = self.header.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let snapshot = DialogHeaderSnapshot {
                id: record.id.clone(),
                native_style: adapter.native_style,
                a11y: session.a11y.clone(),
                session,
            };
            self.header
                .as_mut()
                .expect("registered dialog header")
                .snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let footer = if let Some(record) = self.footer.as_ref() {
            let adapter = self.adapter.snapshot(&record.id)?;
            let session = adapter.session;
            let snapshot = DialogFooterSnapshot {
                id: record.id.clone(),
                native_style: adapter.native_style,
                a11y: session.a11y.clone(),
                session,
            };
            self.footer
                .as_mut()
                .expect("registered dialog footer")
                .snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let content = if self.content.is_some() {
            let id = self.require_content_record()?.id.clone();
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let role = state_string(&session, "dialogRole").unwrap_or_else(|| {
                if root_alert {
                    "alertdialog".to_owned()
                } else {
                    "dialog".to_owned()
                }
            });
            let modal = state_bool(&session, "dialogModal").unwrap_or(true);
            let labelled_by = title_relation.clone();
            let described_by = description_relation.clone();
            let focused = false;
            let refreshed = DialogContentSnapshot {
                id,
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.content_lease.as_ref().map(OverlayLease::id),
                modal,
                role,
                focused,
                labelled_by,
                described_by,
                a11y: session.a11y.clone(),
                session,
            };
            if let Some(record) = self.content.as_mut() {
                record.snapshot = Some(refreshed.clone());
            }
            Some(refreshed)
        } else {
            None
        };

        let mask = if self.mask.is_some() {
            let id = self.require_mask_record()?.id.clone();
            let adapter = self.adapter.snapshot(&id)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let refreshed = DialogMaskSnapshot {
                id,
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open,
                present: open,
                portal_lease_id: self.mask_lease.as_ref().map(OverlayLease::id),
                a11y: session.a11y.clone(),
                session,
            };
            if let Some(record) = self.mask.as_mut() {
                record.snapshot = Some(refreshed.clone());
            }
            Some(refreshed)
        } else {
            None
        };

        Ok(ProtoDialogSnapshot {
            root: root_snapshot,
            trigger,
            mask,
            content,
            title,
            description,
            closes,
            header,
            footer,
        })
    }

    pub fn root(&self) -> Result<&DialogRootSnapshot> {
        self.root
            .as_ref()
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_root("root"))
    }

    pub fn trigger(&self, id: &str) -> Result<&DialogTriggerSnapshot> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_trigger(id))
    }

    pub fn mask(&self, id: &str) -> Result<&DialogMaskSnapshot> {
        self.mask
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_mask(id))
    }

    pub fn content(&self, id: &str) -> Result<&DialogContentSnapshot> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_content(id))
    }

    pub fn title(&self, id: &str) -> Result<&DialogTitleSnapshot> {
        self.title
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_title(id))
    }

    pub fn description(&self, id: &str) -> Result<&DialogDescriptionSnapshot> {
        self.description
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_description(id))
    }

    pub fn close_part(&self, id: &str) -> Result<&DialogCloseSnapshot> {
        self.closes
            .get(id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_close(id))
    }

    pub fn header(&self, id: &str) -> Result<&DialogHeaderSnapshot> {
        self.header
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_header(id))
    }

    pub fn footer(&self, id: &str) -> Result<&DialogFooterSnapshot> {
        self.footer
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_dialog_footer(id))
    }

    pub fn is_open(&mut self) -> Result<bool> {
        Ok(self.snapshot()?.root.open)
    }

    pub fn modal_blocking(&mut self) -> Result<bool> {
        self.is_open()
    }

    pub fn is_modal_open(&mut self) -> Result<bool> {
        self.modal_blocking()
    }

    /// Return whether an input route is allowed while this modal is open.
    /// The route is opaque to this host; only registered dialog members may
    /// continue through the family while the shared mask gates the rest.
    pub fn input_allowed(&mut self, id: &str) -> Result<bool> {
        if !self.is_open()? {
            return Ok(true);
        }
        Ok(self.is_registered_id(id) && id != self.require_trigger_record()?.id)
    }

    /// Dispatch an input to a registered dialog member. Proto remains the
    /// semantic owner of open and close requests.
    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DialogDispatchOutcome> {
        if !self.is_registered_id(id) {
            return Err(unknown_dialog_member(id));
        }
        if kind == InputKind::PressCommit && self.closes.contains_key(id) && !self.is_open()? {
            return Ok(DialogDispatchOutcome::default());
        }
        let is_trigger = self.trigger.as_ref().is_some_and(|record| record.id == id);
        let is_close = self.closes.contains_key(id);
        let mut result = self.dispatch_inner(id, kind, source, detail)?;
        if kind == InputKind::PressCommit {
            if is_trigger {
                result.trigger_press_count = usize::from(result.open_change_count > 0);
            }
            if is_close {
                result.close_press_count = usize::from(result.open_change_count > 0);
            }
        }
        Ok(result)
    }

    pub fn dispatch_trigger(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DialogDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch(id, kind, source, detail)
    }

    pub fn dispatch_content(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DialogDispatchOutcome> {
        self.require_content(id)?;
        self.dispatch(id, kind, source, detail)
    }

    pub fn dispatch_close(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<DialogDispatchOutcome> {
        self.require_close(id)?;
        self.dispatch(id, kind, source, detail)
    }

    pub fn dispatch_key(&mut self, key: &str) -> Result<DialogDispatchOutcome> {
        if key == "Escape" && self.is_open()? {
            return self.dismiss_escape();
        }
        let id = if self.is_open()? {
            self.require_content_record()?.id.clone()
        } else {
            self.require_trigger_record()?.id.clone()
        };
        self.dispatch_inner(
            &id,
            InputKind::KeyDown,
            InputSource::Keyboard,
            Some(serde_json::json!({ "key": key })),
        )
    }

    pub fn press_trigger(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<DialogDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch(id, InputKind::PressCommit, source, None)
    }

    pub fn press_close(&mut self, id: &str, source: InputSource) -> Result<DialogDispatchOutcome> {
        self.require_close(id)?;
        if !self.is_open()? {
            return Ok(DialogDispatchOutcome::default());
        }
        self.dispatch(id, InputKind::PressCommit, source, None)
    }

    pub fn open(&mut self) -> Result<DialogDispatchOutcome> {
        if self.is_open()? {
            return Ok(DialogDispatchOutcome::default());
        }
        let id = self.require_trigger_record()?.id.clone();
        self.dispatch(
            &id,
            InputKind::PressCommit,
            InputSource::Programmatic,
            Some(serde_json::json!({ "key": "Enter" })),
        )
    }

    /// Close through Proto's semantic trigger route and the shared overlay
    /// lease. Alert dialogs retain Proto's outside-press policy and therefore
    /// reject that reason without changing open state.
    pub fn close(&mut self, reason: CloseReason) -> Result<DialogDispatchOutcome> {
        if !self.is_open()? {
            return Ok(DialogDispatchOutcome::default());
        }
        if reason == CloseReason::OutsidePress && self.root()?.alert {
            return Ok(DialogDispatchOutcome::default());
        }
        let root_controlled = self.require_root_record()?.props.open.is_some();
        if !root_controlled {
            if let Some(lease) = self.mask_lease.as_ref() {
                lease.close(reason)?;
            }
            if let Some(lease) = self.content_lease.as_ref() {
                lease.close(reason)?;
            }
        }
        let id = self.require_trigger_record()?.id.clone();
        let outcome =
            self.dispatch(&id, InputKind::PressCommit, InputSource::Programmatic, None)?;
        if !self.is_open()?
            && matches!(
                reason,
                CloseReason::OutsidePress
                    | CloseReason::FocusOutside
                    | CloseReason::Escape
                    | CloseReason::Programmatic
                    | CloseReason::Replaced
            )
        {
            let _ = self.focus_with_source(&id, InputSource::Programmatic);
        }
        Ok(outcome)
    }

    pub fn dismiss_escape(&mut self) -> Result<DialogDispatchOutcome> {
        self.close(CloseReason::Escape)
    }

    pub fn dismiss_outside(&mut self) -> Result<DialogDispatchOutcome> {
        self.close(CloseReason::OutsidePress)
    }

    pub fn set_root_props(&mut self, props: DialogRootProps) -> Result<CommitDisposition> {
        let id = self.require_root_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.root.as_mut().expect("registered dialog root").props = props;
        self.snapshot()?;
        Ok(disposition)
    }

    pub fn set_trigger_props(&mut self, props: DialogTriggerProps) -> Result<CommitDisposition> {
        let id = self.require_trigger_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.trigger
            .as_mut()
            .expect("registered dialog trigger")
            .props = props;
        Ok(disposition)
    }

    pub fn set_mask_props(&mut self, props: DialogMaskProps) -> Result<CommitDisposition> {
        let id = self.require_mask_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.mask.as_mut().expect("registered dialog mask").props = props;
        self.dispose_overlay_leases();
        self.snapshot()?;
        Ok(disposition)
    }

    pub fn set_content_props(&mut self, props: DialogContentProps) -> Result<CommitDisposition> {
        let id = self.require_content_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.content
            .as_mut()
            .expect("registered dialog content")
            .props = props;
        self.dispose_overlay_leases();
        self.snapshot()?;
        Ok(disposition)
    }

    pub fn set_title_props(&mut self, props: DialogTitleProps) -> Result<CommitDisposition> {
        let id = self.require_title_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.title.as_mut().expect("registered dialog title").props = props;
        Ok(disposition)
    }

    pub fn set_description_props(
        &mut self,
        props: DialogDescriptionProps,
    ) -> Result<CommitDisposition> {
        let id = self.require_description_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.description
            .as_mut()
            .expect("registered dialog description")
            .props = props;
        Ok(disposition)
    }

    pub fn set_close_props(
        &mut self,
        id: &str,
        props: DialogCloseProps,
    ) -> Result<CommitDisposition> {
        self.require_close(id)?;
        let disposition = self.adapter.set_props(id, props.to_map())?;
        self.closes
            .get_mut(id)
            .expect("registered dialog close")
            .props = props;
        Ok(disposition)
    }

    pub fn set_header_props(&mut self, props: DialogHeaderProps) -> Result<CommitDisposition> {
        let id = self.require_header_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.header
            .as_mut()
            .expect("registered dialog header")
            .props = props;
        Ok(disposition)
    }

    pub fn set_footer_props(&mut self, props: DialogFooterProps) -> Result<CommitDisposition> {
        let id = self.require_footer_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.footer
            .as_mut()
            .expect("registered dialog footer")
            .props = props;
        Ok(disposition)
    }

    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        if !self.is_registered_id(id) {
            return Err(unknown_dialog_member(id));
        }
        self.adapter.parent_ref(id)
    }

    /// Mark a trigger, content, or close control as natively ready. The
    /// underlying Proto focus scope still decides whether it may receive focus.
    pub fn set_focus_ready(&mut self, id: &str, ready: bool) -> Result<()> {
        if !self.is_focusable_id(id) {
            return Err(unknown_dialog_focus_target(id));
        }
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
        if !self.is_focusable_id(id) {
            return Ok(FocusOperationResult::Rejected);
        }
        let is_trigger = self.trigger.as_ref().is_some_and(|record| record.id == id);
        if self.is_open()? && is_trigger {
            return Ok(FocusOperationResult::Rejected);
        }
        if !self.is_open()? && !is_trigger {
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

    pub fn focus_entry(&mut self) -> Result<FocusOperationResult> {
        if !self.is_open()? {
            return Ok(FocusOperationResult::Rejected);
        }
        let snapshot = self.snapshot()?;
        if snapshot.closes.iter().any(|close| close.focused) {
            Ok(FocusOperationResult::Accepted)
        } else {
            Ok(FocusOperationResult::NotReady)
        }
    }

    pub fn focus_trap_active(&mut self) -> Result<bool> {
        Ok(self.is_open()? && self.content.is_some())
    }

    pub fn blur(&mut self, id: &str, source: InputSource) -> Result<DialogDispatchOutcome> {
        if !self.is_focusable_id(id) {
            return Err(unknown_dialog_focus_target(id));
        }
        self.dispatch(id, InputKind::Blur, source, None)
    }

    pub fn focus_target(&self, id: &str) -> Result<crate::FocusTarget> {
        if !self.is_focusable_id(id) {
            return Err(unknown_dialog_focus_target(id));
        }
        let snapshot = self.adapter.snapshot_current(id)?;
        Ok(crate::FocusTarget {
            session_id: snapshot.session.session_id.as_str().to_owned(),
            instance_id: snapshot.session.instance_id.as_str().to_owned(),
            view_epoch: snapshot.session.projection.view_epoch,
            route_ref: self.family_route.clone(),
            role: self.focus_role(id).to_owned(),
        })
    }

    pub fn focus_with_target(
        &mut self,
        target: crate::FocusTarget,
    ) -> Result<FocusOperationResult> {
        let id = if let Some(trigger) = self.trigger.as_ref()
            && self.focus_target(&trigger.id)? == target
        {
            trigger.id.clone()
        } else if let Some(content) = self.content.as_ref()
            && self.focus_target(&content.id)? == target
        {
            content.id.clone()
        } else if let Some(id) = self.close_order.iter().find(|id| {
            self.focus_target(id)
                .is_ok_and(|candidate| candidate == target)
        }) {
            id.clone()
        } else {
            return Ok(FocusOperationResult::Rejected);
        };
        self.focus(&id)
    }

    pub fn remount_trigger(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_trigger(id)?;
        let epoch = self.adapter.remount(id)?;
        self.focus.remove(id);
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_mask(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_mask(id)?;
        let epoch = self.adapter.remount(id)?;
        self.dispose_overlay_leases();
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_content(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_content(id)?;
        let epoch = self.adapter.remount(id)?;
        self.dispose_overlay_leases();
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_title(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_title(id)?;
        let epoch = self.adapter.remount(id)?;
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_description(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_description(id)?;
        let epoch = self.adapter.remount(id)?;
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_close(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_close(id)?;
        let epoch = self.adapter.remount(id)?;
        self.focus.remove(id);
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_header(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_header(id)?;
        let epoch = self.adapter.remount(id)?;
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn remount_footer(&mut self, id: &str) -> Result<ViewEpoch> {
        self.require_footer(id)?;
        let epoch = self.adapter.remount(id)?;
        self.snapshot()?;
        Ok(epoch)
    }

    pub fn compute_placement(
        &self,
        anchor_rect: OverlayRect,
        floating_size: (f32, f32),
        viewport: OverlayRect,
    ) -> Result<PlacementSnapshot> {
        self.require_content_record()?;
        let (width, height) = floating_size;
        if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
            return Err(BridgeError::InvalidOverlayGeometry {
                detail: "floating size must be finite and non-negative".to_owned(),
            });
        }
        if ![
            anchor_rect.x,
            anchor_rect.y,
            anchor_rect.width,
            anchor_rect.height,
        ]
        .into_iter()
        .all(f32::is_finite)
            || ![viewport.x, viewport.y, viewport.width, viewport.height]
                .into_iter()
                .all(f32::is_finite)
            || anchor_rect.width < 0.0
            || anchor_rect.height < 0.0
            || viewport.width < 0.0
            || viewport.height < 0.0
        {
            return Err(BridgeError::InvalidOverlayGeometry {
                detail: "dialog geometry must contain finite, non-negative dimensions".to_owned(),
            });
        }
        let x = viewport.x + (viewport.width - width) / 2.0;
        let y = viewport.y + (viewport.height - height) / 2.0;
        Ok(PlacementSnapshot::new(
            anchor_rect,
            OverlayRect::new(x, y, width, height),
            viewport,
            crate::Side::Bottom,
            crate::SideAlign::Center,
        ))
    }

    pub fn update_placement(&mut self, placement: PlacementSnapshot) -> Result<()> {
        let lease = self
            .content_lease
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
            .content_lease
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

    /// Reject a stale completion against the current shared content lease.
    pub fn complete_close(
        &mut self,
        revision: ConnectionRevision,
        reason: CloseReason,
    ) -> Result<()> {
        let lease = self
            .content_lease
            .as_ref()
            .ok_or(BridgeError::StaleOverlayLease { lease_id: 0 })?;
        lease.close_with_revision(revision, reason)
    }

    #[must_use]
    pub fn overlay_lease_id(&self) -> Option<u64> {
        self.content_lease.as_ref().map(OverlayLease::id)
    }

    #[must_use]
    pub fn mask_overlay_lease_id(&self) -> Option<u64> {
        self.mask_lease.as_ref().map(OverlayLease::id)
    }

    #[must_use]
    pub fn overlay_revision(&self) -> Option<ConnectionRevision> {
        self.content_lease
            .as_ref()
            .and_then(|lease| self.overlay.current_revision(lease.id()))
    }

    #[must_use]
    pub fn mask_overlay_revision(&self) -> Option<ConnectionRevision> {
        self.mask_lease
            .as_ref()
            .and_then(|lease| self.overlay.current_revision(lease.id()))
    }

    #[must_use]
    pub fn overlay_layer_role(&self) -> Option<LayerRole> {
        self.content_lease
            .as_ref()
            .map(|_| LayerRole::DialogContent)
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

    /// Dispose all sessions and shared leases. Repeated disposal is a no-op.
    pub fn dispose(&mut self) -> Result<()> {
        self.dispose_overlay_leases();
        if self.setup_done {
            for id in self.registered_ids() {
                let _ = self.adapter.dispose(&id);
            }
        }
        self.closes.clear();
        self.close_order.clear();
        self.footer = None;
        self.header = None;
        self.description = None;
        self.title = None;
        self.content = None;
        self.mask = None;
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
    ) -> Result<DialogDispatchOutcome> {
        let mut outcome = self.adapter.dispatch(id, kind, source, detail)?;
        for sibling in self.registered_ids() {
            if sibling != id {
                outcome.absorb(self.adapter.drain(&sibling)?);
            }
        }
        let result = DialogDispatchOutcome {
            open_change_count: outcome.signal_count("openChange"),
            trigger_press_count: 0,
            close_press_count: 0,
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        };
        self.snapshot()?;
        Ok(result)
    }

    fn sync_overlay(&mut self, open: bool, epoch: ViewEpoch) -> Result<()> {
        let mask = self.require_mask_record()?.id.clone();
        let content = self.require_content_record()?.id.clone();
        let trigger = self.require_trigger_record()?.id.clone();
        if !open {
            if let Some(lease) = self.mask_lease.as_ref() {
                lease.close(CloseReason::Programmatic)?;
            }
            if let Some(lease) = self.content_lease.as_ref() {
                lease.close(CloseReason::Programmatic)?;
            }
            return Ok(());
        }

        let replace = self
            .content_lease
            .as_ref()
            .is_some_and(|lease| self.overlay.view_epoch_of(lease.id()) != Some(epoch))
            || self
                .mask_lease
                .as_ref()
                .is_some_and(|lease| self.overlay.view_epoch_of(lease.id()) != Some(epoch));
        if replace {
            self.dispose_overlay_leases();
            self.placement = None;
        }
        if let (Some(mask_lease), Some(content_lease)) =
            (self.mask_lease.as_ref(), self.content_lease.as_ref())
        {
            self.overlay.reopen(mask_lease.id())?;
            self.overlay.reopen(content_lease.id())?;
            return Ok(());
        }

        self.dispose_overlay_leases();
        let root_anchor = AnchorRef::new(format!("{}:anchor:{}", self.family_route, trigger))?;
        let mask_request = OverlayRequest::new(
            root_anchor.clone(),
            OverlaySurfaceRef::new(format!("{}:surface:{}", self.family_route, mask))?,
            epoch,
            LayerRole::DialogMask,
            PlacementPolicy::Popper {
                side: crate::Side::Bottom,
                side_offset: 0.0,
                align: crate::SideAlign::Start,
                align_offset: 0.0,
                avoid_collisions: false,
                collision_padding: 0.0,
            },
            crate::DismissalPolicy {
                outside_press: false,
                escape: false,
                focus_outside: false,
            },
        )?;
        let content_request = OverlayRequest::new(
            root_anchor,
            OverlaySurfaceRef::new(format!("{}:surface:{}", self.family_route, content))?,
            epoch,
            LayerRole::DialogContent,
            PlacementPolicy::Popper {
                side: crate::Side::Bottom,
                side_offset: 0.0,
                align: crate::SideAlign::Center,
                align_offset: 0.0,
                avoid_collisions: false,
                collision_padding: 0.0,
            },
            crate::DismissalPolicy {
                outside_press: false,
                escape: true,
                focus_outside: false,
            },
        )?
        .with_focus_restore_target(trigger);
        let mask_lease = self.overlay.attach(mask_request)?;
        let content_lease = match self.overlay.attach(content_request) {
            Ok(lease) => lease,
            Err(error) => {
                mask_lease.dispose();
                return Err(error);
            }
        };
        self.mask_lease = Some(mask_lease);
        self.content_lease = Some(content_lease);
        Ok(())
    }

    fn start_root(&mut self, id: &str, label: &str, props: &DialogRootProps) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:root"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:root-instance"))?,
            PrototypeKey::ShadcnDialogRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route);
        self.adapter.start(id, label, DIALOG_ROOT_PROFILE, request)
    }

    fn start_trigger(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DialogTriggerProps,
    ) -> Result<()> {
        let label = self.require_root_record()?.label.clone();
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:trigger"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:trigger-instance"))?,
            PrototypeKey::ShadcnDialogTrigger,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dialog trigger", DIALOG_TRIGGER_PROFILE, request)
    }

    fn start_mask(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DialogMaskProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:mask"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:mask-instance"))?,
            PrototypeKey::ShadcnDialogMask,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Dialog mask"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dialog mask", DIALOG_MASK_PROFILE, request)
    }

    fn start_content(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DialogContentProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:content"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:content-instance"))?,
            PrototypeKey::ShadcnDialogContent,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Dialog content"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dialog content", DIALOG_CONTENT_PROFILE, request)
    }

    fn start_title(
        &mut self,
        id: &str,
        label: &str,
        parent: &LogicalParentRef,
        props: &DialogTitleProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:title"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:title-instance"))?,
            PrototypeKey::ShadcnDialogTitle,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter.start(id, label, DIALOG_TITLE_PROFILE, request)
    }

    fn start_description(
        &mut self,
        id: &str,
        label: &str,
        parent: &LogicalParentRef,
        props: &DialogDescriptionProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:description"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:description-instance"))?,
            PrototypeKey::ShadcnDialogDescription,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, label, DIALOG_DESCRIPTION_PROFILE, request)
    }

    fn start_close(
        &mut self,
        id: &str,
        label: &str,
        parent: &LogicalParentRef,
        props: &DialogCloseProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:close"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:close-instance"))?,
            PrototypeKey::ShadcnDialogClose,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter.start(id, label, DIALOG_CLOSE_PROFILE, request)
    }

    fn start_header(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DialogHeaderProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:header"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:header-instance"))?,
            PrototypeKey::ShadcnDialogHeader,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Dialog header"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dialog header", DIALOG_HEADER_PROFILE, request)
    }

    fn start_footer(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &DialogFooterProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:dialog:{id}:footer"))?,
            crate::InstanceId::new(format!("sailbreak:dialog:{id}:footer-instance"))?,
            PrototypeKey::ShadcnDialogFooter,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), "Dialog footer"),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter
            .start(id, "Dialog footer", DIALOG_FOOTER_PROFILE, request)
    }

    fn refresh_cached_content_placement(&mut self, placement: PlacementSnapshot) {
        if let Some(record) = self.content.as_mut()
            && let Some(snapshot) = record.snapshot.as_mut()
        {
            snapshot.placement = Some(placement);
        }
    }

    fn dispose_overlay_leases(&mut self) {
        if let Some(lease) = self.mask_lease.take() {
            lease.dispose();
        }
        if let Some(lease) = self.content_lease.take() {
            lease.dispose();
        }
        self.placement = None;
    }

    fn ensure_graph_open(&self) -> Result<()> {
        if self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "dialog graph is already set up".to_owned(),
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
            || self.mask.as_ref().is_some_and(|record| record.id == id)
            || self.content.as_ref().is_some_and(|record| record.id == id)
            || self.title.as_ref().is_some_and(|record| record.id == id)
            || self
                .description
                .as_ref()
                .is_some_and(|record| record.id == id)
            || self.closes.contains_key(id)
            || self.header.as_ref().is_some_and(|record| record.id == id)
            || self.footer.as_ref().is_some_and(|record| record.id == id)
    }

    fn is_focusable_id(&self, id: &str) -> bool {
        self.trigger.as_ref().is_some_and(|record| record.id == id) || self.closes.contains_key(id)
    }

    fn focus_role(&self, id: &str) -> &'static str {
        if self.trigger.as_ref().is_some_and(|record| record.id == id) {
            "dialog-trigger"
        } else {
            "dialog-close"
        }
    }

    fn registered_ids(&self) -> Vec<String> {
        let mut ids = Vec::with_capacity(self.closes.len() + 8);
        ids.extend(self.close_order.iter().cloned());
        if let Some(footer) = &self.footer {
            ids.push(footer.id.clone());
        }
        if let Some(header) = &self.header {
            ids.push(header.id.clone());
        }
        if let Some(description) = &self.description {
            ids.push(description.id.clone());
        }
        if let Some(title) = &self.title {
            ids.push(title.id.clone());
        }
        if let Some(content) = &self.content {
            ids.push(content.id.clone());
        }
        if let Some(mask) = &self.mask {
            ids.push(mask.id.clone());
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
        self.dispose_overlay_leases();
    }

    fn require_root_record(&self) -> Result<&DialogRootRecord> {
        self.root
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog root is not registered".to_owned(),
            })
    }

    fn require_trigger_record(&self) -> Result<&DialogTriggerRecord> {
        self.trigger
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog trigger is not registered".to_owned(),
            })
    }

    fn require_mask_record(&self) -> Result<&DialogMaskRecord> {
        self.mask
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog mask is not registered".to_owned(),
            })
    }

    fn require_content_record(&self) -> Result<&DialogContentRecord> {
        self.content
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog content is not registered".to_owned(),
            })
    }

    fn require_title_record(&self) -> Result<&DialogTitleRecord> {
        self.title
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog title is not registered".to_owned(),
            })
    }

    fn require_description_record(&self) -> Result<&DialogDescriptionRecord> {
        self.description
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog description is not registered".to_owned(),
            })
    }

    fn require_header_record(&self) -> Result<&DialogHeaderRecord> {
        self.header
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog header is not registered".to_owned(),
            })
    }

    fn require_footer_record(&self) -> Result<&DialogFooterRecord> {
        self.footer
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "dialog footer is not registered".to_owned(),
            })
    }

    fn require_trigger(&self, id: &str) -> Result<&DialogTriggerRecord> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_trigger(id))
    }

    fn require_mask(&self, id: &str) -> Result<&DialogMaskRecord> {
        self.mask
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_mask(id))
    }

    fn require_content(&self, id: &str) -> Result<&DialogContentRecord> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_content(id))
    }

    fn require_title(&self, id: &str) -> Result<&DialogTitleRecord> {
        self.title
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_title(id))
    }

    fn require_description(&self, id: &str) -> Result<&DialogDescriptionRecord> {
        self.description
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_description(id))
    }

    fn require_close(&self, id: &str) -> Result<&DialogCloseRecord> {
        self.closes.get(id).ok_or_else(|| unknown_dialog_close(id))
    }

    fn require_header(&self, id: &str) -> Result<&DialogHeaderRecord> {
        self.header
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_header(id))
    }

    fn require_footer(&self, id: &str) -> Result<&DialogFooterRecord> {
        self.footer
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_dialog_footer(id))
    }
}

fn state_bool(snapshot: &SessionSnapshot, key: &str) -> Option<bool> {
    snapshot.state_values.get(key).and_then(Value::as_bool)
}

fn state_string(snapshot: &SessionSnapshot, key: &str) -> Option<String> {
    snapshot
        .state_values
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn validate_identity(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BridgeError::InvalidIdentity {
            kind: kind.to_owned(),
        });
    }
    Ok(())
}

fn unknown_dialog_member(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog member: {id}"),
    }
}

fn unknown_dialog_focus_target(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog focus target: {id}"),
    }
}

fn unknown_dialog_root(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog root: {id}"),
    }
}

fn unknown_dialog_trigger(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog trigger: {id}"),
    }
}

fn unknown_dialog_mask(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog mask: {id}"),
    }
}

fn unknown_dialog_content(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog content: {id}"),
    }
}

fn unknown_dialog_title(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog title: {id}"),
    }
}

fn unknown_dialog_description(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog description: {id}"),
    }
}

fn unknown_dialog_close(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog close: {id}"),
    }
}

fn unknown_dialog_header(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog header: {id}"),
    }
}

fn unknown_dialog_footer(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown dialog footer: {id}"),
    }
}
