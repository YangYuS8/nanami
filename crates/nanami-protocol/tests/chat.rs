use chrono::DateTime;
use nanami_protocol::{
    ChatRequest, ChatResponse, ChatRole, ErrorPayload, ErrorSeverity, Event, EventEnvelope,
};

#[test]
fn chat_request_serializes_json_shape() {
    let request = ChatRequest {
        session_id: Some("sess_001".into()),
        message: "Hello Nanami".into(),
    };

    let json = serde_json::to_value(request).unwrap();

    assert_eq!(json["session_id"], "sess_001");
    assert_eq!(json["message"], "Hello Nanami");
}

#[test]
fn message_delta_event_serializes_json_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_msg_delta_001",
        timestamp,
        Event::MessageDelta {
            session_id: "sess_001".into(),
            message_id: "msg_001".into(),
            delta: "Hello".into(),
        },
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "message.delta");
    assert_eq!(json["session_id"], "sess_001");
    assert_eq!(json["message_id"], "msg_001");
    assert_eq!(json["delta"], "Hello");
}

#[test]
fn message_completed_event_serializes_json_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:01Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_msg_completed_001",
        timestamp,
        Event::MessageCompleted(ChatResponse {
            session_id: "sess_001".into(),
            message_id: "msg_001".into(),
            content: "Hello user".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "message.completed");
    assert_eq!(json["session_id"], "sess_001");
    assert_eq!(json["message_id"], "msg_001");
    assert_eq!(json["content"], "Hello user");
}

#[test]
fn error_occurred_event_serializes_chat_error_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:02Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_error_001",
        timestamp,
        Event::ErrorOccurred(ErrorPayload {
            task_id: None,
            severity: ErrorSeverity::Error,
            code: "OPENCLAW_CHAT_FAILED".into(),
            message: "OpenClaw chat request failed".into(),
            action_hint: Some("Check OpenClaw Gateway configuration".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "error.occurred");
    assert_eq!(json["severity"], "error");
    assert_eq!(json["code"], "OPENCLAW_CHAT_FAILED");
}

#[test]
fn chat_role_serializes_snake_case() {
    let json = serde_json::to_value(ChatRole::Assistant).unwrap();

    assert_eq!(json, "assistant");
}
