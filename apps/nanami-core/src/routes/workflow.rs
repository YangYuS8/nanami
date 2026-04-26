use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
};
use nanami_protocol::{
    PermissionLevel, PermissionRequestPayload, PermissionScope, ProjectTrustStatus,
};
use serde::Serialize;
use std::convert::Infallible;

use crate::chat_error;
use crate::mock::workflow::{mock_current_project_workflow_events, mock_workflow_events};
use crate::services::project::build_project_structure_summary;
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct WorkflowApplyPatchRequest {
    pub(crate) patch_id: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkflowApplyPatchResponse {
    patch_id: String,
    permission_id: String,
    status: &'static str,
    message: &'static str,
}

pub(crate) async fn workflow_mock_stream() -> Response {
    let events = mock_workflow_events();

    Sse::new(tokio_stream::iter(events.into_iter().map(|event| {
        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    })))
    .keep_alive(KeepAlive::default())
    .into_response()
}

pub(crate) async fn workflow_mock_current_project_stream(
    State(state): State<AppState>,
) -> Response {
    let selected_project = state.selected_project.lock().unwrap();
    let Some(project) = selected_project.as_ref() else {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_SELECTED",
                "No project is currently selected",
                Some("Select and trust a project before running a current-project workflow"),
            ))
            .unwrap(),
        )
            .into_response();
    };

    if project.trust_status != ProjectTrustStatus::SelectedTrusted {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_TRUSTED",
                "Current selected project must be selected_trusted",
                Some("Trust the selected project before running a current-project workflow"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    let structure = match build_project_structure_summary(project) {
        Ok(summary) => summary,
        Err(error) => return error.into_response(),
    };

    let events = mock_current_project_workflow_events(project, &structure);

    Sse::new(tokio_stream::iter(events.into_iter().map(|event| {
        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    })))
    .keep_alive(KeepAlive::default())
    .into_response()
}

pub(crate) async fn workflow_mock_apply_patch(
    State(state): State<AppState>,
    Json(request): Json<WorkflowApplyPatchRequest>,
) -> Json<WorkflowApplyPatchResponse> {
    let permission_id = format!("perm_workflow_patch_{}", request.patch_id);
    let permission_request = PermissionRequestPayload {
        task_id: Some("task_workflow_mock_001".into()),
        permission_id: permission_id.clone(),
        level: PermissionLevel::L3,
        action: "filesystem.write".into(),
        target: format!("mock patch proposal {}", request.patch_id),
        reason: "Mock apply patch request recorded for workflow visualization".into(),
        scope: PermissionScope::Task,
        expires: "task_completed".into(),
    };

    let mut manager = state.permission_manager.lock().unwrap();
    manager.request_permission(permission_request);

    Json(WorkflowApplyPatchResponse {
        patch_id: request.patch_id,
        permission_id,
        status: "waiting_permission",
        message: "Mock apply patch request recorded",
    })
}
