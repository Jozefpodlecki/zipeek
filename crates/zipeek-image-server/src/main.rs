use zipeek_image_server::{ImageServer, options_from_env};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zipeek_image_server=info".into()),
        )
        .init();
    let server = ImageServer::run(options_from_env()?).await?;
    tracing::info!(address = %server.address(), "image server listening");
    tokio::signal::ctrl_c().await?;
    server.stop().await
}
