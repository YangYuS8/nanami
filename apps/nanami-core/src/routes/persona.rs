use axum::response::{
    IntoResponse, Response,
    sse::{Event as SseEvent, KeepAlive, Sse},
};
use nanami_protocol::{
    Event, EventEnvelope, PersonaEmotion, PersonaState, PersonaStatePayload, PersonaStateSource,
};
use std::convert::Infallible;

pub(crate) async fn persona_mock_stream() -> Response {
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
