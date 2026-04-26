use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
};
use nanami_openclaw::OpenClawChatStream;
use nanami_protocol::{
    ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamEventKind, ErrorPayload,
};
use serde::Serialize;
use std::convert::Infallible;
use tokio_stream::once;

use crate::chat_error;
use crate::openclaw::map_openclaw_chat_error;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ChatEndpointResponse {
    Ok(ChatResponse),
    Error(ErrorPayload),
}

pub(crate) async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    if request.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ChatEndpointResponse::Error(ErrorPayload {
                task_id: None,
                severity: nanami_protocol::ErrorSeverity::Error,
                code: "CHAT_EMPTY_MESSAGE".into(),
                message: "Chat message must not be empty".into(),
                action_hint: Some("Enter a message before sending".into()),
            })),
        );
    }

    match state.openclaw.send_chat_message(request).await {
        Ok(response) => (StatusCode::OK, Json(ChatEndpointResponse::Ok(response))),
        Err(error) => (
            StatusCode::BAD_GATEWAY,
            Json(ChatEndpointResponse::Error(error)),
        ),
    }
}

pub(crate) async fn chat_stream(
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

    let stream = match state.openclaw.stream_chat_message(request).await {
        Ok(stream) => stream,
        Err(error) => Box::pin(once(Ok(ChatStreamEvent {
            kind: ChatStreamEventKind::Error,
            session_id: None,
            message_id: None,
            delta: None,
            content: None,
            error: Some(error),
        }))) as OpenClawChatStream,
    };

    Sse::new(tokio_stream::StreamExt::map(stream, |result| {
        let event = match result {
            Ok(event) => event,
            Err(error) => ChatStreamEvent {
                kind: ChatStreamEventKind::Error,
                session_id: None,
                message_id: None,
                delta: None,
                content: None,
                error: Some(map_openclaw_chat_error(error)),
            },
        };

        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    }))
    .keep_alive(KeepAlive::default())
    .into_response()
}
