use proto_ui_gpui::{
    BridgeError, CloseReason, DialogCloseProps, DialogContentProps, DialogDescriptionProps,
    DialogFooterProps, DialogHeaderProps, DialogMaskProps, DialogRootProps, DialogTitleProps,
    DialogTriggerProps, FocusOperationResult, InputKind, InputSource, LayerRole, OverlayEvent,
    ProtoDialogHost,
};

fn dialog(name: &str) -> Result<ProtoDialogHost, BridgeError> {
    let mut host = ProtoDialogHost::new()?;
    let root = format!("{name}-root");
    let trigger = format!("{name}-trigger");
    let mask = format!("{name}-mask");
    let content = format!("{name}-content");
    let title = format!("{name}-title");
    let description = format!("{name}-description");
    let close = format!("{name}-close");
    let header = format!("{name}-header");
    let footer = format!("{name}-footer");
    host.register_root(
        root.clone(),
        format!("{name} confirmation"),
        DialogRootProps::default(),
    )?;
    host.register_trigger(trigger, &root, DialogTriggerProps::default())?;
    host.register_mask(mask, &root, DialogMaskProps::default())?;
    host.register_content(content.clone(), &root, DialogContentProps)?;
    host.register_title(title, "Confirm operation", &content, DialogTitleProps)?;
    host.register_description(
        description,
        "This operation writes through the existing CLI safety gate.",
        &content,
        DialogDescriptionProps,
    )?;
    host.register_close(close, "Cancel", &content, DialogCloseProps::default())?;
    host.register_header(header, &content, DialogHeaderProps)?;
    host.register_footer(footer, &content, DialogFooterProps)?;
    host.setup()?;
    Ok(host)
}

#[test]
fn complete_graph_is_registered_before_runtime_setup_and_projects_all_parts()
-> Result<(), BridgeError> {
    let mut host = ProtoDialogHost::new()?;
    host.register_root("root", "Confirmation", DialogRootProps::default())?;
    host.register_trigger("trigger", "root", DialogTriggerProps::default())?;
    host.register_mask("mask", "root", DialogMaskProps::default())?;
    host.register_content("content", "root", DialogContentProps)?;
    host.register_title("title", "Confirm", "content", DialogTitleProps)?;
    host.register_description("description", "Details", "content", DialogDescriptionProps)?;
    host.register_close("close", "Cancel", "content", DialogCloseProps::default())?;
    host.register_header("header", "content", DialogHeaderProps)?;
    host.register_footer("footer", "content", DialogFooterProps)?;
    assert!(host.snapshot().is_err());

    host.setup()?;
    let snapshot = host.snapshot()?;
    assert_eq!(
        snapshot.root.session.prototype.as_str(),
        "shadcn-dialog-root"
    );
    assert_eq!(
        snapshot
            .trigger
            .as_ref()
            .expect("trigger")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-trigger"
    );
    assert_eq!(
        snapshot
            .mask
            .as_ref()
            .expect("mask")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-mask"
    );
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-content"
    );
    assert_eq!(
        snapshot
            .title
            .as_ref()
            .expect("title")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-title"
    );
    assert_eq!(
        snapshot
            .description
            .as_ref()
            .expect("description")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-description"
    );
    assert_eq!(
        snapshot.closes[0].session.prototype.as_str(),
        "shadcn-dialog-close"
    );
    assert_eq!(
        snapshot
            .header
            .as_ref()
            .expect("header")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-header"
    );
    assert_eq!(
        snapshot
            .footer
            .as_ref()
            .expect("footer")
            .session
            .prototype
            .as_str(),
        "shadcn-dialog-footer"
    );
    assert_ne!(
        snapshot.root.session.instance_id,
        snapshot
            .content
            .as_ref()
            .expect("content")
            .session
            .instance_id
    );
    Ok(())
}

#[test]
fn opening_dialog_projects_modal_mask_and_gates_other_family_input() -> Result<(), BridgeError> {
    let mut host = dialog("modal")?;
    assert!(!host.is_open()?);
    assert!(!host.modal_blocking()?);
    assert!(!host.content("modal-content")?.present);

    let opened = host.open()?;
    assert_eq!(opened.open_change_count, 1);
    assert!(host.is_open()?);
    assert!(host.modal_blocking()?);
    assert!(host.content("modal-content")?.present);
    assert!(host.mask("modal-mask")?.present);
    assert!(host.overlay_lease_id().is_some());
    assert!(host.mask_overlay_lease_id().is_some());
    assert_eq!(host.overlay_layer_role(), Some(LayerRole::DialogContent));
    assert!(!host.input_allowed("outside-action")?);
    assert!(host.input_allowed("modal-close")?);
    Ok(())
}

#[test]
fn escape_and_outside_close_follow_proto_policy_and_restore_trigger_once() -> Result<(), BridgeError>
{
    let mut host = dialog("dismiss")?;
    host.set_focus_ready("dismiss-trigger", true)?;
    assert_eq!(
        host.focus("dismiss-trigger")?,
        FocusOperationResult::Accepted
    );

    host.open()?;
    let escape = host.dismiss_escape()?;
    assert_eq!(escape.open_change_count, 1);
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
    let outside = host.dismiss_outside()?;
    assert_eq!(outside.open_change_count, 1);
    assert!(!host.is_open()?);
    assert!(host.trigger("dismiss-trigger")?.focused);

    let mut alert = dialog("alert")?;
    alert.set_root_props(DialogRootProps {
        alert: true,
        ..DialogRootProps::default()
    })?;
    assert_eq!(alert.dismiss_outside()?.open_change_count, 0);
    assert!(!alert.is_open()?);
    alert.open()?;
    assert_eq!(alert.dismiss_outside()?.open_change_count, 0);
    assert!(alert.is_open()?);
    Ok(())
}
#[test]
fn title_and_description_are_projected_as_content_relations() -> Result<(), BridgeError> {
    let mut host = dialog("relations")?;
    let snapshot = host.snapshot()?;
    let content = snapshot.content.as_ref().expect("content");
    let title = snapshot.title.as_ref().expect("title");
    let description = snapshot.description.as_ref().expect("description");
    assert_eq!(content.labelled_by.as_deref(), title.dialog_id.as_deref());
    assert_eq!(
        content.described_by.as_deref(),
        description.dialog_id.as_deref()
    );
    assert_eq!(content.a11y.as_ref().expect("dialog a11y").role, "dialog");
    assert!(
        title
            .dialog_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        description
            .dialog_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
    );
    Ok(())
}

#[test]
fn focus_enters_content_traps_to_family_and_restores_trigger() -> Result<(), BridgeError> {
    let mut host = dialog("focus")?;
    host.set_focus_ready("focus-trigger", true)?;
    host.set_focus_ready("focus-close", true)?;
    assert_eq!(host.focus("focus-trigger")?, FocusOperationResult::Accepted);
    host.open()?;
    assert_eq!(host.focus_entry()?, FocusOperationResult::Accepted);
    assert!(
        host.snapshot()?
            .closes
            .iter()
            .find(|close| close.id == "focus-close")
            .expect("focus close")
            .focused
    );
    assert!(host.focus_trap_active()?);
    assert_eq!(host.focus("focus-trigger")?, FocusOperationResult::Rejected);
    assert_eq!(host.focus("focus-close")?, FocusOperationResult::Accepted);
    host.close(CloseReason::Programmatic)?;
    assert!(!host.focus_trap_active()?);
    assert!(host.trigger("focus-trigger")?.focused);
    Ok(())
}

#[test]
fn presence_reopens_and_stale_completion_is_rejected_after_remount() -> Result<(), BridgeError> {
    let mut host = dialog("stale")?;
    host.open()?;
    let old_revision = host.overlay_revision().expect("content revision");
    let old_epoch = host.content("stale-content")?.session.projection.view_epoch;
    host.close(CloseReason::Escape)?;
    assert!(!host.content("stale-content")?.present);
    host.open()?;
    assert!(host.content("stale-content")?.present);
    assert!(
        host.drain_overlay_events()
            .iter()
            .any(OverlayEvent::is_presence_enter)
    );

    host.remount_content("stale-content")?;
    let new_epoch = host.content("stale-content")?.session.projection.view_epoch;
    assert!(new_epoch > old_epoch);
    host.open()?;
    assert!(host.overlay_revision().expect("new revision") > old_revision);
    assert!(
        host.complete_close(old_revision, CloseReason::Escape)
            .is_err()
    );
    Ok(())
}

#[test]
fn terminal_disposal_is_idempotent_and_accesskit_close_commits_once() -> Result<(), BridgeError> {
    let mut host = dialog("dispose")?;
    let snapshot = host.snapshot()?;
    assert!(
        snapshot
            .trigger
            .as_ref()
            .expect("trigger")
            .a11y
            .as_ref()
            .expect("trigger a11y")
            .actions
            .iter()
            .any(|action| action == "activate")
    );
    assert_eq!(
        snapshot
            .content
            .as_ref()
            .expect("content")
            .a11y
            .as_ref()
            .expect("content a11y")
            .role,
        "dialog"
    );

    host.open()?;
    let outcome = host.press_close("dispose-close", InputSource::Accessibility)?;
    assert_eq!(outcome.close_press_count, 1);
    assert_eq!(outcome.open_change_count, 1);
    assert!(!host.is_open()?);
    let duplicate = host.press_close("dispose-close", InputSource::Accessibility)?;
    assert_eq!(duplicate.close_press_count, 0);
    assert_eq!(duplicate.open_change_count, 0);

    host.dispose()?;
    host.dispose()?;
    assert!(host.snapshot().is_err());
    assert!(
        host.dispatch(
            "dispose-close",
            InputKind::PressCommit,
            InputSource::Keyboard,
            None
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn disabled_root_and_close_never_emit_dialog_actions() -> Result<(), BridgeError> {
    let mut host = dialog("disabled")?;
    host.set_root_props(DialogRootProps {
        disabled: true,
        ..DialogRootProps::default()
    })?;
    assert_eq!(host.open()?.open_change_count, 0);
    assert_eq!(
        host.press_close("disabled-close", InputSource::Accessibility)?
            .close_press_count,
        0
    );
    Ok(())
}
