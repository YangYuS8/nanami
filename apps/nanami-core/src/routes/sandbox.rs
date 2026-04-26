use axum::response::{
    IntoResponse, Response,
    sse::{Event as SseEvent, KeepAlive, Sse},
};
use std::convert::Infallible;

pub(crate) async fn sandbox_mock_stream() -> Response {
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
