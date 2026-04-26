use nanami_openclaw::{OpenClawChatRequest, OpenClawClient, OpenClawConfig, OpenClawStreamItem};
use tokio_stream::StreamExt;
use wiremock::MockServer;

pub(crate) fn client_for(server: &MockServer) -> OpenClawClient {
    OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: None,
        timeout_ms: 1000,
        chat_path: "/chat".into(),
    })
}

pub(crate) async fn collect_items(server: &MockServer, message: &str) -> Vec<OpenClawStreamItem> {
    client_for(server)
        .stream_agent_events(OpenClawChatRequest {
            message: message.into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap()
}
