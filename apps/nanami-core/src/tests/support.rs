use crate::openclaw::OpenClawService;
use crate::state::NanamiEventStream;
use axum::Router;
use axum::body::Body;
use axum::http::Request;
use nanami_openclaw::OpenClawChatStream;
use nanami_protocol::{ChatRequest, ChatResponse, ChatStreamEvent, ErrorPayload, EventEnvelope};
use serde_json::Value;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use tower::ServiceExt;

#[derive(Clone)]
pub(crate) struct StubOpenClawService {
    pub(crate) response: Result<ChatResponse, ErrorPayload>,
    pub(crate) stream_response: Result<Vec<ChatStreamEvent>, ErrorPayload>,
    pub(crate) agent_stream_response: Result<Vec<EventEnvelope>, ErrorPayload>,
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
    ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            self.stream_response.clone().map(|events| {
                Box::pin(tokio_stream::iter(events.into_iter().map(Ok))) as OpenClawChatStream
            })
        })
    }

    fn stream_agent_events(
        &self,
        _request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            self.agent_stream_response.clone().map(|events| {
                Box::pin(tokio_stream::iter(events.into_iter().map(Ok))) as NanamiEventStream
            })
        })
    }
}

pub(crate) fn router() -> Router {
    crate::router()
}

pub(crate) fn router_with_openclaw(service: StubOpenClawService) -> Router {
    crate::router_with_openclaw(Arc::new(service))
}

pub(crate) async fn body_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub(crate) async fn body_text(response: axum::response::Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

pub(crate) fn temp_project_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ))
}

pub(crate) async fn select_project(app: &Router, path: &Path) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/select")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"project_path":"{}"}}"#,
                    path.display()
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    body_json(response).await
}

pub(crate) async fn select_and_trust_project(app: &Router, path: &Path) -> String {
    let select_json = select_project(app, path).await;
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
    assert_eq!(trust_response.status(), axum::http::StatusCode::OK);

    project_id
}

pub(crate) async fn request_manifest_preview_permission(app: &Router) {
    let response = app
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
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

pub(crate) async fn resolve_permission(app: &Router, permission_id: &str, decision: &str) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/permissions/resolve")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"permission_id":"{}","decision":"{}"}}"#,
                    permission_id, decision
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
