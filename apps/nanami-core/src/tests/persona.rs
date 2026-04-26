use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{body_text, router};

#[tokio::test]
async fn persona_mock_stream_returns_sse_content_type() {
    let response = router()
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
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/persona/mock/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_text(response).await;

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
