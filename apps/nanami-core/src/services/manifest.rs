use axum::http::StatusCode;
use nanami_protocol::{
    ManifestPreview, ManifestSummary, PermissionDecision, ProjectKind, ProjectMetadata,
};

use crate::chat_error;
use crate::state::{AppState, JsonErrorResponse, MANIFEST_PREVIEW_MAX_BYTES};

#[derive(Debug, Clone)]
pub(crate) struct ManifestFile {
    pub(crate) manifest_path: std::path::PathBuf,
    pub(crate) content: String,
    pub(crate) truncated: bool,
    pub(crate) size_bytes: u64,
}

pub(crate) fn top_level_manifest_path(
    project: &ProjectMetadata,
) -> Result<std::path::PathBuf, JsonErrorResponse> {
    let root = std::path::PathBuf::from(&project.project_path);
    let manifest_name = match project.kind {
        ProjectKind::Rust => "Cargo.toml",
        ProjectKind::Node => "package.json",
        ProjectKind::Python => "pyproject.toml",
        ProjectKind::Unknown => {
            return Err((
                StatusCode::BAD_REQUEST,
                [("content-type", "application/json")],
                serde_json::to_string(&chat_error(
                    "PROJECT_MANIFEST_UNAVAILABLE",
                    "No supported top-level manifest is available for the current project",
                    Some("Select a project with Cargo.toml, package.json, or pyproject.toml"),
                ))
                .unwrap(),
            ));
        }
    };

    let manifest_path = root.join(manifest_name);
    if !manifest_path.is_file() {
        return Err((
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_MANIFEST_UNAVAILABLE",
                "The supported top-level manifest file is not available",
                Some("Re-select the project folder to refresh top-level manifest detection"),
            ))
            .unwrap(),
        ));
    }

    Ok(manifest_path)
}

pub(crate) fn build_manifest_preview(
    project: &ProjectMetadata,
) -> Result<ManifestPreview, JsonErrorResponse> {
    let manifest_file = read_manifest_file(project)?;

    Ok(ManifestPreview {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        content_preview: manifest_file.content.clone(),
        truncated: manifest_file.truncated,
        size_bytes: manifest_file.size_bytes,
    })
}

pub(crate) fn ensure_manifest_preview_permission(
    state: &AppState,
    project: &ProjectMetadata,
) -> Result<(), JsonErrorResponse> {
    let permission_id = manifest_preview_permission_id(project);
    let decision = {
        let manager = state.permission_manager.lock().unwrap();
        manager.decision_for(&permission_id)
    };

    if matches!(
        decision,
        Some(PermissionDecision::AllowOnce | PermissionDecision::AllowForTask)
    ) {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        [("content-type", "application/json")],
        serde_json::to_string(&chat_error(
            "MANIFEST_PREVIEW_PERMISSION_REQUIRED",
            "Manifest preview requires an approved filesystem.read permission",
            Some(
                "Request manifest preview permission and approve allow_once or allow_for_task first",
            ),
        ))
        .unwrap(),
    ))
}

pub(crate) fn read_manifest_file(
    project: &ProjectMetadata,
) -> Result<ManifestFile, JsonErrorResponse> {
    let manifest_path = top_level_manifest_path(project)?;
    let bytes = match std::fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                [("content-type", "application/json")],
                serde_json::to_string(&chat_error(
                    "MANIFEST_PREVIEW_UNAVAILABLE",
                    "Unable to read the selected top-level manifest file",
                    Some("Re-select the project folder and request manifest preview again"),
                ))
                .unwrap(),
            ));
        }
    };

    let size_bytes = bytes.len() as u64;
    let preview_bytes = if size_bytes > MANIFEST_PREVIEW_MAX_BYTES {
        &bytes[..MANIFEST_PREVIEW_MAX_BYTES as usize]
    } else {
        &bytes[..]
    };

    Ok(ManifestFile {
        manifest_path,
        content: String::from_utf8_lossy(preview_bytes).into_owned(),
        truncated: size_bytes > MANIFEST_PREVIEW_MAX_BYTES,
        size_bytes,
    })
}

pub(crate) fn build_manifest_summary(
    project: &ProjectMetadata,
) -> Result<ManifestSummary, JsonErrorResponse> {
    let manifest_file = read_manifest_file(project)?;
    Ok(match project.kind {
        ProjectKind::Rust => build_rust_manifest_summary(project, &manifest_file),
        ProjectKind::Node => build_node_manifest_summary(project, &manifest_file),
        ProjectKind::Python => build_python_manifest_summary(project, &manifest_file),
        ProjectKind::Unknown => build_unknown_manifest_summary(project, &manifest_file),
    })
}

fn build_rust_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    let parsed: Result<toml::Value, _> = toml::from_str(&manifest_file.content);
    let (package_name, package_version, dependency_count) = if let Ok(value) = parsed {
        let package = value.get("package").and_then(toml::Value::as_table);
        let package_name = package
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let package_version = package
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let dependency_count = value
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .map(|deps| deps.len() as u64);
        (package_name, package_version, dependency_count)
    } else {
        (None, None, None)
    };

    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: package_name.clone(),
        package_version: package_version.clone(),
        dependency_count,
        script_count: None,
        summary_text: summary_text_for_manifest(
            "Rust",
            package_name.as_deref(),
            package_version.as_deref(),
            dependency_count,
            None,
        ),
    }
}

fn build_node_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&manifest_file.content);
    let (package_name, package_version, dependency_count, script_count) = if let Ok(value) = parsed
    {
        let package_name = value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let package_version = value
            .get("version")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let dependencies = value
            .get("dependencies")
            .and_then(serde_json::Value::as_object)
            .map(|deps| deps.len() as u64)
            .unwrap_or(0);
        let dev_dependencies = value
            .get("devDependencies")
            .and_then(serde_json::Value::as_object)
            .map(|deps| deps.len() as u64)
            .unwrap_or(0);
        let scripts = value
            .get("scripts")
            .and_then(serde_json::Value::as_object)
            .map(|scripts| scripts.len() as u64);
        (
            package_name,
            package_version,
            Some(dependencies + dev_dependencies),
            scripts,
        )
    } else {
        (None, None, None, None)
    };

    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: package_name.clone(),
        package_version: package_version.clone(),
        dependency_count,
        script_count,
        summary_text: summary_text_for_manifest(
            "Node",
            package_name.as_deref(),
            package_version.as_deref(),
            dependency_count,
            script_count,
        ),
    }
}

fn build_python_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    let parsed: Result<toml::Value, _> = toml::from_str(&manifest_file.content);
    let (package_name, package_version, dependency_count) = if let Ok(value) = parsed {
        let project_table = value.get("project").and_then(toml::Value::as_table);
        let package_name = project_table
            .and_then(|project| project.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let package_version = project_table
            .and_then(|project| project.get("version"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let dependency_count = project_table
            .and_then(|project| project.get("dependencies"))
            .and_then(toml::Value::as_array)
            .map(|deps| deps.len() as u64);
        (package_name, package_version, dependency_count)
    } else {
        (None, None, None)
    };

    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: package_name.clone(),
        package_version: package_version.clone(),
        dependency_count,
        script_count: None,
        summary_text: summary_text_for_manifest(
            "Python",
            package_name.as_deref(),
            package_version.as_deref(),
            dependency_count,
            None,
        ),
    }
}

fn build_unknown_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: None,
        package_version: None,
        dependency_count: None,
        script_count: None,
        summary_text: "Manifest summary unavailable".into(),
    }
}

fn summary_text_for_manifest(
    ecosystem: &str,
    package_name: Option<&str>,
    package_version: Option<&str>,
    dependency_count: Option<u64>,
    script_count: Option<u64>,
) -> String {
    if package_name.is_none()
        && package_version.is_none()
        && dependency_count.is_none()
        && script_count.is_none()
    {
        return "Manifest summary unavailable".into();
    }

    let mut summary = format!("{} manifest", ecosystem);
    if let Some(name) = package_name {
        summary.push_str(&format!(" {}", name));
    }
    if let Some(version) = package_version {
        summary.push_str(&format!(" {}", version));
    }
    if let Some(count) = dependency_count {
        summary.push_str(&format!(" with {} dependencies", count));
    }
    if let Some(count) = script_count {
        summary.push_str(&format!(" and {} scripts", count));
    }
    summary
}

pub(crate) fn manifest_preview_permission_id(project: &ProjectMetadata) -> String {
    format!("perm_manifest_preview_{}", project.project_id)
}
