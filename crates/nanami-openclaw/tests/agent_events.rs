use nanami_openclaw::{OpenClawChatRequest, OpenClawClient, OpenClawConfig, OpenClawStreamItem};
use tokio_stream::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> OpenClawClient {
    OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: None,
        timeout_ms: 1000,
        chat_path: "/chat".into(),
    })
}

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run mock".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(matches!(items[0], OpenClawStreamItem::Event(_)));
}

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run mock".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run mock".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "tool.started")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "tool.output" && serde_json::to_value(event).unwrap()["stream"] == "stdout")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "task.completed")));
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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run sandbox".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.started")));
}

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run sandbox".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run sandbox".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run sandbox".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.completed" && serde_json::to_value(event).unwrap()["status"] == "completed")));
    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "sandbox.completed" && serde_json::to_value(event).unwrap()["status"] == "failed")));
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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run workflow".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.started")));
}

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run workflow".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run workflow".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run workflow".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

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

    let items: Vec<_> = client_for(&server)
        .stream_agent_events(OpenClawChatRequest {
            message: "Run workflow".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(items.iter().any(|item| matches!(item, OpenClawStreamItem::Event(event) if serde_json::to_value(event).unwrap()["type"] == "workflow.completed" && serde_json::to_value(event).unwrap()["status"] == "completed")));
}
