use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{body_json, router};

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;

    assert_eq!(json["status"], "ok");
    assert_eq!(json["protocol_version"], nanami_protocol::PROTOCOL_VERSION);
}
