use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{PermissionDecision, PermissionLevel};

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
pub struct PermissionAuditLogResponse {
    pub records: Vec<AuditRecord>,
}
