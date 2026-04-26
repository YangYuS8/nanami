use serde::{Deserialize, Serialize};

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
