use chrono::DateTime;
use nanami_protocol::{Event, EventEnvelope, SessionStatus};

#[test]
fn session_started_serializes_expected_json_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_001",
        timestamp,
        Event::SessionStarted {
            session_id: "sess_001".into(),
            title: "Default Session".into(),
        },
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "session.started");
    assert_eq!(json["id"], "evt_001");
    assert_eq!(json["session_id"], "sess_001");
    assert_eq!(json["title"], "Default Session");
}

#[test]
fn session_updated_serializes_expected_json_shape() {
    let timestamp = DateTime::parse_from_rfc3339("2026-01-01T00:00:01Z")
        .unwrap()
        .to_utc();
    let event = EventEnvelope::new(
        "evt_002",
        timestamp,
        Event::SessionUpdated {
            session_id: "sess_001".into(),
            status: SessionStatus::Connected,
        },
    );

    let json = serde_json::to_value(event).unwrap();

    assert_eq!(json["type"], "session.updated");
    assert_eq!(json["status"], "connected");
}

#[test]
fn session_started_deserializes_from_json() {
    let json = r#"
        {
            "type": "session.started",
            "id": "evt_001",
            "timestamp": "2026-01-01T00:00:00Z",
            "session_id": "sess_001",
            "title": "Default Session"
        }
    "#;

    let event: EventEnvelope = serde_json::from_str(json).unwrap();

    assert_eq!(event.id, "evt_001");
    assert_eq!(
        event.event,
        Event::SessionStarted {
            session_id: "sess_001".into(),
            title: "Default Session".into(),
        }
    );
}
