use proto_ui_gpui::{
    BridgeCommand, BridgeEvent, InstanceId, PrototypeKey, QuickJsBridge, SessionId, SlotProjection,
};

#[test]
fn embedded_runtime_materializes_the_real_shadcn_button() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let command = BridgeCommand::Start {
        session_id: SessionId::new("session-1").expect("session id"),
        instance_id: InstanceId::new("instance-1").expect("instance id"),
        prototype: PrototypeKey::ShadcnButton,
        props: serde_json::json!({
            "variant": "default",
            "size": "default",
            "disabled": false
        })
        .as_object()
        .expect("object props")
        .clone(),
        slot: SlotProjection::new("button-slot", "Apply"),
    };

    let events = bridge.dispatch(&command).expect("button session starts");
    let handshake = events
        .iter()
        .find_map(|event| match event {
            BridgeEvent::Ready { handshake } => Some(handshake),
            _ => None,
        })
        .expect("ready event");
    assert_eq!(handshake.proto_ui, "0.3.0-alpha.0");
    assert!(handshake.registry_digest.starts_with("sha256:"));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, BridgeEvent::Projection { .. }))
    );
}

#[test]
fn projection_ack_unlocks_native_state_and_one_click_signal() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let session_id = SessionId::new("session-1").expect("session id");
    let instance_id = InstanceId::new("instance-1").expect("instance id");
    let start = BridgeCommand::Start {
        session_id: session_id.clone(),
        instance_id: instance_id.clone(),
        prototype: PrototypeKey::ShadcnButton,
        props: serde_json::json!({
            "variant": "default",
            "size": "default",
            "disabled": false
        })
        .as_object()
        .expect("object props")
        .clone(),
        slot: SlotProjection::new("button-slot", "Apply"),
    };
    let start_events = bridge.dispatch(&start).expect("button session starts");
    let projection = start_events
        .iter()
        .find_map(|event| match event {
            BridgeEvent::Projection { projection } => Some(projection),
            _ => None,
        })
        .expect("start emits projection");

    let ack_events = bridge
        .dispatch(&BridgeCommand::ProjectionAck {
            ack: proto_ui_gpui::ProjectionAck::applied(
                session_id.clone(),
                instance_id.clone(),
                projection.view_epoch,
                projection.commit_id,
            ),
        })
        .expect("projection ack succeeds");
    assert!(
        ack_events
            .iter()
            .any(|event| matches!(event, BridgeEvent::A11y { .. }))
    );
    assert!(
        ack_events
            .iter()
            .any(|event| matches!(event, BridgeEvent::State { .. }))
    );

    let input = |kind, sequence, sample_id| BridgeCommand::Input {
        input: proto_ui_gpui::InputEnvelope::new(
            session_id.clone(),
            instance_id.clone(),
            projection.view_epoch,
            sequence,
            proto_ui_gpui::InputPayload::new(
                sample_id,
                "button-1",
                proto_ui_gpui::InputSource::Mouse,
                kind,
            ),
        ),
        detail: None,
    };
    bridge
        .dispatch(&input(
            proto_ui_gpui::InputKind::PointerDown,
            1,
            "mouse-down",
        ))
        .expect("pointer down");
    bridge
        .dispatch(&input(proto_ui_gpui::InputKind::PointerUp, 2, "mouse-up"))
        .expect("pointer up");
    let click_events = bridge
        .dispatch(&input(
            proto_ui_gpui::InputKind::PressCommit,
            3,
            "mouse-click",
        ))
        .expect("press commit");
    assert_eq!(
        click_events
            .iter()
            .filter(|event| matches!(event, BridgeEvent::Signal { key, .. } if key == "click"))
            .count(),
        1
    );

    let duplicate_events = bridge
        .dispatch(&input(
            proto_ui_gpui::InputKind::PressCommit,
            4,
            "mouse-click",
        ))
        .expect("duplicate sample is ignored");
    assert!(
        !duplicate_events
            .iter()
            .any(|event| matches!(event, BridgeEvent::Signal { key, .. } if key == "click"))
    );
}

#[test]
fn embedded_runtime_rejects_an_unregistered_prototype_before_execution() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let command = serde_json::json!({
        "type": "start",
        "session_id": "session-1",
        "instance_id": "instance-1",
        "prototype": "not-a-prototype",
        "props": {},
        "slot": { "slot_id": "button-slot", "accessible_name": "Apply" }
    });

    let error = bridge
        .dispatch_json(&command.to_string())
        .expect_err("unknown key fails");
    assert!(error.to_string().contains("unknown Proto UI prototype"));
}

#[test]
fn embedded_registry_contains_every_published_shadcn_direct_entry() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let events = bridge
        .dispatch(&BridgeCommand::Registry)
        .expect("registry query succeeds");
    let keys = events
        .iter()
        .find_map(|event| match event {
            BridgeEvent::Registry { keys, .. } => Some(keys),
            _ => None,
        })
        .expect("registry event");
    let expected: std::collections::BTreeSet<&str> =
        PrototypeKey::all().iter().map(|key| key.as_str()).collect();
    let actual: std::collections::BTreeSet<&str> = keys.iter().map(String::as_str).collect();
    assert_eq!(actual, expected);
}

#[test]
fn bridge_rejects_oversized_messages_before_quickjs() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let padding = "x".repeat(256 * 1024);
    let serialized = format!(r#"{{"type":"registry","padding":"{padding}"}}"#);
    assert!(matches!(
        bridge.dispatch_json(&serialized),
        Err(proto_ui_gpui::BridgeError::Decode { .. })
    ));
}

#[test]
fn bridge_rejects_json_nested_beyond_the_protocol_limit() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let mut serialized = "null".to_owned();
    for _ in 0..17 {
        serialized = format!("[{serialized}]");
    }
    assert!(matches!(
        bridge.dispatch_json(&serialized),
        Err(proto_ui_gpui::BridgeError::Decode { .. })
    ));
}

#[test]
fn stale_unmount_epoch_is_rejected_before_runtime() {
    let mut bridge = QuickJsBridge::new().expect("embedded QuickJS starts");
    let session_id = SessionId::new("session-1").expect("session id");
    let instance_id = InstanceId::new("instance-1").expect("instance id");
    let start = BridgeCommand::Start {
        session_id: session_id.clone(),
        instance_id: instance_id.clone(),
        prototype: PrototypeKey::ShadcnButton,
        props: serde_json::Map::new(),
        slot: SlotProjection::new("button-slot", "Apply"),
    };
    bridge.dispatch(&start).expect("button starts");
    let unmount = BridgeCommand::Unmount {
        session_id,
        instance_id,
        view_epoch: proto_ui_gpui::ViewEpoch::new(99).expect("view epoch"),
    };
    assert!(matches!(
        bridge.dispatch(&unmount),
        Err(proto_ui_gpui::BridgeError::Runtime { .. })
    ));
}
