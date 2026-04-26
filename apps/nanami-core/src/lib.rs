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
    Event, EventEnvelope, ManifestPreview, ManifestSummary, OpenClawConnectionStatus,
    OpenClawStatusPayload, PermissionAuditLogResponse, PermissionDecision,
    PermissionDecisionStatus, PermissionLevel, PermissionRequestPayload, PermissionResolvedPayload,
    PermissionScope, PersonaEmotion, PersonaState, PersonaStatePayload, PersonaStateSource,
    ProjectKind, ProjectMetadata, ProjectStructureEntry, ProjectStructureEntryType,
    ProjectStructureMarker, ProjectStructureSummary, ProjectTrustStatus, TaskCompletedPayload,
    TaskStartedPayload, TaskStatus, ToolCallStatus, ToolCompletedPayload, ToolOutputPayload,
    ToolOutputStream, ToolStartedPayload, WorkflowChangeType, WorkflowCompletedPayload,
    WorkflowPatchFilePreviewPayload, WorkflowPatchProposedPayload, WorkflowPatchRiskLevel,
    WorkflowStartedPayload, WorkflowStatus, WorkflowStepKind, WorkflowStepPayload,
    WorkflowStepStatus, WorkflowTestResultPayload,
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
type JsonErrorResponse = (StatusCode, [(&'static str, &'static str); 1], String);
const MANIFEST_PREVIEW_MAX_BYTES: u64 = 8 * 1024;

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
            permission_manager: Arc::new(Mutex::new(PermissionManager::new())),
            selected_project: Arc::new(Mutex::new(None)),
        })
}

#[derive(Clone)]
struct AppState {
    openclaw: Arc<dyn OpenClawService>,
    permission_manager: Arc<Mutex<PermissionManager>>,
    selected_project: Arc<Mutex<Option<ProjectMetadata>>>,
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

#[derive(Debug, serde::Deserialize)]
struct ProjectSelectRequest {
    project_path: String,
}

#[derive(Debug, serde::Deserialize)]
struct ProjectTrustRequest {
    project_id: String,
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

async fn workflow_mock_current_project_stream(State(state): State<AppState>) -> Response {
    let selected_project = state.selected_project.lock().unwrap();
    let Some(project) = selected_project.as_ref() else {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_SELECTED",
                "No project is currently selected",
                Some("Select and trust a project before running a current-project workflow"),
            ))
            .unwrap(),
        )
            .into_response();
    };

    if project.trust_status != ProjectTrustStatus::SelectedTrusted {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_TRUSTED",
                "Current selected project must be selected_trusted",
                Some("Trust the selected project before running a current-project workflow"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    let structure = match build_project_structure_summary(project) {
        Ok(summary) => summary,
        Err(error) => return error.into_response(),
    };

    let events = vec![
        EventEnvelope::new(
            "evt_workflow_current_project_started_001",
            chrono::Utc::now(),
            Event::WorkflowStarted(WorkflowStartedPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                project_path: project.project_path.clone(),
                status: WorkflowStatus::Running,
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_open_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::OpenProject,
                status: WorkflowStepStatus::Completed,
                summary: format!(
                    "Selected project {} [{}] ({}, {})",
                    project.display_name,
                    project.project_id,
                    project_kind_label(&project.kind),
                    project_trust_status_label(&project.trust_status)
                ),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_analyze_project_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::AnalyzeProject,
                status: WorkflowStepStatus::Completed,
                summary: format!(
                    "Shallow structure summary includes {} top-level entries",
                    structure.entries.len()
                ),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_run_tests_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::RunTests,
                status: WorkflowStepStatus::Completed,
                summary: "Mock tests executed in current project context".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_test_result_001",
            chrono::Utc::now(),
            Event::WorkflowTestResult(WorkflowTestResultPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
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
            "evt_workflow_current_project_patch_proposed_001",
            chrono::Utc::now(),
            Event::WorkflowPatchProposed(WorkflowPatchProposedPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                patch_id: "patch_current_project_001".into(),
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
            "evt_workflow_current_project_apply_patch_001",
            chrono::Utc::now(),
            Event::WorkflowStep(WorkflowStepPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                step_kind: WorkflowStepKind::ApplyPatch,
                status: WorkflowStepStatus::WaitingPermission,
                summary: "Waiting for patch approval".into(),
            }),
        ),
        EventEnvelope::new(
            "evt_workflow_current_project_completed_001",
            chrono::Utc::now(),
            Event::WorkflowCompleted(WorkflowCompletedPayload {
                workflow_id: "workflow_current_project_001".into(),
                task_id: "task_workflow_current_project_001".into(),
                status: WorkflowStatus::Completed,
                summary: "Mock current-project workflow completed".into(),
            }),
        ),
    ];

    Sse::new(tokio_stream::iter(events.into_iter().map(|event| {
        Ok::<_, Infallible>(SseEvent::default().data(serde_json::to_string(&event).unwrap()))
    })))
    .keep_alive(KeepAlive::default())
    .into_response()
}

fn build_project_structure_summary(
    project: &ProjectMetadata,
) -> Result<ProjectStructureSummary, JsonErrorResponse> {
    let root = std::path::PathBuf::from(&project.project_path);
    let read_dir = match std::fs::read_dir(&root) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                [("content-type", "application/json")],
                serde_json::to_string(&chat_error(
                    "PROJECT_STRUCTURE_UNAVAILABLE",
                    "Unable to read the selected project directory",
                    Some("Select a valid project folder again"),
                ))
                .unwrap(),
            ));
        }
    };

    let mut entries = Vec::new();
    for entry in read_dir.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        let entry_type = if file_type.is_dir() {
            ProjectStructureEntryType::Directory
        } else {
            ProjectStructureEntryType::File
        };

        let marker = match file_name.as_str() {
            "Cargo.toml" | "package.json" | "pyproject.toml" => ProjectStructureMarker::Manifest,
            "src" | "app" | "crates" | "packages" => ProjectStructureMarker::SourceDir,
            ".gitignore" => ProjectStructureMarker::Config,
            "README.md" | "LICENSE" => ProjectStructureMarker::Other,
            _ => ProjectStructureMarker::Other,
        };

        entries.push(ProjectStructureEntry {
            name: file_name.clone(),
            relative_path: file_name,
            entry_type,
            marker,
        });
    }

    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(ProjectStructureSummary {
        project_id: project.project_id.clone(),
        project_path: project.project_path.clone(),
        entries,
    })
}

fn project_kind_label(kind: &ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Rust => "rust",
        ProjectKind::Node => "node",
        ProjectKind::Python => "python",
        ProjectKind::Unknown => "unknown",
    }
}

fn project_trust_status_label(status: &ProjectTrustStatus) -> &'static str {
    match status {
        ProjectTrustStatus::Untrusted => "untrusted",
        ProjectTrustStatus::TrustedMock => "trusted_mock",
        ProjectTrustStatus::SelectedUntrusted => "selected_untrusted",
        ProjectTrustStatus::SelectedTrusted => "selected_trusted",
    }
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

async fn projects_select(
    State(state): State<AppState>,
    Json(request): Json<ProjectSelectRequest>,
) -> impl IntoResponse {
    let project_path = std::path::PathBuf::from(&request.project_path);

    if !project_path.exists() || !project_path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_PATH_INVALID",
                "Selected project path must be an existing directory",
                Some("Choose an existing project folder"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    let kind = if project_path.join("Cargo.toml").is_file() {
        ProjectKind::Rust
    } else if project_path.join("package.json").is_file() {
        ProjectKind::Node
    } else if project_path.join("pyproject.toml").is_file() {
        ProjectKind::Python
    } else {
        ProjectKind::Unknown
    };

    let display_name = project_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("selected-project")
        .to_owned();

    let metadata = ProjectMetadata {
        project_id: format!("project_selected_{}", display_name),
        display_name,
        project_path: request.project_path,
        kind,
        trust_status: ProjectTrustStatus::SelectedUntrusted,
    };

    // Record the current explicitly selected project in memory only.
    // 0.8a/0.8b still do not grant any automatic read/write or execution.
    *state.selected_project.lock().unwrap() = Some(metadata.clone());

    Json(metadata).into_response()
}

async fn projects_trust(
    State(state): State<AppState>,
    Json(request): Json<ProjectTrustRequest>,
) -> impl IntoResponse {
    let mut selected_project = state.selected_project.lock().unwrap();
    let Some(project) = selected_project.as_mut() else {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_SELECTED",
                "No project is currently selected",
                Some("Select a project folder before trusting it"),
            ))
            .unwrap(),
        )
            .into_response();
    };

    if project.project_id != request.project_id {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_ID_MISMATCH",
                "The requested project does not match the current selected project",
                Some("Trust the currently selected project only"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    if project.trust_status != ProjectTrustStatus::SelectedUntrusted {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_TRUST_INVALID_STATE",
                "Only selected_untrusted projects can be trusted",
                Some("Select a project and trust it once"),
            ))
            .unwrap(),
        )
            .into_response();
    }

    project.trust_status = ProjectTrustStatus::SelectedTrusted;

    Json(project.clone()).into_response()
}

async fn projects_current_structure(State(state): State<AppState>) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "loading its structure") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    match build_project_structure_summary(&project) {
        Ok(summary) => Json(summary).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn projects_current_manifest_preview_request(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "requesting manifest preview") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    let manifest_path = match top_level_manifest_path(&project) {
        Ok(path) => path,
        Err(error) => return error.into_response(),
    };

    let permission_id = manifest_preview_permission_id(&project);
    let permission_request = PermissionRequestPayload {
        task_id: None,
        permission_id,
        level: PermissionLevel::L2,
        action: "filesystem.read".into(),
        target: manifest_path.display().to_string(),
        reason: "Read top-level manifest preview for the currently selected trusted project".into(),
        scope: PermissionScope::Task,
        expires: "task_completed".into(),
    };

    let mut manager = state.permission_manager.lock().unwrap();
    Json(manager.request_permission(permission_request)).into_response()
}

async fn projects_current_manifest_preview(State(state): State<AppState>) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "loading manifest preview") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    if let Err(error) = ensure_manifest_preview_permission(&state, &project) {
        return error.into_response();
    }

    match build_manifest_preview(&project) {
        Ok(preview) => Json(preview).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn projects_current_manifest_summary(State(state): State<AppState>) -> impl IntoResponse {
    let project = match selected_trusted_project(&state, "loading manifest summary") {
        Ok(project) => project,
        Err(error) => return error.into_response(),
    };

    if let Err(error) = ensure_manifest_preview_permission(&state, &project) {
        return error.into_response();
    }

    match build_manifest_summary(&project) {
        Ok(summary) => Json(summary).into_response(),
        Err(error) => error.into_response(),
    }
}

fn selected_trusted_project(
    state: &AppState,
    action: &'static str,
) -> Result<ProjectMetadata, JsonErrorResponse> {
    let selected_project = state.selected_project.lock().unwrap();
    let Some(project) = selected_project.as_ref() else {
        return Err((
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_SELECTED",
                "No project is currently selected",
                Some(match action {
                    "requesting manifest preview" => {
                        "Select and trust a project before requesting manifest preview"
                    }
                    "loading manifest preview" => {
                        "Select and trust a project before loading manifest preview"
                    }
                    "loading manifest summary" => {
                        "Select and trust a project before loading manifest summary"
                    }
                    _ => "Select and trust a project before loading its structure",
                }),
            ))
            .unwrap(),
        ));
    };

    if project.trust_status != ProjectTrustStatus::SelectedTrusted {
        return Err((
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_NOT_TRUSTED",
                "Current selected project must be selected_trusted",
                Some(match action {
                    "requesting manifest preview" => {
                        "Trust the selected project before requesting manifest preview"
                    }
                    "loading manifest preview" => {
                        "Trust the selected project before loading manifest preview"
                    }
                    "loading manifest summary" => {
                        "Trust the selected project before loading manifest summary"
                    }
                    _ => "Trust the selected project before loading its structure",
                }),
            ))
            .unwrap(),
        ));
    }

    Ok(project.clone())
}

fn top_level_manifest_path(
    project: &ProjectMetadata,
) -> Result<std::path::PathBuf, JsonErrorResponse> {
    let root = std::path::PathBuf::from(&project.project_path);
    let manifest_name = match project.kind {
        ProjectKind::Rust => "Cargo.toml",
        ProjectKind::Node => "package.json",
        ProjectKind::Python => "pyproject.toml",
        ProjectKind::Unknown => {
            return Err((
                StatusCode::BAD_REQUEST,
                [("content-type", "application/json")],
                serde_json::to_string(&chat_error(
                    "PROJECT_MANIFEST_UNAVAILABLE",
                    "No supported top-level manifest is available for the current project",
                    Some("Select a project with Cargo.toml, package.json, or pyproject.toml"),
                ))
                .unwrap(),
            ));
        }
    };

    let manifest_path = root.join(manifest_name);
    if !manifest_path.is_file() {
        return Err((
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            serde_json::to_string(&chat_error(
                "PROJECT_MANIFEST_UNAVAILABLE",
                "The supported top-level manifest file is not available",
                Some("Re-select the project folder to refresh top-level manifest detection"),
            ))
            .unwrap(),
        ));
    }

    Ok(manifest_path)
}

fn build_manifest_preview(project: &ProjectMetadata) -> Result<ManifestPreview, JsonErrorResponse> {
    let manifest_file = read_manifest_file(project)?;

    Ok(ManifestPreview {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        content_preview: manifest_file.content.clone(),
        truncated: manifest_file.truncated,
        size_bytes: manifest_file.size_bytes,
    })
}

#[derive(Debug, Clone)]
struct ManifestFile {
    manifest_path: std::path::PathBuf,
    content: String,
    truncated: bool,
    size_bytes: u64,
}

fn ensure_manifest_preview_permission(
    state: &AppState,
    project: &ProjectMetadata,
) -> Result<(), JsonErrorResponse> {
    let permission_id = manifest_preview_permission_id(project);
    let decision = {
        let manager = state.permission_manager.lock().unwrap();
        manager.decision_for(&permission_id)
    };

    if matches!(
        decision,
        Some(PermissionDecision::AllowOnce | PermissionDecision::AllowForTask)
    ) {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        [("content-type", "application/json")],
        serde_json::to_string(&chat_error(
            "MANIFEST_PREVIEW_PERMISSION_REQUIRED",
            "Manifest preview requires an approved filesystem.read permission",
            Some(
                "Request manifest preview permission and approve allow_once or allow_for_task first",
            ),
        ))
        .unwrap(),
    ))
}

fn read_manifest_file(project: &ProjectMetadata) -> Result<ManifestFile, JsonErrorResponse> {
    let manifest_path = top_level_manifest_path(project)?;
    let bytes = match std::fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                [("content-type", "application/json")],
                serde_json::to_string(&chat_error(
                    "MANIFEST_PREVIEW_UNAVAILABLE",
                    "Unable to read the selected top-level manifest file",
                    Some("Re-select the project folder and request manifest preview again"),
                ))
                .unwrap(),
            ));
        }
    };

    let size_bytes = bytes.len() as u64;
    let preview_bytes = if size_bytes > MANIFEST_PREVIEW_MAX_BYTES {
        &bytes[..MANIFEST_PREVIEW_MAX_BYTES as usize]
    } else {
        &bytes[..]
    };

    Ok(ManifestFile {
        manifest_path,
        content: String::from_utf8_lossy(preview_bytes).into_owned(),
        truncated: size_bytes > MANIFEST_PREVIEW_MAX_BYTES,
        size_bytes,
    })
}

fn build_manifest_summary(project: &ProjectMetadata) -> Result<ManifestSummary, JsonErrorResponse> {
    let manifest_file = read_manifest_file(project)?;
    Ok(match project.kind {
        ProjectKind::Rust => build_rust_manifest_summary(project, &manifest_file),
        ProjectKind::Node => build_node_manifest_summary(project, &manifest_file),
        ProjectKind::Python => build_python_manifest_summary(project, &manifest_file),
        ProjectKind::Unknown => build_unknown_manifest_summary(project, &manifest_file),
    })
}

fn build_rust_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    let parsed: Result<toml::Value, _> = toml::from_str(&manifest_file.content);
    let (package_name, package_version, dependency_count) = if let Ok(value) = parsed {
        let package = value.get("package").and_then(toml::Value::as_table);
        let package_name = package
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let package_version = package
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let dependency_count = value
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .map(|deps| deps.len() as u64);
        (package_name, package_version, dependency_count)
    } else {
        (None, None, None)
    };

    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: package_name.clone(),
        package_version: package_version.clone(),
        dependency_count,
        script_count: None,
        summary_text: summary_text_for_manifest(
            "Rust",
            package_name.as_deref(),
            package_version.as_deref(),
            dependency_count,
            None,
        ),
    }
}

fn build_node_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&manifest_file.content);
    let (package_name, package_version, dependency_count, script_count) = if let Ok(value) = parsed
    {
        let package_name = value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let package_version = value
            .get("version")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let dependencies = value
            .get("dependencies")
            .and_then(serde_json::Value::as_object)
            .map(|deps| deps.len() as u64)
            .unwrap_or(0);
        let dev_dependencies = value
            .get("devDependencies")
            .and_then(serde_json::Value::as_object)
            .map(|deps| deps.len() as u64)
            .unwrap_or(0);
        let scripts = value
            .get("scripts")
            .and_then(serde_json::Value::as_object)
            .map(|scripts| scripts.len() as u64);
        (
            package_name,
            package_version,
            Some(dependencies + dev_dependencies),
            scripts,
        )
    } else {
        (None, None, None, None)
    };

    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: package_name.clone(),
        package_version: package_version.clone(),
        dependency_count,
        script_count,
        summary_text: summary_text_for_manifest(
            "Node",
            package_name.as_deref(),
            package_version.as_deref(),
            dependency_count,
            script_count,
        ),
    }
}

fn build_python_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    let parsed: Result<toml::Value, _> = toml::from_str(&manifest_file.content);
    let (package_name, package_version, dependency_count) = if let Ok(value) = parsed {
        let project_table = value.get("project").and_then(toml::Value::as_table);
        let package_name = project_table
            .and_then(|project| project.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let package_version = project_table
            .and_then(|project| project.get("version"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let dependency_count = project_table
            .and_then(|project| project.get("dependencies"))
            .and_then(toml::Value::as_array)
            .map(|deps| deps.len() as u64);
        (package_name, package_version, dependency_count)
    } else {
        (None, None, None)
    };

    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: package_name.clone(),
        package_version: package_version.clone(),
        dependency_count,
        script_count: None,
        summary_text: summary_text_for_manifest(
            "Python",
            package_name.as_deref(),
            package_version.as_deref(),
            dependency_count,
            None,
        ),
    }
}

fn build_unknown_manifest_summary(
    project: &ProjectMetadata,
    manifest_file: &ManifestFile,
) -> ManifestSummary {
    ManifestSummary {
        project_id: project.project_id.clone(),
        manifest_path: manifest_file.manifest_path.display().to_string(),
        kind: project.kind.clone(),
        package_name: None,
        package_version: None,
        dependency_count: None,
        script_count: None,
        summary_text: "Manifest summary unavailable".into(),
    }
}

fn summary_text_for_manifest(
    ecosystem: &str,
    package_name: Option<&str>,
    package_version: Option<&str>,
    dependency_count: Option<u64>,
    script_count: Option<u64>,
) -> String {
    if package_name.is_none()
        && package_version.is_none()
        && dependency_count.is_none()
        && script_count.is_none()
    {
        return "Manifest summary unavailable".into();
    }

    let mut summary = format!("{} manifest", ecosystem);
    if let Some(name) = package_name {
        summary.push_str(&format!(" {}", name));
    }
    if let Some(version) = package_version {
        summary.push_str(&format!(" {}", version));
    }
    if let Some(count) = dependency_count {
        summary.push_str(&format!(" with {} dependencies", count));
    }
    if let Some(count) = script_count {
        summary.push_str(&format!(" and {} scripts", count));
    }
    summary
}

fn manifest_preview_permission_id(project: &ProjectMetadata) -> String {
    format!("perm_manifest_preview_{}", project.project_id)
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
    use crate::MANIFEST_PREVIEW_MAX_BYTES;
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
    async fn projects_select_detects_top_level_rust_manifest_and_returns_selected_untrusted() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_project_select_rust_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["kind"], "rust");
        assert_eq!(json["trust_status"], "selected_untrusted");
    }

    #[tokio::test]
    async fn projects_select_returns_unknown_when_no_top_level_manifest_exists() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_project_select_unknown_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(temp_dir.join("nested")).unwrap();
        std::fs::write(temp_dir.join("nested").join("Cargo.toml"), "").unwrap();

        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("nested").join("Cargo.toml"));
        let _ = std::fs::remove_dir(temp_dir.join("nested"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(json["kind"], "unknown");
        assert_eq!(json["trust_status"], "selected_untrusted");
    }

    #[tokio::test]
    async fn projects_trust_updates_selected_project_to_selected_trusted() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_project_trust_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = trust_response.status();
        let trust_body = axum::body::to_bytes(trust_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let trust_json: serde_json::Value = serde_json::from_slice(&trust_body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(trust_json["trust_status"], "selected_trusted");
    }

    #[tokio::test]
    async fn projects_trust_rejects_non_selected_project_id() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"project_id":"project_missing_001"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn projects_current_structure_requires_selected_trusted_project() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/structure")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn projects_current_structure_returns_shallow_summary_for_selected_trusted_project() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_project_structure_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(temp_dir.join("src")).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();
        std::fs::write(temp_dir.join("README.md"), "").unwrap();
        std::fs::write(temp_dir.join(".gitignore"), "").unwrap();
        std::fs::write(temp_dir.join("src").join("nested.rs"), "").unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let structure_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/structure")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = structure_response.status();
        let body = axum::body::to_bytes(structure_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(temp_dir.join("README.md"));
        let _ = std::fs::remove_file(temp_dir.join(".gitignore"));
        let _ = std::fs::remove_file(temp_dir.join("src").join("nested.rs"));
        let _ = std::fs::remove_dir(temp_dir.join("src"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["project_id"], project_id);
        assert!(
            json["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "Cargo.toml"
                    && entry["entry_type"] == "file"
                    && entry["marker"] == "manifest")
        );
        assert!(
            json["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "src"
                    && entry["entry_type"] == "directory"
                    && entry["marker"] == "source_dir")
        );
        assert!(
            json["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == ".gitignore" && entry["marker"] == "config")
        );
        assert!(
            json["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "README.md" && entry["marker"] == "other")
        );
        assert!(
            !json["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["relative_path"] == "src/nested.rs")
        );
    }

    #[tokio::test]
    async fn projects_current_manifest_preview_request_requires_selected_trusted_project() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn projects_current_manifest_preview_request_records_l2_permission_for_top_level_manifest()
     {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_preview_request_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let preview_request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = preview_request_response.status();
        let body = axum::body::to_bytes(preview_request_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["level"], "l2");
        assert_eq!(json["action"], "filesystem.read");
        assert_eq!(
            json["permission_id"],
            format!("perm_manifest_preview_{}", project_id)
        );
        assert_eq!(
            json["target"],
            temp_dir.join("Cargo.toml").display().to_string()
        );
    }

    #[tokio::test]
    async fn projects_current_manifest_preview_requires_permission_decision() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_preview_permission_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let preview_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/preview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(preview_response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn projects_current_manifest_preview_returns_top_level_preview_after_allow_once() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_preview_allow_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(temp_dir.join("nested")).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        std::fs::write(
            temp_dir.join("nested").join("Cargo.toml"),
            "[package]\nname = \"nested\"\n",
        )
        .unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();
        let permission_id = format!("perm_manifest_preview_{}", project_id);

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(request_response.status(), StatusCode::OK);

        let resolve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"permission_id":"{}","decision":"allow_once"}}"#,
                        permission_id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_response.status(), StatusCode::OK);

        let preview_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/preview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = preview_response.status();
        let body = axum::body::to_bytes(preview_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(temp_dir.join("nested").join("Cargo.toml"));
        let _ = std::fs::remove_dir(temp_dir.join("nested"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["project_id"], project_id);
        assert_eq!(json["kind"], "rust");
        assert_eq!(
            json["manifest_path"],
            temp_dir.join("Cargo.toml").display().to_string()
        );
        assert_eq!(json["content_preview"], "[package]\nname = \"demo\"\n");
        assert_eq!(json["truncated"], false);
        assert_eq!(json["size_bytes"], 24);
    }

    #[tokio::test]
    async fn projects_current_manifest_preview_truncates_to_8kb() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_preview_truncate_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let manifest_content = "a".repeat((MANIFEST_PREVIEW_MAX_BYTES as usize) + 17);
        std::fs::write(temp_dir.join("Cargo.toml"), &manifest_content).unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();
        let permission_id = format!("perm_manifest_preview_{}", project_id);

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(request_response.status(), StatusCode::OK);

        let resolve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"permission_id":"{}","decision":"allow_for_task"}}"#,
                        permission_id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_response.status(), StatusCode::OK);

        let preview_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/preview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(preview_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(json["truncated"], true);
        assert_eq!(json["size_bytes"], MANIFEST_PREVIEW_MAX_BYTES + 17);
        assert_eq!(
            json["content_preview"].as_str().unwrap().len(),
            MANIFEST_PREVIEW_MAX_BYTES as usize
        );
    }

    #[tokio::test]
    async fn projects_current_manifest_summary_requires_permission_decision() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_summary_permission_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let summary_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(summary_response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn projects_current_manifest_summary_extracts_rust_fields_after_allow_once() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_summary_rust_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(
            temp_dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\ntokio = \"1\"\n",
        )
        .unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();
        let permission_id = format!("perm_manifest_preview_{}", project_id);

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(request_response.status(), StatusCode::OK);

        let resolve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"permission_id":"{}","decision":"allow_once"}}"#,
                        permission_id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_response.status(), StatusCode::OK);

        let summary_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = summary_response.status();
        let body = axum::body::to_bytes(summary_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["kind"], "rust");
        assert_eq!(json["package_name"], "demo");
        assert_eq!(json["package_version"], "0.1.0");
        assert_eq!(json["dependency_count"], 2);
        assert!(json["script_count"].is_null());
    }

    #[tokio::test]
    async fn projects_current_manifest_summary_extracts_node_fields_with_scripts_and_dependencies()
    {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_summary_node_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(
            temp_dir.join("package.json"),
            r#"{"name":"demo-node","version":"1.2.3","dependencies":{"react":"18"},"devDependencies":{"vite":"5","typescript":"5"},"scripts":{"dev":"vite","build":"vite build"}}"#,
        )
        .unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();
        let permission_id = format!("perm_manifest_preview_{}", project_id);

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(request_response.status(), StatusCode::OK);

        let resolve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"permission_id":"{}","decision":"allow_for_task"}}"#,
                        permission_id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_response.status(), StatusCode::OK);

        let summary_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(summary_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("package.json"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(json["kind"], "node");
        assert_eq!(json["package_name"], "demo-node");
        assert_eq!(json["package_version"], "1.2.3");
        assert_eq!(json["dependency_count"], 3);
        assert_eq!(json["script_count"], 2);
    }

    #[tokio::test]
    async fn projects_current_manifest_summary_extracts_python_fields_and_tolerates_parse_failure()
    {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_manifest_summary_python_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(
            temp_dir.join("pyproject.toml"),
            "[project]\nname = \"demo-py\"\nversion = \"0.2.0\"\ndependencies = [\"fastapi\", \"uvicorn\"]\n",
        )
        .unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();
        let permission_id = format!("perm_manifest_preview_{}", project_id);

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let request_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/current/manifest/preview-request")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(request_response.status(), StatusCode::OK);

        let resolve_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/permissions/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"permission_id":"{}","decision":"allow_for_task"}}"#,
                        permission_id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resolve_response.status(), StatusCode::OK);

        let summary_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = summary_response.status();
        let body = axum::body::to_bytes(summary_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        std::fs::write(temp_dir.join("pyproject.toml"), "not valid toml = [").unwrap();

        let fallback_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/projects/current/manifest/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let fallback_body = axum::body::to_bytes(fallback_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let fallback_json: serde_json::Value = serde_json::from_slice(&fallback_body).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("pyproject.toml"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["kind"], "python");
        assert_eq!(json["package_name"], "demo-py");
        assert_eq!(json["package_version"], "0.2.0");
        assert_eq!(json["dependency_count"], 2);
        assert!(json["script_count"].is_null());

        assert_eq!(fallback_json["kind"], "python");
        assert!(fallback_json["package_name"].is_null());
        assert!(fallback_json["package_version"].is_null());
        assert!(fallback_json["dependency_count"].is_null());
        assert_eq!(
            fallback_json["summary_text"],
            "Manifest summary unavailable"
        );
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
    async fn workflow_mock_current_project_stream_requires_selected_trusted_project() {
        let response = crate::router()
            .oneshot(
                Request::builder()
                    .uri("/workflow/mock/current-project/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn workflow_mock_current_project_stream_uses_selected_project_metadata_and_structure_count()
     {
        let temp_dir = std::env::temp_dir().join(format!(
            "nanami_workflow_current_project_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(temp_dir.join("src")).unwrap();
        std::fs::create_dir_all(temp_dir.join("crates")).unwrap();
        std::fs::write(temp_dir.join("Cargo.toml"), "").unwrap();
        std::fs::write(temp_dir.join("README.md"), "").unwrap();

        let app = crate::router();
        let select_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/select")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"project_path":"{}"}}"#,
                        temp_dir.display()
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        let select_body = axum::body::to_bytes(select_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let select_json: serde_json::Value = serde_json::from_slice(&select_body).unwrap();
        let project_id = select_json["project_id"].as_str().unwrap().to_owned();

        let trust_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/projects/trust")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"project_id":"{}"}}"#, project_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(trust_response.status(), StatusCode::OK);

        let workflow_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/workflow/mock/current-project/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = workflow_response.status();
        let body = axum::body::to_bytes(workflow_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        let _ = std::fs::remove_file(temp_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(temp_dir.join("README.md"));
        let _ = std::fs::remove_dir(temp_dir.join("src"));
        let _ = std::fs::remove_dir(temp_dir.join("crates"));
        let _ = std::fs::remove_dir(&temp_dir);

        assert_eq!(status, StatusCode::OK);
        assert!(text.contains("workflow.started"));
        assert!(text.contains(&project_id));
        assert!(text.contains(&temp_dir.display().to_string()));
        assert!(text.contains("selected_trusted"));
        assert!(text.contains("rust"));
        assert!(text.contains("4 top-level entries"));
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
    async fn tasks_openclaw_stream_contains_workflow_events() {
        let response = crate::router_with_openclaw(Arc::new(StubOpenClawService {
            response: Err(crate::chat_error("unused", "unused", None)),
            stream_response: Ok(Vec::new()),
            agent_stream_response: Ok(vec![
                EventEnvelope::new(
                    "evt_workflow_started_001",
                    chrono::Utc::now(),
                    Event::WorkflowStarted(nanami_protocol::WorkflowStartedPayload {
                        workflow_id: "workflow_mock_001".into(),
                        task_id: "task_workflow_mock_001".into(),
                        project_path: "/mock/project".into(),
                        status: nanami_protocol::WorkflowStatus::Running,
                    }),
                ),
                EventEnvelope::new(
                    "evt_workflow_step_001",
                    chrono::Utc::now(),
                    Event::WorkflowStep(nanami_protocol::WorkflowStepPayload {
                        workflow_id: "workflow_mock_001".into(),
                        task_id: "task_workflow_mock_001".into(),
                        step_kind: nanami_protocol::WorkflowStepKind::AnalyzeProject,
                        status: nanami_protocol::WorkflowStepStatus::Completed,
                        summary: "Mock analysis finished".into(),
                    }),
                ),
                EventEnvelope::new(
                    "evt_workflow_test_result_001",
                    chrono::Utc::now(),
                    Event::WorkflowTestResult(nanami_protocol::WorkflowTestResultPayload {
                        workflow_id: "workflow_mock_001".into(),
                        task_id: "task_workflow_mock_001".into(),
                        status: nanami_protocol::WorkflowStatus::Completed,
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
                    Event::WorkflowPatchProposed(nanami_protocol::WorkflowPatchProposedPayload {
                        workflow_id: "workflow_mock_001".into(),
                        task_id: "task_workflow_mock_001".into(),
                        patch_id: "patch_mock_001".into(),
                        summary: "Mock patch proposal ready".into(),
                        diff_summary: "1 file modified".into(),
                        risk_level: nanami_protocol::WorkflowPatchRiskLevel::Medium,
                        files: vec![nanami_protocol::WorkflowPatchFilePreviewPayload {
                            path: "src/main.rs".into(),
                            change_type: nanami_protocol::WorkflowChangeType::Modified,
                            diff_preview: "- old line\n+ new line".into(),
                        }],
                    }),
                ),
                EventEnvelope::new(
                    "evt_workflow_completed_001",
                    chrono::Utc::now(),
                    Event::WorkflowCompleted(nanami_protocol::WorkflowCompletedPayload {
                        workflow_id: "workflow_mock_001".into(),
                        task_id: "task_workflow_mock_001".into(),
                        status: nanami_protocol::WorkflowStatus::Completed,
                        summary: "Mock workflow completed".into(),
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

        assert!(text.contains("workflow.started"));
        assert!(text.contains("workflow.step"));
        assert!(text.contains("workflow.test_result"));
        assert!(text.contains("workflow.patch_proposed"));
        assert!(text.contains("workflow.completed"));
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
