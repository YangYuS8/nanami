use std::time::Duration;

use nanami_protocol::{OpenClawConnectionStatus, OpenClawStatusPayload};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenClawConfig {
    pub gateway_url: String,
    pub token: Option<String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct OpenClawClient {
    config: OpenClawConfig,
    http: reqwest::Client,
}

#[derive(Debug)]
pub enum OpenClawError {
    InvalidClient(reqwest::Error),
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

fn error_without_secret(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "OpenClaw Gateway request timed out".into()
    } else if error.is_connect() {
        "OpenClaw Gateway is unreachable".into()
    } else {
        "OpenClaw Gateway request failed".into()
    }
}
