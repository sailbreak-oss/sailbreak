use proto_ui_gpui::{
    BridgeError, CloseReason, FocusOperationResult, InputSource, OverlayEvent, OverlayRect,
    PlacementSnapshot, ProtoSelectHost, SelectContentProps, SelectItemProps, SelectPosition,
    SelectRootProps, SelectTriggerProps, SelectValueProps, Side, SideAlign, TemplateNode,
};

fn root_props() -> SelectRootProps {
    SelectRootProps {
        default_value: "balanced".to_owned(),
        ..SelectRootProps::default()
    }
}

fn item(value: &str, text_value: &str, disabled: bool) -> SelectItemProps {
    SelectItemProps {
        value: value.to_owned(),
        text_value: text_value.to_owned(),
        disabled,
        ..SelectItemProps::default()
    }
}

fn select(name: &str, position: SelectPosition) -> Result<ProtoSelectHost, BridgeError> {
    select_with_root(name, position, root_props())
}

fn select_with_root(
    name: &str,
    position: SelectPosition,
    root: SelectRootProps,
) -> Result<ProtoSelectHost, BridgeError> {
    let mut host = ProtoSelectHost::new()?;
    host.register_root(format!("{name}-root"), format!("{name} mode"), root)?;
    host.register_trigger(
        format!("{name}-trigger"),
        &format!("{name}-root"),
        SelectTriggerProps::default(),
    )?;
    host.register_value(
        format!("{name}-value"),
        &format!("{name}-root"),
        SelectValueProps {
            placeholder: "Choose a mode".to_owned(),
        },
    )?;
    host.register_content(
        format!("{name}-content"),
        &format!("{name}-root"),
        SelectContentProps {
            position,
            ..SelectContentProps::default()
        },
    )?;
    host.register_item(
        format!("{name}-item-balanced"),
        "Balanced",
        &format!("{name}-content"),
        item("balanced", "Balanced", false),
    )?;
    host.register_item(
        format!("{name}-item-quiet"),
        "Quiet",
        &format!("{name}-content"),
        item("quiet", "Quiet mode", true),
    )?;
    host.register_item(
        format!("{name}-item-performance"),
        "Performance",
        &format!("{name}-content"),
        item("performance", "Performance mode", false),
    )?;
    host.setup()?;
    Ok(host)
}

fn has_check(nodes: &[TemplateNode]) -> bool {
    nodes.iter().any(|node| match node {
        TemplateNode::Svg {
            tag,
            attributes,
            children,
        } => {
            (tag == "path"
                && attributes
                    .get("d")
                    .is_some_and(|value| value == "m20 6-11 11-5-5"))
                || has_check(children)
        }
        TemplateNode::Container { children, .. } => has_check(children),
        TemplateNode::Text { .. } | TemplateNode::Slot { .. } => false,
    })
}

#[test]
fn graph_is_registered_before_runtime_setup_and_projects_all_parts() -> Result<(), BridgeError> {
    let mut host = ProtoSelectHost::new()?;
    host.register_root("root", "Mode", root_props())?;
    host.register_trigger("trigger", "root", SelectTriggerProps::default())?;
    host.register_value(
        "value",
        "root",
        SelectValueProps {
            placeholder: "Choose".to_owned(),
        },
    )?;
    host.register_content("content", "root", SelectContentProps::default())?;
    host.register_item("first", "First", "content", item("first", "First", false))?;
    assert!(host.snapshot().is_err());

    host.setup()?;
    let snapshot = host.snapshot()?;
    assert_eq!(
        snapshot.root.session.prototype.as_str(),
        "shadcn-select-root"
    );
    assert_eq!(
        snapshot
            .trigger
            .as_ref()
            .expect("trigger")
            .session
            .prototype
            .as_str(),
        "shadcn-select-trigger"
    );
    assert_eq!(
        snapshot
            .value
            .as_ref()
            .expect("value")
            .session
            .prototype
            .as_str(),
        "shadcn-select-value"
    );
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .session
            .prototype
            .as_str(),
        "shadcn-select-content"
    );
    assert_eq!(
        snapshot.items[0].session.prototype.as_str(),
        "shadcn-select-item"
    );
    assert_ne!(
        snapshot.root.session.instance_id,
        snapshot.items[0].session.instance_id
    );
    Ok(())
}

#[test]
fn controlled_and_uncontrolled_values_keep_placeholder_and_display_text_in_sync()
-> Result<(), BridgeError> {
    let mut uncontrolled = select("uncontrolled", SelectPosition::ItemAligned)?;
    assert_eq!(uncontrolled.selected_value()?, "balanced");
    assert_eq!(uncontrolled.display_value()?, "Balanced");

    uncontrolled.open()?;
    let selected =
        uncontrolled.press_item("uncontrolled-item-performance", InputSource::Keyboard)?;
    assert_eq!(selected.item_select_count, 1);
    assert_eq!(selected.value_change_count, 1);
    assert_eq!(selected.selected_values, vec!["performance"]);
    assert_eq!(uncontrolled.selected_value()?, "performance");
    assert_eq!(uncontrolled.display_value()?, "Performance mode");

    let mut controlled = select("controlled", SelectPosition::ItemAligned)?;
    let mut props = root_props();
    props.value = Some("balanced".to_owned());
    controlled.set_root_props(props.clone())?;
    controlled.open()?;
    let outcome = controlled.press_item("controlled-item-performance", InputSource::Keyboard)?;
    assert_eq!(outcome.value_change_count, 1);
    assert_eq!(controlled.selected_value()?, "balanced");
    props.value = Some("performance".to_owned());
    controlled.set_root_props(props)?;
    assert_eq!(controlled.display_value()?, "Performance mode");

    let mut empty = select_with_root(
        "empty",
        SelectPosition::ItemAligned,
        SelectRootProps::default(),
    )?;
    assert_eq!(empty.display_value()?, "Choose a mode");
    Ok(())
}

#[test]
fn keyboard_navigation_typeahead_and_disabled_items_are_proto_owned() -> Result<(), BridgeError> {
    let mut host = select("keys", SelectPosition::ItemAligned)?;
    host.open()?;
    assert_eq!(host.active_value()?, "balanced");
    host.dispatch_key("ArrowDown")?;
    assert_eq!(host.active_value()?, "performance");
    host.dispatch_key("ArrowDown")?;
    assert_eq!(host.active_value()?, "performance");
    host.dispatch_key("Home")?;
    assert_eq!(host.active_value()?, "balanced");
    host.dispatch_key("End")?;
    assert_eq!(host.active_value()?, "performance");
    host.dispatch_key("q")?;
    assert_eq!(host.active_value()?, "performance");
    host.dispatch_key("p")?;
    assert_eq!(host.active_value()?, "performance");
    assert!(host.item("keys-item-quiet")?.disabled);
    Ok(())
}

#[test]
fn selected_indicator_and_combobox_listbox_option_a11y_are_projected() -> Result<(), BridgeError> {
    let mut host = select("a11y", SelectPosition::ItemAligned)?;
    let snapshot = host.snapshot()?;
    let trigger = snapshot.trigger.as_ref().expect("trigger");
    assert_eq!(trigger.a11y.as_ref().expect("combobox").role, "combobox");
    assert_eq!(trigger.a11y.as_ref().expect("combobox").name, "a11y mode");
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .a11y
            .as_ref()
            .expect("listbox")
            .role,
        "listbox"
    );
    let selected = snapshot
        .items
        .iter()
        .find(|item| item.selected)
        .expect("selected item");
    assert_eq!(selected.a11y.as_ref().expect("option").role, "option");
    assert_eq!(selected.a11y.as_ref().expect("option").selected, Some(true));
    assert!(selected.selected_indicator);
    assert!(has_check(&selected.session.projection.template));
    Ok(())
}

#[test]
fn item_aligned_and_popper_placement_stays_in_overlay_host() -> Result<(), BridgeError> {
    let mut item_aligned = select("aligned", SelectPosition::ItemAligned)?;
    item_aligned.open()?;
    let placement = item_aligned.set_anchor_geometry(
        OverlayRect::new(10.0, 20.0, 120.0, 30.0),
        (120.0, 100.0),
        OverlayRect::new(0.0, 0.0, 400.0, 400.0),
    )?;
    assert_eq!(placement.floating_rect.x, 10.0);
    assert_eq!(placement.floating_rect.y, 50.0);
    assert_eq!(
        item_aligned.content("aligned-content")?.placement,
        Some(placement)
    );

    let mut popper = select("popper", SelectPosition::Popper)?;
    popper.open()?;
    let placement = popper.set_anchor_geometry(
        OverlayRect::new(10.0, 20.0, 120.0, 30.0),
        (120.0, 100.0),
        OverlayRect::new(0.0, 0.0, 400.0, 400.0),
    )?;
    assert_eq!(placement.side, Side::Bottom);
    assert_eq!(placement.align, SideAlign::Center);
    assert_eq!(
        popper.content("popper-content")?.portal_lease_id,
        popper.overlay_lease_id()
    );
    Ok(())
}

#[test]
fn open_close_presence_portal_and_trigger_focus_restore_are_epoch_safe() -> Result<(), BridgeError>
{
    let mut host = select("lifecycle", SelectPosition::ItemAligned)?;
    host.set_focus_ready("lifecycle-trigger", true)?;
    assert_eq!(
        host.focus("lifecycle-trigger")?,
        FocusOperationResult::Accepted
    );
    host.open()?;
    assert!(host.content("lifecycle-content")?.present);
    let lease_id = host.overlay_lease_id().expect("portal lease");
    assert!(host.drain_overlay_events().is_empty());

    host.close(CloseReason::Escape)?;
    assert!(!host.content("lifecycle-content")?.present);
    assert_eq!(host.overlay_lease_id(), Some(lease_id));
    assert!(host.trigger("lifecycle-trigger")?.focused);
    assert!(
        host.drain_overlay_events()
            .iter()
            .any(|event| matches!(event, OverlayEvent::Close(CloseReason::Escape)))
    );

    let stale_revision = host.overlay_revision().expect("old revision");
    let before = host
        .content("lifecycle-content")?
        .session
        .projection
        .view_epoch;
    host.remount_content("lifecycle-content")?;
    let after = host
        .content("lifecycle-content")?
        .session
        .projection
        .view_epoch;
    assert!(after > before);
    host.open()?;
    assert!(host.overlay_revision().expect("new revision") > stale_revision);
    assert!(
        host.update_placement_with_revision(
            stale_revision,
            PlacementSnapshot::new(
                OverlayRect::new(1.0, 1.0, 10.0, 10.0),
                OverlayRect::new(1.0, 11.0, 10.0, 10.0),
                OverlayRect::new(0.0, 0.0, 100.0, 100.0),
                Side::Bottom,
                SideAlign::Start,
            )
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn exact_once_selection_remount_dispose_and_family_isolation_hold() -> Result<(), BridgeError> {
    let mut first = select("first", SelectPosition::ItemAligned)?;
    let mut second = select("second", SelectPosition::ItemAligned)?;
    assert_ne!(first.family_route(), second.family_route());
    first.open()?;
    let outcome = first.press_item("first-item-performance", InputSource::Accessibility)?;
    assert_eq!(outcome.item_select_count, 1);
    assert_eq!(outcome.value_change_count, 1);
    assert_eq!(first.selected_value()?, "performance");
    assert_eq!(second.selected_value()?, "balanced");

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
    assert!(first.snapshot().is_err());
    assert!(first.item("first-item-performance").is_err());
    Ok(())
}

#[test]
fn controlled_value_and_disabled_updates_are_atomic_for_the_root_session() -> Result<(), BridgeError>
{
    let mut host = select("atomic", SelectPosition::ItemAligned)?;
    let mut props = root_props();
    props.value = Some("balanced".to_owned());
    props.disabled = true;
    host.set_root_props(props.clone())?;
    let snapshot = host.snapshot()?;
    assert!(snapshot.root.disabled);
    assert!(snapshot.trigger.as_ref().expect("trigger").disabled);
    assert!(snapshot.items.iter().all(|item| item.disabled));
    let outcome = host.press_item("atomic-item-performance", InputSource::Keyboard)?;
    assert_eq!(outcome.item_select_count, 0);
    assert_eq!(outcome.value_change_count, 0);
    assert_eq!(host.selected_value()?, "balanced");
    Ok(())
}
