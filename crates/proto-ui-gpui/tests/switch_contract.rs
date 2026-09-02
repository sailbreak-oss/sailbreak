use proto_ui_gpui::{
    BridgeError, InputKind, InputSource, LogicalParentRef, ProtoSwitchHost, ShadcnTheme,
    SwitchProps,
};

fn props() -> SwitchProps {
    SwitchProps {
        checked: None,
        default_checked: false,
        disabled: false,
    }
}

fn register_switch(host: &mut ProtoSwitchHost, id: &str) -> Result<(), BridgeError> {
    host.register_root(id, "Wi-Fi", props())?;
    host.register_thumb(format!("{id}-thumb"), id)?;
    Ok(())
}

#[test]
fn switch_projects_root_and_thumb_composition_from_runtime() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    let mut initial = props();
    initial.default_checked = true;
    host.register_root("wifi", "Wi-Fi", initial)?;
    host.register_thumb("wifi-thumb", "wifi")?;

    let snapshot = host.snapshot("wifi")?;
    assert!(snapshot.checked);
    assert!(!snapshot.disabled);
    assert_eq!(
        snapshot.root.session.prototype.as_str(),
        "shadcn-switch-root"
    );
    let thumb = snapshot.thumb.as_ref().expect("switch thumb projection");
    assert_eq!(thumb.session.prototype.as_str(), "shadcn-switch-thumb");
    assert!(thumb.checked);
    assert_ne!(snapshot.root.session.instance_id, thumb.session.instance_id);
    assert!(snapshot.root.native_style.unsupported.is_empty());
    assert!(thumb.native_style.unsupported.is_empty());
    let a11y = snapshot.root.session.a11y.as_ref().expect("switch a11y");
    assert_eq!(a11y.role, "switch");
    assert_eq!(a11y.name, "Wi-Fi");
    assert_eq!(a11y.toggled, Some(true));
    Ok(())
}

#[test]
fn uncontrolled_checked_state_changes_once_per_press_commit() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    register_switch(&mut host, "wifi")?;

    let first = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(first.checked_change_count, 1);
    assert_eq!(first.checked_changes, vec![true]);
    assert!(host.snapshot("wifi")?.checked);
    assert!(host.thumb_snapshot("wifi-thumb")?.checked);

    let second = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(second.checked_change_count, 1);
    assert_eq!(second.checked_changes, vec![false]);
    assert!(!host.snapshot("wifi")?.checked);
    assert!(!host.thumb_snapshot("wifi-thumb")?.checked);
    Ok(())
}

#[test]
fn controlled_checked_state_changes_only_through_props() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    let mut controlled = props();
    controlled.checked = Some(false);
    host.register_root("wifi", "Wi-Fi", controlled.clone())?;

    let outcome = host.dispatch("wifi", InputKind::PressCommit, InputSource::Keyboard, None)?;
    assert_eq!(outcome.checked_change_count, 1);
    assert!(!host.snapshot("wifi")?.checked);

    controlled.checked = Some(true);
    host.set_props("wifi", controlled)?;
    assert!(host.snapshot("wifi")?.checked);
    Ok(())
}

#[test]
fn disabled_switch_suppresses_activation_and_clears_transient_state() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    let mut disabled = props();
    disabled.disabled = true;
    host.register_root("wifi", "Wi-Fi", disabled)?;

    let outcome = host.dispatch("wifi", InputKind::PressCommit, InputSource::Mouse, None)?;
    assert_eq!(outcome.checked_change_count, 0);
    let snapshot = host.snapshot("wifi")?;
    assert!(snapshot.disabled);
    assert!(!snapshot.checked);
    assert!(!snapshot.hovered);
    assert!(!snapshot.pressed);
    Ok(())
}

#[test]
fn focus_visible_ring_and_dark_switch_tokens_are_projected() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::with_theme(ShadcnTheme::dark())?;
    register_switch(&mut host, "wifi")?;
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
            .any(|token| token == "ring-ring/50")
    );
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "ring-offset-background")
    );
    assert!(
        snapshot
            .root
            .native_style
            .tokens
            .iter()
            .any(|token| token == "bg-input/50")
    );
    assert!(snapshot.root.native_style.unsupported.is_empty());
    assert!(
        snapshot
            .thumb
            .as_ref()
            .expect("thumb")
            .native_style
            .unsupported
            .is_empty()
    );
    Ok(())
}

#[test]
fn props_remount_preserves_identity_and_advances_epoch() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    register_switch(&mut host, "wifi")?;
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
    let after = host.snapshot("wifi")?;
    assert_eq!(
        after.root.session.session_id,
        before.root.session.session_id
    );
    assert_eq!(
        after.root.session.instance_id,
        before.root.session.instance_id
    );
    Ok(())
}

#[test]
fn stale_parent_link_is_rejected_and_thumb_replacement_uses_current_root_epoch()
-> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    host.register_root("wifi", "Wi-Fi", props())?;
    let stale_parent: LogicalParentRef = host.parent_ref("wifi")?;
    host.register_thumb("wifi-thumb-old", "wifi")?;
    host.remount("wifi")?;

    assert!(matches!(
        host.register_thumb_with_parent("wifi-thumb-stale", stale_parent),
        Err(BridgeError::StaleParent { .. })
    ));
    host.replace_thumb("wifi", "wifi-thumb-new")?;
    assert_eq!(
        host.snapshot("wifi")?.thumb.expect("replacement").id,
        "wifi-thumb-new"
    );
    assert!(host.thumb_snapshot("wifi-thumb-old").is_err());
    Ok(())
}

#[test]
fn disposal_removes_root_and_thumb_sessions() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    register_switch(&mut host, "wifi")?;
    host.dispose("wifi")?;
    assert!(host.snapshot("wifi").is_err());
    assert!(host.thumb_snapshot("wifi-thumb").is_err());
    Ok(())
}

#[test]
fn sibling_sessions_do_not_leak_state_or_signals() -> Result<(), BridgeError> {
    let mut host = ProtoSwitchHost::new()?;
    register_switch(&mut host, "wifi")?;
    register_switch(&mut host, "camera")?;

    let outcome = host.dispatch(
        "wifi",
        InputKind::PressCommit,
        InputSource::Accessibility,
        None,
    )?;
    assert_eq!(outcome.checked_change_count, 1);
    assert!(host.snapshot("wifi")?.checked);
    assert!(host.thumb_snapshot("wifi-thumb")?.checked);
    assert!(!host.snapshot("camera")?.checked);
    assert!(!host.thumb_snapshot("camera-thumb")?.checked);
    Ok(())
}
