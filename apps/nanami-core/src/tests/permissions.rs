use axum::body::Body;
use axum::http::{Request, StatusCode};
use nanami_protocol::{Event, EventEnvelope, ToolStartedPayload};
use tower::ServiceExt;

use super::support::{StubOpenClawService, body_json, body_text, router, router_with_openclaw};

#[tokio::test]
async fn permissions_mock_stream_returns_sse_content_type() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/permissions/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_text(response).await;

    assert!(text.contains("permission.requested"));
    assert!(text.contains("perm_mock_read_project"));
}

#[tokio::test]
async fn permissions_mock_stream_creates_audit_record() {
    let app = router();

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
    let json = body_json(audit_response).await;

    assert_eq!(json["records"][0]["action"], "permission_requested");
}

#[tokio::test]
async fn permissions_resolve_accepts_allow_once() {
    let response = router()
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
    let json = body_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["type"], "permission.resolved");
    assert_eq!(json["decision"], "allow_once");
}

#[tokio::test]
async fn permissions_decision_returns_allow_once_after_resolve() {
    let app = router();

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
    let json = body_json(decision_response).await;

    assert_eq!(json["decision"], "allow_once");
}

#[tokio::test]
async fn permissions_resolve_accepts_allow_for_task() {
    let response = router()
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
    let json = body_json(response).await;

    assert_eq!(json["decision"], "allow_for_task");
}

#[tokio::test]
async fn permissions_decision_returns_allow_for_task_after_resolve() {
    let app = router();

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
    let json = body_json(decision_response).await;

    assert_eq!(json["decision"], "allow_for_task");
}

#[tokio::test]
async fn permissions_resolve_accepts_deny() {
    let response = router()
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
    let json = body_json(response).await;

    assert_eq!(json["decision"], "deny");
}

#[tokio::test]
async fn permissions_decision_returns_deny_after_resolve() {
    let app = router();

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
    let json = body_json(decision_response).await;

    assert_eq!(json["decision"], "deny");
}

#[tokio::test]
async fn dangerous_stream_permission_can_be_resolved_and_audit_includes_requested_and_resolved() {
    let app = router_with_openclaw(StubOpenClawService {
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
    });

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
    let decision_json = body_json(decision_response).await;

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
    let audit_json = body_json(audit_response).await;
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
    let response = router()
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
