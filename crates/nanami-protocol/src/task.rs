use serde::{Deserialize, Serialize};

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
