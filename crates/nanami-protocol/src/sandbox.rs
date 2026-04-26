use serde::{Deserialize, Serialize};

use crate::ToolOutputStream;

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
