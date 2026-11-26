use zipeek_tts_stt::{server::SpeechServer, speech::Speech, web};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(web::bind_address()?).await?;
    let speech = Speech::new()?;
    let server = SpeechServer::start(listener, speech).await?;
    tracing::info!(address = %server.address(), "local Mandarin speech service listening");
    tokio::signal::ctrl_c().await?;
    server.shutdown().await?;
    Ok(())
}
