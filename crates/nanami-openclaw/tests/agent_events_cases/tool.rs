use nanami_openclaw::OpenClawStreamItem;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::support::collect_items;

#[tokio::test]
async fn stream_agent_events_maps_openai_tool_calls_delta() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"id\":\"call_001\",\"type\":\"function\",\"function\":{\"name\":\"mock.shell\",\"arguments\":\"{\\\"command\\\":\\\"cargo check\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n",
        ))
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run mock").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "task.started")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "tool.started")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "tool.output" && serde_json::to_value(event).unwrap()["stream"] == "log")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "task.completed")));
}

#[tokio::test]
async fn stream_agent_events_maps_simple_tool_events() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(
            "data: {\"tool_call_id\":\"tool_001\",\"tool\":\"mock.shell\",\"status\":\"running\",\"summary\":\"checking project\"}\n\ndata: {\"tool_call_id\":\"tool_001\",\"tool\":\"mock.shell\",\"stdout\":\"checking project...\"}\n\ndata: [DONE]\n\n",
        ))
        .mount(&server)
        .await;

    let items = collect_items(&server, "Run mock").await;

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "tool.started")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "tool.output" && serde_json::to_value(event).unwrap()["stream"] == "stdout")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "task.completed")));
}
