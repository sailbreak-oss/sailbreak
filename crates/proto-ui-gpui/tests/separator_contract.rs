use proto_ui_gpui::{BridgeError, ProtoSeparatorHost, SeparatorOrientation, SeparatorProps};

#[test]
fn horizontal_decorative_separator_is_contentless_and_hidden() -> Result<(), BridgeError> {
    let mut host = ProtoSeparatorHost::new()?;
    host.register(
        "main-rule",
        SeparatorProps {
            orientation: SeparatorOrientation::Horizontal,
            decorative: true,
        },
    )?;
    let snapshot = host.snapshot("main-rule")?;

    assert_eq!(snapshot.orientation, SeparatorOrientation::Horizontal);
    assert!(snapshot.decorative);
    assert!(snapshot.session.projection.template.is_empty());
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "h-px")
    );
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "w-full")
    );
    assert!(snapshot.native_style.unsupported.is_empty());
    let a11y = snapshot.session.a11y.as_ref().expect("separator a11y");
    assert!(a11y.hidden);
    assert!(a11y.role.is_empty());
    assert_eq!(a11y.orientation, None);
    assert!(snapshot.profile.signals.is_empty());
    Ok(())
}

#[test]
fn vertical_semantic_separator_projects_role_and_orientation() -> Result<(), BridgeError> {
    let mut host = ProtoSeparatorHost::new()?;
    host.register(
        "side-rule",
        SeparatorProps {
            orientation: SeparatorOrientation::Vertical,
            decorative: false,
        },
    )?;
    let snapshot = host.snapshot("side-rule")?;

    assert_eq!(snapshot.orientation, SeparatorOrientation::Vertical);
    assert!(!snapshot.decorative);
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "h-full")
    );
    assert!(
        snapshot
            .native_style
            .tokens
            .iter()
            .any(|token| token == "w-px")
    );
    let a11y = snapshot.session.a11y.as_ref().expect("separator a11y");
    assert!(!a11y.hidden);
    assert_eq!(a11y.role, "separator");
    assert_eq!(a11y.orientation.as_deref(), Some("vertical"));
    Ok(())
}

#[test]
fn replacement_preserves_identity_without_stale_nodes() -> Result<(), BridgeError> {
    let mut host = ProtoSeparatorHost::new()?;
    host.register("rule", SeparatorProps::default())?;
    let before = host.snapshot("rule")?;

    host.set_props(
        "rule",
        SeparatorProps {
            orientation: SeparatorOrientation::Vertical,
            decorative: false,
        },
    )?;
    let after = host.snapshot("rule")?;
    assert_eq!(after.session.session_id, before.session.session_id);
    assert_eq!(after.session.instance_id, before.session.instance_id);
    assert!(after.session.projection.commit_id > before.session.projection.commit_id);
    assert!(after.removed_semantic_ids.is_empty());

    let epoch = host.remount("rule")?;
    assert!(epoch > before.session.projection.view_epoch);
    host.dispose("rule")?;
    assert!(host.snapshot("rule").is_err());
    Ok(())
}
