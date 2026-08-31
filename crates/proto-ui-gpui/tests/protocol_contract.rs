use proto_ui_gpui::{
    A11ySnapshot, AckDisposition, BridgeError, BridgeHandshake, BridgeState, HostIdentity,
    InputEnvelope, InputKind, InputPayload, InputSource, InstanceId, ProjectionAck,
    ProjectionPayload, ProjectionStatus, ProjectionTransaction, ProtocolVersion, PrototypeKey,
    SessionId, SlotProjection, StyleProjection, TemplateNode, ViewEpoch,
};

fn handshake() -> BridgeHandshake {
    BridgeHandshake::new(
        ProtocolVersion::new(1, 0),
        HostIdentity::new("sailbreak", "gpui-0.2.2", "linux-x11"),
        "sha256:fixture",
    )
}

#[test]
fn handshake_serializes_stable_runtime_identity() {
    let value = serde_json::to_value(handshake()).expect("handshake serializes");
    assert_eq!(
        value,
        serde_json::json!({
            "protocol": { "major": 1, "minor": 0 },
            "proto_ui": "main-snapshot",
            "host": {
                "name": "sailbreak",
                "gpui": "gpui-0.2.2",
                "platform": "linux-x11"
            },
            "registry_digest": "sha256:fixture"
        })
    );
}

#[test]
fn projection_preserves_slot_and_style_a11y_data() {
    let transaction = ProjectionTransaction::new(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
        ViewEpoch::new(1).expect("view epoch"),
        1,
        ProjectionPayload::new(
            vec![TemplateNode::slot("button-slot")],
            SlotProjection::new("button-slot", "Apply"),
            StyleProjection::new(["inline-flex".to_owned(), "bg-primary".to_owned()]),
            Some(A11ySnapshot::button("Apply", false)),
        ),
    )
    .expect("valid projection");

    let value = serde_json::to_value(transaction).expect("projection serializes");
    assert_eq!(value["template"][0]["kind"], "slot");
    assert_eq!(value["slot"]["accessible_name"], "Apply");
    assert_eq!(value["style"]["tokens"][0], "inline-flex");
    assert_eq!(value["a11y"]["role"], "button");
}

#[test]
fn tracker_rejects_stale_ack_and_input_after_epoch_change() {
    let mut state = BridgeState::new(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
    );
    state
        .install_view(ViewEpoch::new(1).expect("view epoch"))
        .expect("install view");
    state
        .accept_ack(ProjectionAck::applied(
            SessionId::new("session-1").expect("session id"),
            InstanceId::new("instance-1").expect("instance id"),
            ViewEpoch::new(1).expect("view epoch"),
            1,
        ))
        .expect("first ack");
    state
        .install_view(ViewEpoch::new(2).expect("view epoch"))
        .expect("replace view");

    let stale_ack = ProjectionAck::applied(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
        ViewEpoch::new(1).expect("view epoch"),
        2,
    );
    assert!(matches!(
        state.accept_ack(stale_ack),
        Err(BridgeError::StaleEpoch { .. })
    ));

    let input = InputEnvelope::new(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
        ViewEpoch::new(1).expect("view epoch"),
        1,
        InputPayload::new(
            "sample-1",
            "surface-1",
            InputSource::Mouse,
            InputKind::PressCommit,
        ),
    );
    assert!(matches!(
        state.accept_input(input),
        Err(BridgeError::StaleEpoch { .. })
    ));
}

#[test]
fn tracker_enforces_sequence_and_terminal_disposal() {
    let mut state = BridgeState::new(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
    );
    state
        .install_view(ViewEpoch::new(1).expect("view epoch"))
        .expect("install view");

    let input = |sequence| {
        InputEnvelope::new(
            SessionId::new("session-1").expect("session id"),
            InstanceId::new("instance-1").expect("instance id"),
            ViewEpoch::new(1).expect("view epoch"),
            sequence,
            InputPayload::new(
                format!("sample-{sequence}"),
                "surface-1",
                InputSource::Keyboard,
                InputKind::KeyDown,
            ),
        )
    };
    state.accept_input(input(1)).expect("first input");
    assert!(matches!(
        state.accept_input(input(1)),
        Err(BridgeError::NonMonotonicSequence { .. })
    ));

    state.dispose();
    assert!(matches!(
        state.accept_input(input(2)),
        Err(BridgeError::Disposed)
    ));
}

#[test]
fn invalid_identity_and_projection_inputs_fail_without_silent_fallback() {
    assert!(matches!(
        SessionId::new(" "),
        Err(BridgeError::InvalidIdentity { .. })
    ));
    assert!(matches!(ViewEpoch::new(0), Err(BridgeError::InvalidEpoch)));

    let result = ProjectionTransaction::new(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
        ViewEpoch::new(1).expect("view epoch"),
        0,
        ProjectionPayload::new(
            vec![],
            SlotProjection::new("slot", "label"),
            StyleProjection::new([]),
            None,
        ),
    );
    assert!(matches!(result, Err(BridgeError::InvalidCommit)));
}

#[test]
fn ack_disposition_is_explicit_for_superseded_and_applied_commits() {
    let mut state = BridgeState::new(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
    );
    state
        .install_view(ViewEpoch::new(1).expect("view epoch"))
        .expect("install view");
    let first = ProjectionAck::with_status(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
        ViewEpoch::new(1).expect("view epoch"),
        1,
        ProjectionStatus::Applied,
    );
    assert_eq!(
        state.accept_ack(first).expect("applied ack"),
        AckDisposition::Applied
    );

    let superseded = ProjectionAck::with_status(
        SessionId::new("session-1").expect("session id"),
        InstanceId::new("instance-1").expect("instance id"),
        ViewEpoch::new(1).expect("view epoch"),
        1,
        ProjectionStatus::Superseded,
    );
    assert_eq!(
        state.accept_ack(superseded).expect("superseded ack"),
        AckDisposition::Superseded
    );
}

#[test]
fn registry_accepts_only_governed_shadcn_prototype_keys() {
    assert_eq!(
        PrototypeKey::parse("shadcn-button").expect("button key"),
        PrototypeKey::ShadcnButton
    );
    assert_eq!(
        PrototypeKey::ShadcnSelectItem.to_string(),
        "shadcn-select-item"
    );
    assert!(matches!(
        PrototypeKey::parse("./arbitrary-import"),
        Err(BridgeError::UnknownPrototype { .. })
    ));
}

#[test]
fn bridge_commands_are_data_only_and_round_trip() {
    let command = proto_ui_gpui::BridgeCommand::Start {
        session_id: SessionId::new("session-1").expect("session id"),
        instance_id: InstanceId::new("instance-1").expect("instance id"),
        prototype: PrototypeKey::ShadcnButton,
        props: serde_json::json!({ "disabled": false })
            .as_object()
            .expect("object props")
            .clone(),
        slot: SlotProjection::new("button-slot", "Apply"),
    };
    let value = serde_json::to_value(&command).expect("command serializes");
    assert_eq!(value["type"], "start");
    assert_eq!(value["prototype"], "shadcn-button");
    assert!(serde_json::from_value::<proto_ui_gpui::BridgeCommand>(value).is_ok());
}

#[test]
fn remount_command_is_data_only_and_round_trips() {
    let command = proto_ui_gpui::BridgeCommand::Remount {
        session_id: SessionId::new("session-1").expect("session id"),
        instance_id: InstanceId::new("instance-1").expect("instance id"),
    };
    let value = serde_json::to_value(&command).expect("command serializes");
    assert_eq!(value["type"], "remount");
    assert!(serde_json::from_value::<proto_ui_gpui::BridgeCommand>(value).is_ok());
}
