use zipeek_tts_stt::{speech::Speech, web};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = web::bind_address()?;
    let speech = Speech::new()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "local Mandarin speech service listening");
    axum::serve(listener, web::router(speech)).await?;
    Ok(())
}
