use proto_ui_gpui::{
    BridgeError, InputKind, InputRequest, InputSource, InstanceId, ProjectionAck, ProtoSessionHost,
    SessionId, StartRequest,
};

fn request(session: &str, instance: &str) -> StartRequest {
    StartRequest::button(
        SessionId::new(session).expect("session id"),
        InstanceId::new(instance).expect("instance id"),
        "Apply",
    )
}

#[test]
fn session_lifecycle_is_epoch_safe() -> Result<(), BridgeError> {
    let request = request("session-1", "instance-1");
    let mut host = ProtoSessionHost::new()?;
    let snapshot = host.start(request.clone())?;
    assert_eq!(snapshot.prototype.as_str(), "shadcn-button");
    assert_eq!(snapshot.projection.view_epoch.get(), 1);
    assert!(snapshot.pending_commit);

    let first_epoch = snapshot.projection.view_epoch;
    host.acknowledge(ProjectionAck::applied(
        request.session_id.clone(),
        request.instance_id.clone(),
        first_epoch,
        snapshot.projection.commit_id,
    ))?;
    let second_epoch = host.remount()?;
    assert!(second_epoch.get() > first_epoch.get());

    let stale = ProjectionAck::applied(
        request.session_id,
        request.instance_id,
        first_epoch,
        snapshot.projection.commit_id + 1,
    );
    assert!(matches!(
        host.acknowledge(stale),
        Err(BridgeError::StaleEpoch { .. })
    ));
    Ok(())
}

#[test]
fn dispose_is_terminal_and_idempotent() -> Result<(), BridgeError> {
    let request = request("session-1", "instance-1");
    let mut host = ProtoSessionHost::new()?;
    let snapshot = host.start(request.clone())?;
    host.acknowledge(ProjectionAck::applied(
        request.session_id.clone(),
        request.instance_id.clone(),
        snapshot.projection.view_epoch,
        snapshot.projection.commit_id,
    ))?;

    host.dispose()?;
    host.dispose()?;
    let input = InputRequest::new(
        proto_ui_gpui::InputEnvelope::new(
            request.session_id,
            request.instance_id,
            snapshot.projection.view_epoch,
            1,
            proto_ui_gpui::InputPayload::new(
                "sample-1",
                "route-1",
                InputSource::Mouse,
                InputKind::PressCommit,
            ),
        ),
        None,
    );
    assert!(matches!(host.input(input), Err(BridgeError::Disposed)));
    Ok(())
}

#[test]
fn duplicate_session_start_is_rejected() -> Result<(), BridgeError> {
    let request = request("session-1", "instance-1");
    let mut host = ProtoSessionHost::new()?;
    host.start(request.clone())?;
    assert!(matches!(
        host.start(request),
        Err(BridgeError::InvalidIdentity { .. })
    ));
    Ok(())
}

#[test]
fn input_rejects_a_stale_route_and_returns_signals() -> Result<(), BridgeError> {
    let request = request("session-1", "instance-1");
    let mut host = ProtoSessionHost::new()?;
    let snapshot = host.start(request.clone())?;
    host.acknowledge(ProjectionAck::applied(
        request.session_id.clone(),
        request.instance_id.clone(),
        snapshot.projection.view_epoch,
        snapshot.projection.commit_id,
    ))?;

    let input = |sequence, route_ref| {
        InputRequest::new(
            proto_ui_gpui::InputEnvelope::new(
                request.session_id.clone(),
                request.instance_id.clone(),
                snapshot.projection.view_epoch,
                sequence,
                proto_ui_gpui::InputPayload::new(
                    format!("sample-{sequence}"),
                    route_ref,
                    InputSource::Mouse,
                    if sequence == 3 {
                        InputKind::PressCommit
                    } else if sequence == 1 {
                        InputKind::PointerDown
                    } else {
                        InputKind::PointerUp
                    },
                ),
            ),
            None,
        )
    };
    host.input(input(1, "route-1"))?;
    host.input(input(2, "route-1"))?;
    let outcome = host.input(input(3, "route-1"))?;
    assert!(outcome.click_emitted);
    assert!(outcome.events.iter().any(
        |event| matches!(event, proto_ui_gpui::BridgeEvent::Signal { key, .. } if key == "click")
    ));

    assert!(matches!(
        host.input(input(4, "stale-route")),
        Err(BridgeError::RouteMismatch { .. })
    ));
    Ok(())
}

#[test]
fn input_waits_for_the_projection_ack_barrier() -> Result<(), BridgeError> {
    let request = request("session-1", "instance-1");
    let mut host = ProtoSessionHost::new()?;
    let snapshot = host.start(request.clone())?;
    let input = InputRequest::new(
        proto_ui_gpui::InputEnvelope::new(
            request.session_id,
            request.instance_id,
            snapshot.projection.view_epoch,
            1,
            proto_ui_gpui::InputPayload::new(
                "sample-1",
                "route-1",
                InputSource::Mouse,
                InputKind::PointerDown,
            ),
        ),
        None,
    );
    assert!(matches!(
        host.input(input),
        Err(BridgeError::ProjectionPending { .. })
    ));
    Ok(())
}
