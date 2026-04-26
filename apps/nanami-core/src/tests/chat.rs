use axum::body::Body;
use axum::http::{Request, StatusCode};
use nanami_protocol::{ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamEventKind};
use tower::ServiceExt;

use crate::openclaw::OpenClawService;

use super::support::{StubOpenClawService, body_json, body_text, router_with_openclaw};

#[tokio::test]
async fn chat_endpoint_rejects_empty_message() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Ok(ChatResponse {
            session_id: "sess_001".into(),
            message_id: "msg_001".into(),
            content: "unused".into(),
        }),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":""}"#))
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn chat_service_unconfigured_gateway_returns_structured_error() {
    let service = crate::openclaw::EnvOpenClawService;

    let error = service
        .send_chat_message(ChatRequest {
            session_id: None,
            message: "Hello".into(),
        })
        .await
        .unwrap_err();

    assert_eq!(error.code, "OPENCLAW_GATEWAY_UNCONFIGURED");
    assert!(!error.message.contains("token"));
}

#[tokio::test]
async fn chat_endpoint_returns_adapter_content() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Ok(ChatResponse {
            session_id: "sess_001".into(),
            message_id: "msg_001".into(),
            content: "Hello from adapter".into(),
        }),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Hello"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let json = body_json(response).await;

    assert_eq!(json["content"], "Hello from adapter");
}

#[tokio::test]
async fn chat_errors_do_not_leak_token() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Err(crate::chat_error(
            "OPENCLAW_AUTH_FAILED",
            "OpenClaw Gateway authentication failed",
            Some("Check NANAMI_OPENCLAW_TOKEN"),
        )),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Hello"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let text = body_text(response).await;

    assert!(!text.contains("secret-token"));
    assert!(!text.contains("Bearer"));
}

#[tokio::test]
async fn chat_stream_endpoint_returns_sse_content_type() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(vec![ChatStreamEvent {
            kind: ChatStreamEventKind::MessageCompleted,
            session_id: Some("sess_001".into()),
            message_id: Some("msg_001".into()),
            delta: None,
            content: Some("Hello".into()),
            error: None,
        }]),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Hello"}"#))
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
}

#[tokio::test]
async fn chat_stream_endpoint_contains_delta_and_completed() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(vec![
            ChatStreamEvent {
                kind: ChatStreamEventKind::MessageDelta,
                session_id: Some("sess_001".into()),
                message_id: Some("msg_001".into()),
                delta: Some("你".into()),
                content: None,
                error: None,
            },
            ChatStreamEvent {
                kind: ChatStreamEventKind::MessageCompleted,
                session_id: Some("sess_001".into()),
                message_id: Some("msg_001".into()),
                delta: None,
                content: Some("你好".into()),
                error: None,
            },
        ]),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Hello"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let text = body_text(response).await;

    assert!(text.contains("message_delta"));
    assert!(text.contains("message_completed"));
}

#[tokio::test]
async fn chat_stream_endpoint_unconfigured_gateway_returns_error_event() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Err(crate::chat_error(
            "OPENCLAW_GATEWAY_UNCONFIGURED",
            "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
            Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
        )),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Hello"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let text = body_text(response).await;

    assert!(text.contains("\"kind\":\"error\""));
    assert!(text.contains("OPENCLAW_GATEWAY_UNCONFIGURED"));
}

#[tokio::test]
async fn chat_stream_endpoint_rejects_empty_message() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    })
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/chat/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":""}"#))
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn chat_stream_service_unconfigured_gateway_returns_structured_error() {
    let service = crate::openclaw::EnvOpenClawService;

    let error = match service
        .stream_chat_message(ChatRequest {
            session_id: None,
            message: "Hello".into(),
        })
        .await
    {
        Ok(_) => panic!("expected unconfigured gateway error"),
        Err(error) => error,
    };

    assert_eq!(error.code, "OPENCLAW_GATEWAY_UNCONFIGURED");
}
