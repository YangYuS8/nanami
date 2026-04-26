use serde::{Deserialize, Serialize};

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
