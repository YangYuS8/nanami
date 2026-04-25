use nanami_openclaw::{OpenClawChatRequest, OpenClawClient, OpenClawConfig, OpenClawError};
use nanami_protocol::ChatStreamEventKind;
use tokio_stream::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> OpenClawClient {
    OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: None,
        timeout_ms: 1000,
        chat_path: "/chat".into(),
    })
}

#[tokio::test]
async fn stream_chat_message_reads_openai_sse_deltas() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(
            "data: {\"choices\":[{\"delta\":{\"content\":\"你\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"好\"}}]}\n\ndata: [DONE]\n\n",
        ))
        .mount(&server)
        .await;

    let mut stream = client_for(&server)
        .stream_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: Some("sess_001".into()),
        })
        .await
        .unwrap();

    let first = stream.next().await.unwrap().unwrap();
    let second = stream.next().await.unwrap().unwrap();
    let completed = stream.next().await.unwrap().unwrap();

    assert_eq!(first.kind, ChatStreamEventKind::MessageDelta);
    assert_eq!(first.delta.as_deref(), Some("你"));
    assert_eq!(second.delta.as_deref(), Some("好"));
    assert_eq!(completed.kind, ChatStreamEventKind::MessageCompleted);
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn stream_chat_message_reads_simple_sse_deltas() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(
            "data: {\"delta\":\"你\"}\n\ndata: {\"delta\":\"好\"}\n\ndata: {\"content\":\"你好\"}\n\n",
        ))
        .mount(&server)
        .await;

    let events: Vec<_> = client_for(&server)
        .stream_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(events[0].delta.as_deref(), Some("你"));
    assert_eq!(events[1].delta.as_deref(), Some("好"));
    assert_eq!(events[2].kind, ChatStreamEventKind::MessageCompleted);
    assert_eq!(events[2].content.as_deref(), Some("你好"));
}

#[tokio::test]
async fn stream_chat_message_falls_back_to_json_completion() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "complete response"
        })))
        .mount(&server)
        .await;

    let events: Vec<_> = client_for(&server)
        .stream_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, ChatStreamEventKind::MessageCompleted);
    assert_eq!(events[0].content.as_deref(), Some("complete response"));
}

#[tokio::test]
async fn stream_chat_message_maps_401_to_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let error = match client_for(&server)
        .stream_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
    {
        Ok(_) => panic!("expected auth error"),
        Err(error) => error,
    };

    assert!(matches!(error, OpenClawError::AuthFailed));
}

#[tokio::test]
async fn stream_chat_message_error_does_not_leak_token() {
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: "http://127.0.0.1:1".into(),
        token: Some("secret-token".into()),
        timeout_ms: 100,
        chat_path: "/chat".into(),
    });

    let error = match client
        .stream_chat_message(OpenClawChatRequest {
            message: "Hello".into(),
            session_id: None,
        })
        .await
    {
        Ok(_) => panic!("expected disconnected error"),
        Err(error) => error,
    };

    assert!(!error.to_string().contains("secret-token"));
}
