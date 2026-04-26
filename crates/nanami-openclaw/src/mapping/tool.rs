use nanami_protocol::{
    Event, EventEnvelope, ToolCallStatus, ToolCompletedPayload, ToolOutputPayload,
    ToolOutputStream, ToolStartedPayload,
};
use serde_json::Value;

use crate::sse::OpenClawStreamItem;
use crate::state::{ToolEventMappingState, build_task_started_event};

pub(crate) fn map_openai_tool_call_delta(
    json: &Value,
    state: &mut ToolEventMappingState,
) -> Option<Vec<OpenClawStreamItem>> {
    let tool_calls = json.pointer("/choices/0/delta/tool_calls")?.as_array()?;
    let mut items = Vec::new();
    let task_id = state.ensure_task_id();

    if !state.task_started {
        state.task_started = true;
        items.push(OpenClawStreamItem::Event(build_task_started_event(
            state.next_event_id(),
            &task_id,
            "OpenClaw task",
        )));
    }

    for tool_call in tool_calls {
        let tool_call_id = tool_call.get("id")?.as_str()?.to_owned();
        let function = tool_call.get("function")?;
        let tool_name = function.get("name")?.as_str()?.to_owned();
        let arguments = function
            .get("arguments")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: task_id.clone(),
                tool_call_id: tool_call_id.clone(),
                tool: tool_name,
                summary: Some("OpenClaw tool call detected".into()),
            }),
        )));

        if !arguments.is_empty() {
            items.push(OpenClawStreamItem::Event(EventEnvelope::new(
                state.next_event_id(),
                chrono::Utc::now(),
                Event::ToolOutput(ToolOutputPayload {
                    task_id: task_id.clone(),
                    tool_call_id,
                    stream: ToolOutputStream::Log,
                    content: arguments,
                }),
            )));
        }
    }

    Some(items)
}

pub(crate) fn map_simple_tool_event(
    json: &Value,
    state: &mut ToolEventMappingState,
) -> Option<Vec<OpenClawStreamItem>> {
    let tool_call_id = json.get("tool_call_id")?.as_str()?.to_owned();
    let tool = json.get("tool")?.as_str()?.to_owned();
    let task_id = state.ensure_task_id();
    let mut items = Vec::new();

    if !state.task_started {
        state.task_started = true;
        items.push(OpenClawStreamItem::Event(build_task_started_event(
            state.next_event_id(),
            &task_id,
            "OpenClaw task",
        )));
    }

    if let Some(status) = json.get("status").and_then(Value::as_str) {
        match status {
            "running" => items.push(OpenClawStreamItem::Event(EventEnvelope::new(
                state.next_event_id(),
                chrono::Utc::now(),
                Event::ToolStarted(ToolStartedPayload {
                    task_id: task_id.clone(),
                    tool_call_id: tool_call_id.clone(),
                    tool: tool.clone(),
                    summary: json
                        .get("summary")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                }),
            ))),
            "completed" => items.push(OpenClawStreamItem::Event(EventEnvelope::new(
                state.next_event_id(),
                chrono::Utc::now(),
                Event::ToolCompleted(ToolCompletedPayload {
                    task_id: task_id.clone(),
                    tool_call_id: tool_call_id.clone(),
                    status: ToolCallStatus::Completed,
                    exit_code: json
                        .get("exit_code")
                        .and_then(Value::as_i64)
                        .map(|value| value as i32),
                }),
            ))),
            "failed" => items.push(OpenClawStreamItem::Event(EventEnvelope::new(
                state.next_event_id(),
                chrono::Utc::now(),
                Event::ToolCompleted(ToolCompletedPayload {
                    task_id: task_id.clone(),
                    tool_call_id: tool_call_id.clone(),
                    status: ToolCallStatus::Failed,
                    exit_code: json
                        .get("exit_code")
                        .and_then(Value::as_i64)
                        .map(|value| value as i32),
                }),
            ))),
            _ => {}
        }
    }

    if let Some(stdout) = json.get("stdout").and_then(Value::as_str) {
        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::ToolOutput(ToolOutputPayload {
                task_id: task_id.clone(),
                tool_call_id: tool_call_id.clone(),
                stream: ToolOutputStream::Stdout,
                content: stdout.to_owned(),
            }),
        )));
    }

    if let Some(stderr) = json.get("stderr").and_then(Value::as_str) {
        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::ToolOutput(ToolOutputPayload {
                task_id,
                tool_call_id,
                stream: ToolOutputStream::Stderr,
                content: stderr.to_owned(),
            }),
        )));
    }

    Some(items)
}
