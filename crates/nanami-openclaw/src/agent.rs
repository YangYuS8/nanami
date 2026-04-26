use nanami_protocol::EventEnvelope;
use serde_json::Value;

use crate::error::OpenClawError;
use crate::mapping::sandbox::map_simple_sandbox_event;
use crate::mapping::tool::{map_openai_tool_call_delta, map_simple_tool_event};
use crate::mapping::workflow::map_simple_workflow_event;
use crate::sse::OpenClawStreamItem;
use crate::state::{ToolEventMappingState, build_task_completed_event};

pub(crate) fn parse_agent_sse_events(
    text: &str,
) -> Result<Vec<Result<OpenClawStreamItem, OpenClawError>>, OpenClawError> {
    let mut state = ToolEventMappingState::default();
    let mut items = Vec::new();

    for frame in text.split("\n\n") {
        let frame = frame.trim();
        if frame.is_empty() {
            continue;
        }

        let events = parse_agent_frame(frame, &mut state)?;
        items.extend(events.into_iter().map(Ok));
    }

    if state.task_started && !state.task_completed {
        items.push(Ok(OpenClawStreamItem::Event(build_task_completed_event(
            &state.task_id,
        ))));
    }

    Ok(items)
}

pub(crate) fn parse_agent_frame(
    frame: &str,
    state: &mut ToolEventMappingState,
) -> Result<Vec<OpenClawStreamItem>, OpenClawError> {
    let data_lines = frame
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("data:"))
        .map(|line| line.trim_start_matches("data:").trim())
        .collect::<Vec<_>>();
    if data_lines.is_empty() {
        return Ok(Vec::new());
    }

    let data = data_lines.join("\n");
    if data == "[DONE]" {
        if state.task_started && !state.task_completed {
            state.task_completed = true;
            return Ok(vec![OpenClawStreamItem::Event(build_task_completed_event(
                &state.ensure_task_id(),
            ))]);
        }
        return Ok(Vec::new());
    }

    let json: Value = serde_json::from_str(&data).map_err(|_| OpenClawError::InvalidResponse)?;

    if let Ok(event) = serde_json::from_value::<EventEnvelope>(json.clone()) {
        return Ok(vec![OpenClawStreamItem::Event(event)]);
    }

    if let Some(items) = map_openai_tool_call_delta(&json, state) {
        return Ok(items);
    }

    if let Some(items) = map_simple_sandbox_event(&json, state) {
        return Ok(items);
    }

    if let Some(items) = map_simple_workflow_event(&json) {
        return Ok(items);
    }

    if let Some(items) = map_simple_tool_event(&json, state) {
        return Ok(items);
    }

    Ok(Vec::new())
}

pub(crate) fn ensure_completed_item(
    state: &ToolEventMappingState,
) -> Option<Result<OpenClawStreamItem, OpenClawError>> {
    if state.task_started && !state.task_completed {
        Some(Ok(OpenClawStreamItem::Event(build_task_completed_event(
            &state.task_id,
        ))))
    } else {
        None
    }
}
