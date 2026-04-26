use crate::openclaw::OpenClawService;
use crate::routes::tasks::maybe_permission_for_tool_event;
use crate::state::{MANIFEST_PREVIEW_MAX_BYTES, NanamiEventStream};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use nanami_openclaw::OpenClawChatStream;
use nanami_protocol::{
    ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamEventKind, ErrorPayload, Event,
    EventEnvelope, OpenClawConnectionStatus, TaskCompletedPayload, TaskStartedPayload, TaskStatus,
    ToolOutputPayload, ToolOutputStream, ToolStartedPayload,
};
use std::pin::Pin;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_endpoint_returns_protocol_version() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["protocol_version"], nanami_protocol::PROTOCOL_VERSION);
}

#[tokio::test]
async fn openclaw_status_unconfigured_returns_disconnected() {
    let status = crate::openclaw_status_from_config(Some("".into())).await;

    assert_eq!(status.status, OpenClawConnectionStatus::Disconnected);
    assert_eq!(status.gateway_url, "");
    assert!(status.message.is_some());
}

#[tokio::test]
async fn openclaw_status_endpoint_returns_ok() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/openclaw/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn openclaw_status_endpoint_returns_status_and_gateway_url() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/openclaw/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("status").is_some());
    assert!(json.get("gateway_url").is_some());
}

#[derive(Clone)]
struct StubOpenClawService {
    response: Result<ChatResponse, ErrorPayload>,
    stream_response: Result<Vec<ChatStreamEvent>, ErrorPayload>,
    agent_stream_response: Result<Vec<EventEnvelope>, ErrorPayload>,
}

impl OpenClawService for StubOpenClawService {
    fn send_chat_message(
        &self,
        _request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>> {
        Box::pin(async { self.response.clone() })
    }

    fn stream_chat_message(
        &self,
        _request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            self.stream_response.clone().map(|events| {
                Box::pin(tokio_stream::iter(events.into_iter().map(Ok))) as OpenClawChatStream
            })
        })
    }

    fn stream_agent_events(
        &self,
        _request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            self.agent_stream_response.clone().map(|events| {
                Box::pin(tokio_stream::iter(events.into_iter().map(Ok))) as NanamiEventStream
            })
        })
    }
}

#[tokio::test]
async fn chat_endpoint_rejects_empty_message() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Ok(ChatResponse {
            session_id: "sess_001".into(),
            message_id: "msg_001".into(),
            content: "unused".into(),
        }),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    }))
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
    let service = crate::EnvOpenClawService;

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
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Ok(ChatResponse {
            session_id: "sess_001".into(),
            message_id: "msg_001".into(),
            content: "Hello from adapter".into(),
        }),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    }))
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["content"], "Hello from adapter");
}

#[tokio::test]
async fn chat_errors_do_not_leak_token() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error(
            "OPENCLAW_AUTH_FAILED",
            "OpenClaw Gateway authentication failed",
            Some("Check NANAMI_OPENCLAW_TOKEN"),
        )),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    }))
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(!text.contains("secret-token"));
    assert!(!text.contains("Bearer"));
}

#[tokio::test]
async fn chat_stream_endpoint_returns_sse_content_type() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
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
    }))
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
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
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
    }))
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("message_delta"));
    assert!(text.contains("message_completed"));
}

#[tokio::test]
async fn chat_stream_endpoint_unconfigured_gateway_returns_error_event() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Err(crate::chat_error(
            "OPENCLAW_GATEWAY_UNCONFIGURED",
            "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
            Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
        )),
        agent_stream_response: Ok(Vec::new()),
    }))
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("\"kind\":\"error\""));
    assert!(text.contains("OPENCLAW_GATEWAY_UNCONFIGURED"));
}

#[tokio::test]
async fn chat_stream_endpoint_rejects_empty_message() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(Vec::new()),
    }))
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
    let service = crate::EnvOpenClawService;

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

#[tokio::test]
async fn tasks_mock_stream_returns_sse_content_type() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/tasks/mock/stream")
                .body(Body::empty())
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
async fn tasks_mock_stream_contains_mock_task_and_tool_events() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/tasks/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("task.started"));
    assert!(text.contains("tool.started"));
    assert!(text.contains("tool.output"));
    assert!(text.contains("tool.completed"));
    assert!(text.contains("task.completed"));
}

#[tokio::test]
async fn sandbox_mock_stream_returns_sse_content_type() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/sandbox/mock/stream")
                .body(Body::empty())
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
async fn sandbox_mock_stream_contains_sandbox_event_sequence() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/sandbox/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("sandbox.started"));
    assert!(text.contains("sandbox.updated"));
    assert!(text.contains("sandbox.output"));
    assert!(text.contains("sandbox.artifact"));
    assert!(text.contains("sandbox.completed"));
}

#[tokio::test]
async fn persona_mock_stream_returns_sse_content_type() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/persona/mock/stream")
                .body(Body::empty())
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
async fn persona_mock_stream_contains_persona_event_sequence() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/persona/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("persona.state"));
    assert!(text.contains("\"state\":\"idle\""));
    assert!(text.contains("\"state\":\"listening\""));
    assert!(text.contains("\"state\":\"thinking\""));
    assert!(text.contains("\"state\":\"tool_call\""));
    assert!(text.contains("\"state\":\"waiting_permission\""));
    assert!(text.contains("\"state\":\"success\""));
    assert!(text.contains("\"state\":\"error\""));
    assert!(text.contains("\"source\":\"mock\""));
}

#[tokio::test]
async fn workflow_mock_stream_returns_sse_content_type() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/workflow/mock/stream")
                .body(Body::empty())
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
async fn workflow_mock_stream_contains_workflow_event_sequence() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/workflow/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("workflow.started"));
    assert!(text.contains("workflow.step"));
    assert!(text.contains("workflow.test_result"));
    assert!(text.contains("workflow.patch_proposed"));
    assert!(text.contains("workflow.completed"));
    assert!(text.contains("\"step_kind\":\"open_project\""));
    assert!(text.contains("\"step_kind\":\"analyze_project\""));
    assert!(text.contains("\"step_kind\":\"run_tests\""));
    assert!(text.contains("\"step_kind\":\"apply_patch\""));
    assert!(text.contains("\"status\":\"waiting_permission\""));
    assert!(text.contains("\"command_preview\":\"cargo test --lib\""));
    assert!(text.contains("\"duration_ms\":1200"));
    assert!(text.contains("\"failed_test_names\":[\"tests::mock_failure\"]"));
    assert!(text.contains("\"risk_level\":\"medium\""));
}

#[tokio::test]
async fn projects_mock_current_returns_mock_project_metadata() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/projects/mock/current")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project_id"], "project_mock_001");
    assert_eq!(json["display_name"], "Nanami Mock Workspace");
    assert_eq!(json["project_path"], "/mock/project");
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["trust_status"], "trusted_mock");
}

#[tokio::test]
async fn projects_select_detects_top_level_rust_manifest_and_returns_selected_untrusted() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_project_select_rust_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["trust_status"], "selected_untrusted");
}

#[tokio::test]
async fn projects_select_returns_unknown_when_no_top_level_manifest_exists() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_project_select_unknown_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(temp_dir.join("nested")).unwrap();
    std::fs::write(temp_dir.join("nested").join("Cargo.toml"), "").unwrap();

    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("nested").join("Cargo.toml"));
    let _ = std::fs::remove_dir(temp_dir.join("nested"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(json["kind"], "unknown");
    assert_eq!(json["trust_status"], "selected_untrusted");
}

#[tokio::test]
async fn projects_trust_updates_selected_project_to_selected_trusted() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_project_trust_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = trust_response.status();
    let trust_body = axum::body::to_bytes(trust_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let trust_json: serde_json::Value = serde_json::from_slice(&trust_body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(trust_json["trust_status"], "selected_trusted");
}

#[tokio::test]
async fn projects_trust_rejects_non_selected_project_id() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project_id":"project_missing_001"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn projects_current_structure_requires_selected_trusted_project() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/projects/current/structure")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn projects_current_structure_returns_shallow_summary_for_selected_trusted_project() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_project_structure_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(temp_dir.join("src")).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();
    std::fs::write(temp_dir.join("README.md"), "").unwrap();
    std::fs::write(temp_dir.join(".gitignore"), "").unwrap();
    std::fs::write(temp_dir.join("src").join("nested.rs"), "").unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let structure_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/structure")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = structure_response.status();
    let body = axum::body::to_bytes(structure_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_file(temp_dir.join("README.md"));
    let _ = std::fs::remove_file(temp_dir.join(".gitignore"));
    let _ = std::fs::remove_file(temp_dir.join("src").join("nested.rs"));
    let _ = std::fs::remove_dir(temp_dir.join("src"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project_id"], project_id);
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "Cargo.toml"
                && entry["entry_type"] == "file"
                && entry["marker"] == "manifest")
    );
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "src"
                && entry["entry_type"] == "directory"
                && entry["marker"] == "source_dir")
    );
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == ".gitignore" && entry["marker"] == "config")
    );
    assert!(
        json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "README.md" && entry["marker"] == "other")
    );
    assert!(
        !json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["relative_path"] == "src/nested.rs")
    );
}

#[tokio::test]
async fn projects_current_manifest_preview_request_requires_selected_trusted_project() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn projects_current_manifest_preview_request_records_l2_permission_for_top_level_manifest() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_preview_request_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let preview_request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = preview_request_response.status();
    let body = axum::body::to_bytes(preview_request_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["level"], "l2");
    assert_eq!(json["action"], "filesystem.read");
    assert_eq!(
        json["permission_id"],
        format!("perm_manifest_preview_{}", project_id)
    );
    assert_eq!(
        json["target"],
        temp_dir.join("Cargo.toml").display().to_string()
    );
}

#[tokio::test]
async fn projects_current_manifest_preview_requires_permission_decision() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_preview_permission_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/preview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(preview_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn projects_current_manifest_preview_returns_top_level_preview_after_allow_once() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_preview_allow_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(temp_dir.join("nested")).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    std::fs::write(
        temp_dir.join("nested").join("Cargo.toml"),
        "[package]\nname = \"nested\"\n",
    )
    .unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"permission_id":"{}","decision":"allow_once"}}"#,
                    permission_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/preview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = preview_response.status();
    let body = axum::body::to_bytes(preview_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_file(temp_dir.join("nested").join("Cargo.toml"));
    let _ = std::fs::remove_dir(temp_dir.join("nested"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["kind"], "rust");
    assert_eq!(
        json["manifest_path"],
        temp_dir.join("Cargo.toml").display().to_string()
    );
    assert_eq!(json["content_preview"], "[package]\nname = \"demo\"\n");
    assert_eq!(json["truncated"], false);
    assert_eq!(json["size_bytes"], 24);
}

#[tokio::test]
async fn projects_current_manifest_preview_truncates_to_8kb() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_preview_truncate_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let manifest_content = "a".repeat((MANIFEST_PREVIEW_MAX_BYTES as usize) + 17);
    std::fs::write(temp_dir.join("Cargo.toml"), &manifest_content).unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"permission_id":"{}","decision":"allow_for_task"}}"#,
                    permission_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/preview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(preview_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(json["truncated"], true);
    assert_eq!(json["size_bytes"], MANIFEST_PREVIEW_MAX_BYTES + 17);
    assert_eq!(
        json["content_preview"].as_str().unwrap().len(),
        MANIFEST_PREVIEW_MAX_BYTES as usize
    );
}

#[tokio::test]
async fn projects_current_manifest_summary_requires_permission_decision() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_summary_permission_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(summary_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn projects_current_manifest_summary_extracts_rust_fields_after_allow_once() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_summary_rust_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
            temp_dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\ntokio = \"1\"\n",
        )
        .unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"permission_id":"{}","decision":"allow_once"}}"#,
                    permission_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = summary_response.status();
    let body = axum::body::to_bytes(summary_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["kind"], "rust");
    assert_eq!(json["package_name"], "demo");
    assert_eq!(json["package_version"], "0.1.0");
    assert_eq!(json["dependency_count"], 2);
    assert!(json["script_count"].is_null());
}

#[tokio::test]
async fn projects_current_manifest_summary_extracts_node_fields_with_scripts_and_dependencies() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_summary_node_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
            temp_dir.join("package.json"),
            r#"{"name":"demo-node","version":"1.2.3","dependencies":{"react":"18"},"devDependencies":{"vite":"5","typescript":"5"},"scripts":{"dev":"vite","build":"vite build"}}"#,
        )
        .unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"permission_id":"{}","decision":"allow_for_task"}}"#,
                    permission_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(summary_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("package.json"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(json["kind"], "node");
    assert_eq!(json["package_name"], "demo-node");
    assert_eq!(json["package_version"], "1.2.3");
    assert_eq!(json["dependency_count"], 3);
    assert_eq!(json["script_count"], 2);
}

#[tokio::test]
async fn projects_current_manifest_summary_extracts_python_fields_and_tolerates_parse_failure() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_manifest_summary_python_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
            temp_dir.join("pyproject.toml"),
            "[project]\nname = \"demo-py\"\nversion = \"0.2.0\"\ndependencies = [\"fastapi\", \"uvicorn\"]\n",
        )
        .unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();
    let permission_id = format!("perm_manifest_preview_{}", project_id);

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let request_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/current/manifest/preview-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"permission_id":"{}","decision":"allow_for_task"}}"#,
                    permission_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = summary_response.status();
    let body = axum::body::to_bytes(summary_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    std::fs::write(temp_dir.join("pyproject.toml"), "not valid toml = [").unwrap();

    let fallback_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects/current/manifest/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let fallback_body = axum::body::to_bytes(fallback_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fallback_json: serde_json::Value = serde_json::from_slice(&fallback_body).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("pyproject.toml"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["kind"], "python");
    assert_eq!(json["package_name"], "demo-py");
    assert_eq!(json["package_version"], "0.2.0");
    assert_eq!(json["dependency_count"], 2);
    assert!(json["script_count"].is_null());

    assert_eq!(fallback_json["kind"], "python");
    assert!(fallback_json["package_name"].is_null());
    assert!(fallback_json["package_version"].is_null());
    assert!(fallback_json["dependency_count"].is_null());
    assert_eq!(
        fallback_json["summary_text"],
        "Manifest summary unavailable"
    );
}

#[tokio::test]
async fn workflow_mock_apply_patch_records_permission_and_returns_waiting_status() {
    let app = crate::router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/workflow/mock/apply-patch")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"patch_id":"patch_mock_001"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["patch_id"], "patch_mock_001");
    assert_eq!(json["status"], "waiting_permission");
    assert_eq!(json["permission_id"], "perm_workflow_patch_patch_mock_001");

    let audit_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let audit_body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let audit_json: serde_json::Value = serde_json::from_slice(&audit_body).unwrap();

    assert!(
        audit_json["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |record| record["permission_id"] == "perm_workflow_patch_patch_mock_001"
                    && record["action"] == "permission_requested"
                    && record["permission_action"] == "filesystem.write"
            )
    );
}

#[tokio::test]
async fn workflow_mock_current_project_stream_requires_selected_trusted_project() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/workflow/mock/current-project/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn workflow_mock_current_project_stream_uses_selected_project_metadata_and_structure_count() {
    let temp_dir = std::env::temp_dir().join(format!(
        "nanami_workflow_current_project_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(temp_dir.join("src")).unwrap();
    std::fs::create_dir_all(temp_dir.join("crates")).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();
    std::fs::write(temp_dir.join("README.md"), "").unwrap();

    let app = crate::router();
    let select_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    temp_dir.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
    let project_id = select_json["project_id"].as_str().unwrap().to_owned();

    let trust_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/trust")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trust_response.status(), StatusCode::OK);

    let workflow_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/workflow/mock/current-project/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = workflow_response.status();
    let body = axum::body::to_bytes(workflow_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
    let _ = std::fs::remove_file(temp_dir.join("README.md"));
    let _ = std::fs::remove_dir(temp_dir.join("src"));
    let _ = std::fs::remove_dir(temp_dir.join("crates"));
    let _ = std::fs::remove_dir(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert!(text.contains("workflow.started"));
    assert!(text.contains(&project_id));
    assert!(text.contains(&temp_dir.display().to_string()));
    assert!(text.contains("selected_trusted"));
    assert!(text.contains("rust"));
    assert!(text.contains("4 top-level entries"));
}

#[tokio::test]
async fn tasks_openclaw_stream_returns_sse_content_type() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![EventEnvelope::new(
            "evt_001",
            chrono::Utc::now(),
            Event::TaskStarted(TaskStartedPayload {
                session_id: None,
                task_id: "task_openclaw_stream_001".into(),
                title: "OpenClaw task".into(),
                status: TaskStatus::Running,
            }),
        )]),
    }))
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/tasks/openclaw/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Run task"}"#))
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
async fn tasks_openclaw_stream_contains_task_and_tool_events() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![
            EventEnvelope::new(
                "evt_001",
                chrono::Utc::now(),
                Event::TaskStarted(TaskStartedPayload {
                    session_id: None,
                    task_id: "task_openclaw_stream_001".into(),
                    title: "OpenClaw task".into(),
                    status: TaskStatus::Running,
                }),
            ),
            EventEnvelope::new(
                "evt_002",
                chrono::Utc::now(),
                Event::ToolStarted(ToolStartedPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    tool_call_id: "call_001".into(),
                    tool: "mock.shell".into(),
                    summary: Some("OpenClaw tool call detected".into()),
                }),
            ),
            EventEnvelope::new(
                "evt_003",
                chrono::Utc::now(),
                Event::ToolOutput(ToolOutputPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    tool_call_id: "call_001".into(),
                    stream: ToolOutputStream::Log,
                    content: "{\"command\":\"cargo check\"}".into(),
                }),
            ),
            EventEnvelope::new(
                "evt_004",
                chrono::Utc::now(),
                Event::TaskCompleted(TaskCompletedPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    status: TaskStatus::Completed,
                    summary: Some("OpenClaw stream completed".into()),
                }),
            ),
        ]),
    }))
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/tasks/openclaw/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Run task"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("task.started"));
    assert!(text.contains("tool.started"));
    assert!(text.contains("tool.output"));
    assert!(text.contains("task.completed"));
}

#[tokio::test]
async fn tasks_openclaw_stream_contains_sandbox_events() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![
            EventEnvelope::new(
                "evt_sandbox_started_001",
                chrono::Utc::now(),
                Event::SandboxStarted(nanami_protocol::SandboxStartedPayload {
                    sandbox_id: "sandbox_001".into(),
                    task_id: "task_openclaw_stream_001".into(),
                    template_id: "rust-workspace".into(),
                    status: nanami_protocol::SandboxStatus::Starting,
                    network_policy: nanami_protocol::SandboxNetworkPolicy::Disabled,
                    mounts: vec![nanami_protocol::SandboxMountPayload {
                        host_path: "/mock/host/project".into(),
                        sandbox_path: "/workspace/project".into(),
                        mode: nanami_protocol::SandboxMountMode::ReadOnly,
                    }],
                }),
            ),
            EventEnvelope::new(
                "evt_sandbox_output_001",
                chrono::Utc::now(),
                Event::SandboxOutput(nanami_protocol::SandboxOutputPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    sandbox_id: "sandbox_001".into(),
                    stream: ToolOutputStream::Stdout,
                    content: "checking workspace...".into(),
                }),
            ),
            EventEnvelope::new(
                "evt_sandbox_artifact_001",
                chrono::Utc::now(),
                Event::SandboxArtifact(nanami_protocol::SandboxArtifactPayload {
                    sandbox_id: "sandbox_001".into(),
                    task_id: "task_openclaw_stream_001".into(),
                    name: "mock-report.txt".into(),
                    path: "/workspace/output/mock-report.txt".into(),
                    media_type: "text/plain".into(),
                    size_bytes: 128,
                }),
            ),
            EventEnvelope::new(
                "evt_sandbox_completed_001",
                chrono::Utc::now(),
                Event::SandboxCompleted(nanami_protocol::SandboxCompletedPayload {
                    sandbox_id: "sandbox_001".into(),
                    task_id: "task_openclaw_stream_001".into(),
                    status: nanami_protocol::SandboxStatus::Completed,
                    exit_code: Some(0),
                    summary: Some("sandbox finished".into()),
                }),
            ),
        ]),
    }))
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/tasks/openclaw/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Run task"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("sandbox.started"));
    assert!(text.contains("sandbox.output"));
    assert!(text.contains("sandbox.artifact"));
    assert!(text.contains("sandbox.completed"));
}

#[tokio::test]
async fn tasks_openclaw_stream_contains_workflow_events() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![
            EventEnvelope::new(
                "evt_workflow_started_001",
                chrono::Utc::now(),
                Event::WorkflowStarted(nanami_protocol::WorkflowStartedPayload {
                    workflow_id: "workflow_mock_001".into(),
                    task_id: "task_workflow_mock_001".into(),
                    project_path: "/mock/project".into(),
                    status: nanami_protocol::WorkflowStatus::Running,
                }),
            ),
            EventEnvelope::new(
                "evt_workflow_step_001",
                chrono::Utc::now(),
                Event::WorkflowStep(nanami_protocol::WorkflowStepPayload {
                    workflow_id: "workflow_mock_001".into(),
                    task_id: "task_workflow_mock_001".into(),
                    step_kind: nanami_protocol::WorkflowStepKind::AnalyzeProject,
                    status: nanami_protocol::WorkflowStepStatus::Completed,
                    summary: "Mock analysis finished".into(),
                }),
            ),
            EventEnvelope::new(
                "evt_workflow_test_result_001",
                chrono::Utc::now(),
                Event::WorkflowTestResult(nanami_protocol::WorkflowTestResultPayload {
                    workflow_id: "workflow_mock_001".into(),
                    task_id: "task_workflow_mock_001".into(),
                    status: nanami_protocol::WorkflowStatus::Completed,
                    summary: "2 tests passed, 1 failed".into(),
                    command_preview: "cargo test --lib".into(),
                    duration_ms: 1200,
                    passed: 2,
                    failed: 1,
                    failed_test_names: vec!["tests::mock_failure".into()],
                }),
            ),
            EventEnvelope::new(
                "evt_workflow_patch_proposed_001",
                chrono::Utc::now(),
                Event::WorkflowPatchProposed(nanami_protocol::WorkflowPatchProposedPayload {
                    workflow_id: "workflow_mock_001".into(),
                    task_id: "task_workflow_mock_001".into(),
                    patch_id: "patch_mock_001".into(),
                    summary: "Mock patch proposal ready".into(),
                    diff_summary: "1 file modified".into(),
                    risk_level: nanami_protocol::WorkflowPatchRiskLevel::Medium,
                    files: vec![nanami_protocol::WorkflowPatchFilePreviewPayload {
                        path: "src/main.rs".into(),
                        change_type: nanami_protocol::WorkflowChangeType::Modified,
                        diff_preview: "- old line\n+ new line".into(),
                    }],
                }),
            ),
            EventEnvelope::new(
                "evt_workflow_completed_001",
                chrono::Utc::now(),
                Event::WorkflowCompleted(nanami_protocol::WorkflowCompletedPayload {
                    workflow_id: "workflow_mock_001".into(),
                    task_id: "task_workflow_mock_001".into(),
                    status: nanami_protocol::WorkflowStatus::Completed,
                    summary: "Mock workflow completed".into(),
                }),
            ),
        ]),
    }))
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/tasks/openclaw/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Run task"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("workflow.started"));
    assert!(text.contains("workflow.step"));
    assert!(text.contains("workflow.test_result"));
    assert!(text.contains("workflow.patch_proposed"));
    assert!(text.contains("workflow.completed"));
}

#[tokio::test]
async fn tasks_openclaw_stream_inserts_permission_for_shell_tool() {
    let event = EventEnvelope::new(
        "evt_shell_started_001",
        chrono::Utc::now(),
        Event::ToolStarted(ToolStartedPayload {
            task_id: "task_openclaw_stream_001".into(),
            tool_call_id: "call_shell_001".into(),
            tool: "command.run".into(),
            summary: Some("cargo check".into()),
        }),
    );

    let permission = maybe_permission_for_tool_event(&event).unwrap();
    let json = serde_json::to_value(permission).unwrap();

    assert_eq!(json["type"], "permission.requested");
    assert_eq!(json["level"], "l4");
}

#[tokio::test]
async fn tasks_openclaw_stream_inserts_shell_permission_once_and_records_single_audit() {
    let app = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![EventEnvelope::new(
            "evt_shell_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_openclaw_stream_001".into(),
                tool_call_id: "call_shell_001".into(),
                tool: "command.run".into(),
                summary: Some("cargo check".into()),
            }),
        )]),
    }));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(text.matches("permission.requested").count(), 1);

    let audit_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let requested_count = json["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["action"] == "permission_requested")
        .count();

    assert_eq!(requested_count, 1);
}

#[tokio::test]
async fn tasks_openclaw_stream_inserts_permission_for_read_file_tool() {
    let event = EventEnvelope::new(
        "evt_read_started_001",
        chrono::Utc::now(),
        Event::ToolStarted(ToolStartedPayload {
            task_id: "task_openclaw_stream_001".into(),
            tool_call_id: "call_read_001".into(),
            tool: "read_file".into(),
            summary: Some("/workspace/project/src/main.rs".into()),
        }),
    );

    let permission = maybe_permission_for_tool_event(&event).unwrap();
    let json = serde_json::to_value(permission).unwrap();

    assert_eq!(json["type"], "permission.requested");
    assert_eq!(json["level"], "l2");
}

#[tokio::test]
async fn tasks_openclaw_stream_does_not_insert_permission_for_harmless_tool() {
    let event = EventEnvelope::new(
        "evt_harmless_started_001",
        chrono::Utc::now(),
        Event::ToolStarted(ToolStartedPayload {
            task_id: "task_openclaw_stream_001".into(),
            tool_call_id: "call_harmless_001".into(),
            tool: "display.message".into(),
            summary: Some("show info".into()),
        }),
    );

    assert!(maybe_permission_for_tool_event(&event).is_none());
}

#[tokio::test]
async fn tasks_openclaw_stream_does_not_insert_permission_for_harmless_tool_route() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![EventEnvelope::new(
            "evt_harmless_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_openclaw_stream_001".into(),
                tool_call_id: "call_harmless_001".into(),
                tool: "display.message".into(),
                summary: Some("show info".into()),
            }),
        )]),
    }))
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/tasks/openclaw/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Run task"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(!text.contains("permission.requested"));
}

#[tokio::test]
async fn tasks_openclaw_stream_unconfigured_gateway_returns_error_event() {
    let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Err(crate::chat_error(
            "OPENCLAW_GATEWAY_UNCONFIGURED",
            "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
            Some("Set NANAMI_OPENCLAW_GATEWAY_URL before starting OpenClaw task streams"),
        )),
    }))
    .oneshot(
        Request::builder()
            .method("POST")
            .uri("/tasks/openclaw/stream")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message":"Run task"}"#))
            .unwrap(),
    )
    .await
    .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("error.occurred"));
    assert!(text.contains("OPENCLAW_GATEWAY_UNCONFIGURED"));
}

#[tokio::test]
async fn permissions_mock_stream_returns_sse_content_type() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/permissions/mock/stream")
                .body(Body::empty())
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
async fn permissions_mock_stream_contains_requested_event() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .uri("/permissions/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert!(text.contains("permission.requested"));
    assert!(text.contains("perm_mock_read_project"));
}

#[tokio::test]
async fn permissions_mock_stream_creates_audit_record() {
    let app = crate::router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let audit_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["records"][0]["action"], "permission_requested");
}

#[tokio::test]
async fn permissions_resolve_accepts_allow_once() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"allow_once"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["type"], "permission.resolved");
    assert_eq!(json["decision"], "allow_once");
}

#[tokio::test]
async fn permissions_decision_returns_allow_once_after_resolve() {
    let app = crate::router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"allow_once"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let decision_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/decision/perm_mock_read_project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["decision"], "allow_once");
}

#[tokio::test]
async fn permissions_resolve_accepts_allow_for_task() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"allow_for_task"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["decision"], "allow_for_task");
}

#[tokio::test]
async fn permissions_decision_returns_allow_for_task_after_resolve() {
    let app = crate::router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"allow_for_task"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let decision_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/decision/perm_mock_read_project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["decision"], "allow_for_task");
}

#[tokio::test]
async fn permissions_resolve_accepts_deny() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"deny"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["decision"], "deny");
}

#[tokio::test]
async fn permissions_decision_returns_deny_after_resolve() {
    let app = crate::router();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"deny"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let decision_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/decision/perm_mock_read_project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["decision"], "deny");
}

#[tokio::test]
async fn dangerous_stream_permission_can_be_resolved_and_audit_includes_requested_and_resolved() {
    let app = crate::router_with_openclaw(Arc::new(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Ok(vec![EventEnvelope::new(
            "evt_shell_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_openclaw_stream_001".into(),
                tool_call_id: "call_shell_001".into(),
                tool: "command.run".into(),
                summary: Some("cargo check".into()),
            }),
        )]),
    }));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let _body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let resolve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_call_shell_001","decision":"allow_once"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve_response.status(), StatusCode::OK);

    let decision_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/decision/perm_call_shell_001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let decision_body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let decision_json: serde_json::Value = serde_json::from_slice(&decision_body).unwrap();

    assert_eq!(decision_json["decision"], "allow_once");

    let audit_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/permissions/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let audit_body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let audit_json: serde_json::Value = serde_json::from_slice(&audit_body).unwrap();
    let actions: Vec<_> = audit_json["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["action"].as_str().unwrap())
        .collect();

    assert!(actions.contains(&"permission_requested"));
    assert!(actions.contains(&"permission_resolved"));
}

#[tokio::test]
async fn permissions_resolve_rejects_invalid_decision() {
    let response = crate::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"permission_id":"perm_mock_read_project","decision":"invalid"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
