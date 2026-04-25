use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use nanami_openclaw::{OpenClawChatRequest, OpenClawClient, OpenClawConfig, OpenClawError};
use nanami_protocol::{
    ChatRequest, ChatResponse, ErrorPayload, ErrorSeverity, OpenClawConnectionStatus,
    OpenClawStatusPayload,
};
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const DEFAULT_OPENCLAW_TIMEOUT_MS: u64 = 3000;

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
        .with_state(AppState { openclaw })
}

#[derive(Clone)]
struct AppState {
    openclaw: Arc<dyn OpenClawService>,
}

trait OpenClawService: Send + Sync {
    fn send_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>>;
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
    use crate::OpenClawService;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use nanami_protocol::{ChatRequest, ChatResponse, ErrorPayload, OpenClawConnectionStatus};
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
    }

    impl OpenClawService for StubOpenClawService {
        fn send_chat_message(
            &self,
            _request: ChatRequest,
        ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>> {
            Box::pin(async { self.response.clone() })
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
}
