use nanami_openclaw::OpenClawStreamItem;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::support::collect_items;

#[tokio::test]
async fn stream_agent_events_reads_nanami_native_tool_started() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(
            "data: {\"type\":\"tool.started\",\"id\":\"evt_001\",\"timestamp\":\"2026-01-01T00:00:00Z\",\"task_id\":\"task_001\",\"tool_call_id\":\"tool_001\",\"tool\":\"mock.shell\",\"summary\":\"running mock shell\"}\n\n",
        ))
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run mock").await;

    assert!(matches!(items[0], OpenClawStreamItem::Event(_)));
}

#[tokio::test]
async fn stream_agent_events_reads_nanami_native_sandbox_started() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"type":"sandbox.started","id":"evt_sandbox_001","timestamp":"2026-01-01T00:00:00Z","sandbox_id":"sandbox_001","task_id":"task_001","template_id":"rust-workspace","status":"starting","network_policy":"disabled","mounts":[{"host_path":"/mock/host/project","sandbox_path":"/workspace/project","mode":"readonly"}]}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run sandbox").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.started")));
}

#[tokio::test]
async fn stream_agent_events_reads_nanami_native_workflow_started() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"type":"workflow.started","id":"evt_workflow_001","timestamp":"2026-01-01T00:00:00Z","workflow_id":"workflow_001","task_id":"task_001","project_path":"/mock/project","status":"running"}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run workflow").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.started")));
}
