use serde::{Deserialize, Serialize};

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
