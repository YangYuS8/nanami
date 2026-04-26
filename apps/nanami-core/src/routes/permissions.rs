use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
};
use nanami_protocol::{
    Event, EventEnvelope, PermissionAuditLogResponse, PermissionDecision, PermissionDecisionStatus,
    PermissionLevel, PermissionRequestPayload, PermissionResolvedPayload, PermissionScope,
};
use std::convert::Infallible;

use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PermissionResolveRequest {
    pub(crate) permission_id: String,
    pub(crate) decision: PermissionDecision,
}

pub(crate) async fn permissions_mock_stream(State(state): State<AppState>) -> Response {
    let event = EventEnvelope::new(
        "evt_permission_mock_requested_001",
        chrono::Utc::now(),
        Event::PermissionRequested(PermissionRequestPayload {
            task_id: Some("task_mock_001".into()),
            permission_id: "perm_mock_read_project".into(),
            level: PermissionLevel::L2,
            action: "filesystem.read".into(),
            target: "/home/user/Code/nanami".into(),
            reason: "Need to read project files for analysis".into(),
            scope: PermissionScope::Task,
            expires: "task_completed".into(),
        }),
    );

    {
        let mut manager = state.permission_manager.lock().unwrap();
        if let Event::PermissionRequested(payload) = &event.event {
            manager.request_permission(payload.clone());
        }
    }

    Sse::new(tokio_stream::iter(vec![Ok::<_, Infallible>(
        SseEvent::default().data(serde_json::to_string(&event).unwrap()),
    )]))
    .keep_alive(KeepAlive::default())
    .into_response()
}

pub(crate) async fn permissions_resolve(
    State(state): State<AppState>,
    Json(request): Json<PermissionResolveRequest>,
) -> impl IntoResponse {
    let mut manager = state.permission_manager.lock().unwrap();
    let resolved = manager.resolve_permission(&request.permission_id, request.decision);
    let event = EventEnvelope::new(
        "evt_permission_mock_resolved_001",
        chrono::Utc::now(),
        Event::PermissionResolved(PermissionResolvedPayload {
            permission_id: resolved.permission_id,
            decision: resolved.decision,
        }),
    );

    (StatusCode::OK, Json(event))
}

pub(crate) async fn permission_decision(
    State(state): State<AppState>,
    Path(permission_id): Path<String>,
) -> Json<PermissionDecisionStatus> {
    let manager = state.permission_manager.lock().unwrap();

    Json(PermissionDecisionStatus {
        permission_id: permission_id.clone(),
        decision: manager.decision_for(&permission_id),
    })
}

pub(crate) async fn permission_audit(
    State(state): State<AppState>,
) -> Json<PermissionAuditLogResponse> {
    let manager = state.permission_manager.lock().unwrap();

    Json(PermissionAuditLogResponse {
        records: manager.audit_records(),
    })
}
