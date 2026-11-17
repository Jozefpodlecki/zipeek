use std::{net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use zipeek_ai_server::{AiServer, fixture_backend::FixtureBackend};
use zipeek_image_server::{ImageServer, ImageServerOptions};
use zipeek_static_server::{ServerConfig, StaticServer};

async fn request_with_origin(
    address: SocketAddr,
    method: &str,
    path: &str,
    body: &str,
    origin: Option<&str>,
) -> String {
    let mut stream = TcpStream::connect(address).await.unwrap();
    let origin_header = origin.map_or(String::new(), |origin| format!("Origin: {origin}\r\n"));
    stream.write_all(format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{origin_header}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len(),
    ).as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8(response).unwrap()
}

async fn request(address: SocketAddr, method: &str, path: &str, body: &str) -> String {
    request_with_origin(address, method, path, body, None).await
}

#[tokio::test]
async fn static_server_has_owned_lifecycle() {
    let root = std::env::temp_dir().join(format!("zipeek-e2e-static-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("index.html"), "static e2e probe").unwrap();
    let config = ServerConfig {
        root: root.clone(),
        address: "127.0.0.1:0".parse().unwrap(),
        base_path: "/".to_owned(),
    };
    let server = StaticServer::run(config).await.unwrap();
    let reply = request(server.address(), "GET", "/", "").await;
    assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    assert!(reply.contains("static e2e probe"));
    server.stop().await.unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn placeholder_image_server_has_owned_lifecycle() {
    let dir = std::env::temp_dir().join(format!("zipeek-e2e-image-{}", std::process::id()));
    let options = ImageServerOptions::placeholder("127.0.0.1:0".parse().unwrap(), dir.clone());
    let server = ImageServer::run(options).await.unwrap();
    let health = request(server.address(), "GET", "/health", "").await;
    assert!(health.starts_with("HTTP/1.1 200"), "{health}");
    assert!(health.contains("placeholder"), "{health}");
    let invalid = request(
        server.address(),
        "POST",
        "/v1/images",
        r#"{"prompt":"","width":512,"height":512}"#,
    )
    .await;
    assert!(invalid.starts_with("HTTP/1.1 400"), "{invalid}");
    server.stop().await.unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn image_server_accepts_a_complete_scene_prompt_and_rejects_oversized_prompts() {
    let dir = std::env::temp_dir().join(format!("zipeek-e2e-image-prompt-{}", std::process::id()));
    let server = ImageServer::run(ImageServerOptions::placeholder(
        "127.0.0.1:0".parse().unwrap(),
        dir.clone(),
    ))
    .await
    .unwrap();

    let prompt = "Photorealistic scene: A cat watches a dog beside a wooden table. No text, captions or watermarks.";
    let valid = serde_json::json!({ "prompt": prompt, "width": 512, "height": 512 });
    let accepted = request(server.address(), "POST", "/v1/images", &valid.to_string()).await;
    assert!(accepted.starts_with("HTTP/1.1 202"), "{accepted}");

    let upper_bound = serde_json::json!({
        "prompt": "x".repeat(500),
        "width": 512,
        "height": 512,
    });
    let within_limit = request(
        server.address(),
        "POST",
        "/v1/images",
        &upper_bound.to_string(),
    )
    .await;
    assert!(within_limit.starts_with("HTTP/1.1 202"), "{within_limit}");

    let oversized = serde_json::json!({
        "prompt": "x".repeat(501),
        "width": 512,
        "height": 512,
    });
    let rejected = request(
        server.address(),
        "POST",
        "/v1/images",
        &oversized.to_string(),
    )
    .await;
    assert!(rejected.starts_with("HTTP/1.1 400"), "{rejected}");

    server.stop().await.unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn image_fixture_allows_only_the_configured_browser_origin() {
    let dir = std::env::temp_dir().join(format!("zipeek-e2e-image-cors-{}", std::process::id()));
    let mut options = ImageServerOptions::placeholder("127.0.0.1:0".parse().unwrap(), dir.clone());
    options.allowed_origins = vec!["http://127.0.0.1:31415".to_owned()];
    let server = ImageServer::run(options).await.unwrap();
    let approved = request_with_origin(
        server.address(),
        "OPTIONS",
        "/v1/images",
        "",
        Some("http://127.0.0.1:31415"),
    )
    .await;
    assert!(approved.starts_with("HTTP/1.1 204"), "{approved}");
    assert!(
        approved.contains("access-control-allow-origin: http://127.0.0.1:31415"),
        "{approved}"
    );
    let other = request_with_origin(
        server.address(),
        "OPTIONS",
        "/v1/images",
        "",
        Some("http://evil.example:31415"),
    )
    .await;
    assert!(!other.contains("access-control-allow-origin:"), "{other}");
    server.stop().await.unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ai_server_fixture_is_available_to_e2e() {
    let server = AiServer::new(Arc::new(FixtureBackend), "fixture", 2).unwrap();
    let _router = server.router();
}

#[tokio::test]
async fn static_server_serves_assets_and_releases_its_port() {
    let root =
        std::env::temp_dir().join(format!("zipeek-e2e-static-assets-{}", std::process::id()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(root.join("index.html"), "<html>e2e app</html>").unwrap();
    std::fs::write(root.join("assets/app.js"), "window.e2eReady = true;").unwrap();

    let server = StaticServer::run(ServerConfig {
        root: root.clone(),
        address: "127.0.0.1:0".parse().unwrap(),
        base_path: "/".to_owned(),
    })
    .await
    .unwrap();
    let address = server.address();
    let index = request(address, "GET", "/", "").await;
    assert!(index.starts_with("HTTP/1.1 200"), "{index}");
    assert!(index.contains("e2e app"), "{index}");
    let asset = request(address, "GET", "/assets/app.js", "").await;
    assert!(asset.starts_with("HTTP/1.1 200"), "{asset}");
    assert!(asset.contains("window.e2eReady = true;"), "{asset}");
    let missing = request(address, "GET", "/missing-file.js", "").await;
    assert!(missing.starts_with("HTTP/1.1 404"), "{missing}");
    server.stop().await.unwrap();
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn static_server_rejects_missing_build_root() {
    let root =
        std::env::temp_dir().join(format!("zipeek-e2e-static-no-index-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let failure = StaticServer::run(ServerConfig {
        root: root.clone(),
        address: "127.0.0.1:0".parse().unwrap(),
        base_path: "/".to_owned(),
    })
    .await;
    assert!(failure.is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn ai_fixture_server_has_owned_lifecycle_and_http_contract() {
    let ai = AiServer::new(Arc::new(FixtureBackend), "fixture", 2)
        .unwrap()
        .with_allowed_origins(vec!["http://127.0.0.1:34567".to_owned()])
        .unwrap();
    let server = ai.run("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let address = server.address();

    let health = request(address, "GET", "/health", "").await;
    assert!(health.starts_with("HTTP/1.1 200"), "{health}");
    let models = request(address, "GET", "/v1/models", "").await;
    assert!(models.starts_with("HTTP/1.1 200"), "{models}");
    assert!(models.contains("fixture"), "{models}");

    let approved = request_with_origin(
        address,
        "OPTIONS",
        "/v1/chat/completions",
        "",
        Some("http://127.0.0.1:34567"),
    )
    .await;
    assert!(approved.starts_with("HTTP/1.1 204"), "{approved}");
    let denied = request_with_origin(
        address,
        "OPTIONS",
        "/v1/chat/completions",
        "",
        Some("http://unapproved.example:34567"),
    )
    .await;
    assert!(denied.starts_with("HTTP/1.1 403"), "{denied}");

    server.stop().await.unwrap();
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
}
