use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    ToolCall,
    WaitingPermission,
    Success,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaEmotion {
    Neutral,
    Happy,
    Focused,
    Worried,
    Surprised,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersonaStateSource {
    Mock,
    Ui,
    System,
    #[serde(rename = "openclaw")]
    OpenClaw,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PersonaStatePayload {
    pub state: PersonaState,
    pub emotion: PersonaEmotion,
    pub text: String,
    pub source: PersonaStateSource,
}
