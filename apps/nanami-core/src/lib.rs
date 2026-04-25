use axum::{Json, Router, routing::get};
use nanami_openclaw::{OpenClawClient, OpenClawConfig};
use nanami_protocol::{OpenClawConnectionStatus, OpenClawStatusPayload};
use serde::Serialize;

const DEFAULT_OPENCLAW_TIMEOUT_MS: u64 = 3000;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    protocol_version: &'static str,
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/openclaw/status", get(openclaw_status))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        protocol_version: nanami_protocol::PROTOCOL_VERSION,
    })
}

async fn openclaw_status() -> Json<nanami_protocol::OpenClawStatusPayload> {
    Json(crate::openclaw_status_from_config(None).await)
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

    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url,
        token: std::env::var("NANAMI_OPENCLAW_TOKEN").ok(),
        timeout_ms: DEFAULT_OPENCLAW_TIMEOUT_MS,
    });

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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use nanami_protocol::OpenClawConnectionStatus;
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
}
