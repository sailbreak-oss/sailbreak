use proto_ui_gpui::{
    AnchorRef, BridgeError, CloseReason, ConnectionRevision, DismissalPolicy, LayerRole,
    OverlayEvent, OverlayHost, OverlayRect, OverlayRequest, OverlaySurfaceRef, PlacementPolicy,
    PlacementSnapshot, Result, Side, SideAlign, ViewEpoch,
};

fn anchor(name: &str) -> AnchorRef {
    AnchorRef::new(format!("anchor-{name}")).expect("anchor ref")
}

fn surface(name: &str) -> OverlaySurfaceRef {
    OverlaySurfaceRef::new(format!("surface-{name}")).expect("surface ref")
}

fn epoch(value: u64) -> ViewEpoch {
    ViewEpoch::new(value).expect("view epoch")
}

fn popper_request(anchor_name: &str, surface_name: &str, view_epoch: ViewEpoch) -> OverlayRequest {
    OverlayRequest::popper(
        anchor(anchor_name),
        surface(surface_name),
        view_epoch,
        LayerRole::SelectContent,
        Side::Bottom,
        SideAlign::Start,
    )
    .expect("popper request")
}

fn request(
    anchor_name: &str,
    surface_name: &str,
    view_epoch: ViewEpoch,
    role: LayerRole,
) -> OverlayRequest {
    OverlayRequest::new(
        anchor(anchor_name),
        surface(surface_name),
        view_epoch,
        role,
        PlacementPolicy::ItemAligned {
            align: SideAlign::Start,
            align_offset: 0.0,
        },
        DismissalPolicy::default(),
    )
    .expect("overlay request")
}

fn placement(floating_height: f32) -> PlacementSnapshot {
    PlacementSnapshot::new(
        OverlayRect::new(10.0, 10.0, 200.0, 30.0),
        OverlayRect::new(10.0, 40.0, 200.0, floating_height),
        OverlayRect::new(0.0, 0.0, 1000.0, 800.0),
        Side::Bottom,
        SideAlign::Start,
    )
}

#[test]
fn portal_attachment_grants_a_live_lease_and_placement_update() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let lease = host.attach(popper_request("attach", "attach", epoch(1)))?;
    assert!(lease.id() != 0);
    assert_eq!(
        host.current_revision(lease.id()),
        Some(ConnectionRevision::new(1)?)
    );
    assert_eq!(host.view_epoch_of(lease.id()), Some(epoch(1)));

    lease.update(placement(120.0))?;
    let events = host.drain_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_placement());

    lease.close(CloseReason::Programmatic)?;
    let events = host.drain_events();
    assert!(events.iter().any(OverlayEvent::is_close));
    Ok(())
}

#[test]
fn replacement_disposes_the_previous_lease_and_bumps_connection_revision() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let first = host.attach(popper_request("replace", "replace", epoch(1)))?;
    let first_id = first.id();
    let first_revision = host.current_revision(first_id).expect("first revision");

    let second = host.attach(popper_request("replace", "replace", epoch(1)))?;
    assert!(second.id() != first_id);
    assert!(host.current_revision(second.id()) > Some(first_revision));
    assert!(host.current_revision(first_id).is_none());

    // The replaced lease rejects further placement updates and closes.
    assert!(matches!(
        first.update(placement(120.0)),
        Err(BridgeError::StaleOverlayLease { .. })
    ));
    assert!(matches!(
        first.close(CloseReason::Programmatic),
        Err(BridgeError::StaleOverlayLease { .. })
    ));

    let events = host.drain_events();
    assert!(events.iter().any(|event| event.is_close()));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(CloseReason::Replaced)))
    );
    Ok(())
}

#[test]
fn stale_connection_revision_placement_is_rejected() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let _prior = host.attach(popper_request("prior", "prior", epoch(1)))?;
    let lease = host.attach(popper_request("stale", "stale", epoch(1)))?;
    lease.update(placement(100.0))?;
    host.drain_events();

    let lease_id = lease.id();
    let expected = host.current_revision(lease_id).expect("current revision");
    let stale = ConnectionRevision::new(expected.get() - 1)?;
    assert!(expected > stale);
    assert!(matches!(
        lease.update_with_revision(stale, placement(90.0)),
        Err(BridgeError::StaleOverlayLease { .. })
    ));
    assert_eq!(host.current_revision(lease_id), Some(expected));
    Ok(())
}

#[test]
fn stale_view_epoch_placement_is_rejected() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let lease = host.attach(popper_request("epoch", "epoch", epoch(3)))?;
    assert!(matches!(
        lease.update(placement_with_epoch(placement(100.0), epoch(2))),
        Err(BridgeError::StaleEpoch { .. })
    ));
    // A current-epoch placement still applies.
    lease.update(placement(100.0))?;
    assert!(host.drain_events().iter().any(OverlayEvent::is_placement));
    Ok(())
}

fn placement_with_epoch(snapshot: PlacementSnapshot, epoch: ViewEpoch) -> PlacementSnapshot {
    snapshot.with_view_epoch(epoch)
}

#[test]
fn layer_order_follows_role_priority_for_sibling_surfaces() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let content = host.attach(request("a", "a", epoch(1), LayerRole::SelectContent))?;
    let mask = host.attach(request("b", "b", epoch(1), LayerRole::DialogMask))?;
    let dialog = host.attach(request("c", "c", epoch(1), LayerRole::DialogContent))?;
    assert!(host.layer_order_of(mask.id()) > host.layer_order_of(content.id()));
    assert!(host.layer_order_of(dialog.id()) > host.layer_order_of(mask.id()));
    Ok(())
}

#[test]
fn shared_event_queue_preserves_lease_attribution() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let first = host.attach(popper_request("tag-a", "tag-a", epoch(1)))?;
    let second = host.attach(popper_request("tag-b", "tag-b", epoch(1)))?;
    first.update(placement(40.0))?;
    second.update(placement(50.0))?;

    let events = host.drain_tagged_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].lease_id, first.id());
    assert_eq!(events[1].lease_id, second.id());
    Ok(())
}

#[test]
fn outside_press_escape_and_focus_outside_emit_semantic_dismissals() -> Result<()> {
    let mut host = OverlayHost::new(16);

    let outside = host.attach(request(
        "outside",
        "outside",
        epoch(1),
        LayerRole::DropdownContent,
    ))?;
    outside.close(CloseReason::OutsidePress)?;

    let escape = host.attach(request(
        "escape",
        "escape",
        epoch(1),
        LayerRole::DropdownContent,
    ))?;
    escape.close(CloseReason::Escape)?;

    let focus = host.attach(request(
        "focus",
        "focus",
        epoch(1),
        LayerRole::HoverCardContent,
    ))?;
    focus.close(CloseReason::FocusOutside)?;

    let events = host.drain_events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(CloseReason::OutsidePress)))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(CloseReason::Escape)))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(CloseReason::FocusOutside)))
    );
    Ok(())
}

#[test]
fn focus_restore_target_is_emitted_exactly_once_per_close() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let lease = host.attach(
        popper_request("focus", "focus", epoch(1)).with_focus_restore_target("trigger-focus"),
    )?;

    lease.close(CloseReason::OutsidePress)?;
    lease.close(CloseReason::Escape)?;
    let events = host.drain_events();
    let restores = events
        .iter()
        .filter(|event| event.is_focus_restore())
        .count();
    assert_eq!(restores, 1);

    // Reopening a fresh lease restores again exactly once.
    let lease = host.attach(
        popper_request("focus", "focus", epoch(1)).with_focus_restore_target("trigger-focus"),
    )?;
    lease.close(CloseReason::Programmatic)?;
    let events = host.drain_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.is_focus_restore())
            .count(),
        1
    );
    Ok(())
}

#[test]
fn close_is_idempotent_and_emits_one_semantic_event() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let lease = host.attach(popper_request("close", "close", epoch(1)))?;
    lease.close(CloseReason::OutsidePress)?;
    lease.close(CloseReason::OutsidePress)?;
    lease.close(CloseReason::Escape)?;
    let events = host.drain_events();
    assert_eq!(events.iter().filter(|event| event.is_close()).count(), 1);
    Ok(())
}

#[test]
fn dispose_is_terminal_and_idempotent() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let lease = host.attach(popper_request("dispose", "dispose", epoch(1)))?;
    let id = lease.id();
    lease.dispose();
    lease.dispose();
    assert!(host.current_revision(id).is_none());
    assert!(matches!(
        lease.update(placement(100.0)),
        Err(BridgeError::StaleOverlayLease { .. })
    ));
    assert!(matches!(
        lease.close(CloseReason::Programmatic),
        Err(BridgeError::StaleOverlayLease { .. })
    ));
    assert!(host.drain_events().is_empty());
    Ok(())
}

#[test]
fn lease_drop_runs_terminal_cleanup_without_events() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let id = {
        let lease = host.attach(popper_request("drop", "drop", epoch(1)))?;
        lease.id()
    };
    assert!(host.current_revision(id).is_none());
    host.dispose_lease(id);
    assert!(host.current_revision(id).is_none());
    assert!(host.drain_events().is_empty());
    Ok(())
}

#[test]
fn bounded_event_queue_overflow_is_fatal() -> Result<()> {
    let mut host = OverlayHost::new(1);
    let lease = host.attach(popper_request("overflow", "overflow", epoch(1)))?;
    // First placement fills the single-slot queue.
    lease.update(placement(50.0))?;
    assert!(matches!(
        lease.update(placement(60.0)),
        Err(BridgeError::OverlayQueueOverflow { .. })
    ));
    // The overflow is fatal: the lease refuses further work.
    assert!(matches!(
        lease.close(CloseReason::Programmatic),
        Err(BridgeError::OverlayQueueOverflow { .. })
    ));
    Ok(())
}

#[test]
fn transition_leave_is_cancelled_by_reopen() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let lease = host.attach(popper_request("transition", "transition", epoch(1)))?;
    lease.close(CloseReason::Escape)?;
    // Reopen cancels the pending leave: presence re-enters, no close completes.
    host.reopen(lease.id())?;
    let events = host.drain_events();
    assert!(!events.iter().any(|event| event.is_close()));
    assert!(events.iter().any(OverlayEvent::is_presence_enter));
    Ok(())
}

#[test]
fn compute_placement_item_aligned_and_popper_are_deterministic() {
    let viewport = OverlayRect::new(0.0, 0.0, 800.0, 600.0);
    let anchor = OverlayRect::new(300.0, 200.0, 200.0, 32.0);

    let item_aligned = PlacementPolicy::ItemAligned {
        align: SideAlign::Start,
        align_offset: 0.0,
    };
    let snapshot = item_aligned
        .compute_placement(anchor, (200.0, 240.0), viewport)
        .expect("item-aligned placement");
    assert_eq!(snapshot.floating_rect.width, 200.0);
    assert_eq!(snapshot.floating_rect.y, anchor.y + anchor.height);
    assert_eq!(snapshot.available_height, 368.0);

    let popper = PlacementPolicy::Popper {
        side: Side::Bottom,
        side_offset: 4.0,
        align: SideAlign::Start,
        align_offset: 0.0,
        avoid_collisions: true,
        collision_padding: 8.0,
    };
    let snapshot = popper
        .compute_placement(anchor, (260.0, 200.0), viewport)
        .expect("popper placement");
    assert_eq!(snapshot.side, Side::Bottom);
    assert_eq!(snapshot.floating_rect.y, anchor.y + anchor.height + 4.0);

    // Near the right edge the popper flips or shifts to stay on-screen.
    let edge_anchor = OverlayRect::new(700.0, 200.0, 100.0, 32.0);
    let edge = popper
        .compute_placement(edge_anchor, (260.0, 200.0), viewport)
        .expect("edge popper placement");
    assert!(edge.floating_rect.x >= 0.0);
    assert!(edge.floating_rect.right() <= viewport.right());
    assert!(edge.flipped || edge.shifted);
}

#[test]
fn connection_revision_is_monotonic_across_replacement() -> Result<()> {
    let mut host = OverlayHost::new(16);
    let first = host.attach(popper_request("mono", "mono", epoch(1)))?;
    let first_revision = host.current_revision(first.id()).expect("revision");
    let second = host.attach(popper_request("mono", "mono", epoch(2)))?;
    let second_revision = host.current_revision(second.id()).expect("revision");
    assert!(second_revision > first_revision);
    assert_eq!(host.view_epoch_of(second.id()), Some(epoch(2)));
    Ok(())
}

#[test]
fn identities_validate_and_reject_empty_refs() {
    assert!(AnchorRef::new("  ").is_err());
    assert!(OverlaySurfaceRef::new("").is_err());
    assert!(matches!(
        AnchorRef::new("").err(),
        Some(BridgeError::InvalidAnchorRef { .. })
    ));
    assert!(matches!(
        OverlaySurfaceRef::new("").err(),
        Some(BridgeError::InvalidSurfaceRef { .. })
    ));
    assert!(matches!(
        ConnectionRevision::new(0),
        Err(BridgeError::InvalidRevision)
    ));
    assert!(ConnectionRevision::new(7).is_ok());
}
