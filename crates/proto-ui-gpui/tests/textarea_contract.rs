use proto_ui_gpui::{
    BridgeError, InputKind, InputSource, ProtoTextareaHost, TextControlEvent, TextControlSelection,
    TextControlWrap, TextareaProps,
};

fn props(default_value: &str) -> TextareaProps {
    TextareaProps {
        default_value: default_value.to_owned(),
        placeholder: "Write a profile".to_owned(),
        rows: 4,
        wrap: TextControlWrap::Soft,
        ..TextareaProps::default()
    }
}

#[test]
fn textarea_projects_initial_value_placeholder_rows_and_a11y() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    host.register("profile", "Profile", props("initial\ntext"))?;

    let snapshot = host.snapshot("profile")?;
    assert_eq!(snapshot.value, "initial\ntext");
    assert_eq!(snapshot.native_value, "initial\ntext");
    assert_eq!(snapshot.placeholder, "Write a profile");
    assert_eq!(snapshot.rows, 4);
    assert_eq!(snapshot.selection, TextControlSelection::caret(0));
    let a11y = snapshot.a11y.expect("textbox a11y");
    assert_eq!(a11y.role, "textbox");
    assert_eq!(a11y.name, "Profile");
    Ok(())
}

#[test]
fn uncontrolled_input_change_and_composition_events_stay_data_only() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    host.register("editor", "Editor", props(""))?;

    host.input("editor", "draft", None, None, false)?;
    let input = host.snapshot("editor")?;
    assert_eq!(input.value, "draft");
    assert_eq!(input.native_value, "draft");
    assert_eq!(input.input_event_count, 1);

    host.dispatch_text(
        "editor",
        TextControlEvent::composition_start("draft", Some("あ".to_owned())),
    )?;
    host.dispatch_text(
        "editor",
        TextControlEvent::composition_update("draft愛", Some("愛".to_owned())),
    )?;
    let committed = host.composition_end("editor", Some("愛".to_owned()))?;
    assert_eq!(
        committed.text_event_count(proto_ui_gpui::TextControlEventType::CompositionEnd),
        1
    );
    assert_eq!(
        committed.text_event_count(proto_ui_gpui::TextControlEventType::Input),
        1
    );
    host.change("editor")?;
    let final_state = host.snapshot("editor")?;
    assert_eq!(final_state.value, "draft愛");
    assert_eq!(final_state.native_value, "draft愛");
    assert!(!final_state.composing);
    assert_eq!(final_state.input_event_count, 2);
    assert_eq!(final_state.change_event_count, 1);
    assert_eq!(final_state.composition_start_count, 1);
    assert_eq!(final_state.composition_update_count, 1);
    assert_eq!(final_state.composition_end_count, 1);
    Ok(())
}

#[test]
fn post_edit_selection_is_not_clamped_against_the_previous_buffer() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    host.register("editor", "Editor", props("abc"))?;
    let epoch = host.snapshot("editor")?.session.projection.view_epoch;

    host.dispatch_text_with_selection_at_epoch(
        "editor",
        epoch,
        TextControlEvent::input("abcd"),
        TextControlSelection::caret(4),
    )?;
    assert_eq!(
        host.snapshot("editor")?.selection,
        TextControlSelection::caret(4)
    );

    host.dispatch_text_with_selection_at_epoch(
        "editor",
        epoch,
        TextControlEvent::input("abcde"),
        TextControlSelection::caret(5),
    )?;
    let snapshot = host.snapshot("editor")?;
    assert_eq!(snapshot.value, "abcde");
    assert_eq!(snapshot.selection, TextControlSelection::caret(5));
    Ok(())
}

#[test]
fn controlled_value_does_not_get_overwritten_by_stale_native_input() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    let mut controlled = props("ignored");
    controlled.value = Some("server".to_owned());
    host.register("editor", "Editor", controlled.clone())?;

    host.input("editor", "client", None, None, false)?;
    assert_eq!(host.snapshot("editor")?.value, "server");
    assert_eq!(host.snapshot("editor")?.native_value, "server");
    host.set_props("editor", controlled)?;
    assert_eq!(host.snapshot("editor")?.native_value, "server");
    Ok(())
}

#[test]
fn uncontrolled_default_value_updates_do_not_overwrite_user_input() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    let mut initial = props("initial");
    initial.default_value = "initial".to_owned();
    host.register("editor", "Editor", initial.clone())?;

    host.input("editor", "user draft", None, None, false)?;
    let mut updated = initial;
    updated.default_value = "new default".to_owned();
    host.set_props("editor", updated)?;

    let snapshot = host.snapshot("editor")?;
    assert_eq!(snapshot.value, "user draft");
    assert_eq!(snapshot.native_value, "user draft");
    Ok(())
}

#[test]
fn composing_controlled_input_preserves_selection_until_commit() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    let mut controlled = props("server");
    controlled.value = Some("server".to_owned());
    host.register("editor", "Editor", controlled)?;
    host.set_selection("editor", TextControlSelection::range(2, 2))?;

    host.composition_start("editor", Some("x".to_owned()))?;
    host.input(
        "editor",
        "sxerver",
        Some("x".to_owned()),
        Some("insertText".to_owned()),
        true,
    )?;
    let composing = host.snapshot("editor")?;
    assert!(composing.composing);
    assert_eq!(composing.selection.start, 2);
    host.composition_end("editor", Some("x".to_owned()))?;
    assert!(!host.snapshot("editor")?.composing);
    assert_eq!(host.snapshot("editor")?.native_value, "server");
    Ok(())
}

#[test]
fn disabled_focus_and_stale_epoch_are_rejected_without_mutating_buffer() -> Result<(), BridgeError>
{
    let mut host = ProtoTextareaHost::new()?;
    let mut disabled = props("safe");
    disabled.disabled = true;
    host.register("editor", "Editor", disabled)?;
    assert!(!host.focus("editor")?);
    assert_eq!(host.snapshot("editor")?.native_value, "safe");

    let epoch = host.snapshot("editor")?.session.projection.view_epoch;
    host.remount("editor")?;
    let stale = TextControlEvent::input("unsafe");
    assert!(matches!(
        host.dispatch_text_at_epoch("editor", epoch, stale),
        Err(BridgeError::StaleEpoch { .. })
    ));
    assert_eq!(host.snapshot("editor")?.native_value, "safe");
    Ok(())
}

#[test]
fn keyboard_focus_projects_focus_visible_ring_until_blur() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    host.register("editor", "Editor", props("draft"))?;
    host.dispatch(
        "editor",
        InputKind::KeyDown,
        InputSource::Keyboard,
        Some(serde_json::json!({ "key": "Tab" })),
    )?;
    host.dispatch("editor", InputKind::Focus, InputSource::Keyboard, None)?;

    let focused = host.snapshot("editor")?;
    assert!(focused.focused);
    assert!(focused.focus_visible);
    assert!(
        focused
            .native_style
            .tokens
            .iter()
            .any(|token| token == "ring-3")
    );

    host.dispatch("editor", InputKind::Blur, InputSource::Keyboard, None)?;
    let blurred = host.snapshot("editor")?;
    assert!(!blurred.focused);
    assert!(!blurred.focus_visible);
    Ok(())
}

#[test]
fn generic_focus_input_routes_remount_and_dispose_are_epoch_safe() -> Result<(), BridgeError> {
    let mut host = ProtoTextareaHost::new()?;
    host.register("editor", "Editor", props("safe"))?;
    host.dispatch("editor", InputKind::Focus, InputSource::Keyboard, None)?;
    assert!(host.snapshot("editor")?.focused);
    host.dispatch("editor", InputKind::Blur, InputSource::Keyboard, None)?;
    assert!(!host.snapshot("editor")?.focused);
    let before = host.snapshot("editor")?.session.projection.view_epoch;
    let after = host.remount("editor")?;
    assert!(after.get() > before.get());
    host.dispose("editor")?;
    assert!(matches!(
        host.snapshot("editor"),
        Err(BridgeError::InvalidIdentity { .. })
    ));
    Ok(())
}
