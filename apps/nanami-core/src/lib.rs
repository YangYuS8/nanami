mod error;
mod mock;
mod openclaw;
mod routes;
mod services;
mod state;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use error::chat_error;
use openclaw::{EnvOpenClawService, OpenClawService, openclaw_status_from_config};
use routes::{
    chat::{chat, chat_stream},
    permissions::{
        permission_audit, permission_decision, permissions_mock_stream, permissions_resolve,
    },
    persona::persona_mock_stream,
    projects::{
        projects_current_manifest_preview, projects_current_manifest_preview_request,
        projects_current_manifest_summary, projects_current_structure, projects_mock_current,
        projects_select, projects_trust,
    },
    sandbox::sandbox_mock_stream,
    tasks::tasks_mock_stream,
    tasks::tasks_openclaw_stream,
    workflow::{
        workflow_mock_apply_patch, workflow_mock_current_project_stream, workflow_mock_stream,
    },
};
use state::AppState;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    protocol_version: &'static str,
}

pub fn router() -> Router {
    router_with_openclaw(Arc::new(EnvOpenClawService))
}

fn router_with_openclaw(openclaw: Arc<dyn OpenClawService>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openclaw/status", get(openclaw_status))
        .route("/chat", post(chat))
        .route("/chat/stream", post(chat_stream))
        .route("/tasks/mock/stream", get(tasks_mock_stream))
        .route("/tasks/openclaw/stream", post(tasks_openclaw_stream))
        .route("/sandbox/mock/stream", get(sandbox_mock_stream))
        .route("/persona/mock/stream", get(persona_mock_stream))
        .route("/workflow/mock/stream", get(workflow_mock_stream))
        .route(
            "/workflow/mock/current-project/stream",
            get(workflow_mock_current_project_stream),
        )
        .route(
            "/workflow/mock/apply-patch",
            post(workflow_mock_apply_patch),
        )
        .route("/projects/select", post(projects_select))
        .route("/projects/trust", post(projects_trust))
        .route("/projects/mock/current", get(projects_mock_current))
        .route(
            "/projects/current/structure",
            get(projects_current_structure),
        )
        .route(
            "/projects/current/manifest/preview-request",
            post(projects_current_manifest_preview_request),
        )
        .route(
            "/projects/current/manifest/preview",
            get(projects_current_manifest_preview),
        )
        .route(
            "/projects/current/manifest/summary",
            get(projects_current_manifest_summary),
        )
        .route("/permissions/mock/stream", get(permissions_mock_stream))
        .route("/permissions/resolve", post(permissions_resolve))
        .route(
            "/permissions/decision/:permission_id",
            get(permission_decision),
        )
        .route("/permissions/audit", get(permission_audit))
        .with_state(AppState {
            openclaw,
            permission_manager: Arc::new(Mutex::new(nanami_permission::PermissionManager::new())),
            selected_project: Arc::new(Mutex::new(None)),
        })
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        protocol_version: nanami_protocol::PROTOCOL_VERSION,
    })
}

async fn openclaw_status() -> Json<nanami_protocol::OpenClawStatusPayload> {
    Json(openclaw_status_from_config(None).await)
}

#[cfg(test)]
mod tests;
