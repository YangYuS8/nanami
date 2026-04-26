use serde::{Deserialize, Serialize};

use crate::ProjectKind;

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
