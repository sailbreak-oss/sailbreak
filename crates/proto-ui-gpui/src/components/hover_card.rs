use serde_json::{Map, Value};

use crate::{
    A11ySnapshot, AdapterDispatchOutcome, BridgeDiagnostic, BridgeError, BridgeEvent, ButtonStyle,
    CloseReason, CommitDisposition, ConnectionRevision, FocusOperationResult, FocusRegistry,
    InputKind, InputSource, LayerRole, LogicalParentRef, NativeStyle, OverlayEvent,
    OverlayEventEnvelope, OverlayHost, OverlayLease, OverlayRect, OverlayRequest,
    OverlaySurfaceRef, PlacementPolicy, PlacementSnapshot, ProtoAdapter, PrototypeKey,
    PrototypeProfile, Result, SessionId, SessionSnapshot, ShadcnTheme, SlotProjection,
    StartRequest, ViewEpoch,
};

const HOVER_CARD_ROOT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnHoverCardRoot,
    exposed_states: &["open", "disabled"],
    signals: &["openChange"],
};

const HOVER_CARD_TRIGGER_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnHoverCardTrigger,
    exposed_states: &["disabled", "hovered", "focused", "focusVisible"],
    signals: &[],
};

const HOVER_CARD_CONTENT_PROFILE: PrototypeProfile = PrototypeProfile {
    prototype: PrototypeKey::ShadcnHoverCardContent,
    exposed_states: &["open"],
    signals: &[],
};

const DEFAULT_OPEN_DELAY: u64 = 700;
const DEFAULT_CLOSE_DELAY: u64 = 300;

/// Props for the semantic Hover Card Root.
///
/// Open and interaction delays are transported to the Proto runtime. The host
/// never starts a native timer; callers advance the bridge's virtual clock with
/// [`ProtoHoverCardHost::advance_time`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoverCardRootProps {
    pub open: Option<bool>,
    pub default_open: bool,
    pub disabled: bool,
    pub open_delay: u64,
    pub close_delay: u64,
}

impl Default for HoverCardRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            disabled: false,
            open_delay: DEFAULT_OPEN_DELAY,
            close_delay: DEFAULT_CLOSE_DELAY,
        }
    }
}

impl HoverCardRootProps {
    fn to_map(&self) -> Map<String, Value> {
        let mut props = Map::new();
        if let Some(open) = self.open {
            props.insert("open".to_owned(), Value::Bool(open));
        }
        props.insert("defaultOpen".to_owned(), Value::Bool(self.default_open));
        props.insert("disabled".to_owned(), Value::Bool(self.disabled));
        props.insert("openDelay".to_owned(), Value::from(self.open_delay));
        props.insert("closeDelay".to_owned(), Value::from(self.close_delay));
        props
    }

    #[must_use]
    pub const fn with_delays(mut self, open_delay: u64, close_delay: u64) -> Self {
        self.open_delay = open_delay;
        self.close_delay = close_delay;
        self
    }
}

/// Props for the native-facing Hover Card Trigger.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HoverCardTriggerProps {
    pub disabled: bool,
}

impl HoverCardTriggerProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::from_iter([(String::from("disabled"), Value::Bool(self.disabled))])
    }
}

/// Placement props for Hover Card Content.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardContentProps {
    pub side: crate::Side,
    pub align: crate::SideAlign,
    pub side_offset: f32,
    pub align_offset: f32,
    pub avoid_collisions: bool,
    pub collision_padding: f32,
}

impl Default for HoverCardContentProps {
    fn default() -> Self {
        Self {
            side: crate::Side::Bottom,
            align: crate::SideAlign::Center,
            side_offset: 4.0,
            align_offset: 0.0,
            avoid_collisions: true,
            collision_padding: 0.0,
        }
    }
}

impl HoverCardContentProps {
    fn to_map(&self) -> Map<String, Value> {
        Map::from_iter([
            (
                String::from("side"),
                Value::String(side_name(self.side).to_owned()),
            ),
            (
                String::from("align"),
                Value::String(align_name(self.align).to_owned()),
            ),
            (String::from("sideOffset"), Value::from(self.side_offset)),
            (String::from("alignOffset"), Value::from(self.align_offset)),
            (
                String::from("avoidCollisions"),
                Value::Bool(self.avoid_collisions),
            ),
            (
                String::from("collisionPadding"),
                Value::from(self.collision_padding),
            ),
        ])
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

#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardRootSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub open: bool,
    pub disabled: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardTriggerSnapshot {
    pub id: String,
    pub label: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub disabled: bool,
    pub hovered: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardContentSnapshot {
    pub id: String,
    pub session: SessionSnapshot,
    pub native_style: NativeStyle,
    pub resolved_style: ButtonStyle,
    pub open: bool,
    pub present: bool,
    pub placement: Option<PlacementSnapshot>,
    pub portal_lease_id: Option<u64>,
    pub slot: SlotProjection,
    pub a11y: Option<A11ySnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoHoverCardSnapshot {
    pub root: HoverCardRootSnapshot,
    pub trigger: Option<HoverCardTriggerSnapshot>,
    pub content: Option<HoverCardContentSnapshot>,
}

/// Alias for callers that use the family name without the host prefix.
pub type HoverCardSnapshot = ProtoHoverCardSnapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HoverCardDispatchOutcome {
    pub open_change_count: usize,
    pub events: Vec<BridgeEvent>,
    pub diagnostics: Vec<BridgeDiagnostic>,
}

impl HoverCardDispatchOutcome {
    #[must_use]
    pub fn signal_count(&self, key: &str) -> usize {
        usize::from(key == "openChange") * self.open_change_count
    }

    fn absorb(&mut self, mut other: Self) {
        self.open_change_count += other.open_change_count;
        self.events.append(&mut other.events);
        self.diagnostics.append(&mut other.diagnostics);
    }
}

impl From<AdapterDispatchOutcome> for HoverCardDispatchOutcome {
    fn from(outcome: AdapterDispatchOutcome) -> Self {
        Self {
            open_change_count: outcome.signal_count("openChange"),
            events: outcome.events,
            diagnostics: outcome.diagnostics,
        }
    }
}

struct HoverCardRootRecord {
    id: String,
    label: String,
    props: HoverCardRootProps,
    snapshot: Option<HoverCardRootSnapshot>,
}

struct HoverCardTriggerRecord {
    id: String,
    props: HoverCardTriggerProps,
    snapshot: Option<HoverCardTriggerSnapshot>,
}

struct HoverCardContentRecord {
    id: String,
    slot: SlotProjection,
    props: HoverCardContentProps,
    snapshot: Option<HoverCardContentSnapshot>,
}

/// Host-side facade for one Shadcn Hover Card Root/Trigger/Content family.
///
/// Registration builds the complete logical graph before Runtime setup. Proto
/// owns interaction, delayed open/close, and transition state. Rust only
/// forwards native pointer/focus facts, advances the existing virtual
/// scheduler, and reconciles one shared OverlayHost lease for the content.
pub struct ProtoHoverCardHost {
    adapter: ProtoAdapter,
    root: Option<HoverCardRootRecord>,
    trigger: Option<HoverCardTriggerRecord>,
    content: Option<HoverCardContentRecord>,
    focus: FocusRegistry,
    overlay: OverlayHost,
    overlay_lease: Option<OverlayLease>,
    placement: Option<PlacementSnapshot>,
    family_route: String,
    setup_done: bool,
}

impl ProtoHoverCardHost {
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
            focus: FocusRegistry::new(),
            overlay: OverlayHost::new(64),
            overlay_lease: None,
            placement: None,
            family_route: format!("hover-card:{family}"),
            setup_done: false,
        })
    }

    pub fn register_root(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        props: HoverCardRootProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let label = label.into();
        validate_identity(&id, "hover card root")?;
        validate_identity(&label, "hover card root accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.root.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate hover card root".to_owned(),
            });
        }
        self.root = Some(HoverCardRootRecord {
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
        props: HoverCardTriggerProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        validate_identity(&id, "hover card trigger")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown hover card root: {root_id}"),
            });
        }
        if self.trigger.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate hover card trigger".to_owned(),
            });
        }
        self.trigger = Some(HoverCardTriggerRecord {
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    /// Register Content with a generic accessible Slot name.
    pub fn register_content(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        props: HoverCardContentProps,
    ) -> Result<()> {
        self.register_content_with_slot(id, root_id, "Hover Card content", props)
    }

    /// Register Content with a caller-owned accessible Slot name. The actual
    /// Slot subtree remains native/Rust-owned and is attached by the caller
    /// when the projected content element is composed.
    pub fn register_content_with_slot(
        &mut self,
        id: impl Into<String>,
        root_id: &str,
        accessible_name: impl Into<String>,
        props: HoverCardContentProps,
    ) -> Result<()> {
        self.ensure_graph_open()?;
        let id = id.into();
        let accessible_name = accessible_name.into();
        validate_identity(&id, "hover card content")?;
        validate_identity(&accessible_name, "hover card content accessible name")?;
        self.ensure_unique_id(&id)?;
        if self.require_root_record()?.id != root_id {
            return Err(BridgeError::InvalidIdentity {
                kind: format!("unknown hover card root: {root_id}"),
            });
        }
        if self.content.is_some() {
            return Err(BridgeError::InvalidIdentity {
                kind: "duplicate hover card content".to_owned(),
            });
        }
        self.content = Some(HoverCardContentRecord {
            slot: SlotProjection::new(format!("{id}:slot"), accessible_name),
            id,
            props,
            snapshot: None,
        });
        Ok(())
    }

    /// Start Root first and then Trigger and Content through fresh opaque
    /// parent references.
    pub fn setup(&mut self) -> Result<()> {
        if self.setup_done {
            return Ok(());
        }
        if self.root.is_none() || self.trigger.is_none() || self.content.is_none() {
            return Err(BridgeError::InvalidIdentity {
                kind: "hover card graph requires root, trigger, and content".to_owned(),
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
        let (content_id, content_slot, content_props) = {
            let content = self.require_content_record()?;
            (
                content.id.clone(),
                content.slot.clone(),
                content.props.clone(),
            )
        };

        let started = (|| -> Result<()> {
            self.start_root(&root_id, &root_label, &root_props)?;
            let root_parent = self.parent_ref(&root_id)?;
            self.start_trigger(&trigger_id, &root_parent, &trigger_props)?;
            self.start_content(&content_id, &root_parent, &content_slot, &content_props)?;
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

    /// Project all members and reconcile the shared content overlay lease.
    pub fn snapshot(&mut self) -> Result<ProtoHoverCardSnapshot> {
        if !self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "hover card graph is not set up".to_owned(),
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
        let root_snapshot = HoverCardRootSnapshot {
            id: root_id,
            label: root_label.clone(),
            native_style: root_adapter.native_style,
            open: root_open,
            disabled: root_disabled,
            a11y: root_session.a11y.clone(),
            session: root_session,
        };
        self.root
            .as_mut()
            .expect("registered hover card root")
            .snapshot = Some(root_snapshot.clone());

        let trigger = if let Some(record) = self.trigger.as_ref() {
            let (trigger_id, trigger_disabled) = (record.id.clone(), record.props.disabled);
            let adapter = self.adapter.snapshot(&trigger_id)?;
            let session = adapter.session;
            let snapshot = HoverCardTriggerSnapshot {
                id: trigger_id,
                label: root_label,
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open: root_open,
                disabled: state_bool(&session, "disabled")
                    .unwrap_or(trigger_disabled || root_disabled),
                hovered: state_bool(&session, "hovered").unwrap_or(false),
                focused: state_bool(&session, "focused").unwrap_or(false),
                focus_visible: state_bool(&session, "focusVisible").unwrap_or(false),
                a11y: session.a11y.clone(),
                session,
            };
            self.trigger
                .as_mut()
                .expect("registered hover card trigger")
                .snapshot = Some(snapshot.clone());
            Some(snapshot)
        } else {
            None
        };

        let content_id = {
            let content = self.require_content_record()?;
            content.id.clone()
        };
        let (content_snapshot, content_open, content_epoch) = {
            let adapter = self.adapter.snapshot(&content_id)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let epoch = session.projection.view_epoch;
            let snapshot = HoverCardContentSnapshot {
                id: content_id.clone(),
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.overlay_lease.as_ref().map(OverlayLease::id),
                slot: session.projection.slot.clone(),
                a11y: session.a11y.clone(),
                session,
            };
            self.content
                .as_mut()
                .expect("registered hover card content")
                .snapshot = Some(snapshot.clone());
            (Some(snapshot), open, Some(epoch))
        };

        if let Some(epoch) = content_epoch {
            self.sync_overlay(content_open, epoch)?;
        }

        // Re-read Content after lease reconciliation so its portal id and
        // placement always describe the current Rust-owned lease.
        let content = if content_snapshot.is_some() {
            let adapter = self.adapter.snapshot(&content_id)?;
            let session = adapter.session;
            let open = state_bool(&session, "open").unwrap_or(root_open);
            let refreshed = HoverCardContentSnapshot {
                id: content_id.clone(),
                native_style: adapter.native_style,
                resolved_style: ButtonStyle::from_projection(&session.style, theme),
                open,
                present: open,
                placement: self.placement.clone(),
                portal_lease_id: self.overlay_lease.as_ref().map(OverlayLease::id),
                slot: session.projection.slot.clone(),
                a11y: session.a11y.clone(),
                session,
            };
            self.content
                .as_mut()
                .expect("registered hover card content")
                .snapshot = Some(refreshed.clone());
            Some(refreshed)
        } else {
            None
        };

        Ok(ProtoHoverCardSnapshot {
            root: root_snapshot,
            trigger,
            content,
        })
    }

    pub fn root(&self) -> Result<&HoverCardRootSnapshot> {
        self.root
            .as_ref()
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_hover_card_root("root"))
    }

    pub fn trigger(&self, id: &str) -> Result<&HoverCardTriggerSnapshot> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_hover_card_trigger(id))
    }

    pub fn content(&self, id: &str) -> Result<&HoverCardContentSnapshot> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .and_then(|record| record.snapshot.as_ref())
            .ok_or_else(|| unknown_hover_card_content(id))
    }

    pub fn is_open(&mut self) -> Result<bool> {
        Ok(self.snapshot()?.root.open)
    }

    /// Dispatch a pointer/focus fact to any registered member. Proto owns the
    /// resulting interaction and delayed open/close request.
    pub fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<HoverCardDispatchOutcome> {
        if !self.is_registered_id(id) {
            return Err(unknown_hover_card_member(id));
        }
        self.dispatch_inner(id, kind, source, detail)
    }

    pub fn dispatch_trigger(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<HoverCardDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch_inner(id, kind, source, detail)
    }

    pub fn dispatch_content(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<HoverCardDispatchOutcome> {
        self.require_content(id)?;
        self.dispatch_inner(id, kind, source, detail)
    }

    pub fn pointer_enter(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<HoverCardDispatchOutcome> {
        self.dispatch_trigger(id, InputKind::PointerEnter, source, None)
    }

    pub fn pointer_leave(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> Result<HoverCardDispatchOutcome> {
        if self.trigger.as_ref().is_some_and(|record| record.id == id) {
            self.dispatch_trigger(id, InputKind::PointerLeave, source, None)
        } else {
            self.dispatch_content(id, InputKind::PointerLeave, source, None)
        }
    }

    /// Advance the existing Proto/bridge virtual scheduler. No native timer is
    /// installed by this host.
    pub fn advance_time(&mut self, milliseconds: u64) -> Result<HoverCardDispatchOutcome> {
        let root_id = self.require_root_record()?.id.clone();
        let mut outcome =
            HoverCardDispatchOutcome::from(self.adapter.advance_time(&root_id, milliseconds)?);
        for sibling in self.registered_ids() {
            if sibling != root_id {
                outcome.absorb(self.adapter.drain(&sibling)?.into());
            }
        }
        self.snapshot()?;
        Ok(outcome)
    }

    /// Convenience programmatic open. It follows the same Proto interaction
    /// request and virtual open delay as pointer entry.
    pub fn open(&mut self) -> Result<HoverCardDispatchOutcome> {
        if self.is_open()? {
            return Ok(HoverCardDispatchOutcome::default());
        }
        let trigger = self.require_trigger_record()?.id.clone();
        let delay = self.require_root_record()?.props.open_delay;
        let mut outcome = self.pointer_enter(&trigger, InputSource::Programmatic)?;
        outcome.absorb(self.advance_time(delay)?);
        Ok(outcome)
    }

    /// Close through a Proto interaction transition while preserving the
    /// supplied semantic reason on the shared overlay lease.
    pub fn close(&mut self, reason: CloseReason) -> Result<HoverCardDispatchOutcome> {
        if !self.is_open()? {
            return Ok(HoverCardDispatchOutcome::default());
        }
        let trigger = self.require_trigger_record()?.id.clone();
        let content = self.require_content_record()?.id.clone();
        let mut outcome = if reason == CloseReason::FocusOutside {
            self.dispatch_trigger(&trigger, InputKind::Blur, InputSource::Programmatic, None)?
        } else {
            self.dispatch_trigger(
                &trigger,
                InputKind::PointerLeave,
                InputSource::Programmatic,
                None,
            )?
        };
        // Content hover keeps a card open while the pointer crosses the gap
        // from Trigger to Content. Explicit dismissal clears that fact.
        outcome.absorb(self.dispatch_content(
            &content,
            InputKind::PointerLeave,
            InputSource::Programmatic,
            None,
        )?);
        if let Some(lease) = self.overlay_lease.as_ref() {
            lease.close(reason)?;
        }
        let delay = self.require_root_record()?.props.close_delay;
        outcome.absorb(self.advance_time(delay)?);
        Ok(outcome)
    }

    pub fn dismiss_outside(&mut self) -> Result<HoverCardDispatchOutcome> {
        self.close(CloseReason::OutsidePress)
    }

    pub fn dismiss_escape(&mut self) -> Result<HoverCardDispatchOutcome> {
        self.close(CloseReason::Escape)
    }

    pub fn dismiss_focus_outside(&mut self) -> Result<HoverCardDispatchOutcome> {
        self.close(CloseReason::FocusOutside)
    }

    pub fn set_root_props(&mut self, props: HoverCardRootProps) -> Result<CommitDisposition> {
        let id = self.require_root_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.root
            .as_mut()
            .expect("registered hover card root")
            .props = props;
        self.snapshot()?;
        Ok(disposition)
    }

    pub fn set_trigger_props(&mut self, props: HoverCardTriggerProps) -> Result<CommitDisposition> {
        let id = self.require_trigger_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.trigger
            .as_mut()
            .expect("registered hover card trigger")
            .props = props;
        self.snapshot()?;
        Ok(disposition)
    }

    pub fn set_content_props(&mut self, props: HoverCardContentProps) -> Result<CommitDisposition> {
        let id = self.require_content_record()?.id.clone();
        let disposition = self.adapter.set_props(&id, props.to_map())?;
        self.content
            .as_mut()
            .expect("registered hover card content")
            .props = props;
        self.dispose_overlay_lease();
        self.snapshot()?;
        Ok(disposition)
    }

    pub fn parent_ref(&mut self, id: &str) -> Result<LogicalParentRef> {
        if !self.is_registered_id(id) {
            return Err(unknown_hover_card_member(id));
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
        if self.focus_target(id)? != *target {
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
        self.drain_siblings(id)?;
        self.snapshot()?;
        Ok(FocusOperationResult::Accepted)
    }

    pub fn blur(&mut self, id: &str, source: InputSource) -> Result<HoverCardDispatchOutcome> {
        self.require_trigger(id)?;
        self.dispatch_inner(id, InputKind::Blur, source, None)
    }

    pub fn focus_target(&self, id: &str) -> Result<crate::FocusTarget> {
        self.require_trigger(id)?;
        let snapshot = self.adapter.snapshot_current(id)?;
        Ok(crate::FocusTarget {
            session_id: snapshot.session.session_id.as_str().to_owned(),
            instance_id: snapshot.session.instance_id.as_str().to_owned(),
            view_epoch: snapshot.session.projection.view_epoch,
            route_ref: self.family_route.clone(),
            role: "hover-card-trigger".to_owned(),
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
        self.dispose_overlay_lease();
        self.snapshot()?;
        Ok(epoch)
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
        if let Some(lease) = self.overlay_lease.as_ref() {
            lease.update(placement.clone())?;
        }
        self.placement = Some(placement.clone());
        self.refresh_cached_content_placement(placement.clone());
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

    /// Reject a stale transition completion against the current content lease.
    pub fn complete_close(
        &mut self,
        revision: ConnectionRevision,
        reason: CloseReason,
    ) -> Result<()> {
        let lease = self
            .overlay_lease
            .as_ref()
            .ok_or(BridgeError::StaleOverlayLease { lease_id: 0 })?;
        lease.close_with_revision(revision, reason)
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
    pub fn overlay_layer_role(&self) -> Option<LayerRole> {
        self.overlay_lease
            .as_ref()
            .map(|_| LayerRole::HoverCardContent)
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
    /// is an idempotent no-op.
    pub fn dispose(&mut self) -> Result<()> {
        self.dispose_overlay_lease();
        if self.setup_done {
            for id in self.registered_ids() {
                let _ = self.adapter.dispose(&id);
            }
        }
        self.content = None;
        self.trigger = None;
        self.root = None;
        self.focus = FocusRegistry::new();
        self.placement = None;
        self.setup_done = false;
        Ok(())
    }

    fn dispatch_inner(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<Value>,
    ) -> Result<HoverCardDispatchOutcome> {
        let mut outcome =
            HoverCardDispatchOutcome::from(self.adapter.dispatch(id, kind, source, detail)?);
        self.drain_siblings_into(id, &mut outcome)?;
        self.snapshot()?;
        Ok(outcome)
    }

    fn drain_siblings(&mut self, id: &str) -> Result<()> {
        let mut ignored = HoverCardDispatchOutcome::default();
        self.drain_siblings_into(id, &mut ignored)
    }

    fn drain_siblings_into(
        &mut self,
        id: &str,
        outcome: &mut HoverCardDispatchOutcome,
    ) -> Result<()> {
        for sibling in self.registered_ids() {
            if sibling != id {
                outcome.absorb(self.adapter.drain(&sibling)?.into());
            }
        }
        Ok(())
    }

    fn sync_overlay(&mut self, open: bool, epoch: ViewEpoch) -> Result<()> {
        let (content_id, content_props) = {
            let content = self.require_content_record()?;
            (content.id.clone(), content.props.clone())
        };
        let trigger_id = self.require_trigger_record()?.id.clone();
        if !open {
            if let Some(lease) = self.overlay_lease.as_ref() {
                lease.close(CloseReason::Programmatic)?;
            }
            return Ok(());
        }

        let replace = self
            .overlay_lease
            .as_ref()
            .is_some_and(|lease| self.overlay.view_epoch_of(lease.id()) != Some(epoch));
        if replace {
            self.dispose_overlay_lease();
        }
        if let Some(id) = self.overlay_lease.as_ref().map(OverlayLease::id) {
            self.overlay.reopen(id)?;
            return Ok(());
        }

        let request = OverlayRequest::new(
            crate::AnchorRef::new(format!("{}:anchor:{}", self.family_route, trigger_id))?,
            OverlaySurfaceRef::new(format!("{}:surface:{}", self.family_route, content_id))?,
            epoch,
            LayerRole::HoverCardContent,
            content_props.placement_policy(),
            Default::default(),
        )?;
        self.overlay_lease = Some(self.overlay.attach(request)?);
        if let Some(placement) = self.placement.clone()
            && let Some(lease) = self.overlay_lease.as_ref()
        {
            lease.update(placement)?;
        }
        Ok(())
    }

    fn start_root(&mut self, id: &str, label: &str, props: &HoverCardRootProps) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:hover-card:{id}:root"))?,
            crate::InstanceId::new(format!("sailbreak:hover-card:{id}:root-instance"))?,
            PrototypeKey::ShadcnHoverCardRoot,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label.to_owned()),
        )
        .with_route_ref(&self.family_route);
        self.adapter
            .start(id, label, HOVER_CARD_ROOT_PROFILE, request)
    }

    fn start_trigger(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        props: &HoverCardTriggerProps,
    ) -> Result<()> {
        let label = self.require_root_record()?.label.clone();
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:hover-card:{id}:trigger"))?,
            crate::InstanceId::new(format!("sailbreak:hover-card:{id}:trigger-instance"))?,
            PrototypeKey::ShadcnHoverCardTrigger,
            props.to_map(),
            SlotProjection::new(format!("{id}:slot"), label),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter.start(
            id,
            "Hover Card trigger",
            HOVER_CARD_TRIGGER_PROFILE,
            request,
        )
    }

    fn start_content(
        &mut self,
        id: &str,
        parent: &LogicalParentRef,
        slot: &SlotProjection,
        props: &HoverCardContentProps,
    ) -> Result<()> {
        let request = StartRequest::new(
            SessionId::new(format!("sailbreak:hover-card:{id}:content"))?,
            crate::InstanceId::new(format!("sailbreak:hover-card:{id}:content-instance"))?,
            PrototypeKey::ShadcnHoverCardContent,
            props.to_map(),
            slot.clone(),
        )
        .with_route_ref(&self.family_route)
        .with_parent(parent.clone());
        self.adapter.start(
            id,
            "Hover Card content",
            HOVER_CARD_CONTENT_PROFILE,
            request,
        )
    }

    fn refresh_cached_content_placement(&mut self, placement: PlacementSnapshot) {
        if let Some(record) = self.content.as_mut()
            && let Some(snapshot) = record.snapshot.as_mut()
        {
            snapshot.placement = Some(placement);
        }
    }

    fn dispose_overlay_lease(&mut self) {
        if let Some(lease) = self.overlay_lease.take() {
            lease.dispose();
        }
        self.placement = None;
    }

    fn ensure_graph_open(&self) -> Result<()> {
        if self.setup_done {
            return Err(BridgeError::InvalidIdentity {
                kind: "hover card graph is already set up".to_owned(),
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
    }

    fn registered_ids(&self) -> Vec<String> {
        let mut ids = Vec::with_capacity(3);
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
        self.dispose_overlay_lease();
    }

    fn require_root_record(&self) -> Result<&HoverCardRootRecord> {
        self.root
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "hover card root is not registered".to_owned(),
            })
    }

    fn require_trigger_record(&self) -> Result<&HoverCardTriggerRecord> {
        self.trigger
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "hover card trigger is not registered".to_owned(),
            })
    }

    fn require_content_record(&self) -> Result<&HoverCardContentRecord> {
        self.content
            .as_ref()
            .ok_or_else(|| BridgeError::InvalidIdentity {
                kind: "hover card content is not registered".to_owned(),
            })
    }

    fn require_trigger(&self, id: &str) -> Result<&HoverCardTriggerRecord> {
        self.trigger
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_hover_card_trigger(id))
    }

    fn require_content(&self, id: &str) -> Result<&HoverCardContentRecord> {
        self.content
            .as_ref()
            .filter(|record| record.id == id)
            .ok_or_else(|| unknown_hover_card_content(id))
    }
}

fn state_bool(snapshot: &SessionSnapshot, key: &str) -> Option<bool> {
    snapshot.state_values.get(key).and_then(Value::as_bool)
}

fn side_name(side: crate::Side) -> &'static str {
    match side {
        crate::Side::Top => "top",
        crate::Side::Right => "right",
        crate::Side::Bottom => "bottom",
        crate::Side::Left => "left",
    }
}

fn align_name(align: crate::SideAlign) -> &'static str {
    match align {
        crate::SideAlign::Start => "start",
        crate::SideAlign::Center => "center",
        crate::SideAlign::End => "end",
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

fn unknown_hover_card_member(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown hover card member: {id}"),
    }
}

fn unknown_hover_card_root(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown hover card root: {id}"),
    }
}

fn unknown_hover_card_trigger(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown hover card trigger: {id}"),
    }
}

fn unknown_hover_card_content(id: &str) -> BridgeError {
    BridgeError::InvalidIdentity {
        kind: format!("unknown hover card content: {id}"),
    }
}
