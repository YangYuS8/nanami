use std::time::Duration;

use async_stream::try_stream;
use nanami_protocol::{
    ChatStreamEvent, ChatStreamEventKind, Event, EventEnvelope, OpenClawConnectionStatus,
    OpenClawStatusPayload, SandboxArtifactPayload, SandboxCompletedPayload, SandboxMountMode,
    SandboxMountPayload, SandboxNetworkPolicy, SandboxOutputPayload, SandboxStartedPayload,
    SandboxStatus, SandboxUpdatedPayload, TaskCompletedPayload, TaskStartedPayload, TaskStatus,
    ToolCallStatus, ToolCompletedPayload, ToolOutputPayload, ToolOutputStream, ToolStartedPayload,
    WorkflowChangeType, WorkflowCompletedPayload, WorkflowPatchFilePreviewPayload,
    WorkflowPatchProposedPayload, WorkflowPatchRiskLevel, WorkflowStartedPayload, WorkflowStatus,
    WorkflowStepKind, WorkflowStepPayload, WorkflowStepStatus, WorkflowTestResultPayload,
};
use serde_json::Value;
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt, iter};

pub type OpenClawChatStream =
    Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, OpenClawError>> + Send>>;
pub type OpenClawAgentStream =
    Pin<Box<dyn Stream<Item = Result<OpenClawStreamItem, OpenClawError>> + Send>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenClawStreamItem {
    Chat(ChatStreamEvent),
    Event(EventEnvelope),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawConfig {
    pub gateway_url: String,
    pub token: Option<String>,
    pub timeout_ms: u64,
    pub chat_path: String,
}

impl OpenClawConfig {
    pub fn with_default_chat_path(
        gateway_url: String,
        token: Option<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            gateway_url,
            token,
            timeout_ms,
            // 0.2b placeholder: OpenClaw Gateway chat endpoint is not stable yet.
            // Keep this default centralized in the adapter so UI/core do not depend on it.
            chat_path: "/chat".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatResponse {
    pub content: String,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OpenClawClient {
    config: OpenClawConfig,
    http: reqwest::Client,
}

#[derive(Debug)]
pub enum OpenClawError {
    InvalidClient(reqwest::Error),
    AuthFailed,
    Disconnected,
    InvalidResponse,
    UnexpectedStatus(u16),
}

impl std::fmt::Display for OpenClawError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidClient(_) => write!(formatter, "OpenClaw client configuration failed"),
            Self::AuthFailed => write!(formatter, "OpenClaw Gateway authentication failed"),
            Self::Disconnected => write!(formatter, "OpenClaw Gateway is unreachable"),
            Self::InvalidResponse => write!(
                formatter,
                "OpenClaw Gateway returned an unsupported response"
            ),
            Self::UnexpectedStatus(status) => {
                write!(formatter, "OpenClaw Gateway returned HTTP {status}")
            }
        }
    }
}

impl std::error::Error for OpenClawError {}

impl OpenClawClient {
    pub fn new(config: OpenClawConfig) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("reqwest client configuration should be valid");

        Self { config, http }
    }

    pub async fn check_status(&self) -> Result<OpenClawStatusPayload, OpenClawError> {
        if self.config.gateway_url.trim().is_empty() {
            return Ok(self.payload(
                OpenClawConnectionStatus::Disconnected,
                Some("OpenClaw Gateway URL is not configured"),
            ));
        }

        // 0.2 placeholder: OpenClaw Gateway health endpoint is not stable yet.
        // Use the configured gateway URL as the conservative reachability probe.
        let mut request = self.http.get(&self.config.gateway_url);
        if let Some(token) = &self.config.token {
            request = request.bearer_auth(token);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Ok(self.payload(
                    OpenClawConnectionStatus::Disconnected,
                    Some("OpenClaw Gateway is unreachable"),
                ));
            }
            Err(error) => {
                return Ok(self.payload(
                    OpenClawConnectionStatus::Error,
                    Some(&error_without_secret(&error)),
                ));
            }
        };

        let status_code = response.status();
        let body = response.text().await.unwrap_or_default().to_lowercase();

        if body.contains("pairing_required") || body.contains("pairing required") {
            return Ok(self.payload(
                OpenClawConnectionStatus::PairingRequired,
                Some("OpenClaw Gateway requires pairing"),
            ));
        }

        if body.contains("scope_missing") || body.contains("scope missing") {
            return Ok(self.payload(
                OpenClawConnectionStatus::ScopeMissing,
                Some("OpenClaw Gateway reports missing scope"),
            ));
        }

        if status_code.is_success() {
            return Ok(self.payload(OpenClawConnectionStatus::Connected, None));
        }

        if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
            return Ok(self.payload(
                OpenClawConnectionStatus::AuthFailed,
                Some("OpenClaw Gateway authentication failed"),
            ));
        }

        Ok(self.payload(
            OpenClawConnectionStatus::Error,
            Some("OpenClaw Gateway returned an unexpected status"),
        ))
    }

    pub async fn send_chat_message(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawChatResponse, OpenClawError> {
        let url = format!(
            "{}{}",
            self.config.gateway_url.trim_end_matches('/'),
            normalized_path(&self.config.chat_path)
        );
        let body = serde_json::json!({
            "message": request.message,
            "session_id": request.session_id,
        });

        let mut http_request = self.http.post(url).json(&body);
        if let Some(token) = &self.config.token {
            http_request = http_request.bearer_auth(token);
        }

        let response = match http_request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Err(OpenClawError::Disconnected);
            }
            Err(_) => return Err(OpenClawError::InvalidResponse),
        };

        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(OpenClawError::AuthFailed);
        }
        if !status.is_success() {
            return Err(OpenClawError::UnexpectedStatus(status.as_u16()));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|_| OpenClawError::InvalidResponse)?;
        let content = extract_content(&json).ok_or(OpenClawError::InvalidResponse)?;

        Ok(OpenClawChatResponse {
            content,
            session_id: json
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message_id: json
                .get("message_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }

    pub async fn stream_chat_message(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawChatStream, OpenClawError> {
        let url = format!(
            "{}{}",
            self.config.gateway_url.trim_end_matches('/'),
            normalized_path(&self.config.chat_path)
        );
        let body = serde_json::json!({
            "message": request.message,
            "session_id": request.session_id,
            "stream": true,
        });

        let mut http_request = self.http.post(url).json(&body);
        if let Some(token) = &self.config.token {
            http_request = http_request.bearer_auth(token);
        }

        let response = match http_request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Err(OpenClawError::Disconnected);
            }
            Err(_) => return Err(OpenClawError::InvalidResponse),
        };

        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(OpenClawError::AuthFailed);
        }
        if !status.is_success() {
            return Err(OpenClawError::UnexpectedStatus(status.as_u16()));
        }

        let header_is_sse = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("text/event-stream"));
        if !header_is_sse {
            let text = response
                .text()
                .await
                .map_err(|_| OpenClawError::InvalidResponse)?;
            let events = if text.trim_start().starts_with("data:") {
                parse_sse_events(&text)?
            } else {
                let json: Value =
                    serde_json::from_str(&text).map_err(|_| OpenClawError::InvalidResponse)?;
                let content = extract_content(&json).ok_or(OpenClawError::InvalidResponse)?;
                vec![Ok(ChatStreamEvent {
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
                })]
            };

            return Ok(Box::pin(iter(events)));
        }

        let stream = try_stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut accumulated = String::new();
            let mut completed = false;

            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk.map_err(|_| OpenClawError::InvalidResponse)?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(separator) = buffer.find("\n\n") {
                    let frame = buffer[..separator].to_owned();
                    buffer.drain(..separator + 2);
                    if let Some(event) = parse_sse_frame(&frame, &mut accumulated, &mut completed)? {
                        yield event;
                    }
                }
            }

            if !buffer.trim().is_empty() {
                let pending_event = parse_sse_frame(&buffer, &mut accumulated, &mut completed)?;
                if let Some(event) = pending_event {
                    yield event;
                }
            }

            if !completed {
                yield ChatStreamEvent {
                    kind: ChatStreamEventKind::MessageCompleted,
                    session_id: None,
                    message_id: None,
                    delta: None,
                    content: Some(accumulated),
                    error: None,
                };
            }
        };

        Ok(Box::pin(stream))
    }

    pub async fn stream_agent_events(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawAgentStream, OpenClawError> {
        let url = format!(
            "{}{}",
            self.config.gateway_url.trim_end_matches('/'),
            normalized_path(&self.config.chat_path)
        );
        let body = serde_json::json!({
            "message": request.message,
            "session_id": request.session_id,
            "stream": true,
        });

        let mut http_request = self.http.post(url).json(&body);
        if let Some(token) = &self.config.token {
            http_request = http_request.bearer_auth(token);
        }

        let response = match http_request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Err(OpenClawError::Disconnected);
            }
            Err(_) => return Err(OpenClawError::InvalidResponse),
        };

        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(OpenClawError::AuthFailed);
        }
        if !status.is_success() {
            return Err(OpenClawError::UnexpectedStatus(status.as_u16()));
        }

        let header_is_sse = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("text/event-stream"));

        if !header_is_sse {
            let text = response
                .text()
                .await
                .map_err(|_| OpenClawError::InvalidResponse)?;
            let items = if text.trim_start().starts_with("data:") {
                parse_agent_sse_events(&text)?
            } else {
                Vec::new()
            };

            return Ok(Box::pin(iter(items)));
        }

        let stream = try_stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut state = ToolEventMappingState::default();

            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk.map_err(|_| OpenClawError::InvalidResponse)?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(separator) = buffer.find("\n\n") {
                    let frame = buffer[..separator].to_owned();
                    buffer.drain(..separator + 2);
                    let items = parse_agent_frame(&frame, &mut state)?;
                    for item in items {
                        yield item;
                    }
                }
            }

            if !buffer.trim().is_empty() {
                let items = parse_agent_frame(&buffer, &mut state)?;
                for item in items {
                    yield item;
                }
            }

            if state.task_started && !state.task_completed {
                yield OpenClawStreamItem::Event(build_task_completed_event(&state.task_id));
            }
        };

        Ok(Box::pin(stream))
    }

    fn payload(
        &self,
        status: OpenClawConnectionStatus,
        message: Option<&str>,
    ) -> OpenClawStatusPayload {
        OpenClawStatusPayload {
            status,
            gateway_url: self.config.gateway_url.clone(),
            message: message.map(str::to_owned),
            agent: None,
            profile: None,
        }
    }
}

fn parse_sse_events(
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

fn parse_agent_sse_events(
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

#[derive(Default)]
struct ToolEventMappingState {
    counter: usize,
    task_started: bool,
    task_completed: bool,
    task_id: String,
}

impl ToolEventMappingState {
    fn next_event_id(&mut self) -> String {
        self.counter += 1;
        format!("evt_openclaw_tool_{:03}", self.counter)
    }

    fn ensure_task_id(&mut self) -> String {
        if self.task_id.is_empty() {
            self.task_id = "task_openclaw_stream_001".into();
        }
        self.task_id.clone()
    }
}

fn parse_agent_frame(
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

fn map_openai_tool_call_delta(
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

fn map_simple_workflow_event(json: &Value) -> Option<Vec<OpenClawStreamItem>> {
    json.get("workflow_id")?;

    let workflow_id = json.get("workflow_id")?.as_str()?.to_owned();
    let task_id = json.get("task_id")?.as_str()?.to_owned();

    if let Some(project_path) = json.get("project_path").and_then(Value::as_str) {
        let status = workflow_status(json.get("status")?.as_str()?)?;
        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_started_001",
            chrono::Utc::now(),
            Event::WorkflowStarted(WorkflowStartedPayload {
                workflow_id,
                task_id,
                project_path: project_path.to_owned(),
                status,
            }),
        ))]);
    }

    if let Some(step_kind) = json.get("step_kind").and_then(Value::as_str) {
        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_step_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id,
                task_id,
                step_kind: workflow_step_kind(step_kind)?,
                status: workflow_step_status(json.get("status")?.as_str()?)?,
                summary: json.get("summary")?.as_str()?.to_owned(),
            }),
        ))]);
    }

    if json.get("command_preview").is_some() {
        let failed_test_names = json
            .get("failed_test_names")?
            .as_array()?
            .iter()
            .map(|value| value.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()?;

        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_test_result_001",
            chrono::Utc::now(),
            Event::WorkflowTestResult(WorkflowTestResultPayload {
                workflow_id,
                task_id,
                status: workflow_status(json.get("status")?.as_str()?)?,
                summary: json.get("summary")?.as_str()?.to_owned(),
                command_preview: json.get("command_preview")?.as_str()?.to_owned(),
                duration_ms: json.get("duration_ms")?.as_u64()?,
                passed: json.get("passed")?.as_u64()? as u32,
                failed: json.get("failed")?.as_u64()? as u32,
                failed_test_names,
            }),
        ))]);
    }

    if json.get("patch_id").is_some() {
        let files = json
            .get("files")?
            .as_array()?
            .iter()
            .map(workflow_patch_file_preview)
            .collect::<Option<Vec<_>>>()?;

        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_patch_001",
            chrono::Utc::now(),
            Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
                workflow_id,
                task_id,
                patch_id: json.get("patch_id")?.as_str()?.to_owned(),
                summary: json.get("summary")?.as_str()?.to_owned(),
                diff_summary: json.get("diff_summary")?.as_str()?.to_owned(),
                risk_level: workflow_patch_risk_level(json.get("risk_level")?.as_str()?)?,
                files,
            }),
        ))]);
    }

    if let Some(summary) = json.get("summary").and_then(Value::as_str)
        && let Some(status) = json.get("status").and_then(Value::as_str)
        && matches!(status, "completed" | "failed")
    {
        return Some(vec![OpenClawStreamItem::Event(EventEnvelope::new(
            "evt_openclaw_workflow_completed_001",
            chrono::Utc::now(),
            Event::WorkflowCompleted(WorkflowCompletedPayload {
                workflow_id,
                task_id,
                status: workflow_status(status)?,
                summary: summary.to_owned(),
            }),
        ))]);
    }

    None
}

fn workflow_status(value: &str) -> Option<WorkflowStatus> {
    match value {
        "running" => Some(WorkflowStatus::Running),
        "waiting_permission" => Some(WorkflowStatus::WaitingPermission),
        "completed" => Some(WorkflowStatus::Completed),
        "failed" => Some(WorkflowStatus::Failed),
        _ => None,
    }
}

fn workflow_step_kind(value: &str) -> Option<WorkflowStepKind> {
    match value {
        "open_project" => Some(WorkflowStepKind::OpenProject),
        "analyze_project" => Some(WorkflowStepKind::AnalyzeProject),
        "run_tests" => Some(WorkflowStepKind::RunTests),
        "patch_proposed" => Some(WorkflowStepKind::PatchProposed),
        "apply_patch" => Some(WorkflowStepKind::ApplyPatch),
        "verify" => Some(WorkflowStepKind::Verify),
        _ => None,
    }
}

fn workflow_step_status(value: &str) -> Option<WorkflowStepStatus> {
    match value {
        "pending" => Some(WorkflowStepStatus::Pending),
        "running" => Some(WorkflowStepStatus::Running),
        "completed" => Some(WorkflowStepStatus::Completed),
        "waiting_permission" => Some(WorkflowStepStatus::WaitingPermission),
        "failed" => Some(WorkflowStepStatus::Failed),
        _ => None,
    }
}

fn workflow_patch_risk_level(value: &str) -> Option<WorkflowPatchRiskLevel> {
    match value {
        "low" => Some(WorkflowPatchRiskLevel::Low),
        "medium" => Some(WorkflowPatchRiskLevel::Medium),
        "high" => Some(WorkflowPatchRiskLevel::High),
        _ => None,
    }
}

fn workflow_patch_file_preview(value: &Value) -> Option<WorkflowPatchFilePreviewPayload> {
    let file = value.as_object()?;
    Some(WorkflowPatchFilePreviewPayload {
        path: file.get("path")?.as_str()?.to_owned(),
        change_type: match file.get("change_type")?.as_str()? {
            "added" => WorkflowChangeType::Added,
            "modified" => WorkflowChangeType::Modified,
            "deleted" => WorkflowChangeType::Deleted,
            "renamed" => WorkflowChangeType::Renamed,
            _ => return None,
        },
        diff_preview: file.get("diff_preview")?.as_str()?.to_owned(),
    })
}

fn map_simple_sandbox_event(
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

fn map_simple_tool_event(
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

fn build_task_started_event(id: String, task_id: &str, title: &str) -> EventEnvelope {
    EventEnvelope::new(
        id,
        chrono::Utc::now(),
        Event::TaskStarted(TaskStartedPayload {
            session_id: None,
            task_id: task_id.to_owned(),
            title: title.to_owned(),
            status: TaskStatus::Running,
        }),
    )
}

fn build_task_completed_event(task_id: &str) -> EventEnvelope {
    EventEnvelope::new(
        "evt_openclaw_task_completed_001",
        chrono::Utc::now(),
        Event::TaskCompleted(TaskCompletedPayload {
            task_id: task_id.to_owned(),
            status: TaskStatus::Completed,
            summary: Some("OpenClaw stream completed".into()),
        }),
    )
}

fn parse_sse_frame(
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

fn normalized_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

fn extract_content(json: &Value) -> Option<String> {
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

fn extract_delta(json: &Value) -> Option<String> {
    json.get("delta")
        .and_then(Value::as_str)
        .or_else(|| {
            json.pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
        })
        .map(str::to_owned)
}

fn error_without_secret(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "OpenClaw Gateway request timed out".into()
    } else if error.is_connect() {
        "OpenClaw Gateway is unreachable".into()
    } else {
        "OpenClaw Gateway request failed".into()
    }
}
