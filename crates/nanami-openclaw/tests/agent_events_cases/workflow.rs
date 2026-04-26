use nanami_openclaw::OpenClawStreamItem;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::support::collect_items;

#[tokio::test]
async fn stream_agent_events_maps_simple_workflow_step_event() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"workflow_id":"workflow_001","task_id":"task_001","step_kind":"analyze_project","status":"completed","summary":"Mock analysis finished"}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run workflow").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.step" && serde_json::to_value(event).unwrap()["step_kind"] == "analyze_project")));
}

#[tokio::test]
async fn stream_agent_events_maps_simple_workflow_test_result_event() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"workflow_id":"workflow_001","task_id":"task_001","status":"completed","summary":"2 tests passed, 1 failed","command_preview":"cargo test --lib","duration_ms":1200,"passed":2,"failed":1,"failed_test_names":["tests::mock_failure"]}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run workflow").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.test_result" && serde_json::to_value(event).unwrap()["failed"] == 1)));
}

#[tokio::test]
async fn stream_agent_events_maps_simple_workflow_patch_proposed_event() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"workflow_id":"workflow_001","task_id":"task_001","patch_id":"patch_001","summary":"Mock patch proposal ready","diff_summary":"1 file modified","risk_level":"medium","files":[{"path":"src/main.rs","change_type":"modified","diff_preview":"- old line\n+ new line"}]}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run workflow").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.patch_proposed" && serde_json::to_value(event).unwrap()["risk_level"] == "medium")));
}

#[tokio::test]
async fn stream_agent_events_maps_simple_workflow_completed_event() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"workflow_id":"workflow_001","task_id":"task_001","status":"completed","summary":"Mock workflow completed"}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run workflow").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.completed" && serde_json::to_value(event).unwrap()["status"] == "completed")));
}
