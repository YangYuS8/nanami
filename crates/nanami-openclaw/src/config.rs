#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawConfig {
    pub gateway_url: String,
    pub token: Option<String>,
    pub timeout_ms: u64,
    pub chat_path: String,
}

impl OpenClawConfig {
    pub fn with_default_chat_path(
        gateway_url: String,
        token: Option<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            gateway_url,
            token,
            timeout_ms,
            chat_path: "/chat".into(),
        }
    }
}
