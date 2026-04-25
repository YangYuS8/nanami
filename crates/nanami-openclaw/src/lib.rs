use std::time::Duration;

use nanami_protocol::{OpenClawConnectionStatus, OpenClawStatusPayload};
use serde_json::Value;

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
            // 0.2b placeholder: OpenClaw Gateway chat endpoint is not stable yet.
            // Keep this default centralized in the adapter so UI/core do not depend on it.
            chat_path: "/chat".into(),
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct OpenClawClient {
    config: OpenClawConfig,
    http: reqwest::Client,
}

#[derive(Debug)]
pub enum OpenClawError {
    InvalidClient(reqwest::Error),
    AuthFailed,
    Disconnected,
    InvalidResponse,
    UnexpectedStatus(u16),
}

impl std::fmt::Display for OpenClawError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidClient(_) => write!(formatter, "OpenClaw client configuration failed"),
            Self::AuthFailed => write!(formatter, "OpenClaw Gateway authentication failed"),
            Self::Disconnected => write!(formatter, "OpenClaw Gateway is unreachable"),
            Self::InvalidResponse => write!(
                formatter,
                "OpenClaw Gateway returned an unsupported response"
            ),
            Self::UnexpectedStatus(status) => {
                write!(formatter, "OpenClaw Gateway returned HTTP {status}")
            }
        }
    }
}

impl std::error::Error for OpenClawError {}

impl OpenClawClient {
    pub fn new(config: OpenClawConfig) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("reqwest client configuration should be valid");

        Self { config, http }
    }

    pub async fn check_status(&self) -> Result<OpenClawStatusPayload, OpenClawError> {
        if self.config.gateway_url.trim().is_empty() {
            return Ok(self.payload(
                OpenClawConnectionStatus::Disconnected,
                Some("OpenClaw Gateway URL is not configured"),
            ));
        }

        // 0.2 placeholder: OpenClaw Gateway health endpoint is not stable yet.
        // Use the configured gateway URL as the conservative reachability probe.
        let mut request = self.http.get(&self.config.gateway_url);
        if let Some(token) = &self.config.token {
            request = request.bearer_auth(token);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Ok(self.payload(
                    OpenClawConnectionStatus::Disconnected,
                    Some("OpenClaw Gateway is unreachable"),
                ));
            }
            Err(error) => {
                return Ok(self.payload(
                    OpenClawConnectionStatus::Error,
                    Some(&error_without_secret(&error)),
                ));
            }
        };

        let status_code = response.status();
        let body = response.text().await.unwrap_or_default().to_lowercase();

        if body.contains("pairing_required") || body.contains("pairing required") {
            return Ok(self.payload(
                OpenClawConnectionStatus::PairingRequired,
                Some("OpenClaw Gateway requires pairing"),
            ));
        }

        if body.contains("scope_missing") || body.contains("scope missing") {
            return Ok(self.payload(
                OpenClawConnectionStatus::ScopeMissing,
                Some("OpenClaw Gateway reports missing scope"),
            ));
        }

        if status_code.is_success() {
            return Ok(self.payload(OpenClawConnectionStatus::Connected, None));
        }

        if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
            return Ok(self.payload(
                OpenClawConnectionStatus::AuthFailed,
                Some("OpenClaw Gateway authentication failed"),
            ));
        }

        Ok(self.payload(
            OpenClawConnectionStatus::Error,
            Some("OpenClaw Gateway returned an unexpected status"),
        ))
    }

    pub async fn send_chat_message(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawChatResponse, OpenClawError> {
        let url = format!(
            "{}{}",
            self.config.gateway_url.trim_end_matches('/'),
            normalized_path(&self.config.chat_path)
        );
        let body = serde_json::json!({
            "message": request.message,
            "session_id": request.session_id,
        });

        let mut http_request = self.http.post(url).json(&body);
        if let Some(token) = &self.config.token {
            http_request = http_request.bearer_auth(token);
        }

        let response = match http_request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Err(OpenClawError::Disconnected);
            }
            Err(_) => return Err(OpenClawError::InvalidResponse),
        };

        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(OpenClawError::AuthFailed);
        }
        if !status.is_success() {
            return Err(OpenClawError::UnexpectedStatus(status.as_u16()));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|_| OpenClawError::InvalidResponse)?;
        let content = extract_content(&json).ok_or(OpenClawError::InvalidResponse)?;

        Ok(OpenClawChatResponse {
            content,
            session_id: json
                .get("session_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message_id: json
                .get("message_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }

    fn payload(
        &self,
        status: OpenClawConnectionStatus,
        message: Option<&str>,
    ) -> OpenClawStatusPayload {
        OpenClawStatusPayload {
            status,
            gateway_url: self.config.gateway_url.clone(),
            message: message.map(str::to_owned),
            agent: None,
            profile: None,
        }
    }
}

fn normalized_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

fn extract_content(json: &Value) -> Option<String> {
    json.get("content")
        .and_then(Value::as_str)
        .or_else(|| {
            json.pointer("/choices/0/message/content")
                .and_then(Value::as_str)
        })
        .or_else(|| {
            json.pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
        })
        .map(str::to_owned)
}

fn error_without_secret(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "OpenClaw Gateway request timed out".into()
    } else if error.is_connect() {
        "OpenClaw Gateway is unreachable".into()
    } else {
        "OpenClaw Gateway request failed".into()
    }
}
