use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use nanami_protocol::{
    ManifestSummary, PermissionLevel, PermissionRequestPayload, PermissionScope, ProjectMetadata,
    ProjectTrustStatus,
};

use crate::chat_error;
use crate::services::manifest::{
    build_manifest_preview, build_manifest_summary, ensure_manifest_preview_permission,
    manifest_preview_permission_id, top_level_manifest_path,
};
use crate::services::project::{
    build_project_structure_summary, detect_project_kind, mock_project_metadata,
    selected_trusted_project,
};
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ProjectSelectRequest {
    pub(crate) project_path: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ProjectTrustRequest {
    pub(crate) project_id: String,
}

pub(crate) async fn projects_mock_current() -> Json<ProjectMetadata> {
    Json(mock_project_metadata())
}

pub(crate) async fn projects_select(
    State(state): State<AppState>,
    Json(request): Json<ProjectSelectRequest>,
) -> impl IntoResponse {
    let project_path = std::path::PathBuf::from(&request.project_path);

    if !project_path.exists() || !project_path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_PATH_INVALID",
                "Selected project path must be an existing directory",
                Some("Choose an existing project folder"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    let kind = detect_project_kind(&project_path);

    let display_name = project_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("selected-project")
        .to_owned();

    let metadata = ProjectMetadata {
        project_id: format!("project_selected_{}", display_name),
        display_name,
        project_path: request.project_path,
        kind,
        trust_status: ProjectTrustStatus::SelectedUntrusted,
    };

    *state.selected_project.lock().unwrap() = Some(metadata.clone());

    Json(metadata).into_response()
}

pub(crate) async fn projects_trust(
    State(state): State<AppState>,
    Json(request): Json<ProjectTrustRequest>,
) -> impl IntoResponse {
    let mut selected_project = state.selected_project.lock().unwrap();
    let Some(project) = selected_project.as_mut() else {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_SELECTED",
                "No project is currently selected",
                Some("Select a project folder before trusting it"),
            ))
            .unwrap(),
        )
            .into_response();
    };

    if project.project_id != request.project_id {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_ID_MISMATCH",
                "The requested project does not match the current selected project",
                Some("Trust the currently selected project only"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    if project.trust_status != ProjectTrustStatus::SelectedUntrusted {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_TRUST_INVALID_STATE",
                "Only selected_untrusted projects can be trusted",
                Some("Select a project and trust it once"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    project.trust_status = ProjectTrustStatus::SelectedTrusted;

    Json(project.clone()).into_response()
}

pub(crate) async fn projects_current_structure(State(state): State<AppState>) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "loading its structure") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    match build_project_structure_summary(&project) {
        Ok(summary) => Json(summary).into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn projects_current_manifest_preview_request(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "requesting manifest preview") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    let manifest_path = match top_level_manifest_path(&project) {
        Ok(path) => path,
        Err(error) => return error.into_response(),
    };

    let permission_id = manifest_preview_permission_id(&project);
    let permission_request = PermissionRequestPayload {
        task_id: None,
        permission_id,
        level: PermissionLevel::L2,
        action: "filesystem.read".into(),
        target: manifest_path.display().to_string(),
        reason: "Read top-level manifest preview for the currently selected trusted project".into(),
        scope: PermissionScope::Task,
        expires: "task_completed".into(),
    };

    let mut manager = state.permission_manager.lock().unwrap();
    Json(manager.request_permission(permission_request)).into_response()
}

pub(crate) async fn projects_current_manifest_preview(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "loading manifest preview") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    if let Err(error) = ensure_manifest_preview_permission(&state, &project) {
        return error.into_response();
    }

    match build_manifest_preview(&project) {
        Ok(preview) => Json(preview).into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn projects_current_manifest_summary(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "loading manifest summary") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    if let Err(error) = ensure_manifest_preview_permission(&state, &project) {
        return error.into_response();
    }

    match build_manifest_summary(&project) {
        Ok(summary) => Json::<ManifestSummary>(summary).into_response(),
        Err(error) => error.into_response(),
    }
}
