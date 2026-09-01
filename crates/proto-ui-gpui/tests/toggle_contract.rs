use proto_ui_gpui::{
    BridgeError, InputKind, InputSource, ProtoToggleHost, ToggleProps, ToggleSize, ToggleVariant,
};

fn props() -> ToggleProps {
    ToggleProps {
        variant: ToggleVariant::Outline,
        size: ToggleSize::Sm,
        active: None,
        default_active: true,
        disabled: false,
    }
}

#[test]
fn default_active_variant_size_and_a11y_project_from_runtime() -> Result<(), BridgeError> {
    let mut host = ProtoToggleHost::new()?;
    host.register("wifi", "Wi-Fi", props())?;
    let snapshot = host.snapshot("wifi")?;

    assert!(snapshot.active);
    assert!(!snapshot.disabled);
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "border-input")
    );
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "h-7")
    );
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "min-w-7")
    );
    assert!(
        snapshot.native_style.unsupported.is_empty(),
        "unsupported: {:?}",
        snapshot.native_style.unsupported
    );
    assert!(
        snapshot.resolved_style.unsupported.is_empty(),
        "resolved unsupported: {:?}",
        snapshot.resolved_style.unsupported
    );
    let a11y = snapshot
        .session
        .a11y
        .as_ref()
        .expect("toggle a11y snapshot");
    assert_eq!(a11y.role, "button");
    assert_eq!(a11y.name, "Wi-Fi");
    assert_eq!(a11y.toggled, Some(true));
    Ok(())
}

#[test]
fn uncontrolled_activation_emits_once_and_updates_exposed_active() -> Result<(), BridgeError> {
    let mut host = ProtoToggleHost::new()?;
    let mut initial = props();
    initial.default_active = false;
    host.register("wifi", "Wi-Fi", initial)?;

    let outcome = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(outcome.active_change_count, 1);
    assert!(host.snapshot("wifi")?.active);

    let outcome = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(outcome.active_change_count, 1);
    assert!(!host.snapshot("wifi")?.active);
    Ok(())
}

#[test]
fn controlled_active_changes_only_through_props() -> Result<(), BridgeError> {
    let mut host = ProtoToggleHost::new()?;
    let mut controlled = props();
    controlled.active = Some(false);
    controlled.default_active = false;
    host.register("wifi", "Wi-Fi", controlled.clone())?;

    let outcome = host.dispatch("wifi", InputKind::PressCommit, InputSource::Keyboard, None)?;
    assert_eq!(outcome.active_change_count, 1);
    assert!(!host.snapshot("wifi")?.active);

    controlled.active = Some(true);
    host.set_props("wifi", controlled)?;
    assert!(host.snapshot("wifi")?.active);
    Ok(())
}

#[test]
fn disabled_toggle_suppresses_activation() -> Result<(), BridgeError> {
    let mut host = ProtoToggleHost::new()?;
    let mut disabled = props();
    disabled.default_active = false;
    disabled.disabled = true;
    host.register("wifi", "Wi-Fi", disabled)?;

    let outcome = host.dispatch("wifi", InputKind::PressCommit, InputSource::Mouse, None)?;
    assert_eq!(outcome.active_change_count, 0);
    let snapshot = host.snapshot("wifi")?;
    assert!(!snapshot.active);
    assert!(snapshot.disabled);
    Ok(())
}

#[test]
fn focus_style_replacement_and_disposal_are_epoch_safe() -> Result<(), BridgeError> {
    let mut host = ProtoToggleHost::new()?;
    host.register("wifi", "Wi-Fi", props())?;
    host.dispatch(
        "wifi",
        InputKind::KeyDown,
        InputSource::Keyboard,
        Some(serde_json::json!({ "key": "Tab" })),
    )?;
    host.dispatch("wifi", InputKind::Focus, InputSource::Keyboard, None)?;
    let before = host.snapshot("wifi")?;
    assert!(
        before
            .native_style
            .tokens
            .iter()
            .any(|token| token == "ring-3")
    );

    let mut replacement = props();
    replacement.variant = ToggleVariant::Default;
    replacement.size = ToggleSize::Lg;
    host.set_props("wifi", replacement)?;
    let after = host.snapshot("wifi")?;
    assert_eq!(after.session.session_id, before.session.session_id);
    assert_eq!(after.session.instance_id, before.session.instance_id);
    assert!(after.session.projection.commit_id > before.session.projection.commit_id);
    assert!(after.native_style.tokens.iter().any(|token| token == "h-9"));

    host.dispose("wifi")?;
    assert!(matches!(
        host.snapshot("wifi"),
        Err(BridgeError::InvalidIdentity { .. })
    ));
    Ok(())
}
