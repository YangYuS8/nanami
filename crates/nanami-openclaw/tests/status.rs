use nanami_openclaw::{OpenClawClient, OpenClawConfig};
use nanami_protocol::OpenClawConnectionStatus;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn check_status_maps_http_200_to_connected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: None,
        timeout_ms: 1000,
    });

    let status = client.check_status().await.unwrap();

    assert_eq!(status.status, OpenClawConnectionStatus::Connected);
    assert_eq!(status.gateway_url, server.uri());
}

#[tokio::test]
async fn check_status_maps_unauthorized_to_auth_failed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: Some("token".into()),
        timeout_ms: 1000,
    });

    let status = client.check_status().await.unwrap();

    assert_eq!(status.status, OpenClawConnectionStatus::AuthFailed);
}

#[tokio::test]
async fn check_status_maps_pairing_required_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(409).set_body_string("pairing required"))
        .mount(&server)
        .await;
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: None,
        timeout_ms: 1000,
    });

    let status = client.check_status().await.unwrap();

    assert_eq!(status.status, OpenClawConnectionStatus::PairingRequired);
}

#[tokio::test]
async fn check_status_maps_scope_missing_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403).set_body_string("scope missing"))
        .mount(&server)
        .await;
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: server.uri(),
        token: Some("token".into()),
        timeout_ms: 1000,
    });

    let status = client.check_status().await.unwrap();

    assert_eq!(status.status, OpenClawConnectionStatus::ScopeMissing);
}

#[tokio::test]
async fn check_status_maps_connection_failure_to_disconnected() {
    let client = OpenClawClient::new(OpenClawConfig {
        gateway_url: "http://127.0.0.1:1".into(),
        token: None,
        timeout_ms: 100,
    });

    let status = client.check_status().await.unwrap();

    assert_eq!(status.status, OpenClawConnectionStatus::Disconnected);
}
