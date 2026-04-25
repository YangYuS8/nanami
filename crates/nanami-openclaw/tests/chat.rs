use nanami_openclaw::{OpenClawChatRequest, OpenClawClient, OpenClawConfig, OpenClawError};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer, token: Option<String>) -> OpenClawClient {
    OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token,
        timeout_ms: 1000,
        chat_path: "/chat".into(),
    })
}

#[tokio::test]
async fn send_chat_message_reads_simple_json_content() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "Hello from OpenClaw",
            "session_id": "sess_remote",
            "message_id": "msg_remote"
        })))
        .mount(&server)
        .await;

    let response = client_for(&server, None)
        .send_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: Some("sess_local".into()),
        })
        .await
        .unwrap();

    assert_eq!(response.content, "Hello from OpenClaw");
    assert_eq!(response.session_id.as_deref(), Some("sess_remote"));
    assert_eq!(response.message_id.as_deref(), Some("msg_remote"));
}

#[tokio::test]
async fn send_chat_message_reads_openai_compatible_content() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [
                { "message": { "content": "OpenAI compatible response" } }
            ]
        })))
        .mount(&server)
        .await;

    let response = client_for(&server, None)
        .send_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
        .unwrap();

    assert_eq!(response.content, "OpenAI compatible response");
}

#[tokio::test]
async fn send_chat_message_sends_bearer_token_without_exposing_it() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .and(header("authorization", "Bearer secret-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "ok"
        })))
        .mount(&server)
        .await;

    let response = client_for(&server, Some("secret-token".into()))
        .send_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
        .unwrap();

    assert_eq!(response.content, "ok");
}

#[tokio::test]
async fn send_chat_message_maps_401_to_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let error = client_for(&server, Some("secret-token".into()))
        .send_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, OpenClawError::AuthFailed));
    assert!(!error.to_string().contains("secret-token"));
}

#[tokio::test]
async fn send_chat_message_maps_gateway_unreachable_without_token_leak() {
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: "http://127.0.0.1:1".into(),
        token: Some("secret-token".into()),
        timeout_ms: 100,
        chat_path: "/chat".into(),
    });

    let error = client
        .send_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, OpenClawError::Disconnected));
    assert!(!error.to_string().contains("secret-token"));
}
