use nanami_protocol::{ChatStreamEvent, ChatStreamEventKind};
use serde_json::Value;
use std::pin::Pin;
use tokio_stream::{Stream, iter};

use crate::error::OpenClawError;

pub type OpenClawChatStream =
    Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, OpenClawError>> + Send>>;
pub type OpenClawAgentStream =
    Pin<Box<dyn Stream<Item = Result<crate::OpenClawStreamItem, OpenClawError>> + Send>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenClawStreamItem {
    Chat(ChatStreamEvent),
    Event(nanami_protocol::EventEnvelope),
}

pub(crate) fn parse_sse_events(
    text: &str,
) -> Result<Vec<Result<ChatStreamEvent, OpenClawError>>, OpenClawError> {
    let mut events = Vec::new();
    let mut content = String::new();
    let mut completed = false;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with("data:") {
            continue;
        }

        let data = line.trim_start_matches("data:").trim();
        if data == "[DONE]" {
            completed = true;
            events.push(Ok(ChatStreamEvent {
                kind: ChatStreamEventKind::MessageCompleted,
                session_id: None,
                message_id: None,
                delta: None,
                content: Some(content.clone()),
                error: None,
            }));
            continue;
        }

        let json: Value = serde_json::from_str(data).map_err(|_| OpenClawError::InvalidResponse)?;
        if let Some(delta) = extract_delta(&json) {
            content.push_str(&delta);
            events.push(Ok(ChatStreamEvent {
                kind: ChatStreamEventKind::MessageDelta,
                session_id: json
                    .get("session_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                message_id: json
                    .get("message_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                delta: Some(delta),
                content: None,
                error: None,
            }));
        } else if let Some(final_content) = extract_content(&json) {
            completed = true;
            events.push(Ok(ChatStreamEvent {
                kind: ChatStreamEventKind::MessageCompleted,
                session_id: json
                    .get("session_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                message_id: json
                    .get("message_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                delta: None,
                content: Some(final_content),
                error: None,
            }));
        }
    }

    if events.is_empty() {
        return Err(OpenClawError::InvalidResponse);
    }

    if !completed {
        events.push(Ok(ChatStreamEvent {
            kind: ChatStreamEventKind::MessageCompleted,
            session_id: None,
            message_id: None,
            delta: None,
            content: Some(content),
            error: None,
        }));
    }

    Ok(events)
}

pub(crate) fn parse_sse_frame(
    frame: &str,
    accumulated: &mut String,
    completed: &mut bool,
) -> Result<Option<ChatStreamEvent>, OpenClawError> {
    let data_lines = frame
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("data:"))
        .map(|line| line.trim_start_matches("data:").trim())
        .collect::<Vec<_>>();
    if data_lines.is_empty() {
        return Ok(None);
    }

    let data = data_lines.join("\n");
    if data == "[DONE]" {
        *completed = true;
        return Ok(Some(ChatStreamEvent {
            kind: ChatStreamEventKind::MessageCompleted,
            session_id: None,
            message_id: None,
            delta: None,
            content: Some(accumulated.clone()),
            error: None,
        }));
    }

    let json: Value = serde_json::from_str(&data).map_err(|_| OpenClawError::InvalidResponse)?;
    if let Some(delta) = extract_delta(&json) {
        accumulated.push_str(&delta);
        return Ok(Some(ChatStreamEvent {
            kind: ChatStreamEventKind::MessageDelta,
            session_id: json
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message_id: json
                .get("message_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            delta: Some(delta),
            content: None,
            error: None,
        }));
    }

    if let Some(content) = extract_content(&json) {
        *completed = true;
        return Ok(Some(ChatStreamEvent {
            kind: ChatStreamEventKind::MessageCompleted,
            session_id: json
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message_id: json
                .get("message_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            delta: None,
            content: Some(content),
            error: None,
        }));
    }

    Err(OpenClawError::InvalidResponse)
}

pub(crate) fn normalized_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

pub(crate) fn extract_content(json: &Value) -> Option<String> {
    json.get("content")
        .and_then(Value::as_str)
        .or_else(|| {
            json.pointer("/choices/0/message/content")
                .and_then(Value::as_str)
        })
        .or_else(|| {
            json.pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
        })
        .map(str::to_owned)
}

pub(crate) fn extract_delta(json: &Value) -> Option<String> {
    json.get("delta")
        .and_then(Value::as_str)
        .or_else(|| {
            json.pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
        })
        .map(str::to_owned)
}

pub(crate) fn json_completion_stream(json: Value) -> Result<OpenClawChatStream, OpenClawError> {
    let content = extract_content(&json).ok_or(OpenClawError::InvalidResponse)?;
    Ok(Box::pin(iter(vec![Ok(ChatStreamEvent {
        kind: ChatStreamEventKind::MessageCompleted,
        session_id: json
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        message_id: json
            .get("message_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        delta: None,
        content: Some(content),
        error: None,
    })])))
}
