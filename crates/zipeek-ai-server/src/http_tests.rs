use std::sync::Arc;

use super::{AiServer, fixture_backend::FixtureBackend};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

async fn send(method: &str, path: &str, body: &str) -> String {
    send_with_origin(method, path, body, None).await
}

async fn send_with_origin(method: &str, path: &str, body: &str, origin: Option<&str>) -> String {
    let server = AiServer::new(Arc::new(FixtureBackend), "fixture", 2).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, server.router()).await.unwrap();
    });
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let origin_header = origin
        .map(|value| format!("Origin: {value}\r\n"))
        .unwrap_or_default();
    let message = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\n{origin_header}Content-Length: {}\r\n\r\n{body}",
        body.len(),
    );
    stream.write_all(message.as_bytes()).await.unwrap();
    let mut data = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read_to_end(&mut data),
    )
    .await
    .unwrap()
    .unwrap();
    task.abort();
    String::from_utf8(data).unwrap()
}

#[tokio::test]
async fn lists_only_the_configured_model() {
    let response = send("GET", "/v1/models", "").await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("\"id\":\"fixture\""), "{response}");
}

#[tokio::test]
async fn validates_model_identity_before_running_a_backend() {
    let body = r#"{"model":"missing","messages":[{"role":"user","content":"你好"}]}"#;
    let response = send("POST", "/v1/chat/completions", body).await;
    assert!(response.starts_with("HTTP/1.1 400"), "{response}");
    assert!(response.contains("model unavailable"), "{response}");
}

#[tokio::test]
async fn returns_openai_compatible_json_completion() {
    let body =
        r#"{"model":"fixture","messages":[{"role":"user","content":"你好"}],"stream":false}"#;
    let response = send("POST", "/v1/chat/completions", body).await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("chat.completion"), "{response}");
    assert!(response.contains("你好"), "{response}");
}

#[tokio::test]
async fn sends_sse_deltas_and_done_marker() {
    let body = r#"{"model":"fixture","messages":[{"role":"user","content":"你好"}],"stream":true}"#;
    let response = send("POST", "/v1/chat/completions", body).await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("text/event-stream"), "{response}");
    assert!(response.contains("你好"), "{response}");
    assert!(response.contains("[DONE]"), "{response}");
}

#[tokio::test]
async fn rejects_untrusted_browser_origins() {
    let response =
        send_with_origin("GET", "/v1/models", "", Some("https://untrusted.example")).await;
    assert!(response.starts_with("HTTP/1.1 403"), "{response}");
}

#[tokio::test]
async fn allows_explicit_local_browser_origin() {
    let response = send_with_origin(
        "OPTIONS",
        "/v1/chat/completions",
        "",
        Some("http://localhost:1420"),
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 204"), "{response}");
    assert!(
        response
            .to_ascii_lowercase()
            .contains("access-control-allow-origin: http://localhost:1420"),
        "{response}"
    );
}

#[tokio::test]
async fn rejects_invalid_roles_and_empty_content_without_calling_backend() {
    for payload in [
        r#"{"model":"fixture","messages":[{"role":"tool","content":"你好"}]}"#,
        r#"{"model":"fixture","messages":[{"role":"user","content":"  "}]}"#,
        r#"{"model":"fixture","messages":[]}"#,
    ] {
        let response = send("POST", "/v1/chat/completions", payload).await;
        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
    }
}

#[tokio::test]
async fn rejects_invalid_generation_options_and_unsupported_response_formats() {
    for payload in [
        r#"{"model":"fixture","messages":[{"role":"user","content":"你好"}],"max_tokens":0}"#,
        r#"{"model":"fixture","messages":[{"role":"user","content":"你好"}],"temperature":2.5}"#,
        r#"{"model":"fixture","messages":[{"role":"user","content":"你好"}],"response_format":{"type":"json_schema"}}"#,
    ] {
        let response = send("POST", "/v1/chat/completions", payload).await;
        assert!(response.starts_with("HTTP/1.1 400"), "{response}");
    }
}
