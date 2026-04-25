use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 17878));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "nanami-core listening");

    axum::serve(listener, nanami_core::router()).await?;

    Ok(())
}
