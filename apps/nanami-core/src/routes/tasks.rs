use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
};
use futures_util::StreamExt as FuturesStreamExt;
use nanami_permission::PermissionManager;
use nanami_protocol::{
    ChatRequest, Event, EventEnvelope, TaskCompletedPayload, TaskStartedPayload, TaskStatus,
    ToolCallStatus, ToolCompletedPayload, ToolOutputPayload, ToolOutputStream, ToolStartedPayload,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::once;

use crate::chat_error;
use crate::state::{AppState, NanamiEventStream};

pub(crate) async fn tasks_mock_stream() -> Response {
    let events = vec![
        EventEnvelope::new(
            "evt_task_mock_started_001",
            chrono::Utc::now(),
            Event::TaskStarted(TaskStartedPayload {
                session_id: Some("sess_mock_001".into()),
                task_id: "task_mock_001".into(),
                title: "Mock project check".into(),
                status: TaskStatus::Running,
            }),
        ),
        EventEnvelope::new(
            "evt_tool_mock_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_mock_001".into(),
                tool_call_id: "tool_mock_001".into(),
                tool: "mock.shell".into(),
                summary: Some("Mock shell run".into()),
            }),
        ),
        EventEnvelope::new(
            "evt_tool_mock_stdout_001",
            chrono::Utc::now(),
            Event::ToolOutput(ToolOutputPayload {
                task_id: "task_mock_001".into(),
                tool_call_id: "tool_mock_001".into(),
                stream: ToolOutputStream::Stdout,
                content: "checking project...".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_tool_mock_stderr_001",
            chrono::Utc::now(),
            Event::ToolOutput(ToolOutputPayload {
                task_id: "task_mock_001".into(),
                tool_call_id: "tool_mock_001".into(),
                stream: ToolOutputStream::Stderr,
                content: "warning: mock warning".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_tool_mock_completed_001",
            chrono::Utc::now(),
            Event::ToolCompleted(ToolCompletedPayload {
                task_id: "task_mock_001".into(),
                tool_call_id: "tool_mock_001".into(),
                status: ToolCallStatus::Completed,
                exit_code: Some(0),
            }),
        ),
        EventEnvelope::new(
            "evt_task_mock_completed_001",
            chrono::Utc::now(),
            Event::TaskCompleted(TaskCompletedPayload {
                task_id: "task_mock_001".into(),
                status: TaskStatus::Completed,
                summary: Some("Mock task completed".into()),
            }),
        ),
    ];

    Sse::new(tokio_stream::iter(events.into_iter().map(|event| {
        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    })))
    .keep_alive(KeepAlive::default())
    .into_response()
}

pub(crate) async fn tasks_openclaw_stream(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Response {
    if request.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "CHAT_EMPTY_MESSAGE",
                "Chat message must not be empty",
                Some("Enter a message before sending"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    let events = match state.openclaw.stream_agent_events(request).await {
        Ok(events) => events,
        Err(error) => Box::pin(once(Ok(EventEnvelope::new(
            "evt_openclaw_error_001",
            chrono::Utc::now(),
            Event::ErrorOccurred(error),
        )))) as NanamiEventStream,
    };

    let permission_manager = Arc::clone(&state.permission_manager);
    Sse::new(FuturesStreamExt::flat_map(events, move |event| {
        let permission_manager = Arc::clone(&permission_manager);
        let event = match event {
            Ok(event) => event,
            Err(error) => EventEnvelope::new(
                "evt_openclaw_error_001",
                chrono::Utc::now(),
                Event::ErrorOccurred(error),
            ),
        };

        let mut response_events = vec![event.clone()];
        if let Some(permission_event) = maybe_permission_for_tool_event(&event) {
            let mut manager = permission_manager.lock().unwrap();
            if let Event::PermissionRequested(payload) = &permission_event.event {
                manager.request_permission(payload.clone());
            }
            response_events.push(permission_event);
        }

        tokio_stream::iter(response_events.into_iter().map(|event| {
            Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
        }))
    }))
    .keep_alive(KeepAlive::default())
    .into_response()
}

pub(crate) fn maybe_permission_for_tool_event(event: &EventEnvelope) -> Option<EventEnvelope> {
    let tool_started = match &event.event {
        Event::ToolStarted(payload) => payload,
        _ => return None,
    };

    let manager = PermissionManager::new();
    let permission = manager.classify_tool_request(nanami_permission::DangerousToolRequest {
        task_id: Some(tool_started.task_id.clone()),
        tool_call_id: tool_started.tool_call_id.clone(),
        tool: tool_started.tool.clone(),
        arguments: Some(
            [
                tool_started.tool.clone(),
                tool_started.summary.clone().unwrap_or_default(),
            ]
            .join(" ")
            .trim()
            .to_owned(),
        ),
        summary: tool_started.summary.clone(),
    })?;

    Some(EventEnvelope::new(
        format!("evt_perm_{}", tool_started.tool_call_id),
        chrono::Utc::now(),
        Event::PermissionRequested(permission),
    ))
}
