use serde::{Deserialize, Serialize};

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
pub struct PermissionDecisionStatus {
    pub permission_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<PermissionDecision>,
}
