use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{body_json, router};

#[tokio::test]
async fn openclaw_status_unconfigured_returns_disconnected() {
    let status = crate::openclaw::openclaw_status_from_config(Some("".into())).await;

    assert_eq!(
        status.status,
        nanami_protocol::OpenClawConnectionStatus::Disconnected
    );
    assert_eq!(status.gateway_url, "");
    assert!(status.message.is_some());
}

#[tokio::test]
async fn openclaw_status_endpoint_returns_ok() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/openclaw/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;

    assert!(json.get("status").is_some());
    assert!(json.get("gateway_url").is_some());
}
