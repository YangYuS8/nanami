use chrono::DateTime;
use nanami_protocol::{
    Event, EventEnvelope, SandboxArtifactPayload, SandboxCompletedPayload, SandboxMountMode,
    SandboxMountPayload, SandboxNetworkPolicy, SandboxStartedPayload, SandboxStatus,
    SandboxUpdatedPayload, ToolOutputPayload, ToolOutputStream,
};

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc()
}

fn mount() -> SandboxMountPayload {
    SandboxMountPayload {
        host_path: "/mock/host/project".into(),
        sandbox_path: "/workspace/project".into(),
        mode: SandboxMountMode::ReadOnly,
    }
}

#[test]
fn sandbox_started_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_sandbox_started_001",
        timestamp(),
        Event::SandboxStarted(SandboxStartedPayload {
            sandbox_id: "sandbox_mock_001".into(),
            task_id: "task_mock_001".into(),
            template_id: "rust-workspace".into(),
            status: SandboxStatus::Starting,
            network_policy: SandboxNetworkPolicy::Disabled,
            mounts: vec![mount()],
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "sandbox.started");
    assert_eq!(json["sandbox_id"], "sandbox_mock_001");
    assert_eq!(json["task_id"], "task_mock_001");
    assert_eq!(json["template_id"], "rust-workspace");
    assert_eq!(json["status"], "starting");
    assert_eq!(json["network_policy"], "disabled");
    assert_eq!(json["mounts"][0]["host_path"], "/mock/host/project");
    assert_eq!(json["mounts"][0]["sandbox_path"], "/workspace/project");
    assert_eq!(json["mounts"][0]["mode"], "read_only");
}

#[test]
fn sandbox_updated_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_sandbox_updated_001",
        timestamp(),
        Event::SandboxUpdated(SandboxUpdatedPayload {
            sandbox_id: "sandbox_mock_001".into(),
            task_id: "task_mock_001".into(),
            status: SandboxStatus::Running,
            summary: Some("Mock sandbox is running".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "sandbox.updated");
    assert_eq!(json["sandbox_id"], "sandbox_mock_001");
    assert_eq!(json["status"], "running");
    assert_eq!(json["summary"], "Mock sandbox is running");
}

#[test]
fn sandbox_output_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_sandbox_output_001",
        timestamp(),
        Event::SandboxOutput(ToolOutputPayload {
            task_id: "task_mock_001".into(),
            tool_call_id: "sandbox_mock_001".into(),
            stream: ToolOutputStream::Stdout,
            content: "Checking sandbox...".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "sandbox.output");
    assert_eq!(json["task_id"], "task_mock_001");
    assert_eq!(json["tool_call_id"], "sandbox_mock_001");
    assert_eq!(json["stream"], "stdout");
    assert_eq!(json["content"], "Checking sandbox...");
}

#[test]
fn sandbox_artifact_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_sandbox_artifact_001",
        timestamp(),
        Event::SandboxArtifact(SandboxArtifactPayload {
            sandbox_id: "sandbox_mock_001".into(),
            task_id: "task_mock_001".into(),
            name: "mock-report.txt".into(),
            path: "/workspace/output/mock-report.txt".into(),
            media_type: "text/plain".into(),
            size_bytes: 128,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "sandbox.artifact");
    assert_eq!(json["sandbox_id"], "sandbox_mock_001");
    assert_eq!(json["name"], "mock-report.txt");
    assert_eq!(json["path"], "/workspace/output/mock-report.txt");
    assert_eq!(json["media_type"], "text/plain");
    assert_eq!(json["size_bytes"], 128);
}

#[test]
fn sandbox_completed_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_sandbox_completed_001",
        timestamp(),
        Event::SandboxCompleted(SandboxCompletedPayload {
            sandbox_id: "sandbox_mock_001".into(),
            task_id: "task_mock_001".into(),
            status: SandboxStatus::Completed,
            exit_code: Some(0),
            summary: Some("Mock sandbox completed".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "sandbox.completed");
    assert_eq!(json["sandbox_id"], "sandbox_mock_001");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["summary"], "Mock sandbox completed");
}
