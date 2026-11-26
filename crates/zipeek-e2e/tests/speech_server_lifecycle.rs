use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use zipeek_tts_stt::{server::SpeechServer, speech::Speech};

async fn request(address: SocketAddr, path: &str) -> io::Result<String> {
    let mut connection = TcpStream::connect(address).await?;
    connection
        .write_all(
            format!(
                "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                address.port()
            )
            .as_bytes(),
        )
        .await?;
    let mut response = Vec::new();
    connection.read_to_end(&mut response).await?;
    Ok(String::from_utf8_lossy(&response).into_owned())
}

#[tokio::test]
async fn e2e_owns_speech_server_lifecycle() -> io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let server = SpeechServer::start(listener, Speech::new()?).await?;
    let address = server.address();
    assert!(address.ip().is_loopback());
    assert_ne!(address.port(), 0);
    assert_eq!(server.url(), format!("http://{address}"));

    let health = tokio::time::timeout(Duration::from_secs(5), request(address, "/health"))
        .await
        .map_err(io::Error::other)??;
    assert!(health.starts_with("HTTP/1.1 200"), "{health}");
    assert!(health.ends_with("ok"), "{health}");

    let capabilities = request(address, "/v1/capabilities").await?;
    assert!(capabilities.starts_with("HTTP/1.1 200"), "{capabilities}");
    assert!(capabilities.contains("provider=local"));
    assert!(capabilities.contains("language=zh-CN"));
    assert!(capabilities.contains("synthesis="));
    assert!(capabilities.contains("recognition="));

    server.shutdown().await?;
    assert!(TcpStream::connect(address).await.is_err());
    Ok(())
}

#[tokio::test]
async fn dropping_speech_server_releases_listener() -> io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let server = SpeechServer::start(listener, Speech::new()?).await?;
    let address = server.address();
    drop(server);
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if TcpListener::bind(address).await.is_ok() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(io::Error::other)?;
    Ok(())
}

#[tokio::test]
async fn speech_server_rejects_non_loopback_bind() -> io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let result = SpeechServer::start(listener, Speech::new()?).await;
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::InvalidInput);
    Ok(())
}
