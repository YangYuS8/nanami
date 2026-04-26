use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{body_json, body_text, router, select_and_trust_project, temp_project_dir};

#[tokio::test]
async fn workflow_mock_stream_returns_sse_content_type() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/workflow/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_text(response).await;

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
async fn workflow_mock_apply_patch_records_permission_and_returns_waiting_status() {
    let app = router();

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
    let json = body_json(response).await;

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
    let audit_json = body_json(audit_response).await;

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
    let response = router()
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
    let temp_dir = temp_project_dir("nanami_workflow_current_project");
    std::fs::create_dir_all(temp_dir.join("src")).unwrap();
    std::fs::create_dir_all(temp_dir.join("crates")).unwrap();
    std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();
    std::fs::write(temp_dir.join("README.md"), "").unwrap();

    let app = router();
    let project_id = select_and_trust_project(&app, &temp_dir).await;

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
    let text = body_text(workflow_response).await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert_eq!(status, StatusCode::OK);
    assert!(text.contains("workflow.started"));
    assert!(text.contains(&project_id));
    assert!(text.contains(&temp_dir.display().to_string()));
    assert!(text.contains("selected_trusted"));
    assert!(text.contains("rust"));
    assert!(text.contains("4 top-level entries"));
}
