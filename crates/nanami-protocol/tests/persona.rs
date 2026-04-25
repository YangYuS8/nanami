use chrono::DateTime;
use nanami_protocol::{
    Event, EventEnvelope, PersonaEmotion, PersonaState, PersonaStatePayload, PersonaStateSource,
};

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc()
}

#[test]
fn persona_state_source_mock_serializes_snake_case() {
    let json = serde_json::to_value(PersonaStateSource::Mock).unwrap();

    assert_eq!(json, "mock");
}

#[test]
fn persona_state_source_openclaw_serializes_snake_case() {
    let json = serde_json::to_value(PersonaStateSource::OpenClaw).unwrap();

    assert_eq!(json, "openclaw");
}

#[test]
fn persona_state_event_serializes_json_shape() {
    let event = EventEnvelope::new(
        "evt_persona_mock_001",
        timestamp(),
        Event::PersonaState(PersonaStatePayload {
            state: PersonaState::Thinking,
            emotion: PersonaEmotion::Focused,
            text: "Analyzing task".into(),
            source: PersonaStateSource::Mock,
        }),
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "persona.state");
    assert_eq!(json["state"], "thinking");
    assert_eq!(json["emotion"], "focused");
    assert_eq!(json["text"], "Analyzing task");
    assert_eq!(json["source"], "mock");
}
