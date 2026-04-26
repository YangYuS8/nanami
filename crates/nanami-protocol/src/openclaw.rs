use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenClawConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    PairingRequired,
    AuthFailed,
    ScopeMissing,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OpenClawStatusPayload {
    pub status: OpenClawConnectionStatus,
    pub gateway_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}
