use nanami_openclaw::OpenClawStreamItem;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::support::collect_items;

#[tokio::test]
async fn stream_agent_events_maps_simple_sandbox_output_events() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"sandbox_id":"sandbox_001","template_id":"rust-workspace","network_policy":"disabled","mounts":[{"host_path":"/mock/host/project","sandbox_path":"/workspace/project","mode":"readonly"}]}

data: {"sandbox_id":"sandbox_001","stdout":"checking workspace..."}

data: {"sandbox_id":"sandbox_001","stderr":"warning: mock stderr"}

data: [DONE]

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run sandbox").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.output" && serde_json::to_value(event).unwrap()["stream"] == "stdout")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.output" && serde_json::to_value(event).unwrap()["stream"] == "stderr")));
}

#[tokio::test]
async fn stream_agent_events_maps_simple_sandbox_artifact_event() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"sandbox_id":"sandbox_001","artifact":{"name":"mock-report.txt","path":"/workspace/output/mock-report.txt","media_type":"text/plain","size_bytes":128}}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run sandbox").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.artifact")));
}

#[tokio::test]
async fn stream_agent_events_maps_simple_sandbox_completed_and_failed_events() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    r#"data: {"sandbox_id":"sandbox_001","status":"completed","exit_code":0,"summary":"sandbox finished"}

data: {"sandbox_id":"sandbox_002","status":"failed","exit_code":1,"summary":"sandbox failed"}

"#,
                ),
        )
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run sandbox").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.completed" && serde_json::to_value(event).unwrap()["status"] == "completed")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.completed" && serde_json::to_value(event).unwrap()["status"] == "failed")));
}
