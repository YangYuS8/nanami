use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{body_text, router};

#[tokio::test]
async fn sandbox_mock_stream_returns_sse_content_type() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/sandbox/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_text(response).await;

    assert!(text.contains("sandbox.started"));
    assert!(text.contains("sandbox.updated"));
    assert!(text.contains("sandbox.output"));
    assert!(text.contains("sandbox.artifact"));
    assert!(text.contains("sandbox.completed"));
}
