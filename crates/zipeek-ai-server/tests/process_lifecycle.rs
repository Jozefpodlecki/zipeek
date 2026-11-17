use std::{net::SocketAddr, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    process::Command,
    time::{sleep, timeout},
};

async fn request(
    address: SocketAddr,
    path: &str,
    method: &str,
    body: &str,
) -> std::io::Result<String> {
    let mut connection = TcpStream::connect(address).await?;
    let message = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    connection.write_all(message.as_bytes()).await?;
    let mut response = String::new();
    connection.read_to_string(&mut response).await?;
    Ok(response)
}

#[tokio::test]
async fn executable_serves_fixture_health_models_and_streaming_then_shuts_down() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let executable = env!("CARGO_BIN_EXE_zipeek-ai-server");
    let mut child = Command::new(executable)
        .args(["--fixture", "--bind", &address.to_string()])
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let health = timeout(Duration::from_secs(10), async {
        loop {
            if let Ok(response) = request(address, "/health", "GET", "").await {
                if response.starts_with("HTTP/1.1 200") {
                    break response;
                }
            }
            if let Some(status) = child.try_wait().unwrap() {
                panic!("fixture process terminated before readiness: {status}");
            }
            sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("fixture process did not become ready");
    assert!(health.contains("200"));

    let models = request(address, "/v1/models", "GET", "").await.unwrap();
    assert!(models.starts_with("HTTP/1.1 200"), "{models}");
    assert!(models.contains("\"id\":\"fixture\""), "{models}");

    let payload =
        r#"{"model":"fixture","messages":[{"role":"user","content":"你好"}],"stream":true}"#;
    let streaming = request(address, "/v1/chat/completions", "POST", payload)
        .await
        .unwrap();
    assert!(streaming.starts_with("HTTP/1.1 200"), "{streaming}");
    assert!(streaming.contains("text/event-stream"), "{streaming}");
    assert!(streaming.contains("[DONE]"), "{streaming}");

    let non_streaming = payload.replace("\"stream\":true", "\"stream\":false");
    let json = request(address, "/v1/chat/completions", "POST", &non_streaming)
        .await
        .unwrap();
    assert!(json.starts_with("HTTP/1.1 200"), "{json}");
    assert!(json.contains("chat.completion"), "{json}");

    let invalid = request(
        address,
        "/v1/chat/completions",
        "POST",
        r#"{"model":"invalid","messages":[{"role":"user","content":"test"}]}"#,
    )
    .await
    .unwrap();
    assert!(invalid.starts_with("HTTP/1.1 400"), "{invalid}");

    child.start_kill().unwrap();
    timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("fixture process failed to stop")
        .unwrap();
}
