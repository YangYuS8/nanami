use futures_util::StreamExt as FuturesStreamExt;
use nanami_openclaw::{
    OpenClawChatRequest, OpenClawChatStream, OpenClawClient, OpenClawConfig, OpenClawError,
    OpenClawStreamItem,
};
use nanami_protocol::{
    ChatRequest, ChatResponse, ErrorPayload, EventEnvelope, OpenClawConnectionStatus,
    OpenClawStatusPayload,
};
use std::future::Future;
use std::pin::Pin;

use crate::error::chat_error;
use crate::state::{DEFAULT_OPENCLAW_TIMEOUT_MS, NanamiEventStream};

pub(crate) trait OpenClawService: Send + Sync {
    fn send_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>>;
    fn stream_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>>;
    fn stream_agent_events(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>>;
}

#[derive(Clone)]
pub(crate) struct EnvOpenClawService;

pub(crate) async fn openclaw_status_from_config(
    gateway_url: Option<String>,
) -> OpenClawStatusPayload {
    let gateway_url = gateway_url
        .unwrap_or_else(|| std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default());
    if gateway_url.trim().is_empty() {
        return OpenClawStatusPayload {
            status: OpenClawConnectionStatus::Disconnected,
            gateway_url,
            message: Some("NANAMI_OPENCLAW_GATEWAY_URL is not configured".into()),
            agent: None,
            profile: None,
        };
    }

    let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));

    match client.check_status().await {
        Ok(status) => status,
        Err(_) => OpenClawStatusPayload {
            status: OpenClawConnectionStatus::Error,
            gateway_url: String::new(),
            message: Some("OpenClaw status check failed".into()),
            agent: None,
            profile: None,
        },
    }
}

impl OpenClawService for EnvOpenClawService {
    fn send_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            let gateway_url = std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default();
            if gateway_url.trim().is_empty() {
                return Err(chat_error(
                    "OPENCLAW_GATEWAY_UNCONFIGURED",
                    "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                    Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
                ));
            }

            let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));
            client
                .send_chat_message(OpenClawChatRequest {
                    message: request.message,
                    session_id: request.session_id.clone(),
                })
                .await
                .map(|response| ChatResponse {
                    session_id: response
                        .session_id
                        .or(request.session_id)
                        .unwrap_or_else(|| "default".into()),
                    message_id: response.message_id.unwrap_or_else(|| "msg_openclaw".into()),
                    content: response.content,
                })
                .map_err(map_openclaw_chat_error)
        })
    }

    fn stream_chat_message(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OpenClawChatStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            let gateway_url = std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default();
            if gateway_url.trim().is_empty() {
                return Err(chat_error(
                    "OPENCLAW_GATEWAY_UNCONFIGURED",
                    "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                    Some("Set NANAMI_OPENCLAW_GATEWAY_URL before sending chat messages"),
                ));
            }

            let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));
            let stream = client
                .stream_chat_message(OpenClawChatRequest {
                    message: request.message,
                    session_id: request.session_id,
                })
                .await
                .map_err(map_openclaw_chat_error)?;
            Ok(stream)
        })
    }

    fn stream_agent_events(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<NanamiEventStream, ErrorPayload>> + Send + '_>> {
        Box::pin(async move {
            let gateway_url = std::env::var("NANAMI_OPENCLAW_GATEWAY_URL").unwrap_or_default();
            if gateway_url.trim().is_empty() {
                return Err(chat_error(
                    "OPENCLAW_GATEWAY_UNCONFIGURED",
                    "NANAMI_OPENCLAW_GATEWAY_URL is not configured",
                    Some("Set NANAMI_OPENCLAW_GATEWAY_URL before starting OpenClaw task streams"),
                ));
            }

            let client = OpenClawClient::new(openclaw_config_from_env(gateway_url));
            let stream = client
                .stream_agent_events(OpenClawChatRequest {
                    message: request.message,
                    session_id: request.session_id,
                })
                .await
                .map_err(map_openclaw_chat_error)?;
            let mapped = FuturesStreamExt::flat_map(stream, |item| match item {
                Ok(OpenClawStreamItem::Event(event)) => {
                    tokio_stream::iter(vec![Ok::<_, ErrorPayload>(event)])
                }
                Ok(OpenClawStreamItem::Chat(_)) => tokio_stream::iter(Vec::new()),
                Err(error) => tokio_stream::iter(vec![Err::<EventEnvelope, _>(
                    map_openclaw_chat_error(error),
                )]),
            });

            Ok(Box::pin(mapped) as NanamiEventStream)
        })
    }
}

fn openclaw_config_from_env(gateway_url: String) -> OpenClawConfig {
    let chat_path = std::env::var("NANAMI_OPENCLAW_CHAT_PATH").unwrap_or_else(|_| "/chat".into());
    OpenClawConfig {
        gateway_url,
        token: std::env::var("NANAMI_OPENCLAW_TOKEN").ok(),
        timeout_ms: DEFAULT_OPENCLAW_TIMEOUT_MS,
        chat_path,
    }
}

pub(crate) fn map_openclaw_chat_error(error: OpenClawError) -> ErrorPayload {
    match error {
        OpenClawError::AuthFailed => chat_error(
            "OPENCLAW_AUTH_FAILED",
            "OpenClaw Gateway authentication failed",
            Some("Check NANAMI_OPENCLAW_TOKEN"),
        ),
        OpenClawError::Disconnected => chat_error(
            "OPENCLAW_DISCONNECTED",
            "OpenClaw Gateway is unreachable",
            Some("Check NANAMI_OPENCLAW_GATEWAY_URL"),
        ),
        OpenClawError::InvalidResponse => chat_error(
            "OPENCLAW_INVALID_RESPONSE",
            "OpenClaw Gateway returned an unsupported chat response",
            None,
        ),
        OpenClawError::UnexpectedStatus(_) | OpenClawError::InvalidClient(_) => {
            chat_error("OPENCLAW_CHAT_FAILED", "OpenClaw chat request failed", None)
        }
    }
}
