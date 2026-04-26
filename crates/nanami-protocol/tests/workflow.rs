use chrono::DateTime;
use nanami_protocol::{
    Event, EventEnvelope, WorkflowChangeType, WorkflowCompletedPayload,
    WorkflowPatchFilePreviewPayload, WorkflowPatchProposedPayload, WorkflowStartedPayload,
    WorkflowPatchRiskLevel, WorkflowStatus, WorkflowStepKind, WorkflowStepPayload,
    WorkflowStepStatus, WorkflowTestResultPayload,
};

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc()
}

#[test]
fn workflow_step_kind_verify_serializes_snake_case() {
    let json = serde_json::to_value(WorkflowStepKind::Verify).unwrap();

    assert_eq!(json, "verify");
}

#[test]
fn workflow_started_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_workflow_started_001",
        timestamp(),
        Event::WorkflowStarted(WorkflowStartedPayload {
            workflow_id: "workflow_mock_001".into(),
            task_id: "task_workflow_mock_001".into(),
            project_path: "/mock/project".into(),
            status: WorkflowStatus::Running,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "workflow.started");
    assert_eq!(json["workflow_id"], "workflow_mock_001");
    assert_eq!(json["project_path"], "/mock/project");
    assert_eq!(json["status"], "running");
}

#[test]
fn workflow_step_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_workflow_step_001",
        timestamp(),
        Event::WorkflowStep(WorkflowStepPayload {
            workflow_id: "workflow_mock_001".into(),
            task_id: "task_workflow_mock_001".into(),
            step_kind: WorkflowStepKind::AnalyzeProject,
            status: WorkflowStepStatus::Completed,
            summary: "Mock analysis completed".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "workflow.step");
    assert_eq!(json["step_kind"], "analyze_project");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["summary"], "Mock analysis completed");
}

#[test]
fn workflow_test_result_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_workflow_test_result_001",
        timestamp(),
        Event::WorkflowTestResult(WorkflowTestResultPayload {
            workflow_id: "workflow_mock_001".into(),
            task_id: "task_workflow_mock_001".into(),
            status: WorkflowStatus::Completed,
            summary: "2 tests passed".into(),
            command_preview: "cargo test --lib".into(),
            duration_ms: 1200,
            passed: 2,
            failed: 0,
            failed_test_names: Vec::new(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "workflow.test_result");
    assert_eq!(json["command_preview"], "cargo test --lib");
    assert_eq!(json["duration_ms"], 1200);
    assert_eq!(json["passed"], 2);
    assert_eq!(json["failed"], 0);
}

#[test]
fn workflow_patch_proposed_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_workflow_patch_001",
        timestamp(),
        Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
            workflow_id: "workflow_mock_001".into(),
            task_id: "task_workflow_mock_001".into(),
            patch_id: "patch_mock_001".into(),
            summary: "Mock patch proposal".into(),
            diff_summary: "1 file modified".into(),
            risk_level: WorkflowPatchRiskLevel::Medium,
            files: vec![WorkflowPatchFilePreviewPayload {
                path: "src/main.rs".into(),
                change_type: WorkflowChangeType::Modified,
                diff_preview: "- old line\n+ new line".into(),
            }],
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "workflow.patch_proposed");
    assert_eq!(json["patch_id"], "patch_mock_001");
    assert_eq!(json["diff_summary"], "1 file modified");
    assert_eq!(json["risk_level"], "medium");
    assert_eq!(json["files"][0]["path"], "src/main.rs");
    assert_eq!(json["files"][0]["change_type"], "modified");
}

#[test]
fn workflow_completed_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_workflow_completed_001",
        timestamp(),
        Event::WorkflowCompleted(WorkflowCompletedPayload {
            workflow_id: "workflow_mock_001".into(),
            task_id: "task_workflow_mock_001".into(),
            status: WorkflowStatus::Completed,
            summary: "Mock workflow completed".into(),
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "workflow.completed");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["summary"], "Mock workflow completed");
}
