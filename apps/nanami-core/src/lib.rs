use axum::{
    Json, Router,
    extract::Path,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures_util::StreamExt as FuturesStreamExt;
use nanami_openclaw::{
    OpenClawChatRequest, OpenClawChatStream, OpenClawClient, OpenClawConfig, OpenClawError,
    OpenClawStreamItem,
};
use nanami_permission::PermissionManager;
use nanami_protocol::{
    ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamEventKind, ErrorPayload, ErrorSeverity,
    Event, EventEnvelope, OpenClawConnectionStatus, OpenClawStatusPayload,
    PermissionAuditLogResponse, PermissionDecision, PermissionDecisionStatus, PermissionLevel,
    PermissionRequestPayload, PermissionResolvedPayload, PermissionScope, PersonaEmotion,
    PersonaState, PersonaStatePayload, PersonaStateSource, ProjectKind, ProjectMetadata,
    ProjectTrustStatus, TaskCompletedPayload, TaskStartedPayload, TaskStatus, ToolCallStatus,
    ToolCompletedPayload, ToolOutputPayload, ToolOutputStream, ToolStartedPayload,
    WorkflowChangeType, WorkflowCompletedPayload, WorkflowPatchFilePreviewPayload,
    WorkflowPatchProposedPayload, WorkflowPatchRiskLevel, WorkflowStartedPayload, WorkflowStatus,
    WorkflowStepKind, WorkflowStepPayload, WorkflowStepStatus, WorkflowTestResultPayload,
};
use serde::Serialize;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use tokio_stream::once;

const DEFAULT_OPENCLAW_TIMEOUT_MS: u64 = 3000;
type NanamiEventStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<EventEnvelope, ErrorPayload>> + Send>>;

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
            "/workflow/mock/apply-patch",
            post(workflow_mock_apply_patch),
        )
        .route("/projects/mock/current", get(projects_mock_current))
        .route("/permissions/mock/stream", get(permissions_mock_stream))
        .route("/permissions/resolve", post(permissions_resolve))
        .route(
            "/permissions/decision/:permission_id",
            get(permission_decision),
        )
        .route("/permissions/audit", get(permission_audit))
        .with_state(AppState {
            openclaw,
            permission_manager: Arc::new(Mutex::new(PermissionManager::new())),
        })
}

#[derive(Clone)]
struct AppState {
    openclaw: Arc<dyn OpenClawService>,
    permission_manager: Arc<Mutex<PermissionManager>>,
}

#[derive(Debug, serde::Deserialize)]
struct PermissionResolveRequest {
    permission_id: String,
    decision: PermissionDecision,
}

#[derive(Debug, serde::Deserialize)]
struct WorkflowApplyPatchRequest {
    patch_id: String,
}

#[derive(Debug, Serialize)]
struct WorkflowApplyPatchResponse {
    patch_id: String,
    permission_id: String,
    status: &'static str,
    message: &'static str,
}

trait OpenClawService: Send + Sync {
    fn send_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>>;
    fn stream_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>>;
    fn stream_agent_events(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>>;
}

#[derive(Clone)]
struct EnvOpenClawService;

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        protocol_version: nanami_protocol::PROTOCOL_VERSION,
    })
}

async fn openclaw_status() -> Json<nanami_protocol::OpenClawStatusPayload> {
    Json(crate::openclaw_status_from_config(None).await)
}

async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    if request.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ChatEndpointResponse::Error(ErrorPayload {
                task_id: None,
                severity: ErrorSeverity::Error,
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

async fn chat_stream(State(state): State<AppState>, Json(request): Json<ChatRequest>) -> Response {
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

async fn tasks_mock_stream() -> Response {
    // 0.3a mock only: fixed task/tool event sequence for Task Panel visualization.
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

async fn tasks_openclaw_stream(
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

async fn sandbox_mock_stream() -> Response {
    Sse::new(tokio_stream::iter(
        nanami_sandbox::mock_sandbox_events()
            .into_iter()
            .map(|event| {
                Ok::<_, Infallible>(
                    SseEvent::default().data(serde_json::to_string(&event).unwrap()),
                )
            }),
    ))
    .keep_alive(KeepAlive::default())
    .into_response()
}

async fn persona_mock_stream() -> Response {
    let events = vec![
        EventEnvelope::new(
            "evt_persona_mock_idle_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::Idle,
                emotion: PersonaEmotion::Neutral,
                text: "Standing by".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
        EventEnvelope::new(
            "evt_persona_mock_listening_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::Listening,
                emotion: PersonaEmotion::Focused,
                text: "Listening to your request".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
        EventEnvelope::new(
            "evt_persona_mock_thinking_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::Thinking,
                emotion: PersonaEmotion::Focused,
                text: "Thinking through the task".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
        EventEnvelope::new(
            "evt_persona_mock_tool_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::ToolCall,
                emotion: PersonaEmotion::Surprised,
                text: "Preparing a tool call".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
        EventEnvelope::new(
            "evt_persona_mock_waiting_permission_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::WaitingPermission,
                emotion: PersonaEmotion::Worried,
                text: "Waiting for permission".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
        EventEnvelope::new(
            "evt_persona_mock_success_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::Success,
                emotion: PersonaEmotion::Happy,
                text: "Task finished successfully".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
        EventEnvelope::new(
            "evt_persona_mock_error_001",
            chrono::Utc::now(),
            Event::PersonaState(PersonaStatePayload {
                state: PersonaState::Error,
                emotion: PersonaEmotion::Worried,
                text: "Something went wrong".into(),
                source: PersonaStateSource::Mock,
            }),
        ),
    ];

    Sse::new(tokio_stream::iter(events.into_iter().map(|event| {
        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    })))
    .keep_alive(KeepAlive::default())
    .into_response()
}

async fn workflow_mock_stream() -> Response {
    let events = vec![
        EventEnvelope::new(
            "evt_workflow_started_001",
            chrono::Utc::now(),
            Event::WorkflowStarted(WorkflowStartedPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                project_path: "/mock/project".into(),
                status: WorkflowStatus::Running,
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_open_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::OpenProject,
                status: WorkflowStepStatus::Completed,
                summary: "Mock project context opened".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_analyze_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::AnalyzeProject,
                status: WorkflowStepStatus::Completed,
                summary: "Mock analysis finished".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_run_tests_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::RunTests,
                status: WorkflowStepStatus::Completed,
                summary: "Mock tests executed".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_test_result_001",
            chrono::Utc::now(),
            Event::WorkflowTestResult(WorkflowTestResultPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                status: WorkflowStatus::Completed,
                summary: "2 tests passed, 1 failed".into(),
                command_preview: "cargo test --lib".into(),
                duration_ms: 1200,
                passed: 2,
                failed: 1,
                failed_test_names: vec!["tests::mock_failure".into()],
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_patch_proposed_001",
            chrono::Utc::now(),
            Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                patch_id: "patch_mock_001".into(),
                summary: "Mock patch proposal ready".into(),
                diff_summary: "1 file modified".into(),
                risk_level: WorkflowPatchRiskLevel::Medium,
                files: vec![WorkflowPatchFilePreviewPayload {
                    path: "src/main.rs".into(),
                    change_type: WorkflowChangeType::Modified,
                    diff_preview: "- old line\n+ new line".into(),
                }],
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_step_apply_patch_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                step_kind: WorkflowStepKind::ApplyPatch,
                status: WorkflowStepStatus::WaitingPermission,
                summary: "Waiting for patch approval".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_completed_001",
            chrono::Utc::now(),
            Event::WorkflowCompleted(WorkflowCompletedPayload {
                workflow_id: "workflow_mock_001".into(),
                task_id: "task_workflow_mock_001".into(),
                status: WorkflowStatus::Completed,
                summary: "Mock workflow completed".into(),
            }),
        ),
    ];

    Sse::new(tokio_stream::iter(events.into_iter().map(|event| {
        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    })))
    .keep_alive(KeepAlive::default())
    .into_response()
}

async fn projects_mock_current() -> Json<ProjectMetadata> {
    Json(ProjectMetadata {
        project_id: "project_mock_001".into(),
        display_name: "Nanami Mock Workspace".into(),
        project_path: "/mock/project".into(),
        kind: ProjectKind::Rust,
        trust_status: ProjectTrustStatus::TrustedMock,
    })
}

async fn workflow_mock_apply_patch(
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

fn maybe_permission_for_tool_event(event: &EventEnvelope) -> Option<EventEnvelope> {
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

async fn permissions_mock_stream(State(state): State<AppState>) -> Response {
    // 0.4a mock only: fixed permission request for UI permission flow.
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

async fn permissions_resolve(
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

async fn permission_decision(
    State(state): State<AppState>,
    Path(permission_id): Path<String>,
) -> Json<PermissionDecisionStatus> {
    let manager = state.permission_manager.lock().unwrap();

    Json(PermissionDecisionStatus {
        permission_id: permission_id.clone(),
        decision: manager.decision_for(&permission_id),
    })
}

async fn permission_audit(State(state): State<AppState>) -> Json<PermissionAuditLogResponse> {
    let manager = state.permission_manager.lock().unwrap();

    Json(PermissionAuditLogResponse {
        records: manager.audit_records(),
    })
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ChatEndpointResponse {
    Ok(ChatResponse),
    Error(ErrorPayload),
}

async fn openclaw_status_from_config(gateway_url: Option<String>) -> OpenClawStatusPayload {
    let gateway_url = gateway_url
        .unwrap_or_else(|| std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default());
    if gateway_url.trim().is_empty() {
        return OpenClawStatusPayload {
            status: OpenClawConnectionStatus::Disconnected,
            gateway_url,
            message: Some("NANAMI_OPENCLAW_GATEWAY_URL is not configured".into()),
            agent: None,
            profile: None,
        };
    }

    let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));

    match client.check_status().await {
        Ok(status) => status,
        Err(_) => OpenClawStatusPayload {
            status: OpenClawConnectionStatus::Error,
            gateway_url: String::new(),
            message: Some("OpenClaw status check failed".into()),
            agent: None,
            profile: None,
        },
    }
}

impl OpenClawService for EnvOpenClawService {
    fn send_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            let gateway_url = std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default();
            if gateway_url.trim().is_empty() {
                return Err(chat_error(
                    "OPENCLAW_GATEWAY_UNCONFIGURED",
                    "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                    Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
                ));
            }

            let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));
            client
                .send_chat_message(OpenClawChatRequest {
                    message: request.message,
                    session_id: request.session_id.clone(),
                })
                .await
                .map(|response| ChatResponse {
                    session_id: response
                        .session_id
                        .or(request.session_id)
                        .unwrap_or_else(|| "default".into()),
                    message_id: response.message_id.unwrap_or_else(|| "msg_openclaw".into()),
                    content: response.content,
                })
                .map_err(map_openclaw_chat_error)
        })
    }

    fn stream_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            let gateway_url = std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default();
            if gateway_url.trim().is_empty() {
                return Err(chat_error(
                    "OPENCLAW_GATEWAY_UNCONFIGURED",
                    "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                    Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
                ));
            }

            let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));
            let stream = client
                .stream_chat_message(OpenClawChatRequest {
                    message: request.message,
                    session_id: request.session_id,
                })
                .await
                .map_err(map_openclaw_chat_error)?;
            Ok(stream)
        })
    }

    fn stream_agent_events(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            let gateway_url = std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default();
            if gateway_url.trim().is_empty() {
                return Err(chat_error(
                    "OPENCLAW_GATEWAY_UNCONFIGURED",
                    "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                    Some("Set NANAMI_OPENCLAW_GATEWAY_URL before starting OpenClaw task streams"),
                ));
            }

            let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));
            let stream = client
                .stream_agent_events(OpenClawChatRequest {
                    message: request.message,
                    session_id: request.session_id,
                })
                .await
                .map_err(map_openclaw_chat_error)?;
            let mapped = futures_util::StreamExt::flat_map(stream, |item| match item {
                Ok(OpenClawStreamItem::Event(event)) => {
                    tokio_stream::iter(vec![Ok::<_, ErrorPayload>(event)])
                }
                Ok(OpenClawStreamItem::Chat(_)) => tokio_stream::iter(Vec::new()),
                Err(error) => tokio_stream::iter(vec![Err::<EventEnvelope, _>(
                    map_openclaw_chat_error(error),
                )]),
            });

            Ok(Box::pin(mapped) as NanamiEventStream)
        })
    }
}

fn openclaw_config_from_env(gateway_url: String) -> OpenClawConfig {
    let chat_path = std::env::var("NANAMI_OPENCLAW_CHAT_PATH").unwrap_or_else(|_| "/chat".into());
    OpenClawConfig {
        gateway_url,
        token: std::env::var("NANAMI_OPENCLAW_TOKEN").ok(),
        timeout_ms: DEFAULT_OPENCLAW_TIMEOUT_MS,
        chat_path,
    }
}

fn map_openclaw_chat_error(error: OpenClawError) -> ErrorPayload {
    match error {
        OpenClawError::AuthFailed => chat_error(
            "OPENCLAW_AUTH_FAILED",
            "OpenClaw Gateway authentication failed",
            Some("Check NANAMI_OPENCLAW_TOKEN"),
        ),
        OpenClawError::Disconnected => chat_error(
            "OPENCLAW_DISCONNECTED",
            "OpenClaw Gateway is unreachable",
            Some("Check NANAMI_OPENCLAW_GATEWAY_URL"),
        ),
        OpenClawError::InvalidResponse => chat_error(
            "OPENCLAW_INVALID_RESPONSE",
            "OpenClaw Gateway returned an unsupported chat response",
            None,
        ),
        OpenClawError::UnexpectedStatus(_) | OpenClawError::InvalidClient(_) => {
            chat_error("OPENCLAW_CHAT_FAILED", "OpenClaw chat request failed", None)
        }
    }
}

fn chat_error(code: &str, message: &str, action_hint: Option<&str>) -> ErrorPayload {
    ErrorPayload {
        task_id: None,
        severity: ErrorSeverity::Error,
        code: code.into(),
        message: message.into(),
        action_hint: action_hint.map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use crate::NanamiEventStream;
    use crate::OpenClawService;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use nanami_openclaw::OpenClawChatStream;
    use nanami_protocol::{
        ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamEventKind, ErrorPayload, Event,
        EventEnvelope, OpenClawConnectionStatus, TaskCompletedPayload, TaskStartedPayload,
        TaskStatus, ToolOutputPayload, ToolOutputStream, ToolStartedPayload,
    };
    use std::pin::Pin;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_returns_protocol_version() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "ok");
        assert_eq!(json["protocol_version"], nanami_protocol::PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn openclaw_status_unconfigured_returns_disconnected() {
        let status = crate::openclaw_status_from_config(Some("".into())).await;

        assert_eq!(status.status, OpenClawConnectionStatus::Disconnected);
        assert_eq!(status.gateway_url, "");
        assert!(status.message.is_some());
    }

    #[tokio::test]
    async fn openclaw_status_endpoint_returns_ok() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/openclaw/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn openclaw_status_endpoint_returns_status_and_gateway_url() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/openclaw/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("status").is_some());
        assert!(json.get("gateway_url").is_some());
    }

    #[derive(Clone)]
    struct StubOpenClawService {
        response: Result<ChatResponse, ErrorPayload>,
        stream_response: Result<Vec<ChatStreamEvent>, ErrorPayload>,
        agent_stream_response: Result<Vec<EventEnvelope>, ErrorPayload>,
    }

    impl OpenClawService for StubOpenClawService {
        fn send_chat_message(
            &self,
            _request: ChatRequest,
        ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>> {
            Box::pin(async { self.response.clone() })
        }

        fn stream_chat_message(
            &self,
            _request: ChatRequest,
        ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>>
        {
            Box::pin(async move {
                self.stream_response.clone().map(|events| {
                    Box::pin(tokio_stream::iter(events.into_iter().map(Ok))) as OpenClawChatStream
                })
            })
        }

        fn stream_agent_events(
            &self,
            _request: ChatRequest,
        ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>>
        {
            Box::pin(async move {
                self.agent_stream_response.clone().map(|events| {
                    Box::pin(tokio_stream::iter(events.into_iter().map(Ok))) as NanamiEventStream
                })
            })
        }
    }

    #[tokio::test]
    async fn chat_endpoint_rejects_empty_message() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Ok(ChatResponse {
                session_id: "sess_001".into(),
                message_id: "msg_001".into(),
                content: "unused".into(),
            }),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn chat_service_unconfigured_gateway_returns_structured_error() {
        let service = crate::EnvOpenClawService;

        let error = service
            .send_chat_message(ChatRequest {
                session_id: None,
                message: "Hello".into(),
            })
            .await
            .unwrap_err();

        assert_eq!(error.code, "OPENCLAW_GATEWAY_UNCONFIGURED");
        assert!(!error.message.contains("token"));
    }

    #[tokio::test]
    async fn chat_endpoint_returns_adapter_content() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Ok(ChatResponse {
                session_id: "sess_001".into(),
                message_id: "msg_001".into(),
                content: "Hello from adapter".into(),
            }),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["content"], "Hello from adapter");
    }

    #[tokio::test]
    async fn chat_errors_do_not_leak_token() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error(
                "OPENCLAW_AUTH_FAILED",
                "OpenClaw Gateway authentication failed",
                Some("Check NANAMI_OPENCLAW_TOKEN"),
            )),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(!text.contains("secret-token"));
        assert!(!text.contains("Bearer"));
    }

    #[tokio::test]
    async fn chat_stream_endpoint_returns_sse_content_type() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(vec![ChatStreamEvent {
                kind: ChatStreamEventKind::MessageCompleted,
                session_id: Some("sess_001".into()),
                message_id: Some("msg_001".into()),
                delta: None,
                content: Some("Hello".into()),
                error: None,
            }]),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn chat_stream_endpoint_contains_delta_and_completed() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(vec![
                ChatStreamEvent {
                    kind: ChatStreamEventKind::MessageDelta,
                    session_id: Some("sess_001".into()),
                    message_id: Some("msg_001".into()),
                    delta: Some("你".into()),
                    content: None,
                    error: None,
                },
                ChatStreamEvent {
                    kind: ChatStreamEventKind::MessageCompleted,
                    session_id: Some("sess_001".into()),
                    message_id: Some("msg_001".into()),
                    delta: None,
                    content: Some("你好".into()),
                    error: None,
                },
            ]),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("message_delta"));
        assert!(text.contains("message_completed"));
    }

    #[tokio::test]
    async fn chat_stream_endpoint_unconfigured_gateway_returns_error_event() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Err(crate::chat_error(
                "OPENCLAW_GATEWAY_UNCONFIGURED",
                "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
            )),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("\"kind\":\"error\""));
        assert!(text.contains("OPENCLAW_GATEWAY_UNCONFIGURED"));
    }

    #[tokio::test]
    async fn chat_stream_endpoint_rejects_empty_message() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(Vec::new()),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn chat_stream_service_unconfigured_gateway_returns_structured_error() {
        let service = crate::EnvOpenClawService;

        let error = match service
            .stream_chat_message(ChatRequest {
                session_id: None,
                message: "Hello".into(),
            })
            .await
        {
            Ok(_) => panic!("expected unconfigured gateway error"),
            Err(error) => error,
        };

        assert_eq!(error.code, "OPENCLAW_GATEWAY_UNCONFIGURED");
    }

    #[tokio::test]
    async fn tasks_mock_stream_returns_sse_content_type() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/tasks/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn tasks_mock_stream_contains_mock_task_and_tool_events() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/tasks/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("task.started"));
        assert!(text.contains("tool.started"));
        assert!(text.contains("tool.output"));
        assert!(text.contains("tool.completed"));
        assert!(text.contains("task.completed"));
    }

    #[tokio::test]
    async fn sandbox_mock_stream_returns_sse_content_type() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/sandbox/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn sandbox_mock_stream_contains_sandbox_event_sequence() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/sandbox/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("sandbox.started"));
        assert!(text.contains("sandbox.updated"));
        assert!(text.contains("sandbox.output"));
        assert!(text.contains("sandbox.artifact"));
        assert!(text.contains("sandbox.completed"));
    }

    #[tokio::test]
    async fn persona_mock_stream_returns_sse_content_type() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/persona/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn persona_mock_stream_contains_persona_event_sequence() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/persona/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("persona.state"));
        assert!(text.contains("\"state\":\"idle\""));
        assert!(text.contains("\"state\":\"listening\""));
        assert!(text.contains("\"state\":\"thinking\""));
        assert!(text.contains("\"state\":\"tool_call\""));
        assert!(text.contains("\"state\":\"waiting_permission\""));
        assert!(text.contains("\"state\":\"success\""));
        assert!(text.contains("\"state\":\"error\""));
        assert!(text.contains("\"source\":\"mock\""));
    }

    #[tokio::test]
    async fn workflow_mock_stream_returns_sse_content_type() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/workflow/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn workflow_mock_stream_contains_workflow_event_sequence() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/workflow/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("workflow.started"));
        assert!(text.contains("workflow.step"));
        assert!(text.contains("workflow.test_result"));
        assert!(text.contains("workflow.patch_proposed"));
        assert!(text.contains("workflow.completed"));
        assert!(text.contains("\"step_kind\":\"open_project\""));
        assert!(text.contains("\"step_kind\":\"analyze_project\""));
        assert!(text.contains("\"step_kind\":\"run_tests\""));
        assert!(text.contains("\"step_kind\":\"apply_patch\""));
        assert!(text.contains("\"status\":\"waiting_permission\""));
        assert!(text.contains("\"command_preview\":\"cargo test --lib\""));
        assert!(text.contains("\"duration_ms\":1200"));
        assert!(text.contains("\"failed_test_names\":[\"tests::mock_failure\"]"));
        assert!(text.contains("\"risk_level\":\"medium\""));
    }

    #[tokio::test]
    async fn projects_mock_current_returns_mock_project_metadata() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/projects/mock/current")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["project_id"], "project_mock_001");
        assert_eq!(json["display_name"], "Nanami Mock Workspace");
        assert_eq!(json["project_path"], "/mock/project");
        assert_eq!(json["kind"], "rust");
        assert_eq!(json["trust_status"], "trusted_mock");
    }

    #[tokio::test]
    async fn workflow_mock_apply_patch_records_permission_and_returns_waiting_status() {
        let app = crate::router();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/workflow/mock/apply-patch")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"patch_id":"patch_mock_001"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["patch_id"], "patch_mock_001");
        assert_eq!(json["status"], "waiting_permission");
        assert_eq!(json["permission_id"], "perm_workflow_patch_patch_mock_001");

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/audit")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let audit_body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let audit_json: serde_json::Value = serde_json::from_slice(&audit_body).unwrap();

        assert!(audit_json["records"].as_array().unwrap().iter().any(
            |record| record["permission_id"] == "perm_workflow_patch_patch_mock_001"
                && record["action"] == "permission_requested"
                && record["permission_action"] == "filesystem.write"
        ));
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_returns_sse_content_type() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![EventEnvelope::new(
                "evt_001",
                chrono::Utc::now(),
                Event::TaskStarted(TaskStartedPayload {
                    session_id: None,
                    task_id: "task_openclaw_stream_001".into(),
                    title: "OpenClaw task".into(),
                    status: TaskStatus::Running,
                }),
            )]),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_contains_task_and_tool_events() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![
                EventEnvelope::new(
                    "evt_001",
                    chrono::Utc::now(),
                    Event::TaskStarted(TaskStartedPayload {
                        session_id: None,
                        task_id: "task_openclaw_stream_001".into(),
                        title: "OpenClaw task".into(),
                        status: TaskStatus::Running,
                    }),
                ),
                EventEnvelope::new(
                    "evt_002",
                    chrono::Utc::now(),
                    Event::ToolStarted(ToolStartedPayload {
                        task_id: "task_openclaw_stream_001".into(),
                        tool_call_id: "call_001".into(),
                        tool: "mock.shell".into(),
                        summary: Some("OpenClaw tool call detected".into()),
                    }),
                ),
                EventEnvelope::new(
                    "evt_003",
                    chrono::Utc::now(),
                    Event::ToolOutput(ToolOutputPayload {
                        task_id: "task_openclaw_stream_001".into(),
                        tool_call_id: "call_001".into(),
                        stream: ToolOutputStream::Log,
                        content: "{\"command\":\"cargo check\"}".into(),
                    }),
                ),
                EventEnvelope::new(
                    "evt_004",
                    chrono::Utc::now(),
                    Event::TaskCompleted(TaskCompletedPayload {
                        task_id: "task_openclaw_stream_001".into(),
                        status: TaskStatus::Completed,
                        summary: Some("OpenClaw stream completed".into()),
                    }),
                ),
            ]),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("task.started"));
        assert!(text.contains("tool.started"));
        assert!(text.contains("tool.output"));
        assert!(text.contains("task.completed"));
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_contains_sandbox_events() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![
                EventEnvelope::new(
                    "evt_sandbox_started_001",
                    chrono::Utc::now(),
                    Event::SandboxStarted(nanami_protocol::SandboxStartedPayload {
                        sandbox_id: "sandbox_001".into(),
                        task_id: "task_openclaw_stream_001".into(),
                        template_id: "rust-workspace".into(),
                        status: nanami_protocol::SandboxStatus::Starting,
                        network_policy: nanami_protocol::SandboxNetworkPolicy::Disabled,
                        mounts: vec![nanami_protocol::SandboxMountPayload {
                            host_path: "/mock/host/project".into(),
                            sandbox_path: "/workspace/project".into(),
                            mode: nanami_protocol::SandboxMountMode::ReadOnly,
                        }],
                    }),
                ),
                EventEnvelope::new(
                    "evt_sandbox_output_001",
                    chrono::Utc::now(),
                    Event::SandboxOutput(nanami_protocol::SandboxOutputPayload {
                        task_id: "task_openclaw_stream_001".into(),
                        sandbox_id: "sandbox_001".into(),
                        stream: ToolOutputStream::Stdout,
                        content: "checking workspace...".into(),
                    }),
                ),
                EventEnvelope::new(
                    "evt_sandbox_artifact_001",
                    chrono::Utc::now(),
                    Event::SandboxArtifact(nanami_protocol::SandboxArtifactPayload {
                        sandbox_id: "sandbox_001".into(),
                        task_id: "task_openclaw_stream_001".into(),
                        name: "mock-report.txt".into(),
                        path: "/workspace/output/mock-report.txt".into(),
                        media_type: "text/plain".into(),
                        size_bytes: 128,
                    }),
                ),
                EventEnvelope::new(
                    "evt_sandbox_completed_001",
                    chrono::Utc::now(),
                    Event::SandboxCompleted(nanami_protocol::SandboxCompletedPayload {
                        sandbox_id: "sandbox_001".into(),
                        task_id: "task_openclaw_stream_001".into(),
                        status: nanami_protocol::SandboxStatus::Completed,
                        exit_code: Some(0),
                        summary: Some("sandbox finished".into()),
                    }),
                ),
            ]),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("sandbox.started"));
        assert!(text.contains("sandbox.output"));
        assert!(text.contains("sandbox.artifact"));
        assert!(text.contains("sandbox.completed"));
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_inserts_permission_for_shell_tool() {
        let event = EventEnvelope::new(
            "evt_shell_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_openclaw_stream_001".into(),
                tool_call_id: "call_shell_001".into(),
                tool: "command.run".into(),
                summary: Some("cargo check".into()),
            }),
        );

        let permission = crate::maybe_permission_for_tool_event(&event).unwrap();
        let json = serde_json::to_value(permission).unwrap();

        assert_eq!(json["type"], "permission.requested");
        assert_eq!(json["level"], "l4");
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_inserts_shell_permission_once_and_records_single_audit() {
        let app = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![EventEnvelope::new(
                "evt_shell_started_001",
                chrono::Utc::now(),
                Event::ToolStarted(ToolStartedPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    tool_call_id: "call_shell_001".into(),
                    tool: "command.run".into(),
                    summary: Some("cargo check".into()),
                }),
            )]),
        }));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/openclaw/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message":"Run task"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(text.matches("permission.requested").count(), 1);

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/audit")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let requested_count = json["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|record| record["action"] == "permission_requested")
            .count();

        assert_eq!(requested_count, 1);
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_inserts_permission_for_read_file_tool() {
        let event = EventEnvelope::new(
            "evt_read_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_openclaw_stream_001".into(),
                tool_call_id: "call_read_001".into(),
                tool: "read_file".into(),
                summary: Some("/workspace/project/src/main.rs".into()),
            }),
        );

        let permission = crate::maybe_permission_for_tool_event(&event).unwrap();
        let json = serde_json::to_value(permission).unwrap();

        assert_eq!(json["type"], "permission.requested");
        assert_eq!(json["level"], "l2");
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_does_not_insert_permission_for_harmless_tool() {
        let event = EventEnvelope::new(
            "evt_harmless_started_001",
            chrono::Utc::now(),
            Event::ToolStarted(ToolStartedPayload {
                task_id: "task_openclaw_stream_001".into(),
                tool_call_id: "call_harmless_001".into(),
                tool: "display.message".into(),
                summary: Some("show info".into()),
            }),
        );

        assert!(crate::maybe_permission_for_tool_event(&event).is_none());
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_does_not_insert_permission_for_harmless_tool_route() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![EventEnvelope::new(
                "evt_harmless_started_001",
                chrono::Utc::now(),
                Event::ToolStarted(ToolStartedPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    tool_call_id: "call_harmless_001".into(),
                    tool: "display.message".into(),
                    summary: Some("show info".into()),
                }),
            )]),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(!text.contains("permission.requested"));
    }

    #[tokio::test]
    async fn tasks_openclaw_stream_unconfigured_gateway_returns_error_event() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Err(crate::chat_error(
                "OPENCLAW_GATEWAY_UNCONFIGURED",
                "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                Some("Set NANAMI_OPENCLAW_GATEWAY_URL before starting OpenClaw task streams"),
            )),
        }))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks/openclaw/stream")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"Run task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("error.occurred"));
        assert!(text.contains("OPENCLAW_GATEWAY_UNCONFIGURED"));
    }

    #[tokio::test]
    async fn permissions_mock_stream_returns_sse_content_type() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/permissions/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn permissions_mock_stream_contains_requested_event() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/permissions/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("permission.requested"));
        assert!(text.contains("perm_mock_read_project"));
    }

    #[tokio::test]
    async fn permissions_mock_stream_creates_audit_record() {
        let app = crate::router();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/mock/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/audit")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["records"][0]["action"], "permission_requested");
    }

    #[tokio::test]
    async fn permissions_resolve_accepts_allow_once() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"allow_once"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["type"], "permission.resolved");
        assert_eq!(json["decision"], "allow_once");
    }

    #[tokio::test]
    async fn permissions_decision_returns_allow_once_after_resolve() {
        let app = crate::router();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"allow_once"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let decision_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/decision/perm_mock_read_project")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["decision"], "allow_once");
    }

    #[tokio::test]
    async fn permissions_resolve_accepts_allow_for_task() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"allow_for_task"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["decision"], "allow_for_task");
    }

    #[tokio::test]
    async fn permissions_decision_returns_allow_for_task_after_resolve() {
        let app = crate::router();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"allow_for_task"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let decision_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/decision/perm_mock_read_project")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["decision"], "allow_for_task");
    }

    #[tokio::test]
    async fn permissions_resolve_accepts_deny() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"deny"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["decision"], "deny");
    }

    #[tokio::test]
    async fn permissions_decision_returns_deny_after_resolve() {
        let app = crate::router();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"deny"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let decision_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/decision/perm_mock_read_project")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["decision"], "deny");
    }

    #[tokio::test]
    async fn dangerous_stream_permission_can_be_resolved_and_audit_includes_requested_and_resolved()
    {
        let app = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![EventEnvelope::new(
                "evt_shell_started_001",
                chrono::Utc::now(),
                Event::ToolStarted(ToolStartedPayload {
                    task_id: "task_openclaw_stream_001".into(),
                    tool_call_id: "call_shell_001".into(),
                    tool: "command.run".into(),
                    summary: Some("cargo check".into()),
                }),
            )]),
        }));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/openclaw/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message":"Run task"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let _body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let resolve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_call_shell_001","decision":"allow_once"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_response.status(), StatusCode::OK);

        let decision_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/decision/perm_call_shell_001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let decision_body = axum::body::to_bytes(decision_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let decision_json: serde_json::Value = serde_json::from_slice(&decision_body).unwrap();

        assert_eq!(decision_json["decision"], "allow_once");

        let audit_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/permissions/audit")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let audit_body = axum::body::to_bytes(audit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let audit_json: serde_json::Value = serde_json::from_slice(&audit_body).unwrap();
        let actions: Vec<_> = audit_json["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["action"].as_str().unwrap())
            .collect();

        assert!(actions.contains(&"permission_requested"));
        assert!(actions.contains(&"permission_resolved"));
    }

    #[tokio::test]
    async fn permissions_resolve_rejects_invalid_decision() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"permission_id":"perm_mock_read_project","decision":"invalid"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
