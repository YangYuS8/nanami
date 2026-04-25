use chrono::DateTime;
use nanami_protocol::{
    Event, EventEnvelope, TaskCompletedPayload, TaskStartedPayload, TaskStatus, TaskUpdatedPayload,
    ToolCallStatus, ToolCompletedPayload, ToolOutputPayload, ToolOutputStream, ToolStartedPayload,
};

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc()
}

#[test]
fn task_started_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_task_started_001",
        timestamp(),
        Event::TaskStarted(TaskStartedPayload {
            session_id: Some("sess_001".into()),
            task_id: "task_001".into(),
            title: "Mock project check".into(),
            status: TaskStatus::Running,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "task.started");
    assert_eq!(json["task_id"], "task_001");
    assert_eq!(json["status"], "running");
}

#[test]
fn task_updated_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_task_updated_001",
        timestamp(),
        Event::TaskUpdated(TaskUpdatedPayload {
            task_id: "task_001".into(),
            status: TaskStatus::WaitingPermission,
            summary: Some("Need approval".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "task.updated");
    assert_eq!(json["status"], "waiting_permission");
    assert_eq!(json["summary"], "Need approval");
}

#[test]
fn task_completed_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_task_completed_001",
        timestamp(),
        Event::TaskCompleted(TaskCompletedPayload {
            task_id: "task_001".into(),
            status: TaskStatus::Completed,
            summary: Some("Mock task completed".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "task.completed");
    assert_eq!(json["status"], "completed");
}

#[test]
fn tool_started_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_tool_started_001",
        timestamp(),
        Event::ToolStarted(ToolStartedPayload {
            task_id: "task_001".into(),
            tool_call_id: "tool_001".into(),
            tool: "mock.shell".into(),
            summary: Some("Mock shell run".into()),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "tool.started");
    assert_eq!(json["tool"], "mock.shell");
}

#[test]
fn tool_output_stdout_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_tool_output_stdout_001",
        timestamp(),
        Event::ToolOutput(ToolOutputPayload {
            task_id: "task_001".into(),
            tool_call_id: "tool_001".into(),
            stream: ToolOutputStream::Stdout,
            content: "checking project...".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "tool.output");
    assert_eq!(json["stream"], "stdout");
}

#[test]
fn tool_output_stderr_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_tool_output_stderr_001",
        timestamp(),
        Event::ToolOutput(ToolOutputPayload {
            task_id: "task_001".into(),
            tool_call_id: "tool_001".into(),
            stream: ToolOutputStream::Stderr,
            content: "warning: mock warning".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "tool.output");
    assert_eq!(json["stream"], "stderr");
}

#[test]
fn tool_completed_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_tool_completed_001",
        timestamp(),
        Event::ToolCompleted(ToolCompletedPayload {
            task_id: "task_001".into(),
            tool_call_id: "tool_001".into(),
            status: ToolCallStatus::Completed,
            exit_code: Some(0),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "tool.completed");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["exit_code"], 0);
}
