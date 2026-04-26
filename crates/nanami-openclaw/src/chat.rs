#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawChatResponse {
    pub content: String,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
}
