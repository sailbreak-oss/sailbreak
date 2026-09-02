use proto_ui_gpui::{
    BridgeError, FocusOperationResult, InputSource, ProtoTabsHost, TabsActivationMode,
    TabsContentProps, TabsListProps, TabsOrientation, TabsRootProps, TabsTriggerProps,
};

fn root_props(activation_mode: TabsActivationMode) -> TabsRootProps {
    TabsRootProps {
        value: None,
        default_value: "overview".to_owned(),
        orientation: TabsOrientation::Horizontal,
        activation_mode,
    }
}

fn list_props(loop_navigation: bool) -> TabsListProps {
    TabsListProps::default().with_loop(loop_navigation)
}

fn trigger(value: &str, disabled: bool) -> TabsTriggerProps {
    TabsTriggerProps {
        value: value.to_owned(),
        disabled,
    }
}

fn content(value: &str) -> TabsContentProps {
    TabsContentProps {
        value: value.to_owned(),
        keep_mounted: false,
    }
}

fn tabs(
    name: &str,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
) -> Result<ProtoTabsHost, BridgeError> {
    let mut host = ProtoTabsHost::new()?;
    host.register_root(
        format!("{name}-root"),
        format!("{name} tabs"),
        root_props(activation_mode),
    )?;
    host.register_list(
        format!("{name}-list"),
        &format!("{name}-root"),
        list_props(loop_navigation),
    )?;
    for value in ["overview", "power", "diagnostics"] {
        host.register_trigger(
            format!("{name}-trigger-{value}"),
            value.to_uppercase(),
            &format!("{name}-list"),
            trigger(value, value == "power" && name == "disabled"),
        )?;
        host.register_content(
            format!("{name}-content-{value}"),
            &format!("{name}-root"),
            content(value),
        )?;
    }
    host.setup()?;
    Ok(host)
}

#[test]
fn complete_logical_graph_is_registered_before_tabs_setup() -> Result<(), BridgeError> {
    let mut host = ProtoTabsHost::new()?;
    host.register_root(
        "graph-root",
        "Graph tabs",
        root_props(TabsActivationMode::Automatic),
    )?;
    host.register_list("graph-list", "graph-root", list_props(false))?;
    for value in ["overview", "power", "diagnostics"] {
        host.register_trigger(
            format!("graph-trigger-{value}"),
            value.to_uppercase(),
            "graph-list",
            trigger(value, false),
        )?;
        host.register_content(
            format!("graph-content-{value}"),
            "graph-root",
            content(value),
        )?;
    }
    assert!(host.parent_ref("graph-root").is_err());
    assert!(host.snapshot().is_err());

    host.setup()?;
    let snapshot = host.snapshot()?;
    assert_eq!(snapshot.triggers.len(), 3);
    assert_eq!(snapshot.contents.len(), 3);
    assert_eq!(
        snapshot.list.a11y.as_ref().expect("tablist a11y").role,
        "tablist"
    );
    assert!(
        snapshot
            .triggers
            .iter()
            .all(|trigger| trigger.session.projection.view_epoch.get() > 0)
    );
    assert!(
        snapshot
            .contents
            .iter()
            .all(|content| content.session.projection.view_epoch.get() > 0)
    );
    Ok(())
}

#[test]
fn keyboard_navigation_supports_both_orientations_and_home_end() -> Result<(), BridgeError> {
    let mut host = tabs("keys", TabsActivationMode::Manual, false)?;
    host.set_focus_ready("keys-trigger-overview", true)?;
    assert_eq!(
        host.focus("keys-trigger-overview")?,
        FocusOperationResult::Accepted
    );

    host.dispatch_key("ArrowRight")?;
    assert_eq!(host.active_value()?, "power");
    host.dispatch_key("ArrowDown")?;
    assert_eq!(host.active_value()?, "power");
    host.dispatch_key("End")?;
    assert_eq!(host.active_value()?, "diagnostics");
    host.dispatch_key("Home")?;
    assert_eq!(host.active_value()?, "overview");

    let mut vertical = tabs("vertical", TabsActivationMode::Manual, false)?;
    let mut vertical_props = root_props(TabsActivationMode::Manual);
    vertical_props.orientation = TabsOrientation::Vertical;
    vertical.set_root_props(vertical_props)?;
    vertical.set_focus_ready("vertical-trigger-overview", true)?;
    assert_eq!(
        vertical.focus("vertical-trigger-overview")?,
        FocusOperationResult::Accepted
    );
    vertical.dispatch_key("ArrowDown")?;
    assert_eq!(vertical.active_value()?, "power");
    vertical.dispatch_key("ArrowRight")?;
    assert_eq!(vertical.active_value()?, "power");
    Ok(())
}

#[test]
fn loop_policy_skips_disabled_triggers_and_stops_at_boundaries() -> Result<(), BridgeError> {
    let mut bounded = tabs("bounded", TabsActivationMode::Manual, false)?;
    for id in [
        "bounded-trigger-overview",
        "bounded-trigger-power",
        "bounded-trigger-diagnostics",
    ] {
        bounded.set_focus_ready(id, true)?;
    }
    bounded.focus("bounded-trigger-overview")?;
    bounded.dispatch_key("ArrowLeft")?;
    assert_eq!(bounded.active_value()?, "overview");
    bounded.dispatch_key("ArrowRight")?;
    assert_eq!(bounded.active_value()?, "power");

    let mut looping = tabs("looping", TabsActivationMode::Manual, true)?;
    for id in [
        "looping-trigger-overview",
        "looping-trigger-power",
        "looping-trigger-diagnostics",
    ] {
        looping.set_focus_ready(id, true)?;
    }
    looping.focus("looping-trigger-overview")?;
    looping.dispatch_key("ArrowLeft")?;
    assert_eq!(looping.active_value()?, "diagnostics");

    let mut disabled = tabs("disabled", TabsActivationMode::Manual, true)?;
    for id in [
        "disabled-trigger-overview",
        "disabled-trigger-power",
        "disabled-trigger-diagnostics",
    ] {
        disabled.set_focus_ready(id, true)?;
    }
    disabled.focus("disabled-trigger-overview")?;
    disabled.dispatch_key("ArrowRight")?;
    assert_eq!(disabled.active_value()?, "diagnostics");
    let activation = disabled.press_commit("disabled-trigger-power", InputSource::Keyboard)?;
    assert_eq!(activation.click_count, 0);
    assert_eq!(disabled.selected_value()?, "overview");
    Ok(())
}

#[test]
fn manual_focus_keeps_selected_separate_from_active_until_activation() -> Result<(), BridgeError> {
    let mut host = tabs("manual", TabsActivationMode::Manual, false)?;
    host.set_focus_ready("manual-trigger-overview", true)?;
    host.set_focus_ready("manual-trigger-power", true)?;
    host.focus("manual-trigger-overview")?;
    host.dispatch_key("ArrowRight")?;
    assert_eq!(host.selected_value()?, "overview");
    assert_eq!(host.active_value()?, "power");

    let outcome = host.press_commit("manual-trigger-power", InputSource::Keyboard)?;
    assert_eq!(outcome.click_count, 1);
    assert_eq!(host.selected_value()?, "power");
    assert!(!host.content("manual-content-overview")?.present);
    assert!(host.content("manual-content-power")?.present);
    Ok(())
}

#[test]
fn automatic_focus_selects_and_keyboard_activation_has_one_commit() -> Result<(), BridgeError> {
    let mut host = tabs("automatic", TabsActivationMode::Automatic, false)?;
    host.set_focus_ready("automatic-trigger-overview", true)?;
    host.set_focus_ready("automatic-trigger-power", true)?;
    host.focus("automatic-trigger-overview")?;
    host.dispatch_key("ArrowRight")?;
    assert_eq!(host.selected_value()?, "power");
    let first = host.press_commit("automatic-trigger-power", InputSource::Keyboard)?;
    assert_eq!(first.click_count, 1);
    let second = host.press_commit("automatic-trigger-power", InputSource::Keyboard)?;
    assert_eq!(second.click_count, 1);
    assert_eq!(host.selected_value()?, "power");
    Ok(())
}

#[test]
fn accesskit_activation_uses_the_same_press_commit_path() -> Result<(), BridgeError> {
    let mut host = tabs("accesskit", TabsActivationMode::Manual, false)?;
    let trigger = host.trigger("accesskit-trigger-power")?;
    let a11y = trigger.a11y.as_ref().expect("tab a11y");
    assert!(a11y.actions.iter().any(|action| action == "activate"));
    assert_eq!(a11y.role, "tab");
    let outcome = host.press_commit("accesskit-trigger-power", InputSource::Accessibility)?;
    assert_eq!(outcome.click_count, 1);
    assert_eq!(host.selected_value()?, "power");
    Ok(())
}

#[test]
fn tab_roles_relations_and_presence_are_stable() -> Result<(), BridgeError> {
    let host = tabs("relations", TabsActivationMode::Automatic, false)?;
    let trigger = host.trigger("relations-trigger-overview")?;
    let panel = host.content("relations-content-overview")?;
    assert_eq!(trigger.a11y.as_ref().expect("tab").role, "tab");
    assert_eq!(panel.a11y.as_ref().expect("tabpanel").role, "tabpanel");
    assert_eq!(trigger.tab_id, panel.labelled_by);
    assert_eq!(trigger.controls, panel.tabpanel_id);
    assert!(host.content("relations-content-overview")?.present);
    assert!(!host.content("relations-content-power")?.present);
    Ok(())
}

#[test]
fn focus_result_distinguishes_not_ready_rejected_and_accepted() -> Result<(), BridgeError> {
    let mut host = tabs("focus", TabsActivationMode::Manual, false)?;
    assert_eq!(
        host.focus("focus-trigger-overview")?,
        FocusOperationResult::NotReady
    );
    host.set_focus_ready("focus-trigger-overview", true)?;
    assert_eq!(
        host.focus("focus-trigger-overview")?,
        FocusOperationResult::Accepted
    );
    let stale = host.focus_target("focus-trigger-overview")?;
    host.remount_trigger("focus-trigger-overview")?;
    assert_eq!(
        host.focus_with_target(stale)?,
        FocusOperationResult::Rejected
    );
    assert_eq!(
        host.focus("missing-trigger")?,
        FocusOperationResult::Rejected
    );
    Ok(())
}

#[test]
fn remount_rejects_stale_focus_and_dispose_removes_the_family() -> Result<(), BridgeError> {
    let mut host = tabs("lifecycle", TabsActivationMode::Manual, false)?;
    host.set_focus_ready("lifecycle-trigger-overview", true)?;
    host.focus("lifecycle-trigger-overview")?;
    let before = host.focus_target("lifecycle-trigger-overview")?;
    host.remount_trigger("lifecycle-trigger-overview")?;
    assert!(
        host.focus_with_target(before)
            .is_ok_and(|result| { result == FocusOperationResult::Rejected })
    );
    host.dispose()?;
    assert!(host.snapshot().is_err());
    assert!(host.trigger("lifecycle-trigger-overview").is_err());
    Ok(())
}

#[test]
fn separate_tabs_families_do_not_receive_each_others_arrow_keys() -> Result<(), BridgeError> {
    let mut first = tabs("first", TabsActivationMode::Manual, true)?;
    let mut second = tabs("second", TabsActivationMode::Manual, true)?;
    for id in ["first-trigger-overview", "first-trigger-power"] {
        first.set_focus_ready(id, true)?;
    }
    for id in ["second-trigger-overview", "second-trigger-power"] {
        second.set_focus_ready(id, true)?;
    }
    first.focus("first-trigger-overview")?;
    second.focus("second-trigger-overview")?;
    first.dispatch_key("ArrowRight")?;
    assert_eq!(first.active_value()?, "power");
    assert_eq!(second.active_value()?, "overview");
    Ok(())
}
