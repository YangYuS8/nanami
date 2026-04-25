use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "0.1";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "session.started")]
    SessionStarted { session_id: String, title: String },
    #[serde(rename = "session.updated")]
    SessionUpdated {
        session_id: String,
        status: SessionStatus,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EventEnvelope {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub event: Event,
}

impl EventEnvelope {
    pub fn new(id: impl Into<String>, timestamp: DateTime<Utc>, event: Event) -> Self {
        Self {
            id: id.into(),
            timestamp,
            event,
        }
    }
}
