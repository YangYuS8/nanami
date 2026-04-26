use nanami_protocol::{
    Event, EventEnvelope, WorkflowChangeType, WorkflowCompletedPayload,
    WorkflowPatchFilePreviewPayload, WorkflowPatchProposedPayload, WorkflowPatchRiskLevel,
    WorkflowStartedPayload, WorkflowStatus, WorkflowStepKind, WorkflowStepPayload,
    WorkflowStepStatus, WorkflowTestResultPayload,
};
use serde_json::Value;

use crate::sse::OpenClawStreamItem;

pub(crate) fn map_simple_workflow_event(json: &Value) -> Option<Vec<OpenClawStreamItem>> {
    json.get("workflow_id")?;

    let workflow_id = json.get("workflow_id")?.as_str()?.to_owned();
    let task_id = json.get("task_id")?.as_str()?.to_owned();

    if let Some(project_path) = json.get("project_path").and_then(Value::as_str) {
        let status = workflow_status(json.get("status")?.as_str()?)?;
        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_started_001",
            chrono::Utc::now(),
            Event::WorkflowStarted(WorkflowStartedPayload {
                workflow_id,
                task_id,
                project_path: project_path.to_owned(),
                status,
            }),
        ))]);
    }

    if let Some(step_kind) = json.get("step_kind").and_then(Value::as_str) {
        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_step_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id,
                task_id,
                step_kind: workflow_step_kind(step_kind)?,
                status: workflow_step_status(json.get("status")?.as_str()?)?,
                summary: json.get("summary")?.as_str()?.to_owned(),
            }),
        ))]);
    }

    if json.get("command_preview").is_some() {
        let failed_test_names = json
            .get("failed_test_names")?
            .as_array()?
            .iter()
            .map(|value| value.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()?;

        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_test_result_001",
            chrono::Utc::now(),
            Event::WorkflowTestResult(WorkflowTestResultPayload {
                workflow_id,
                task_id,
                status: workflow_status(json.get("status")?.as_str()?)?,
                summary: json.get("summary")?.as_str()?.to_owned(),
                command_preview: json.get("command_preview")?.as_str()?.to_owned(),
                duration_ms: json.get("duration_ms")?.as_u64()?,
                passed: json.get("passed")?.as_u64()? as u32,
                failed: json.get("failed")?.as_u64()? as u32,
                failed_test_names,
            }),
        ))]);
    }

    if json.get("patch_id").is_some() {
        let files = json
            .get("files")?
            .as_array()?
            .iter()
            .map(workflow_patch_file_preview)
            .collect::<Option<Vec<_>>>()?;

        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_patch_001",
            chrono::Utc::now(),
            Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
                workflow_id,
                task_id,
                patch_id: json.get("patch_id")?.as_str()?.to_owned(),
                summary: json.get("summary")?.as_str()?.to_owned(),
                diff_summary: json.get("diff_summary")?.as_str()?.to_owned(),
                risk_level: workflow_patch_risk_level(json.get("risk_level")?.as_str()?)?,
                files,
            }),
        ))]);
    }

    if let Some(summary) = json.get("summary").and_then(Value::as_str)
        && let Some(status) = json.get("status").and_then(Value::as_str)
        && matches!(status, "completed" | "failed")
    {
        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_completed_001",
            chrono::Utc::now(),
            Event::WorkflowCompleted(WorkflowCompletedPayload {
                workflow_id,
                task_id,
                status: workflow_status(status)?,
                summary: summary.to_owned(),
            }),
        ))]);
    }

    None
}

fn workflow_status(value: &str) -> Option<WorkflowStatus> {
    match value {
        "running" => Some(WorkflowStatus::Running),
        "waiting_permission" => Some(WorkflowStatus::WaitingPermission),
        "completed" => Some(WorkflowStatus::Completed),
        "failed" => Some(WorkflowStatus::Failed),
        _ => None,
    }
}

fn workflow_step_kind(value: &str) -> Option<WorkflowStepKind> {
    match value {
        "open_project" => Some(WorkflowStepKind::OpenProject),
        "analyze_project" => Some(WorkflowStepKind::AnalyzeProject),
        "run_tests" => Some(WorkflowStepKind::RunTests),
        "patch_proposed" => Some(WorkflowStepKind::PatchProposed),
        "apply_patch" => Some(WorkflowStepKind::ApplyPatch),
        "verify" => Some(WorkflowStepKind::Verify),
        _ => None,
    }
}

fn workflow_step_status(value: &str) -> Option<WorkflowStepStatus> {
    match value {
        "pending" => Some(WorkflowStepStatus::Pending),
        "running" => Some(WorkflowStepStatus::Running),
        "completed" => Some(WorkflowStepStatus::Completed),
        "waiting_permission" => Some(WorkflowStepStatus::WaitingPermission),
        "failed" => Some(WorkflowStepStatus::Failed),
        _ => None,
    }
}

fn workflow_patch_risk_level(value: &str) -> Option<WorkflowPatchRiskLevel> {
    match value {
        "low" => Some(WorkflowPatchRiskLevel::Low),
        "medium" => Some(WorkflowPatchRiskLevel::Medium),
        "high" => Some(WorkflowPatchRiskLevel::High),
        _ => None,
    }
}

fn workflow_patch_file_preview(value: &Value) -> Option<WorkflowPatchFilePreviewPayload> {
    let file = value.as_object()?;
    Some(WorkflowPatchFilePreviewPayload {
        path: file.get("path")?.as_str()?.to_owned(),
        change_type: match file.get("change_type")?.as_str()? {
            "added" => WorkflowChangeType::Added,
            "modified" => WorkflowChangeType::Modified,
            "deleted" => WorkflowChangeType::Deleted,
            "renamed" => WorkflowChangeType::Renamed,
            _ => return None,
        },
        diff_preview: file.get("diff_preview")?.as_str()?.to_owned(),
    })
}
