use axum::http::StatusCode;
use nanami_protocol::{
    ProjectKind, ProjectMetadata, ProjectStructureEntry, ProjectStructureEntryType,
    ProjectStructureMarker, ProjectStructureSummary, ProjectTrustStatus,
};

use crate::chat_error;
use crate::state::{AppState, JsonErrorResponse};

pub(crate) fn build_project_structure_summary(
    project: &ProjectMetadata,
) -> Result<ProjectStructureSummary, JsonErrorResponse> {
    let root = std::path::PathBuf::from(&project.project_path);
    let read_dir = match std::fs::read_dir(&root) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                [("content-type", "application/json")],
                serde_json::to_string(&chat_error(
                    "PROJECT_STRUCTURE_UNAVAILABLE",
                    "Unable to read the selected project directory",
                    Some("Select a valid project folder again"),
                ))
                .unwrap(),
            ));
        }
    };

    let mut entries = Vec::new();
    for entry in read_dir.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        let entry_type = if file_type.is_dir() {
            ProjectStructureEntryType::Directory
        } else {
            ProjectStructureEntryType::File
        };

        let marker = match file_name.as_str() {
            "Cargo.toml" | "package.json" | "pyproject.toml" => ProjectStructureMarker::Manifest,
            "src" | "app" | "crates" | "packages" => ProjectStructureMarker::SourceDir,
            ".gitignore" => ProjectStructureMarker::Config,
            "README.md" | "LICENSE" => ProjectStructureMarker::Other,
            _ => ProjectStructureMarker::Other,
        };

        entries.push(ProjectStructureEntry {
            name: file_name.clone(),
            relative_path: file_name,
            entry_type,
            marker,
        });
    }

    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(ProjectStructureSummary {
        project_id: project.project_id.clone(),
        project_path: project.project_path.clone(),
        entries,
    })
}

pub(crate) fn project_kind_label(kind: &ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Rust => "rust",
        ProjectKind::Node => "node",
        ProjectKind::Python => "python",
        ProjectKind::Unknown => "unknown",
    }
}

pub(crate) fn project_trust_status_label(status: &ProjectTrustStatus) -> &'static str {
    match status {
        ProjectTrustStatus::Untrusted => "untrusted",
        ProjectTrustStatus::TrustedMock => "trusted_mock",
        ProjectTrustStatus::SelectedUntrusted => "selected_untrusted",
        ProjectTrustStatus::SelectedTrusted => "selected_trusted",
    }
}

pub(crate) fn mock_project_metadata() -> ProjectMetadata {
    ProjectMetadata {
        project_id: "project_mock_001".into(),
        display_name: "Nanami Mock Workspace".into(),
        project_path: "/mock/project".into(),
        kind: ProjectKind::Rust,
        trust_status: ProjectTrustStatus::TrustedMock,
    }
}

pub(crate) fn detect_project_kind(project_path: &std::path::Path) -> ProjectKind {
    if project_path.join("Cargo.toml").is_file() {
        ProjectKind::Rust
    } else if project_path.join("package.json").is_file() {
        ProjectKind::Node
    } else if project_path.join("pyproject.toml").is_file() {
        ProjectKind::Python
    } else {
        ProjectKind::Unknown
    }
}

pub(crate) fn selected_trusted_project(
    state: &AppState,
    action: &'static str,
) -> Result<ProjectMetadata, JsonErrorResponse> {
    let selected_project = state.selected_project.lock().unwrap();
    let Some(project) = selected_project.as_ref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_SELECTED",
                "No project is currently selected",
                Some(match action {
                    "requesting manifest preview" => {
                        "Select and trust a project before requesting manifest preview"
                    }
                    "loading manifest preview" => {
                        "Select and trust a project before loading manifest preview"
                    }
                    "loading manifest summary" => {
                        "Select and trust a project before loading manifest summary"
                    }
                    _ => "Select and trust a project before loading its structure",
                }),
            ))
            .unwrap(),
        ));
    };

    if project.trust_status != ProjectTrustStatus::SelectedTrusted {
        return Err((
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_TRUSTED",
                "Current selected project must be selected_trusted",
                Some(match action {
                    "requesting manifest preview" => {
                        "Trust the selected project before requesting manifest preview"
                    }
                    "loading manifest preview" => {
                        "Trust the selected project before loading manifest preview"
                    }
                    "loading manifest summary" => {
                        "Trust the selected project before loading manifest summary"
                    }
                    _ => "Trust the selected project before loading its structure",
                }),
            ))
            .unwrap(),
        ));
    }

    Ok(project.clone())
}
