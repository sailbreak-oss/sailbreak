use proto_ui_gpui::A11ySnapshot;
use sailbreak_gui::project_a11y;

#[test]
fn button_snapshot_projects_native_role_label_and_click_action() {
    let projection = project_a11y(&A11ySnapshot::button("Apply", false));

    assert_eq!(projection.role, gpui::accesskit::Role::Button);
    assert_eq!(projection.label, "Apply");
    assert_eq!(projection.action, Some(gpui::AccessibleAction::Click));
    assert!(!projection.disabled);
    assert_eq!(projection.selected, None);
    assert_eq!(projection.toggled, None);
}

#[test]
fn native_state_projection_preserves_disabled_selected_and_toggled() {
    let snapshot = A11ySnapshot {
        role: "button".to_owned(),
        name: "Pinned".to_owned(),
        disabled: true,
        focused: false,
        focus_visible: false,
        selected: Some(true),
        toggled: Some(true),
        actions: vec!["activate".to_owned()],
    };
    let projection = project_a11y(&snapshot);

    assert!(projection.disabled);
    assert_eq!(projection.selected, Some(true));
    assert_eq!(projection.toggled, Some(gpui::Toggled::True));
}
