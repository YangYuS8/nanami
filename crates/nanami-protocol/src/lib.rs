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
