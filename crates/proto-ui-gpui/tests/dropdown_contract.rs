use proto_ui_gpui::{
    BridgeError, CloseReason, DropdownContentProps, DropdownItemProps, DropdownRootProps,
    DropdownTriggerProps, FocusOperationResult, InputSource, OverlayEvent, OverlayRect,
    PlacementSnapshot, ProtoDropdownHost, Side, SideAlign,
};

fn item(value: &str, text_value: &str, disabled: bool) -> DropdownItemProps {
    DropdownItemProps {
        value: value.to_owned(),
        text_value: text_value.to_owned(),
        disabled,
        ..DropdownItemProps::default()
    }
}

fn dropdown(name: &str) -> Result<ProtoDropdownHost, BridgeError> {
    let mut host = ProtoDropdownHost::new()?;
    let root_id = format!("{name}-root");
    let trigger_id = format!("{name}-trigger");
    let content_id = format!("{name}-content");
    host.register_root(
        root_id.clone(),
        format!("{name} actions"),
        DropdownRootProps::default(),
    )?;
    host.register_trigger(
        trigger_id.clone(),
        &root_id,
        DropdownTriggerProps::default(),
    )?;
    host.register_content(
        content_id.clone(),
        &root_id,
        DropdownContentProps::default(),
    )?;
    host.register_item(
        format!("{name}-item-first"),
        "First",
        &content_id,
        item("first", "First", false),
    )?;
    host.register_item(
        format!("{name}-item-disabled"),
        "Disabled",
        &content_id,
        item("disabled", "Disabled", true),
    )?;
    host.register_item(
        format!("{name}-item-last"),
        "Last",
        &content_id,
        item("last", "Last", false),
    )?;
    host.setup()?;
    Ok(host)
}

#[test]
fn complete_graph_is_registered_before_runtime_setup_and_projects_menu_parts()
-> Result<(), BridgeError> {
    let mut host = ProtoDropdownHost::new()?;
    host.register_root("root", "Actions", DropdownRootProps::default())?;
    host.register_trigger("trigger", "root", DropdownTriggerProps::default())?;
    host.register_content("content", "root", DropdownContentProps::default())?;
    host.register_item("first", "First", "content", item("first", "First", false))?;
    assert!(host.snapshot().is_err());

    host.setup()?;
    let snapshot = host.snapshot()?;
    assert_eq!(
        snapshot.root.session.prototype.as_str(),
        "shadcn-dropdown-root"
    );
    assert_eq!(
        snapshot
            .trigger
            .as_ref()
            .expect("trigger")
            .session
            .prototype
            .as_str(),
        "shadcn-dropdown-trigger"
    );
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .session
            .prototype
            .as_str(),
        "shadcn-dropdown-content"
    );
    assert_eq!(
        snapshot.items[0].session.prototype.as_str(),
        "shadcn-dropdown-item"
    );
    assert_ne!(
        snapshot.root.session.instance_id,
        snapshot.items[0].session.instance_id
    );
    Ok(())
}

#[test]
fn open_close_and_presence_are_owned_by_proto_and_overlay() -> Result<(), BridgeError> {
    let mut host = dropdown("presence")?;
    assert!(!host.is_open()?);
    assert!(!host.content("presence-content")?.present);
    let opened = host.open()?;
    assert_eq!(opened.open_change_count, 1);
    assert!(host.is_open()?);
    assert!(host.content("presence-content")?.present);
    let lease = host.overlay_lease_id().expect("portal lease");

    let closed = host.close(CloseReason::Programmatic)?;
    assert_eq!(closed.open_change_count, 1);
    assert!(!host.is_open()?);
    assert!(!host.content("presence-content")?.present);
    assert_eq!(host.overlay_lease_id(), Some(lease));
    assert_eq!(host.open()?.open_change_count, 1);
    Ok(())
}

#[test]
fn keyboard_navigation_home_end_and_disabled_items_are_proto_owned() -> Result<(), BridgeError> {
    let mut host = dropdown("keys")?;
    host.open()?;
    assert_eq!(host.active_value()?, "first");
    host.dispatch_key("ArrowDown")?;
    assert_eq!(host.active_value()?, "disabled");
    host.dispatch_key("ArrowDown")?;
    assert_eq!(host.active_value()?, "last");
    host.dispatch_key("Home")?;
    assert_eq!(host.active_value()?, "first");
    host.dispatch_key("End")?;
    assert_eq!(host.active_value()?, "last");
    assert!(host.item("keys-item-disabled")?.disabled);
    Ok(())
}

#[test]
fn escape_and_outside_press_dismiss_once_and_restore_trigger_focus() -> Result<(), BridgeError> {
    let mut host = dropdown("dismiss")?;
    host.set_focus_ready("dismiss-trigger", true)?;
    assert_eq!(
        host.focus("dismiss-trigger")?,
        FocusOperationResult::Accepted
    );
    host.open()?;
    host.dismiss_escape()?;
    assert!(!host.is_open()?);
    assert!(host.trigger("dismiss-trigger")?.focused);
    let events = host.drain_overlay_events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(CloseReason::Escape)))
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.is_focus_restore())
            .count(),
        1
    );

    host.open()?;
    host.dismiss_outside()?;
    assert!(!host.is_open()?);
    assert!(host.trigger("dismiss-trigger")?.focused);
    Ok(())
}

#[test]
fn item_activation_emits_one_select_signal_and_closes_once() -> Result<(), BridgeError> {
    let mut host = dropdown("activate")?;
    host.open()?;
    let outcome = host.press_item("activate-item-last", InputSource::Accessibility)?;
    assert_eq!(outcome.item_select_count, 1);
    assert_eq!(outcome.selection_count(), 1);
    assert_eq!(outcome.selected_values, vec!["last"]);
    assert_eq!(outcome.open_change_count, 1);
    assert!(!host.is_open()?);
    let duplicate = host.press_item("activate-item-last", InputSource::Accessibility)?;
    assert_eq!(duplicate.item_select_count, 0);
    assert_eq!(duplicate.open_change_count, 0);
    Ok(())
}

#[test]
fn menu_and_menuitem_a11y_roles_are_projected() -> Result<(), BridgeError> {
    let mut host = dropdown("a11y")?;
    let snapshot = host.snapshot()?;
    assert_eq!(
        snapshot
            .trigger
            .as_ref()
            .expect("trigger")
            .a11y
            .as_ref()
            .expect("button")
            .role,
        "button"
    );
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .a11y
            .as_ref()
            .expect("menu")
            .role,
        "menu"
    );
    assert!(
        snapshot
            .items
            .iter()
            .all(|item| item.a11y.as_ref().expect("menuitem").role == "menuitem")
    );
    Ok(())
}

#[test]
fn stale_portal_epoch_and_revision_are_rejected_after_content_remount() -> Result<(), BridgeError> {
    let mut host = dropdown("stale")?;
    host.open()?;
    let old_revision = host.overlay_revision().expect("revision");
    let old_epoch = host.content("stale-content")?.session.projection.view_epoch;
    host.remount_content("stale-content")?;
    let new_epoch = host.content("stale-content")?.session.projection.view_epoch;
    assert!(new_epoch > old_epoch);
    host.open()?;
    assert!(host.overlay_revision().expect("new revision") > old_revision);
    let placement = PlacementSnapshot::new(
        OverlayRect::new(1.0, 1.0, 10.0, 10.0),
        OverlayRect::new(1.0, 11.0, 10.0, 10.0),
        OverlayRect::new(0.0, 0.0, 100.0, 100.0),
        Side::Bottom,
        SideAlign::Start,
    );
    assert!(
        host.update_placement_with_revision(old_revision, placement)
            .is_err()
    );
    Ok(())
}

#[test]
fn remount_and_dispose_are_idempotent_and_isolate_families() -> Result<(), BridgeError> {
    let mut first = dropdown("first")?;
    let mut second = dropdown("second")?;
    assert_ne!(first.family_route(), second.family_route());
    first.open()?;
    first.dispatch_key("ArrowDown")?;
    assert_eq!(first.active_value()?, "disabled");
    assert_eq!(second.active_value()?, "");

    let before = first
        .trigger("first-trigger")?
        .session
        .projection
        .view_epoch;
    first.remount_trigger("first-trigger")?;
    assert!(
        first
            .trigger("first-trigger")?
            .session
            .projection
            .view_epoch
            > before
    );
    assert_eq!(
        first.focus("first-trigger")?,
        FocusOperationResult::NotReady
    );

    first.dispose()?;
    first.dispose()?;
    assert!(first.snapshot().is_err());
    assert!(first.item("first-item-last").is_err());
    assert_eq!(second.active_value()?, "");
    Ok(())
}

#[test]
fn disabled_root_and_items_never_emit_activation() -> Result<(), BridgeError> {
    let mut host = dropdown("disabled")?;
    host.set_root_props(DropdownRootProps {
        disabled: true,
        ..DropdownRootProps::default()
    })?;
    let open = host.open()?;
    assert_eq!(open.open_change_count, 0);
    let outcome = host.press_item("disabled-item-last", InputSource::Keyboard)?;
    assert_eq!(outcome.item_select_count, 0);
    assert_eq!(outcome.open_change_count, 0);
    Ok(())
}
