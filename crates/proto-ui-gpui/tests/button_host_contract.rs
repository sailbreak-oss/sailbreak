use proto_ui_gpui::{
    InputKind, InputSource, ProtoButtonHost, ShadcnButtonSize, ShadcnButtonVariant,
};

#[test]
fn registered_button_uses_real_shadcn_projection_and_accessibility() {
    let mut host = ProtoButtonHost::new().expect("QuickJS host starts");
    host.register_button(
        "apply",
        "Apply",
        ShadcnButtonVariant::Default,
        ShadcnButtonSize::Default,
    )
    .expect("button registers");

    let button = host.button("apply").expect("button state");
    let a11y = button.a11y.as_ref().expect("a11y projection");
    assert_eq!(a11y.role, "button");
    assert_eq!(a11y.name, "Apply");
    assert!(!a11y.disabled);
    assert!(button.style.unsupported.is_empty());
    assert_eq!(button.style.height, 32.0);
    assert_eq!(button.style.padding_x, 10.0);
}

#[test]
fn native_pointer_state_and_click_follow_one_proto_signal_path() {
    let mut host = ProtoButtonHost::new().expect("QuickJS host starts");
    host.register_button(
        "refresh",
        "Refresh",
        ShadcnButtonVariant::Default,
        ShadcnButtonSize::Default,
    )
    .expect("button registers");

    host.dispatch("refresh", InputKind::PointerEnter, InputSource::Mouse, None)
        .expect("pointer enter");
    let hovered = host.button("refresh").expect("button state");
    assert_eq!(hovered.state("hovered"), Some(true));

    let outcome = host
        .dispatch(
            "refresh",
            InputKind::PressCommit,
            InputSource::Accessibility,
            None,
        )
        .expect("press commit");
    assert!(outcome.click_emitted);

    let duplicate = host
        .dispatch(
            "refresh",
            InputKind::PressCommit,
            InputSource::Accessibility,
            None,
        )
        .expect("second press gets a distinct sample");
    assert!(duplicate.click_emitted);
    assert_eq!(host.button("refresh").expect("button state").click_count, 2);
}

#[test]
fn disabled_button_suppresses_activation_and_projects_disabled_style() {
    let mut host = ProtoButtonHost::new().expect("QuickJS host starts");
    host.register_button(
        "danger",
        "Delete",
        ShadcnButtonVariant::Destructive,
        ShadcnButtonSize::Sm,
    )
    .expect("button registers");
    host.set_disabled("danger", true).expect("disable button");

    let button = host.button("danger").expect("button state");
    assert_eq!(button.state("disabled"), Some(true));
    assert_eq!(button.style.opacity, 0.5);
    assert!(button.style.pointer_events_none);
    assert!(button.a11y.as_ref().expect("a11y projection").disabled);

    let outcome = host
        .dispatch("danger", InputKind::PressCommit, InputSource::Mouse, None)
        .expect("disabled press is handled");
    assert!(!outcome.click_emitted);
}

#[test]
fn shadcn_variant_and_size_props_change_translated_surface() {
    let mut host = ProtoButtonHost::new().expect("QuickJS host starts");
    host.register_button(
        "link",
        "Open",
        ShadcnButtonVariant::Link,
        ShadcnButtonSize::Lg,
    )
    .expect("button registers");
    let button = host.button("link").expect("button state");
    assert!(!button.style.underline);
    host.dispatch("link", InputKind::PointerEnter, InputSource::Mouse, None)
        .expect("hover link");
    let button = host.button("link").expect("button state");
    assert!(button.style.underline);
    assert_eq!(button.style.height, 36.0);
    assert_eq!(button.style.background.alpha, 0.0);
}

#[test]
fn variant_updates_reproject_the_same_logical_button() {
    let mut host = ProtoButtonHost::new().expect("QuickJS host starts");
    host.register_button(
        "section",
        "Overview",
        ShadcnButtonVariant::Ghost,
        ShadcnButtonSize::Sm,
    )
    .expect("button registers");
    let initial_epoch = host
        .button("section")
        .expect("button state")
        .projection
        .view_epoch;
    host.set_variant("section", ShadcnButtonVariant::Secondary)
        .expect("variant updates");
    let button = host.button("section").expect("button state");
    assert_eq!(button.variant(), ShadcnButtonVariant::Secondary);
    assert_eq!(button.projection.view_epoch, initial_epoch);
    assert_eq!(button.style.background.rgb, host.theme().secondary);
}

#[test]
fn every_published_button_variant_has_a_complete_native_translation() {
    let mut host = ProtoButtonHost::new().expect("QuickJS host starts");
    let variants = [
        ShadcnButtonVariant::Default,
        ShadcnButtonVariant::Destructive,
        ShadcnButtonVariant::Outline,
        ShadcnButtonVariant::Secondary,
        ShadcnButtonVariant::Ghost,
        ShadcnButtonVariant::Link,
    ];
    for (index, variant) in variants.into_iter().enumerate() {
        let id = format!("variant-{index}");
        host.register_button(&id, "Action", variant, ShadcnButtonSize::Default)
            .expect("button registers");
        assert!(
            host.button(&id)
                .expect("button state")
                .style
                .unsupported
                .is_empty(),
            "variant {variant:?} emitted an unsupported token"
        );
    }
}
