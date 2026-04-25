use chrono::DateTime;
use nanami_protocol::{
    ErrorPayload, ErrorSeverity, Event, EventEnvelope, OpenClawConnectionStatus,
    OpenClawStatusPayload,
};

#[test]
fn openclaw_status_payload_serializes_snake_case_status() {
    let payload = OpenClawStatusPayload {
        status: OpenClawConnectionStatus::PairingRequired,
        gateway_url: "http://127.0.0.1:18789".into(),
        message: Some("Pairing required".into()),
        agent: Some("default-agent".into()),
        profile: Some("desktop".into()),
    };

    let json = serde_json::to_value(payload).unwrap();

    assert_eq!(json["status"], "pairing_required");
    assert_eq!(json["gateway_url"], "http://127.0.0.1:18789");
    assert_eq!(json["message"], "Pairing required");
    assert_eq!(json["agent"], "default-agent");
    assert_eq!(json["profile"], "desktop");
}

#[test]
fn openclaw_status_event_serializes_json_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_openclaw_001",
        timestamp,
        Event::OpenClawStatus(OpenClawStatusPayload {
            status: OpenClawConnectionStatus::Connected,
            gateway_url: "http://127.0.0.1:18789".into(),
            message: None,
            agent: None,
            profile: None,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "openclaw.status");
    assert_eq!(json["id"], "evt_openclaw_001");
    assert_eq!(json["status"], "connected");
    assert_eq!(json["gateway_url"], "http://127.0.0.1:18789");
}

#[test]
fn openclaw_status_event_deserializes_from_json() {
    let json = r#"
        {
            "type": "openclaw.status",
            "id": "evt_openclaw_001",
            "timestamp": "2026-01-01T00:00:00Z",
            "status": "auth_failed",
            "gateway_url": "http://127.0.0.1:18789",
            "message": "Authentication failed"
        }
    "#;

    let event: EventEnvelope = serde_json::from_str(json).unwrap();

    assert_eq!(
        event.event,
        Event::OpenClawStatus(OpenClawStatusPayload {
            status: OpenClawConnectionStatus::AuthFailed,
            gateway_url: "http://127.0.0.1:18789".into(),
            message: Some("Authentication failed".into()),
            agent: None,
            profile: None,
        })
    );
}

#[test]
fn error_occurred_event_serializes_json_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:01Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_error_001",
        timestamp,
        Event::ErrorOccurred(ErrorPayload {
            task_id: None,
            severity: ErrorSeverity::Error,
            code: "OPENCLAW_UNREACHABLE".into(),
            message: "OpenClaw Gateway is unreachable".into(),
            action_hint: Some("Check NANAMI_OPENCLAW_GATEWAY_URL".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "error.occurred");
    assert_eq!(json["severity"], "error");
    assert_eq!(json["code"], "OPENCLAW_UNREACHABLE");
    assert_eq!(json["action_hint"], "Check NANAMI_OPENCLAW_GATEWAY_URL");
}
