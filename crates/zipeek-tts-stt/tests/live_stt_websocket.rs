use std::path::PathBuf;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use zipeek_tts_stt::{server::SpeechServer, speech::Speech};

fn speech_without_models() -> Speech {
    Speech::with_paths(
        PathBuf::from("zipeek-live-test-missing-espeak"),
        PathBuf::from("zipeek-live-test-missing-data"),
        PathBuf::from("zipeek-live-test-missing-asr/model.safetensors"),
    )
}

async fn handshake(stream: &mut TcpStream, host: &str, origin: &str) -> String {
    let request = format!(
        "GET /v1/stt/live HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nOrigin: {origin}\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await.unwrap();
        bytes.push(byte[0]);
        if bytes.ends_with(b"\r\n\r\n") {
            break;
        }
        assert!(bytes.len() < 8192);
    }
    String::from_utf8(bytes).unwrap()
}

async fn message(stream: &mut TcpStream) -> Vec<u8> {
    let mut header = [0; 2];
    stream.read_exact(&mut header).await.unwrap();
    assert_eq!(header[0] & 0x0f, 2);
    let length = match header[1] & 0x7f {
        n @ 0..=125 => n as usize,
        126 => {
            let mut bytes = [0; 2];
            stream.read_exact(&mut bytes).await.unwrap();
            u16::from_be_bytes(bytes) as usize
        }
        _ => panic!("unexpected large server frame"),
    };
    let mut data = vec![0; length];
    stream.read_exact(&mut data).await.unwrap();
    data
}

async fn send_binary(stream: &mut TcpStream, payload: &[u8]) {
    let length = payload.len();
    assert!(length <= 65535);
    let mut bytes = vec![0x82];
    if length < 126 {
        bytes.push(0x80 | length as u8);
    } else {
        bytes.push(0x80 | 126);
        bytes.extend_from_slice(&(length as u16).to_be_bytes());
    }
    let mask = [1, 2, 3, 4];
    bytes.extend_from_slice(&mask);
    for (idx, byte) in payload.iter().enumerate() {
        bytes.push(byte ^ mask[idx % 4]);
    }
    stream.write_all(&bytes).await.unwrap();
}

#[tokio::test]
async fn websocket_connects_and_rejects_no_speech_without_model() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server = SpeechServer::start(listener, speech_without_models())
        .await
        .unwrap();
    let mut stream = TcpStream::connect(server.address()).await.unwrap();
    let host = format!("127.0.0.1:{}", server.address().port());
    let result = handshake(&mut stream, &host, "http://localhost:1420").await;
    assert!(result.starts_with("HTTP/1.1 101"), "{result}");
    assert_eq!(
        message(&mut stream).await,
        vec![0x81, 0x80, 0x3e, 0, 0, 30, 0, 1]
    );
    send_binary(&mut stream, &[1, 0x80, 0x3e, 0, 0]).await;
    assert_eq!(message(&mut stream).await, vec![0x82]);
    send_binary(&mut stream, &[2]).await;
    assert_eq!(message(&mut stream).await, vec![0x85, 1]);
    drop(stream);
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn websocket_rejects_external_browser_origin() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server = SpeechServer::start(listener, speech_without_models())
        .await
        .unwrap();
    let mut stream = TcpStream::connect(server.address()).await.unwrap();
    let host = format!("127.0.0.1:{}", server.address().port());
    let response = handshake(&mut stream, &host, "https://evil.example").await;
    assert!(!response.starts_with("HTTP/1.1 101"));
    drop(stream);
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn websocket_rejects_audio_before_start() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server = SpeechServer::start(listener, speech_without_models())
        .await
        .unwrap();
    let mut stream = TcpStream::connect(server.address()).await.unwrap();
    let host = format!("127.0.0.1:{}", server.address().port());
    assert!(
        handshake(&mut stream, &host, "http://localhost:1420")
            .await
            .starts_with("HTTP/1.1 101")
    );
    assert_eq!(message(&mut stream).await[0], 0x81);
    send_binary(&mut stream, &[3, 0, 0]).await;
    assert_eq!(message(&mut stream).await, vec![0x85, 5]);
    drop(stream);
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn websocket_rejects_invalid_sample_rate() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server = SpeechServer::start(listener, speech_without_models())
        .await
        .unwrap();
    let mut stream = TcpStream::connect(server.address()).await.unwrap();
    let host = format!("127.0.0.1:{}", server.address().port());
    assert!(
        handshake(&mut stream, &host, "http://localhost:1420")
            .await
            .starts_with("HTTP/1.1 101")
    );
    assert_eq!(message(&mut stream).await[0], 0x81);
    send_binary(&mut stream, &[1, 0x80, 0xbb, 0, 0]).await;
    assert_eq!(message(&mut stream).await, vec![0x85, 5]);
    drop(stream);
    server.shutdown().await.unwrap();
}
