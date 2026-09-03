use proto_ui_gpui::{
    BridgeError, CloseReason, FocusOperationResult, HoverCardContentProps, HoverCardRootProps,
    HoverCardSnapshot, HoverCardTriggerProps, InputKind, InputSource, LayerRole, OverlayEvent,
    OverlayRect, PlacementSnapshot, ProtoHoverCardHost, Side, SideAlign,
};

fn hover_card(name: &str) -> Result<ProtoHoverCardHost, BridgeError> {
    hover_card_with_props(
        name,
        HoverCardRootProps::default().with_delays(10, 20),
        HoverCardContentProps::default(),
    )
}

fn hover_card_with_props(
    name: &str,
    root_props: HoverCardRootProps,
    content_props: HoverCardContentProps,
) -> Result<ProtoHoverCardHost, BridgeError> {
    let mut host = ProtoHoverCardHost::new()?;
    let root = format!("{name}-root");
    let trigger = format!("{name}-trigger");
    let content = format!("{name}-content");
    host.register_root(root.clone(), format!("{name} details"), root_props)?;
    host.register_trigger(trigger, &root, HoverCardTriggerProps::default())?;
    host.register_content(content, &root, content_props)?;
    host.setup()?;
    Ok(host)
}

fn open(host: &mut ProtoHoverCardHost, trigger: &str) -> Result<(), BridgeError> {
    host.dispatch_trigger(trigger, InputKind::PointerEnter, InputSource::Mouse, None)?;
    host.advance_time(10)?;
    Ok(())
}

fn placement() -> PlacementSnapshot {
    PlacementSnapshot::new(
        OverlayRect::new(20.0, 30.0, 100.0, 24.0),
        OverlayRect::new(20.0, 58.0, 180.0, 80.0),
        OverlayRect::new(0.0, 0.0, 600.0, 400.0),
        Side::Bottom,
        SideAlign::Center,
    )
}

#[test]
fn complete_graph_is_deferred_until_setup_and_projects_root_trigger_content()
-> Result<(), BridgeError> {
    let mut host = ProtoHoverCardHost::new()?;
    host.register_root("root", "Capability details", HoverCardRootProps::default())?;
    host.register_trigger("trigger", "root", HoverCardTriggerProps::default())?;
    host.register_content("content", "root", HoverCardContentProps::default())?;
    assert!(host.snapshot().is_err());

    host.setup()?;
    let snapshot = host.snapshot()?;
    assert_eq!(
        snapshot.root.session.prototype.as_str(),
        "shadcn-hover-card-root"
    );
    assert_eq!(
        snapshot
            .trigger
            .as_ref()
            .expect("trigger")
            .session
            .prototype
            .as_str(),
        "shadcn-hover-card-trigger"
    );
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .session
            .prototype
            .as_str(),
        "shadcn-hover-card-content"
    );
    assert_ne!(
        snapshot.root.session.instance_id,
        snapshot
            .content
            .as_ref()
            .expect("content")
            .session
            .instance_id
    );
    assert!(!snapshot.root.open);
    assert!(!snapshot.content.as_ref().expect("content").present);
    Ok(())
}

#[test]
fn pointer_entry_uses_proto_delay_and_exact_boundary() -> Result<(), BridgeError> {
    let mut host = hover_card("pointer")?;
    let entered = host.dispatch_trigger(
        "pointer-trigger",
        InputKind::PointerEnter,
        InputSource::Mouse,
        None,
    )?;
    assert_eq!(entered.open_change_count, 0);
    assert!(!host.is_open()?);

    assert_eq!(host.advance_time(9)?.open_change_count, 0);
    assert!(!host.is_open()?);
    assert_eq!(host.advance_time(1)?.open_change_count, 1);
    assert!(host.is_open()?);
    assert!(host.content("pointer-content")?.present);
    assert_eq!(host.overlay_layer_role(), Some(LayerRole::HoverCardContent));
    Ok(())
}

#[test]
fn focus_entry_forwards_native_fact_and_opens_after_the_same_delay() -> Result<(), BridgeError> {
    let mut host = hover_card("focus")?;
    host.set_focus_ready("focus-trigger", true)?;
    assert_eq!(
        host.focus_with_source("focus-trigger", InputSource::Keyboard)?,
        FocusOperationResult::Accepted
    );
    assert!(host.trigger("focus-trigger")?.focused);
    assert!(!host.is_open()?);
    host.advance_time(9)?;
    assert!(!host.is_open()?);
    host.advance_time(1)?;
    assert!(host.is_open()?);
    assert!(host.trigger("focus-trigger")?.focused);
    Ok(())
}

#[test]
fn pointer_leave_uses_close_delay_and_boundary() -> Result<(), BridgeError> {
    let mut host = hover_card("leave")?;
    open(&mut host, "leave-trigger")?;
    host.dispatch_trigger(
        "leave-trigger",
        InputKind::PointerLeave,
        InputSource::Mouse,
        None,
    )?;
    assert_eq!(host.advance_time(19)?.open_change_count, 0);
    assert!(host.is_open()?);
    assert_eq!(host.advance_time(1)?.open_change_count, 1);
    assert!(!host.is_open()?);
    assert!(!host.content("leave-content")?.present);
    Ok(())
}

#[test]
fn content_entry_cancels_close_and_explicit_outside_dismissal_closes_once()
-> Result<(), BridgeError> {
    let mut host = hover_card("outside")?;
    open(&mut host, "outside-trigger")?;
    host.dispatch_trigger(
        "outside-trigger",
        InputKind::PointerLeave,
        InputSource::Mouse,
        None,
    )?;
    host.advance_time(10)?;
    host.dispatch_content(
        "outside-content",
        InputKind::PointerEnter,
        InputSource::Mouse,
        None,
    )?;
    host.advance_time(20)?;
    assert!(host.is_open()?);

    let dismissed = host.dismiss_outside()?;
    assert_eq!(dismissed.open_change_count, 1);
    assert!(!host.is_open()?);
    let events = host.drain_overlay_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, OverlayEvent::Close(CloseReason::OutsidePress)))
            .count(),
        1
    );
    let duplicate = host.dismiss_outside()?;
    assert_eq!(duplicate.open_change_count, 0);
    Ok(())
}

#[test]
fn placement_is_rust_owned_and_content_uses_one_hover_card_lease() -> Result<(), BridgeError> {
    let mut host = hover_card_with_props(
        "placement",
        HoverCardRootProps::default().with_delays(0, 0),
        HoverCardContentProps {
            side: Side::Top,
            align: SideAlign::End,
            side_offset: 8.0,
            align_offset: 3.0,
            avoid_collisions: true,
            collision_padding: 4.0,
        },
    )?;
    let computed = host.set_anchor_geometry(
        OverlayRect::new(20.0, 100.0, 100.0, 24.0),
        (180.0, 80.0),
        OverlayRect::new(0.0, 0.0, 600.0, 400.0),
    )?;
    assert_eq!(computed.side, Side::Top);
    assert!(host.overlay_lease_id().is_none());
    open(&mut host, "placement-trigger")?;
    assert_eq!(host.content("placement-content")?.placement, Some(computed));
    assert_eq!(
        host.content("placement-content")?.portal_lease_id,
        host.overlay_lease_id()
    );
    assert_eq!(host.overlay_layer_role(), Some(LayerRole::HoverCardContent));

    let explicit = placement();
    host.update_placement(explicit.clone())?;
    assert_eq!(host.content("placement-content")?.placement, Some(explicit));
    Ok(())
}

#[test]
fn reopening_before_close_delay_cancels_pending_transition() -> Result<(), BridgeError> {
    let mut host = hover_card("cancel")?;
    open(&mut host, "cancel-trigger")?;
    host.dispatch_trigger(
        "cancel-trigger",
        InputKind::PointerLeave,
        InputSource::Mouse,
        None,
    )?;
    host.advance_time(19)?;
    host.dispatch_trigger(
        "cancel-trigger",
        InputKind::PointerEnter,
        InputSource::Mouse,
        None,
    )?;
    host.advance_time(1)?;
    assert!(host.is_open()?);
    assert!(
        !host
            .drain_overlay_events()
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(_)))
    );
    Ok(())
}

#[test]
fn remount_replaces_stale_epoch_and_revision_and_dispose_is_terminal() -> Result<(), BridgeError> {
    let mut host = hover_card("lifecycle")?;
    open(&mut host, "lifecycle-trigger")?;
    let old_epoch = host
        .content("lifecycle-content")?
        .session
        .projection
        .view_epoch;
    let old_revision = host.overlay_revision().expect("overlay revision");
    host.remount_content("lifecycle-content")?;
    let new_epoch = host
        .content("lifecycle-content")?
        .session
        .projection
        .view_epoch;
    assert!(new_epoch > old_epoch);
    let new_revision = host.overlay_revision().expect("new overlay revision");
    assert!(new_revision > old_revision);
    assert!(
        host.complete_close(old_revision, CloseReason::Programmatic)
            .is_err()
    );

    host.set_focus_ready("lifecycle-trigger", true)?;
    host.remount_trigger("lifecycle-trigger")?;
    assert_eq!(
        host.focus("lifecycle-trigger")?,
        FocusOperationResult::NotReady
    );
    host.dispose()?;
    host.dispose()?;
    assert!(host.snapshot().is_err());
    assert!(
        host.dispatch_trigger(
            "lifecycle-trigger",
            InputKind::PointerEnter,
            InputSource::Mouse,
            None
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn families_and_open_close_signals_are_isolated_and_not_duplicated() -> Result<(), BridgeError> {
    let mut first = hover_card("first")?;
    let mut second = hover_card("second")?;
    assert_ne!(first.family_route(), second.family_route());

    first.dispatch_trigger(
        "first-trigger",
        InputKind::PointerEnter,
        InputSource::Mouse,
        None,
    )?;
    assert_eq!(first.advance_time(10)?.open_change_count, 1);
    assert!(first.is_open()?);
    assert!(!second.is_open()?);

    first.dispatch_trigger(
        "first-trigger",
        InputKind::PointerEnter,
        InputSource::Mouse,
        None,
    )?;
    assert_eq!(first.advance_time(10)?.open_change_count, 0);
    let closed = first.close(CloseReason::Programmatic)?;
    assert_eq!(closed.open_change_count, 1);
    let duplicate = first.close(CloseReason::Programmatic)?;
    assert_eq!(duplicate.open_change_count, 0);
    assert!(!first.is_open()?);
    assert!(!second.is_open()?);
    Ok(())
}

#[test]
fn rust_owned_slot_detail_is_projected_without_replacing_proto_template() -> Result<(), BridgeError>
{
    let mut host = ProtoHoverCardHost::new()?;
    host.register_root("root", "battery.status", HoverCardRootProps::default())?;
    host.register_trigger("trigger", "root", HoverCardTriggerProps::default())?;
    host.register_content_with_slot(
        "content",
        "root",
        "UNAVAILABLE: battery telemetry is not exposed by this platform",
        HoverCardContentProps::default(),
    )?;
    host.setup()?;
    let snapshot: HoverCardSnapshot = host.snapshot()?;
    let content = snapshot.content.expect("content");
    assert_eq!(
        content.slot.accessible_name,
        "UNAVAILABLE: battery telemetry is not exposed by this platform"
    );
    assert!(
        content
            .session
            .projection
            .template
            .iter()
            .any(|node| matches!(node, proto_ui_gpui::TemplateNode::Slot { .. }))
    );
    Ok(())
}

#[test]
fn disabled_root_does_not_schedule_an_open_request() -> Result<(), BridgeError> {
    let mut host = hover_card_with_props(
        "disabled",
        HoverCardRootProps {
            disabled: true,
            ..HoverCardRootProps::default().with_delays(0, 0)
        },
        HoverCardContentProps::default(),
    )?;
    host.dispatch_trigger(
        "disabled-trigger",
        InputKind::PointerEnter,
        InputSource::Mouse,
        None,
    )?;
    assert_eq!(host.advance_time(0)?.open_change_count, 0);
    assert!(!host.is_open()?);
    Ok(())
}
