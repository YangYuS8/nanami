use nanami_protocol::{
    Event, EventEnvelope, ProjectMetadata, ProjectStructureSummary, WorkflowChangeType,
    WorkflowCompletedPayload, WorkflowPatchFilePreviewPayload, WorkflowPatchProposedPayload,
    WorkflowPatchRiskLevel, WorkflowStartedPayload, WorkflowStatus, WorkflowStepKind,
    WorkflowStepPayload, WorkflowStepStatus, WorkflowTestResultPayload,
};

use crate::services::project::{project_kind_label, project_trust_status_label};

pub(crate) fn mock_workflow_events() -> Vec<EventEnvelope> {
    vec![
        EventEnvelope::new(
            "evt_workflow_started_001",
            chrono::Utc::now(),
            Event::WorkflowStarted(WorkflowStartedPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                project_path: "/mock/project".into(),
                status: WorkflowStatus::Running,
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_open_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::OpenProject,
                status: WorkflowStepStatus::Completed,
                summary: "Mock project context opened".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_analyze_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::AnalyzeProject,
                status: WorkflowStepStatus::Completed,
                summary: "Mock analysis finished".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_run_tests_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::RunTests,
                status: WorkflowStepStatus::Completed,
                summary: "Mock tests executed".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_test_result_001",
            chrono::Utc::now(),
            Event::WorkflowTestResult(WorkflowTestResultPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                status: WorkflowStatus::Completed,
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
            Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                patch_id: "patch_mock_001".into(),
                summary: "Mock patch proposal ready".into(),
                diff_summary: "1 file modified".into(),
                risk_level: WorkflowPatchRiskLevel::Medium,
                files: vec![WorkflowPatchFilePreviewPayload {
                    path: "src/main.rs".into(),
                    change_type: WorkflowChangeType::Modified,
                    diff_preview: "- old line\n+ new line".into(),
                }],
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_apply_patch_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::ApplyPatch,
                status: WorkflowStepStatus::WaitingPermission,
                summary: "Waiting for patch approval".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_completed_001",
            chrono::Utc::now(),
            Event::WorkflowCompleted(WorkflowCompletedPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                status: WorkflowStatus::Completed,
                summary: "Mock workflow completed".into(),
            }),
        ),
    ]
}

pub(crate) fn mock_current_project_workflow_events(
    project: &ProjectMetadata,
    structure: &ProjectStructureSummary,
) -> Vec<EventEnvelope> {
    vec![
        EventEnvelope::new(
            "evt_workflow_current_project_started_001",
            chrono::Utc::now(),
            Event::WorkflowStarted(WorkflowStartedPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                project_path: project.project_path.clone(),
                status: WorkflowStatus::Running,
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_open_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::OpenProject,
                status: WorkflowStepStatus::Completed,
                summary: format!(
                    "Selected project {} [{}] ({}, {})",
                    project.display_name,
                    project.project_id,
                    project_kind_label(&project.kind),
                    project_trust_status_label(&project.trust_status)
                ),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_analyze_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::AnalyzeProject,
                status: WorkflowStepStatus::Completed,
                summary: format!(
                    "Shallow structure summary includes {} top-level entries",
                    structure.entries.len()
                ),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_run_tests_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::RunTests,
                status: WorkflowStepStatus::Completed,
                summary: "Mock tests executed in current project context".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_test_result_001",
            chrono::Utc::now(),
            Event::WorkflowTestResult(WorkflowTestResultPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                status: WorkflowStatus::Completed,
                summary: "2 tests passed, 1 failed".into(),
                command_preview: "cargo test --lib".into(),
                duration_ms: 1200,
                passed: 2,
                failed: 1,
                failed_test_names: vec!["tests::mock_failure".into()],
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_patch_proposed_001",
            chrono::Utc::now(),
            Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                patch_id: "patch_current_project_001".into(),
                summary: "Mock patch proposal ready".into(),
                diff_summary: "1 file modified".into(),
                risk_level: WorkflowPatchRiskLevel::Medium,
                files: vec![WorkflowPatchFilePreviewPayload {
                    path: "src/main.rs".into(),
                    change_type: WorkflowChangeType::Modified,
                    diff_preview: "- old line\n+ new line".into(),
                }],
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_apply_patch_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::ApplyPatch,
                status: WorkflowStepStatus::WaitingPermission,
                summary: "Waiting for patch approval".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_completed_001",
            chrono::Utc::now(),
            Event::WorkflowCompleted(WorkflowCompletedPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                status: WorkflowStatus::Completed,
                summary: "Mock current-project workflow completed".into(),
            }),
        ),
    ]
}
