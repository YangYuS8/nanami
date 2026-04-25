use nanami_protocol::{ChatStreamEvent, ChatStreamEventKind, ErrorPayload, ErrorSeverity};

#[test]
fn chat_stream_delta_serializes_json_shape() {
    let event = ChatStreamEvent {
        kind: ChatStreamEventKind::MessageDelta,
        session_id: Some("sess_001".into()),
        message_id: Some("msg_001".into()),
        delta: Some("你".into()),
        content: None,
        error: None,
    };

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["kind"], "message_delta");
    assert_eq!(json["session_id"], "sess_001");
    assert_eq!(json["message_id"], "msg_001");
    assert_eq!(json["delta"], "你");
}

#[test]
fn chat_stream_completed_serializes_json_shape() {
    let event = ChatStreamEvent {
        kind: ChatStreamEventKind::MessageCompleted,
        session_id: Some("sess_001".into()),
        message_id: Some("msg_001".into()),
        delta: None,
        content: Some("你好".into()),
        error: None,
    };

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["kind"], "message_completed");
    assert_eq!(json["content"], "你好");
}

#[test]
fn chat_stream_error_serializes_json_shape() {
    let event = ChatStreamEvent {
        kind: ChatStreamEventKind::Error,
        session_id: None,
        message_id: None,
        delta: None,
        content: None,
        error: Some(ErrorPayload {
            task_id: None,
            severity: ErrorSeverity::Error,
            code: "OPENCLAW_CHAT_FAILED".into(),
            message: "stream failed".into(),
            action_hint: None,
        }),
    };

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["kind"], "error");
    assert_eq!(json["error"]["code"], "OPENCLAW_CHAT_FAILED");
}
