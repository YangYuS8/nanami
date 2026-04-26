use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "0.1";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenClawConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    PairingRequired,
    AuthFailed,
    ScopeMissing,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OpenClawStatusPayload {
    pub status: OpenClawConnectionStatus,
    pub gateway_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChatResponse {
    pub session_id: String,
    pub message_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatStreamEventKind {
    MessageDelta,
    MessageCompleted,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChatStreamEvent {
    pub kind: ChatStreamEventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ErrorPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub severity: ErrorSeverity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_hint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    WaitingPermission,
    Failed,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    Pending,
    Running,
    Failed,
    Completed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutputStream {
    Stdout,
    Stderr,
    Log,
    Artifact,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStatus {
    Starting,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxNetworkPolicy {
    Disabled,
    Limited,
    Enabled,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    ToolCall,
    WaitingPermission,
    Success,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaEmotion {
    Neutral,
    Happy,
    Focused,
    Worried,
    Surprised,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaStateSource {
    Mock,
    Ui,
    System,
    #[serde(rename = "openclaw")]
    OpenClaw,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PersonaStatePayload {
    pub state: PersonaState,
    pub emotion: PersonaEmotion,
    pub text: String,
    pub source: PersonaStateSource,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Rust,
    Node,
    Python,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTrustStatus {
    Untrusted,
    TrustedMock,
    SelectedUntrusted,
    SelectedTrusted,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub project_id: String,
    pub display_name: String,
    pub project_path: String,
    pub kind: ProjectKind,
    pub trust_status: ProjectTrustStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStructureEntryType {
    File,
    Directory,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStructureMarker {
    Manifest,
    SourceDir,
    Config,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProjectStructureEntry {
    pub name: String,
    pub relative_path: String,
    pub entry_type: ProjectStructureEntryType,
    pub marker: ProjectStructureMarker,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProjectStructureSummary {
    pub project_id: String,
    pub project_path: String,
    pub entries: Vec<ProjectStructureEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ManifestPreview {
    pub project_id: String,
    pub manifest_path: String,
    pub kind: ProjectKind,
    pub content_preview: String,
    pub truncated: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ManifestSummary {
    pub project_id: String,
    pub manifest_path: String,
    pub kind: ProjectKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_count: Option<u64>,
    pub summary_text: String,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum SandboxMountMode {
    #[serde(rename = "readonly")]
    ReadOnly,
    #[serde(rename = "readwrite")]
    ReadWrite,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SandboxMountPayload {
    pub host_path: String,
    pub sandbox_path: String,
    pub mode: SandboxMountMode,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SandboxStartedPayload {
    pub sandbox_id: String,
    pub task_id: String,
    pub template_id: String,
    pub status: SandboxStatus,
    pub network_policy: SandboxNetworkPolicy,
    pub mounts: Vec<SandboxMountPayload>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SandboxUpdatedPayload {
    pub sandbox_id: String,
    pub task_id: String,
    pub status: SandboxStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SandboxArtifactPayload {
    pub sandbox_id: String,
    pub task_id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SandboxOutputPayload {
    pub task_id: String,
    pub sandbox_id: String,
    pub stream: ToolOutputStream,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SandboxCompletedPayload {
    pub sandbox_id: String,
    pub task_id: String,
    pub status: SandboxStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskStartedPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub task_id: String,
    pub title: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskUpdatedPayload {
    pub task_id: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskCompletedPayload {
    pub task_id: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolStartedPayload {
    pub task_id: String,
    pub tool_call_id: String,
    pub tool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolOutputPayload {
    pub task_id: String,
    pub tool_call_id: String,
    pub stream: ToolOutputStream,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolCompletedPayload {
    pub task_id: String,
    pub tool_call_id: String,
    pub status: ToolCallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    AllowOnce,
    AllowForTask,
    Deny,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    Once,
    Task,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PermissionRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub permission_id: String,
    pub level: PermissionLevel,
    pub action: String,
    pub target: String,
    pub reason: String,
    pub scope: PermissionScope,
    pub expires: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PermissionResolvedPayload {
    pub permission_id: String,
    pub decision: PermissionDecision,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    PermissionRequested,
    PermissionResolved,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuditRecord {
    pub audit_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub permission_id: String,
    pub action: AuditAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<PermissionLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<PermissionDecision>,
    pub result: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PermissionDecisionStatus {
    pub permission_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<PermissionDecision>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PermissionAuditLogResponse {
    pub records: Vec<AuditRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "session.started")]
    SessionStarted { session_id: String, title: String },
    #[serde(rename = "session.updated")]
    SessionUpdated {
        session_id: String,
        status: SessionStatus,
    },
    #[serde(rename = "openclaw.status")]
    OpenClawStatus(OpenClawStatusPayload),
    #[serde(rename = "message.user")]
    MessageUser {
        session_id: String,
        message_id: String,
        content: String,
    },
    #[serde(rename = "message.delta")]
    MessageDelta {
        session_id: String,
        message_id: String,
        delta: String,
    },
    #[serde(rename = "message.completed")]
    MessageCompleted(ChatResponse),
    #[serde(rename = "task.started")]
    TaskStarted(TaskStartedPayload),
    #[serde(rename = "task.updated")]
    TaskUpdated(TaskUpdatedPayload),
    #[serde(rename = "task.completed")]
    TaskCompleted(TaskCompletedPayload),
    #[serde(rename = "tool.started")]
    ToolStarted(ToolStartedPayload),
    #[serde(rename = "tool.output")]
    ToolOutput(ToolOutputPayload),
    #[serde(rename = "tool.completed")]
    ToolCompleted(ToolCompletedPayload),
    #[serde(rename = "sandbox.started")]
    SandboxStarted(SandboxStartedPayload),
    #[serde(rename = "sandbox.updated")]
    SandboxUpdated(SandboxUpdatedPayload),
    #[serde(rename = "sandbox.output")]
    SandboxOutput(SandboxOutputPayload),
    #[serde(rename = "sandbox.artifact")]
    SandboxArtifact(SandboxArtifactPayload),
    #[serde(rename = "sandbox.completed")]
    SandboxCompleted(SandboxCompletedPayload),
    #[serde(rename = "persona.state")]
    PersonaState(PersonaStatePayload),
    #[serde(rename = "workflow.started")]
    WorkflowStarted(WorkflowStartedPayload),
    #[serde(rename = "workflow.step")]
    WorkflowStep(WorkflowStepPayload),
    #[serde(rename = "workflow.test_result")]
    WorkflowTestResult(WorkflowTestResultPayload),
    #[serde(rename = "workflow.patch_proposed")]
    WorkflowPatchProposed(WorkflowPatchProposedPayload),
    #[serde(rename = "workflow.completed")]
    WorkflowCompleted(WorkflowCompletedPayload),
    #[serde(rename = "permission.requested")]
    PermissionRequested(PermissionRequestPayload),
    #[serde(rename = "permission.resolved")]
    PermissionResolved(PermissionResolvedPayload),
    #[serde(rename = "error.occurred")]
    ErrorOccurred(ErrorPayload),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EventEnvelope {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub event: Event,
}

impl EventEnvelope {
    pub fn new(id: impl Into<String>, timestamp: DateTime<Utc>, event: Event) -> Self {
        Self {
            id: id.into(),
            timestamp,
            event,
        }
    }
}
