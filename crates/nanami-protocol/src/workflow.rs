use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    Running,
    WaitingPermission,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStepKind {
    OpenProject,
    AnalyzeProject,
    RunTests,
    PatchProposed,
    ApplyPatch,
    Verify,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStepStatus {
    Pending,
    Running,
    Completed,
    WaitingPermission,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPatchRiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowStartedPayload {
    pub workflow_id: String,
    pub task_id: String,
    pub project_path: String,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowStepPayload {
    pub workflow_id: String,
    pub task_id: String,
    pub step_kind: WorkflowStepKind,
    pub status: WorkflowStepStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowTestResultPayload {
    pub workflow_id: String,
    pub task_id: String,
    pub status: WorkflowStatus,
    pub summary: String,
    pub command_preview: String,
    pub duration_ms: u64,
    pub passed: u32,
    pub failed: u32,
    pub failed_test_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowPatchFilePreviewPayload {
    pub path: String,
    pub change_type: WorkflowChangeType,
    pub diff_preview: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowPatchProposedPayload {
    pub workflow_id: String,
    pub task_id: String,
    pub patch_id: String,
    pub summary: String,
    pub diff_summary: String,
    pub risk_level: WorkflowPatchRiskLevel,
    pub files: Vec<WorkflowPatchFilePreviewPayload>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowCompletedPayload {
    pub workflow_id: String,
    pub task_id: String,
    pub status: WorkflowStatus,
    pub summary: String,
}
