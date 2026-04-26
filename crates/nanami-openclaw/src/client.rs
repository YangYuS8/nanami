use std::time::Duration;

use async_stream::try_stream;
use serde_json::Value;
use tokio_stream::{StreamExt, iter};

use crate::agent::{ensure_completed_item, parse_agent_frame, parse_agent_sse_events};
use crate::chat::{OpenClawChatRequest, OpenClawChatResponse};
use crate::config::OpenClawConfig;
use crate::error::{OpenClawError, error_without_secret};
use crate::sse::{
    OpenClawAgentStream, OpenClawChatStream, json_completion_stream, normalized_path,
    parse_sse_events, parse_sse_frame,
};
use crate::state::ToolEventMappingState;
use crate::status::payload;

#[derive(Debug, Clone)]
pub struct OpenClawClient {
    config: OpenClawConfig,
    http: reqwest::Client,
}

impl OpenClawClient {
    pub fn new(config: OpenClawConfig) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("reqwest client configuration should be valid");

        Self { config, http }
    }

    pub async fn check_status(
        &self,
    ) -> Result<nanami_protocol::OpenClawStatusPayload, OpenClawError> {
        if self.config.gateway_url.trim().is_empty() {
            return Ok(payload(
                &self.config,
                nanami_protocol::OpenClawConnectionStatus::Disconnected,
                Some("OpenClaw Gateway URL is not configured"),
            ));
        }

        let mut request = self.http.get(&self.config.gateway_url);
        if let Some(token) = &self.config.token {
            request = request.bearer_auth(token);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) if error.is_timeout() || error.is_connect() => {
                return Ok(payload(
                    &self.config,
                    nanami_protocol::OpenClawConnectionStatus::Disconnected,
                    Some("OpenClaw Gateway is unreachable"),
                ));
            }
            Err(error) => {
                return Ok(payload(
                    &self.config,
                    nanami_protocol::OpenClawConnectionStatus::Error,
                    Some(&error_without_secret(&error)),
                ));
            }
        };

        let status_code = response.status();
        let body = response.text().await.unwrap_or_default().to_lowercase();

        if body.contains("pairing_required") || body.contains("pairing required") {
            return Ok(payload(
                &self.config,
                nanami_protocol::OpenClawConnectionStatus::PairingRequired,
                Some("OpenClaw Gateway requires pairing"),
            ));
        }

        if body.contains("scope_missing") || body.contains("scope missing") {
            return Ok(payload(
                &self.config,
                nanami_protocol::OpenClawConnectionStatus::ScopeMissing,
                Some("OpenClaw Gateway reports missing scope"),
            ));
        }

        if status_code.is_success() {
            return Ok(payload(
                &self.config,
                nanami_protocol::OpenClawConnectionStatus::Connected,
                None,
            ));
        }

        if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
            return Ok(payload(
                &self.config,
                nanami_protocol::OpenClawConnectionStatus::AuthFailed,
                Some("OpenClaw Gateway authentication failed"),
            ));
        }

        Ok(payload(
            &self.config,
            nanami_protocol::OpenClawConnectionStatus::Error,
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
        let content = crate::sse::extract_content(&json).ok_or(OpenClawError::InvalidResponse)?;

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

    pub async fn stream_chat_message(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawChatStream, OpenClawError> {
        let url = format!(
            "{}{}",
            self.config.gateway_url.trim_end_matches('/'),
            normalized_path(&self.config.chat_path)
        );
        let body = serde_json::json!({
            "message": request.message,
            "session_id": request.session_id,
            "stream": true,
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

        let header_is_sse = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("text/event-stream"));
        if !header_is_sse {
            let text = response
                .text()
                .await
                .map_err(|_| OpenClawError::InvalidResponse)?;
            if text.trim_start().starts_with("data:") {
                let events = parse_sse_events(&text)?;
                return Ok(Box::pin(iter(events)));
            }

            let json: Value =
                serde_json::from_str(&text).map_err(|_| OpenClawError::InvalidResponse)?;
            return json_completion_stream(json);
        }

        let stream = try_stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut accumulated = String::new();
            let mut completed = false;

            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk.map_err(|_| OpenClawError::InvalidResponse)?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(separator) = buffer.find("\n\n") {
                    let frame = buffer[..separator].to_owned();
                    buffer.drain(..separator + 2);
                    if let Some(event) = parse_sse_frame(&frame, &mut accumulated, &mut completed)? {
                        yield event;
                    }
                }
            }

            if !buffer.trim().is_empty() {
                let pending_event = parse_sse_frame(&buffer, &mut accumulated, &mut completed)?;
                if let Some(event) = pending_event {
                    yield event;
                }
            }

            if !completed {
                yield nanami_protocol::ChatStreamEvent {
                    kind: nanami_protocol::ChatStreamEventKind::MessageCompleted,
                    session_id: None,
                    message_id: None,
                    delta: None,
                    content: Some(accumulated),
                    error: None,
                };
            }
        };

        Ok(Box::pin(stream))
    }

    pub async fn stream_agent_events(
        &self,
        request: OpenClawChatRequest,
    ) -> Result<OpenClawAgentStream, OpenClawError> {
        let url = format!(
            "{}{}",
            self.config.gateway_url.trim_end_matches('/'),
            normalized_path(&self.config.chat_path)
        );
        let body = serde_json::json!({
            "message": request.message,
            "session_id": request.session_id,
            "stream": true,
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

        let header_is_sse = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("text/event-stream"));

        if !header_is_sse {
            let text = response
                .text()
                .await
                .map_err(|_| OpenClawError::InvalidResponse)?;
            let items = if text.trim_start().starts_with("data:") {
                parse_agent_sse_events(&text)?
            } else {
                Vec::new()
            };

            return Ok(Box::pin(iter(items)));
        }

        let stream = try_stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut state = ToolEventMappingState::default();

            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk.map_err(|_| OpenClawError::InvalidResponse)?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(separator) = buffer.find("\n\n") {
                    let frame = buffer[..separator].to_owned();
                    buffer.drain(..separator + 2);
                    let items = parse_agent_frame(&frame, &mut state)?;
                    for item in items {
                        yield item;
                    }
                }
            }

            if !buffer.trim().is_empty() {
                let items = parse_agent_frame(&buffer, &mut state)?;
                for item in items {
                    yield item;
                }
            }

            if let Some(item) = ensure_completed_item(&state) {
                yield item?;
            }
        };

        Ok(Box::pin(stream))
    }
}
