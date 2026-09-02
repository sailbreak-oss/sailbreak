use proto_ui_gpui::{
    BridgeError, CheckboxProps, InputKind, InputSource, LogicalParentRef, ProtoCheckboxHost,
    ShadcnTheme, TemplateNode,
};

fn props() -> CheckboxProps {
    CheckboxProps {
        checked: None,
        default_checked: false,
        disabled: false,
        indeterminate: None,
        default_indeterminate: false,
    }
}

fn register_checkbox(host: &mut ProtoCheckboxHost, id: &str) -> Result<(), BridgeError> {
    host.register_root(id, "Wi-Fi", props())?;
    host.register_indicator(format!("{id}-indicator"), id)?;
    Ok(())
}

fn has_svg_path(nodes: &[TemplateNode], path: &str) -> bool {
    nodes.iter().any(|node| match node {
        TemplateNode::Svg {
            tag,
            attributes,
            children,
        } => {
            (tag == "path" && attributes.get("d").is_some_and(|value| value == path))
                || has_svg_path(children, path)
        }
        TemplateNode::Container { children, .. } => has_svg_path(children, path),
        TemplateNode::Text { .. } | TemplateNode::Slot { .. } => false,
    })
}

fn has_slot(nodes: &[TemplateNode], slot_id: &str) -> bool {
    nodes.iter().any(|node| match node {
        TemplateNode::Slot { slot_id: candidate } => candidate == slot_id,
        TemplateNode::Container { children, .. } | TemplateNode::Svg { children, .. } => {
            has_slot(children, slot_id)
        }
        TemplateNode::Text { .. } => false,
    })
}

#[test]
fn checkbox_projects_root_indicator_tri_state_and_a11y() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::new()?;
    let mut initial = props();
    initial.default_checked = true;
    initial.default_indeterminate = true;
    host.register_root("wifi", "Wi-Fi", initial)?;
    host.register_indicator("wifi-indicator", "wifi")?;

    let snapshot = host.snapshot("wifi")?;
    assert!(snapshot.checked);
    assert!(snapshot.indeterminate);
    assert!(!snapshot.disabled);
    assert_eq!(
        snapshot.root.session.prototype.as_str(),
        "shadcn-checkbox-root"
    );

    let indicator = snapshot
        .indicator
        .as_ref()
        .expect("checkbox indicator projection");
    assert_eq!(
        indicator.session.prototype.as_str(),
        "shadcn-checkbox-indicator"
    );
    assert!(indicator.checked);
    assert!(indicator.indeterminate);
    assert_ne!(
        snapshot.root.session.instance_id,
        indicator.session.instance_id
    );
    assert!(has_slot(
        &indicator.session.projection.template,
        "wifi-indicator:slot"
    ));
    assert!(has_svg_path(
        &indicator.session.projection.template,
        "M5 12h14"
    ));
    assert!(snapshot.root.native_style.unsupported.is_empty());
    assert!(snapshot.root.resolved_style.unsupported.is_empty());
    assert!(indicator.resolved_style.unsupported.is_empty());
    assert!(indicator.native_style.unsupported.is_empty());
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "size-4")
    );
    assert!(
        indicator
            .native_style
            .tokens
            .iter()
            .any(|token| token == "transition-none")
    );

    let a11y = snapshot.root.session.a11y.as_ref().expect("checkbox a11y");
    assert_eq!(a11y.role, "checkbox");
    assert_eq!(a11y.name, "Wi-Fi");
    assert!(!a11y.disabled);
    assert!(a11y.actions.iter().any(|action| action == "activate"));
    Ok(())
}

#[test]
fn uncontrolled_activation_toggles_checked_and_clears_indeterminate() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::new()?;
    let mut initial = props();
    initial.default_indeterminate = true;
    host.register_root("wifi", "Wi-Fi", initial)?;
    host.register_indicator("wifi-indicator", "wifi")?;

    let first = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(first.checked_change_count, 1);
    assert_eq!(first.checked_changes, vec![true]);
    assert_eq!(first.indeterminate_change_count, 1);
    assert_eq!(first.indeterminate_changes, vec![false]);
    let first_snapshot = host.snapshot("wifi")?;
    assert!(first_snapshot.checked);
    assert!(!first_snapshot.indeterminate);
    assert!(has_svg_path(
        &first_snapshot
            .indicator
            .as_ref()
            .expect("indicator")
            .session
            .projection
            .template,
        "m20 6-11 11-5-5"
    ));

    let second = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(second.checked_change_count, 1);
    assert_eq!(second.checked_changes, vec![false]);
    assert_eq!(second.indeterminate_change_count, 0);
    let second_snapshot = host.snapshot("wifi")?;
    assert!(!second_snapshot.checked);
    assert!(!second_snapshot.indeterminate);
    assert!(!has_svg_path(
        &second_snapshot
            .indicator
            .as_ref()
            .expect("indicator")
            .session
            .projection
            .template,
        "m20 6-11 11-5-5"
    ));
    Ok(())
}

#[test]
fn controlled_checked_and_indeterminate_values_change_only_through_props() -> Result<(), BridgeError>
{
    let mut host = ProtoCheckboxHost::new()?;
    let mut controlled = props();
    controlled.checked = Some(false);
    controlled.indeterminate = Some(true);
    host.register_root("wifi", "Wi-Fi", controlled.clone())?;
    host.register_indicator("wifi-indicator", "wifi")?;

    let outcome = host.dispatch("wifi", InputKind::PressCommit, InputSource::Keyboard, None)?;
    assert_eq!(outcome.checked_change_count, 1);
    assert_eq!(outcome.checked_changes, vec![true]);
    assert_eq!(outcome.indeterminate_change_count, 1);
    assert_eq!(outcome.indeterminate_changes, vec![false]);
    let unchanged = host.snapshot("wifi")?;
    assert!(!unchanged.checked);
    assert!(unchanged.indeterminate);

    controlled.checked = Some(true);
    controlled.indeterminate = Some(false);
    host.set_props("wifi", controlled)?;
    let updated = host.snapshot("wifi")?;
    assert!(updated.checked);
    assert!(!updated.indeterminate);
    Ok(())
}

#[test]
fn disabled_checkbox_suppresses_activation_and_clears_transient_state() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::new()?;
    let mut disabled = props();
    disabled.disabled = true;
    disabled.default_indeterminate = true;
    host.register_root("wifi", "Wi-Fi", disabled)?;
    host.register_indicator("wifi-indicator", "wifi")?;

    let outcome = host.dispatch("wifi", InputKind::PressCommit, InputSource::Mouse, None)?;
    assert_eq!(outcome.checked_change_count, 0);
    assert_eq!(outcome.indeterminate_change_count, 0);
    let snapshot = host.snapshot("wifi")?;
    assert!(snapshot.disabled);
    assert!(!snapshot.checked);
    assert!(snapshot.indeterminate);
    assert!(!snapshot.hovered);
    assert!(!snapshot.pressed);
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "cursor-not-allowed")
    );
    let a11y = snapshot.root.session.a11y.as_ref().expect("checkbox a11y");
    assert!(a11y.disabled);
    Ok(())
}

#[test]
fn keyboard_space_is_exact_activation_and_enter_is_not() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::new()?;
    host.register_root("wifi", "Wi-Fi", props())?;
    host.register_indicator("wifi-indicator", "wifi")?;

    host.dispatch(
        "wifi",
        InputKind::KeyDown,
        InputSource::Keyboard,
        Some(serde_json::json!({ "key": "Tab" })),
    )?;
    host.dispatch("wifi", InputKind::Focus, InputSource::Keyboard, None)?;

    let enter = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Keyboard,
        Some(serde_json::json!({ "key": "Enter" })),
    )?;
    assert_eq!(enter.checked_change_count, 0);
    assert!(!host.snapshot("wifi")?.checked);

    let space = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Keyboard,
        Some(serde_json::json!({ "key": " " })),
    )?;
    assert_eq!(space.checked_change_count, 1);
    assert_eq!(space.checked_changes, vec![true]);
    assert!(host.snapshot("wifi")?.checked);
    Ok(())
}

#[test]
fn focus_visible_and_dark_unfilled_tokens_are_projected() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::with_theme(ShadcnTheme::dark())?;
    host.register_root("wifi", "Wi-Fi", props())?;
    host.register_indicator("wifi-indicator", "wifi")?;
    host.dispatch(
        "wifi",
        InputKind::KeyDown,
        InputSource::Keyboard,
        Some(serde_json::json!({ "key": "Tab" })),
    )?;
    host.dispatch("wifi", InputKind::Focus, InputSource::Keyboard, None)?;

    let snapshot = host.snapshot("wifi")?;
    assert!(snapshot.focus_visible);
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "ring-3")
    );
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "ring-ring/50")
    );
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "bg-input/30")
    );
    assert!(snapshot.root.native_style.unsupported.is_empty());
    assert!(
        snapshot
            .indicator
            .as_ref()
            .expect("indicator")
            .native_style
            .unsupported
            .is_empty()
    );

    host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    let checked = host.snapshot("wifi")?;
    assert!(
        !checked
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "bg-input/30")
    );
    assert!(
        checked
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "bg-primary")
    );
    Ok(())
}

#[test]
fn stale_parent_replacement_remount_and_disposal_are_epoch_safe() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::new()?;
    host.register_root("wifi", "Wi-Fi", props())?;
    let stale_parent: LogicalParentRef = host.parent_ref("wifi")?;
    host.register_indicator("wifi-indicator-old", "wifi")?;
    let before = host.snapshot("wifi")?;

    host.set_props("wifi", props())?;
    let after_props = host.snapshot("wifi")?;
    assert_eq!(
        after_props.root.session.session_id,
        before.root.session.session_id
    );
    assert_eq!(
        after_props.root.session.instance_id,
        before.root.session.instance_id
    );
    assert!(
        after_props.root.session.projection.commit_id > before.root.session.projection.commit_id
    );

    let before_epoch = after_props.root.session.projection.view_epoch;
    let after_epoch = host.remount("wifi")?;
    assert!(after_epoch.get() > before_epoch.get());
    assert!(matches!(
        host.register_indicator_with_parent("wifi-indicator-stale", stale_parent),
        Err(BridgeError::StaleParent { .. })
    ));

    host.replace_indicator("wifi", "wifi-indicator-new")?;
    assert_eq!(
        host.snapshot("wifi")?.indicator.expect("replacement").id,
        "wifi-indicator-new"
    );
    assert!(host.indicator_snapshot("wifi-indicator-old").is_err());

    host.dispose("wifi")?;
    assert!(host.snapshot("wifi").is_err());
    assert!(host.indicator_snapshot("wifi-indicator-new").is_err());
    Ok(())
}

#[test]
fn sibling_checkbox_sessions_do_not_leak_state_or_signals() -> Result<(), BridgeError> {
    let mut host = ProtoCheckboxHost::new()?;
    register_checkbox(&mut host, "wifi")?;
    register_checkbox(&mut host, "camera")?;

    let outcome = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(outcome.checked_change_count, 1);
    assert!(host.snapshot("wifi")?.checked);
    assert!(host.indicator_snapshot("wifi-indicator")?.checked);
    assert!(!host.snapshot("camera")?.checked);
    assert!(!host.indicator_snapshot("camera-indicator")?.checked);
    Ok(())
}
