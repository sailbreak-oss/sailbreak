use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    rc::{Rc, Weak},
};

use crate::{BridgeError, Result, ViewEpoch};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AnchorRef(String);

impl AnchorRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BridgeError::InvalidAnchorRef { value });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OverlaySurfaceRef(String);

impl OverlaySurfaceRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BridgeError::InvalidSurfaceRef { value });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConnectionRevision(u64);

impl ConnectionRevision {
    pub fn new(value: u64) -> Result<Self> {
        if value == 0 {
            return Err(BridgeError::InvalidRevision);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LayerRole {
    HoverCardContent,
    SelectContent,
    DropdownContent,
    DialogMask,
    DialogContent,
}

impl LayerRole {
    const fn priority(self) -> u64 {
        match self {
            Self::HoverCardContent => 10,
            Self::SelectContent => 20,
            Self::DropdownContent => 30,
            Self::DialogMask => 40,
            Self::DialogContent => 50,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SideAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl OverlayRect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    #[must_use]
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    fn validate(self, name: &str) -> Result<()> {
        if [self.x, self.y, self.width, self.height]
            .into_iter()
            .all(f32::is_finite)
            && self.width >= 0.0
            && self.height >= 0.0
        {
            return Ok(());
        }
        Err(BridgeError::InvalidOverlayGeometry {
            detail: format!("{name} must contain finite, non-negative dimensions"),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlacementPolicy {
    ItemAligned {
        align: SideAlign,
        align_offset: f32,
    },
    Popper {
        side: Side,
        side_offset: f32,
        align: SideAlign,
        align_offset: f32,
        avoid_collisions: bool,
        collision_padding: f32,
    },
}

impl PlacementPolicy {
    pub fn compute_placement(
        &self,
        anchor_rect: OverlayRect,
        floating_size: (f32, f32),
        viewport: OverlayRect,
    ) -> Result<PlacementSnapshot> {
        anchor_rect.validate("anchor rectangle")?;
        viewport.validate("viewport rectangle")?;
        let (width, height) = floating_size;
        if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
            return Err(BridgeError::InvalidOverlayGeometry {
                detail: "floating size must be finite and non-negative".to_owned(),
            });
        }

        match *self {
            Self::ItemAligned {
                align,
                align_offset,
            } => {
                if !align_offset.is_finite() {
                    return Err(BridgeError::InvalidOverlayGeometry {
                        detail: "item alignment offset must be finite".to_owned(),
                    });
                }
                let x = aligned_coordinate(
                    anchor_rect.x,
                    anchor_rect.width,
                    width,
                    align,
                    align_offset,
                );
                Ok(PlacementSnapshot::new(
                    anchor_rect,
                    OverlayRect::new(x, anchor_rect.bottom(), width, height),
                    viewport,
                    Side::Bottom,
                    align,
                ))
            }
            Self::Popper {
                side,
                side_offset,
                align,
                align_offset,
                avoid_collisions,
                collision_padding,
            } => {
                if !side_offset.is_finite()
                    || !align_offset.is_finite()
                    || !collision_padding.is_finite()
                    || collision_padding < 0.0
                {
                    return Err(BridgeError::InvalidOverlayGeometry {
                        detail: "popper offsets and collision padding must be finite".to_owned(),
                    });
                }
                let mut resolved_side = side;
                let mut rect = popper_rect(
                    anchor_rect,
                    width,
                    height,
                    resolved_side,
                    side_offset,
                    align,
                    align_offset,
                );
                let mut flipped = false;
                let mut shifted = false;
                if avoid_collisions {
                    let bounds = inset(viewport, collision_padding);
                    if main_axis_overflow(rect, bounds, resolved_side) > 0.0 {
                        let opposite = resolved_side.opposite();
                        let candidate = popper_rect(
                            anchor_rect,
                            width,
                            height,
                            opposite,
                            side_offset,
                            align,
                            align_offset,
                        );
                        if main_axis_overflow(candidate, bounds, opposite)
                            < main_axis_overflow(rect, bounds, resolved_side)
                        {
                            rect = candidate;
                            resolved_side = opposite;
                            flipped = true;
                        }
                    }
                    let shifted_x = rect
                        .x
                        .clamp(bounds.x, (bounds.right() - rect.width).max(bounds.x));
                    let shifted_y = rect
                        .y
                        .clamp(bounds.y, (bounds.bottom() - rect.height).max(bounds.y));
                    shifted = shifted_x != rect.x || shifted_y != rect.y;
                    rect.x = shifted_x;
                    rect.y = shifted_y;
                }
                let mut snapshot =
                    PlacementSnapshot::new(anchor_rect, rect, viewport, resolved_side, align);
                snapshot.flipped = flipped;
                snapshot.shifted = shifted;
                Ok(snapshot)
            }
        }
    }
}

fn aligned_coordinate(
    anchor_start: f32,
    anchor_size: f32,
    floating_size: f32,
    align: SideAlign,
    offset: f32,
) -> f32 {
    let base = match align {
        SideAlign::Start => anchor_start,
        SideAlign::Center => anchor_start + (anchor_size - floating_size) / 2.0,
        SideAlign::End => anchor_start + anchor_size - floating_size,
    };
    base + offset
}

fn popper_rect(
    anchor: OverlayRect,
    width: f32,
    height: f32,
    side: Side,
    side_offset: f32,
    align: SideAlign,
    align_offset: f32,
) -> OverlayRect {
    match side {
        Side::Top => OverlayRect::new(
            aligned_coordinate(anchor.x, anchor.width, width, align, align_offset),
            anchor.y - height - side_offset,
            width,
            height,
        ),
        Side::Bottom => OverlayRect::new(
            aligned_coordinate(anchor.x, anchor.width, width, align, align_offset),
            anchor.bottom() + side_offset,
            width,
            height,
        ),
        Side::Left => OverlayRect::new(
            anchor.x - width - side_offset,
            aligned_coordinate(anchor.y, anchor.height, height, align, align_offset),
            width,
            height,
        ),
        Side::Right => OverlayRect::new(
            anchor.right() + side_offset,
            aligned_coordinate(anchor.y, anchor.height, height, align, align_offset),
            width,
            height,
        ),
    }
}

fn inset(rect: OverlayRect, amount: f32) -> OverlayRect {
    OverlayRect::new(
        rect.x + amount,
        rect.y + amount,
        (rect.width - amount * 2.0).max(0.0),
        (rect.height - amount * 2.0).max(0.0),
    )
}

fn main_axis_overflow(rect: OverlayRect, bounds: OverlayRect, side: Side) -> f32 {
    match side {
        Side::Top => (bounds.y - rect.y).max(0.0),
        Side::Bottom => (rect.bottom() - bounds.bottom()).max(0.0),
        Side::Left => (bounds.x - rect.x).max(0.0),
        Side::Right => (rect.right() - bounds.right()).max(0.0),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlacementSnapshot {
    pub anchor_rect: OverlayRect,
    pub floating_rect: OverlayRect,
    pub viewport: OverlayRect,
    pub side: Side,
    pub align: SideAlign,
    pub flipped: bool,
    pub shifted: bool,
    pub available_width: f32,
    pub available_height: f32,
    view_epoch: Option<ViewEpoch>,
}

impl PlacementSnapshot {
    #[must_use]
    pub fn new(
        anchor_rect: OverlayRect,
        floating_rect: OverlayRect,
        viewport: OverlayRect,
        side: Side,
        align: SideAlign,
    ) -> Self {
        let (available_width, available_height) = match side {
            Side::Top => (viewport.width, (anchor_rect.y - viewport.y).max(0.0)),
            Side::Bottom => (
                viewport.width,
                (viewport.bottom() - anchor_rect.bottom()).max(0.0),
            ),
            Side::Left => ((anchor_rect.x - viewport.x).max(0.0), viewport.height),
            Side::Right => (
                (viewport.right() - anchor_rect.right()).max(0.0),
                viewport.height,
            ),
        };
        Self {
            anchor_rect,
            floating_rect,
            viewport,
            side,
            align,
            flipped: false,
            shifted: false,
            available_width,
            available_height,
            view_epoch: None,
        }
    }

    #[must_use]
    pub fn with_view_epoch(mut self, view_epoch: ViewEpoch) -> Self {
        self.view_epoch = Some(view_epoch);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DismissalPolicy {
    pub outside_press: bool,
    pub escape: bool,
    pub focus_outside: bool,
}

impl Default for DismissalPolicy {
    fn default() -> Self {
        Self {
            outside_press: true,
            escape: true,
            focus_outside: true,
        }
    }
}

impl DismissalPolicy {
    const fn allows(self, reason: CloseReason) -> bool {
        match reason {
            CloseReason::OutsidePress => self.outside_press,
            CloseReason::Escape => self.escape,
            CloseReason::FocusOutside => self.focus_outside,
            CloseReason::Programmatic | CloseReason::Replaced => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseReason {
    OutsidePress,
    Escape,
    FocusOutside,
    Programmatic,
    Replaced,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayEvent {
    Placement(PlacementSnapshot),
    Close(CloseReason),
    FocusRestore(String),
    PresenceEnter,
}

impl OverlayEvent {
    #[must_use]
    pub const fn is_placement(&self) -> bool {
        matches!(self, Self::Placement(_))
    }

    #[must_use]
    pub const fn is_close(&self) -> bool {
        matches!(self, Self::Close(_))
    }

    #[must_use]
    pub const fn is_focus_restore(&self) -> bool {
        matches!(self, Self::FocusRestore(_))
    }

    #[must_use]
    pub const fn is_presence_enter(&self) -> bool {
        matches!(self, Self::PresenceEnter)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayEventEnvelope {
    pub lease_id: u64,
    pub event: OverlayEvent,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayRequest {
    pub anchor_ref: AnchorRef,
    pub surface_ref: OverlaySurfaceRef,
    pub view_epoch: ViewEpoch,
    pub layer_role: LayerRole,
    pub placement_policy: PlacementPolicy,
    pub dismissal_policy: DismissalPolicy,
    pub focus_restore_target: Option<String>,
}

impl OverlayRequest {
    pub fn new(
        anchor_ref: AnchorRef,
        surface_ref: OverlaySurfaceRef,
        view_epoch: ViewEpoch,
        layer_role: LayerRole,
        placement_policy: PlacementPolicy,
        dismissal_policy: DismissalPolicy,
    ) -> Result<Self> {
        Ok(Self {
            anchor_ref,
            surface_ref,
            view_epoch,
            layer_role,
            placement_policy,
            dismissal_policy,
            focus_restore_target: None,
        })
    }

    pub fn popper(
        anchor_ref: AnchorRef,
        surface_ref: OverlaySurfaceRef,
        view_epoch: ViewEpoch,
        layer_role: LayerRole,
        side: Side,
        align: SideAlign,
    ) -> Result<Self> {
        Self::new(
            anchor_ref,
            surface_ref,
            view_epoch,
            layer_role,
            PlacementPolicy::Popper {
                side,
                side_offset: 0.0,
                align,
                align_offset: 0.0,
                avoid_collisions: true,
                collision_padding: 0.0,
            },
            DismissalPolicy::default(),
        )
    }

    #[must_use]
    pub fn with_focus_restore_target(mut self, target: impl Into<String>) -> Self {
        self.focus_restore_target = Some(target.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PresenceState {
    Entered,
    Leaving,
}

struct OverlayRecord {
    request: OverlayRequest,
    revision: ConnectionRevision,
    layer_order: u64,
    placement: Option<PlacementSnapshot>,
    presence: PresenceState,
    closed: bool,
}

type QueuedEvent = OverlayEventEnvelope;

struct OverlayCore {
    capacity: usize,
    failed: bool,
    next_id: u64,
    next_revision: u64,
    records: BTreeMap<u64, OverlayRecord>,
    surfaces: BTreeMap<OverlaySurfaceRef, u64>,
    events: VecDeque<QueuedEvent>,
}

impl OverlayCore {
    fn terminal_error(&self) -> BridgeError {
        BridgeError::OverlayQueueOverflow {
            capacity: self.capacity,
        }
    }

    fn ensure_live(&self) -> Result<()> {
        if self.failed {
            return Err(self.terminal_error());
        }
        Ok(())
    }

    fn validate_lease(&self, id: u64, revision: ConnectionRevision) -> Result<&OverlayRecord> {
        self.ensure_live()?;
        self.records
            .get(&id)
            .filter(|record| record.revision == revision)
            .ok_or(BridgeError::StaleOverlayLease { lease_id: id })
    }

    fn push_events(&mut self, id: u64, events: Vec<OverlayEvent>) -> Result<()> {
        self.ensure_live()?;
        if self.events.len().saturating_add(events.len()) > self.capacity {
            self.failed = true;
            return Err(self.terminal_error());
        }
        self.events
            .extend(events.into_iter().map(|event| OverlayEventEnvelope {
                lease_id: id,
                event,
            }));
        Ok(())
    }

    fn remove_record(&mut self, id: u64) -> bool {
        let Some(record) = self.records.remove(&id) else {
            return false;
        };
        if self.surfaces.get(&record.request.surface_ref) == Some(&id) {
            self.surfaces.remove(&record.request.surface_ref);
        }
        true
    }

    fn dispose(&mut self, id: u64) {
        if self.remove_record(id) {
            self.events.retain(|queued| queued.lease_id != id);
        }
    }

    fn close(&mut self, id: u64, revision: ConnectionRevision, reason: CloseReason) -> Result<()> {
        let record = self.validate_lease(id, revision)?;
        if record.closed || !record.request.dismissal_policy.allows(reason) {
            return Ok(());
        }
        let restore = record.request.focus_restore_target.clone();
        let mut events = vec![OverlayEvent::Close(reason)];
        if let Some(target) = restore {
            events.push(OverlayEvent::FocusRestore(target));
        }
        self.push_events(id, events)?;
        let record = self
            .records
            .get_mut(&id)
            .ok_or(BridgeError::StaleOverlayLease { lease_id: id })?;
        record.closed = true;
        record.presence = PresenceState::Leaving;
        Ok(())
    }
}

pub struct OverlayHost {
    core: Rc<RefCell<OverlayCore>>,
}

impl OverlayHost {
    #[must_use]
    pub fn new(event_capacity: usize) -> Self {
        Self {
            core: Rc::new(RefCell::new(OverlayCore {
                capacity: event_capacity,
                failed: false,
                next_id: 0,
                next_revision: 0,
                records: BTreeMap::new(),
                surfaces: BTreeMap::new(),
                events: VecDeque::with_capacity(event_capacity),
            })),
        }
    }

    pub fn attach(&mut self, request: OverlayRequest) -> Result<OverlayLease> {
        if request
            .focus_restore_target
            .as_deref()
            .is_some_and(|target| target.trim().is_empty())
        {
            return Err(BridgeError::InvalidIdentity {
                kind: "overlay focus restore target".to_owned(),
            });
        }
        let mut core = self.core.borrow_mut();
        core.ensure_live()?;
        if let Some(previous_id) = core.surfaces.get(&request.surface_ref).copied() {
            let previous_revision = core
                .records
                .get(&previous_id)
                .map(|record| record.revision)
                .ok_or(BridgeError::StaleOverlayLease {
                    lease_id: previous_id,
                })?;
            core.close(previous_id, previous_revision, CloseReason::Replaced)?;
            core.remove_record(previous_id);
        }
        core.next_id = core
            .next_id
            .checked_add(1)
            .ok_or_else(|| BridgeError::Runtime {
                detail: "overlay lease id overflow".to_owned(),
            })?;
        core.next_revision =
            core.next_revision
                .checked_add(1)
                .ok_or_else(|| BridgeError::Runtime {
                    detail: "overlay connection revision overflow".to_owned(),
                })?;
        let id = core.next_id;
        let revision = ConnectionRevision::new(core.next_revision)?;
        let layer_order = request
            .layer_role
            .priority()
            .saturating_mul(1_000_000)
            .saturating_add(id);
        core.surfaces.insert(request.surface_ref.clone(), id);
        core.records.insert(
            id,
            OverlayRecord {
                request: request.clone(),
                revision,
                layer_order,
                placement: None,
                presence: PresenceState::Entered,
                closed: false,
            },
        );
        Ok(OverlayLease {
            core: Rc::downgrade(&self.core),
            id,
            revision,
            view_epoch: request.view_epoch,
        })
    }

    #[must_use]
    pub fn current_revision(&self, id: u64) -> Option<ConnectionRevision> {
        self.core
            .borrow()
            .records
            .get(&id)
            .map(|record| record.revision)
    }

    #[must_use]
    pub fn view_epoch_of(&self, id: u64) -> Option<ViewEpoch> {
        self.core
            .borrow()
            .records
            .get(&id)
            .map(|record| record.request.view_epoch)
    }

    #[must_use]
    pub fn layer_order_of(&self, id: u64) -> Option<u64> {
        self.core
            .borrow()
            .records
            .get(&id)
            .map(|record| record.layer_order)
    }

    pub fn reopen(&mut self, id: u64) -> Result<()> {
        let mut core = self.core.borrow_mut();
        core.ensure_live()?;
        let record = core
            .records
            .get_mut(&id)
            .ok_or(BridgeError::StaleOverlayLease { lease_id: id })?;
        if !record.closed {
            return Ok(());
        }
        record.closed = false;
        record.presence = PresenceState::Entered;
        core.events.retain(|queued| queued.lease_id != id);
        core.push_events(id, vec![OverlayEvent::PresenceEnter])
    }

    pub fn dispose_lease(&mut self, id: u64) {
        self.core.borrow_mut().dispose(id);
    }

    pub fn drain_tagged_events(&mut self) -> Vec<OverlayEventEnvelope> {
        self.core.borrow_mut().events.drain(..).collect()
    }

    pub fn drain_events(&mut self) -> Vec<OverlayEvent> {
        self.drain_tagged_events()
            .into_iter()
            .map(|queued| queued.event)
            .collect()
    }
}

pub struct OverlayLease {
    core: Weak<RefCell<OverlayCore>>,
    id: u64,
    revision: ConnectionRevision,
    view_epoch: ViewEpoch,
}

impl OverlayLease {
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    pub fn update(&self, placement: PlacementSnapshot) -> Result<()> {
        self.update_with_revision(self.revision, placement)
    }

    pub fn update_with_revision(
        &self,
        revision: ConnectionRevision,
        placement: PlacementSnapshot,
    ) -> Result<()> {
        let core = self
            .core
            .upgrade()
            .ok_or(BridgeError::StaleOverlayLease { lease_id: self.id })?;
        let mut core = core.borrow_mut();
        let record = core.validate_lease(self.id, revision)?;
        let received_epoch = placement.view_epoch.unwrap_or(self.view_epoch);
        if received_epoch != record.request.view_epoch {
            return Err(BridgeError::StaleEpoch {
                expected: record.request.view_epoch,
                received: received_epoch,
            });
        }
        placement.anchor_rect.validate("anchor rectangle")?;
        placement.floating_rect.validate("floating rectangle")?;
        placement.viewport.validate("viewport rectangle")?;
        core.push_events(self.id, vec![OverlayEvent::Placement(placement.clone())])?;
        let record = core
            .records
            .get_mut(&self.id)
            .ok_or(BridgeError::StaleOverlayLease { lease_id: self.id })?;
        record.placement = Some(placement);
        Ok(())
    }

    pub fn close(&self, reason: CloseReason) -> Result<()> {
        let core = self
            .core
            .upgrade()
            .ok_or(BridgeError::StaleOverlayLease { lease_id: self.id })?;
        core.borrow_mut().close(self.id, self.revision, reason)
    }

    pub fn dispose(&self) {
        if let Some(core) = self.core.upgrade() {
            core.borrow_mut().dispose(self.id);
        }
    }
}

impl Drop for OverlayLease {
    fn drop(&mut self) {
        self.dispose();
    }
}
