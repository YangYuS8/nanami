use axum::body::Body;
use axum::http::{Request, StatusCode};
use nanami_protocol::{
    Event, EventEnvelope, TaskCompletedPayload, TaskStartedPayload, TaskStatus, ToolOutputPayload,
    ToolOutputStream, ToolStartedPayload,
};
use tower::ServiceExt;

use super::support::{StubOpenClawService, body_json, body_text, router, router_with_openclaw};
use crate::routes::tasks::maybe_permission_for_tool_event;

#[tokio::test]
async fn tasks_mock_stream_returns_sse_content_type() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/tasks/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_text(response).await;

    assert!(text.contains("task.started"));
    assert!(text.contains("tool.started"));
    assert!(text.contains("tool.output"));
    assert!(text.contains("tool.completed"));
    assert!(text.contains("task.completed"));
}

#[tokio::test]
async fn tasks_openclaw_stream_returns_sse_content_type() {
    let response = router_with_openclaw(StubOpenClawService {
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
    })
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
    let response = router_with_openclaw(StubOpenClawService {
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
    })
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
    let text = body_text(response).await;

    assert!(text.contains("task.started"));
    assert!(text.contains("tool.started"));
    assert!(text.contains("tool.output"));
    assert!(text.contains("task.completed"));
}

#[tokio::test]
async fn tasks_openclaw_stream_contains_sandbox_events() {
    let response = router_with_openclaw(StubOpenClawService {
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
    })
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
    let text = body_text(response).await;

    assert!(text.contains("sandbox.started"));
    assert!(text.contains("sandbox.output"));
    assert!(text.contains("sandbox.artifact"));
    assert!(text.contains("sandbox.completed"));
}

#[tokio::test]
async fn tasks_openclaw_stream_contains_workflow_events() {
    let response = router_with_openclaw(StubOpenClawService {
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
    })
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
    let text = body_text(response).await;

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
    let text = body_text(response).await;

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
    let json = body_json(audit_response).await;
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
    let response = router_with_openclaw(StubOpenClawService {
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
    })
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
    let text = body_text(response).await;

    assert!(!text.contains("permission.requested"));
}

#[tokio::test]
async fn tasks_openclaw_stream_unconfigured_gateway_returns_error_event() {
    let response = router_with_openclaw(StubOpenClawService {
        response: Err(crate::chat_error("unused", "unused", None)),
        stream_response: Ok(Vec::new()),
        agent_stream_response: Err(crate::chat_error(
            "OPENCLAW_GATEWAY_UNCONFIGURED",
            "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
            Some("Set NANAMI_OPENCLAW_GATEWAY_URL before starting OpenClaw task streams"),
        )),
    })
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
    let text = body_text(response).await;

    assert!(text.contains("error.occurred"));
    assert!(text.contains("OPENCLAW_GATEWAY_UNCONFIGURED"));
}
