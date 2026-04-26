use nanami_protocol::{
    Event, EventEnvelope, SandboxArtifactPayload, SandboxCompletedPayload, SandboxMountMode,
    SandboxMountPayload, SandboxNetworkPolicy, SandboxOutputPayload, SandboxStartedPayload,
    SandboxStatus, SandboxUpdatedPayload, ToolOutputStream,
};
use serde_json::Value;

use crate::sse::OpenClawStreamItem;
use crate::state::{ToolEventMappingState, build_task_started_event};

pub(crate) fn map_simple_sandbox_event(
    json: &Value,
    state: &mut ToolEventMappingState,
) -> Option<Vec<OpenClawStreamItem>> {
    let sandbox_id = json.get("sandbox_id")?.as_str()?.to_owned();
    let task_id = state.ensure_task_id();
    let mut items = Vec::new();
    let mut mapped_sandbox_event = false;

    if !state.task_started {
        state.task_started = true;
        items.push(OpenClawStreamItem::Event(build_task_started_event(
            state.next_event_id(),
            &task_id,
            "OpenClaw task",
        )));
    }

    if let Some(started) = build_sandbox_started_event(json, state, &task_id, &sandbox_id) {
        items.push(OpenClawStreamItem::Event(started));
        mapped_sandbox_event = true;
    }

    if let Some(updated) = build_sandbox_updated_event(json, state, &task_id, &sandbox_id) {
        items.push(OpenClawStreamItem::Event(updated));
        mapped_sandbox_event = true;
    }

    if let Some(stdout) = json.get("stdout").and_then(Value::as_str) {
        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::SandboxOutput(SandboxOutputPayload {
                task_id: task_id.clone(),
                sandbox_id: sandbox_id.clone(),
                stream: ToolOutputStream::Stdout,
                content: stdout.to_owned(),
            }),
        )));
        mapped_sandbox_event = true;
    }

    if let Some(stderr) = json.get("stderr").and_then(Value::as_str) {
        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::SandboxOutput(SandboxOutputPayload {
                task_id: task_id.clone(),
                sandbox_id: sandbox_id.clone(),
                stream: ToolOutputStream::Stderr,
                content: stderr.to_owned(),
            }),
        )));
        mapped_sandbox_event = true;
    }

    if let Some(log) = json.get("log").and_then(Value::as_str) {
        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::SandboxOutput(SandboxOutputPayload {
                task_id: task_id.clone(),
                sandbox_id: sandbox_id.clone(),
                stream: ToolOutputStream::Log,
                content: log.to_owned(),
            }),
        )));
        mapped_sandbox_event = true;
    }

    if let Some(artifact) = json.get("artifact") {
        let name = artifact.get("name")?.as_str()?.to_owned();
        let path = artifact.get("path")?.as_str()?.to_owned();
        let media_type = artifact.get("media_type")?.as_str()?.to_owned();
        let size_bytes = artifact.get("size_bytes")?.as_u64()?;

        items.push(OpenClawStreamItem::Event(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::SandboxArtifact(SandboxArtifactPayload {
                sandbox_id: sandbox_id.clone(),
                task_id: task_id.clone(),
                name,
                path,
                media_type,
                size_bytes,
            }),
        )));
        mapped_sandbox_event = true;
    }

    if !mapped_sandbox_event {
        return None;
    }

    Some(items)
}

fn build_sandbox_started_event(
    json: &Value,
    state: &mut ToolEventMappingState,
    task_id: &str,
    sandbox_id: &str,
) -> Option<EventEnvelope> {
    let template_id = json.get("template_id")?.as_str()?.to_owned();
    let network_policy = sandbox_network_policy(json.get("network_policy")?)?;
    let mounts = json
        .get("mounts")?
        .as_array()?
        .iter()
        .map(sandbox_mount_payload)
        .collect::<Option<Vec<_>>>()?;

    Some(EventEnvelope::new(
        state.next_event_id(),
        chrono::Utc::now(),
        Event::SandboxStarted(SandboxStartedPayload {
            sandbox_id: sandbox_id.to_owned(),
            task_id: task_id.to_owned(),
            template_id,
            status: SandboxStatus::Starting,
            network_policy,
            mounts,
        }),
    ))
}

fn build_sandbox_updated_event(
    json: &Value,
    state: &mut ToolEventMappingState,
    task_id: &str,
    sandbox_id: &str,
) -> Option<EventEnvelope> {
    let status = json.get("status")?.as_str()?;

    match status {
        "running" => Some(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::SandboxUpdated(SandboxUpdatedPayload {
                sandbox_id: sandbox_id.to_owned(),
                task_id: task_id.to_owned(),
                status: SandboxStatus::Running,
                summary: json
                    .get("summary")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            }),
        )),
        "completed" | "failed" => Some(EventEnvelope::new(
            state.next_event_id(),
            chrono::Utc::now(),
            Event::SandboxCompleted(SandboxCompletedPayload {
                sandbox_id: sandbox_id.to_owned(),
                task_id: task_id.to_owned(),
                status: if status == "completed" {
                    SandboxStatus::Completed
                } else {
                    SandboxStatus::Failed
                },
                exit_code: json
                    .get("exit_code")
                    .and_then(Value::as_i64)
                    .map(|value| value as i32),
                summary: json
                    .get("summary")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            }),
        )),
        _ => None,
    }
}

fn sandbox_network_policy(value: &Value) -> Option<SandboxNetworkPolicy> {
    match value.as_str()? {
        "disabled" => Some(SandboxNetworkPolicy::Disabled),
        "limited" => Some(SandboxNetworkPolicy::Limited),
        "enabled" => Some(SandboxNetworkPolicy::Enabled),
        _ => None,
    }
}

fn sandbox_mount_payload(value: &Value) -> Option<SandboxMountPayload> {
    let mount = value.as_object()?;
    Some(SandboxMountPayload {
        host_path: mount.get("host_path")?.as_str()?.to_owned(),
        sandbox_path: mount.get("sandbox_path")?.as_str()?.to_owned(),
        mode: match mount.get("mode")?.as_str()? {
            "readonly" => SandboxMountMode::ReadOnly,
            "readwrite" => SandboxMountMode::ReadWrite,
            _ => return None,
        },
    })
}
