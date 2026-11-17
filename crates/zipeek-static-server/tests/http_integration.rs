use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};
use zipeek_static_server::{ServerConfig, StaticServer};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct SiteFixture {
    root: PathBuf,
}

impl SiteFixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zipeek-static-integration-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir(&root).unwrap();
        Self { root }
    }

    fn write(&self, path: &str, contents: &[u8]) {
        let file = self.root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, contents).unwrap();
    }

    fn config(&self, base_path: &str) -> ServerConfig {
        ServerConfig {
            root: self.root.clone(),
            address: "127.0.0.1:0".parse().unwrap(),
            base_path: base_path.to_owned(),
        }
    }
}

impl Drop for SiteFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

async fn request(address: SocketAddr, method: &str, path: &str) -> String {
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(address))
        .await
        .expect("connection timed out")
        .expect("server must accept connections");
    stream
        .write_all(
            format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
    let mut reply = Vec::new();
    timeout(Duration::from_secs(5), stream.read_to_end(&mut reply))
        .await
        .expect("response timed out")
        .unwrap();
    String::from_utf8(reply).unwrap()
}

fn expect_status(reply: &str, code: u16) {
    assert!(
        reply.starts_with(&format!("HTTP/1.1 {code} ")),
        "unexpected HTTP status: {reply}"
    );
}

#[tokio::test]
async fn serves_index_and_assets_from_root() {
    let fixture = SiteFixture::new();
    fixture.write("index.html", b"<html>static integration</html>");
    fixture.write("assets/app.js", b"window.testReady = true;");
    fixture.write("assets/app.wasm", b"wasm-test-bytes");

    let server = StaticServer::run(fixture.config("/")).await.unwrap();
    let address = server.address();
    assert_eq!(server.url(), format!("http://{address}"));

    for (path, contents) in [
        ("/", "static integration"),
        ("/index.html", "static integration"),
        ("/assets/app.js", "window.testReady = true;"),
        ("/assets/app.wasm", "wasm-test-bytes"),
    ] {
        let reply = request(address, "GET", path).await;
        expect_status(&reply, 200);
        assert!(reply.contains(contents), "{path}: {reply}");
    }

    server.stop().await.unwrap();
}

#[tokio::test]
async fn wasm_assets_have_a_wasm_mime_type_and_head_has_no_body() {
    let fixture = SiteFixture::new();
    fixture.write("index.html", b"index");
    fixture.write("assets/application.wasm", b"fixture-wasm-payload");
    let server = StaticServer::run(fixture.config("/")).await.unwrap();

    let get = request(server.address(), "GET", "/assets/application.wasm").await;
    expect_status(&get, 200);
    assert!(
        get.to_ascii_lowercase()
            .contains("content-type: application/wasm"),
        "WASM needs the correct MIME type for browser streaming: {get}"
    );
    assert!(get.contains("fixture-wasm-payload"));

    let head = request(server.address(), "HEAD", "/assets/application.wasm").await;
    expect_status(&head, 200);
    assert!(
        head.ends_with("\r\n\r\n"),
        "HEAD must not include the asset body: {head}"
    );
    server.stop().await.unwrap();
}

#[tokio::test]
async fn missing_paths_do_not_fall_back_to_index() {
    let fixture = SiteFixture::new();
    fixture.write("index.html", b"sentinel-index-body");
    let server = StaticServer::run(fixture.config("/")).await.unwrap();

    for path in ["/missing.js", "/unknown/route", "/assets/missing.wasm"] {
        let reply = request(server.address(), "GET", path).await;
        expect_status(&reply, 404);
        assert!(!reply.contains("sentinel-index-body"), "{path}: {reply}");
    }

    server.stop().await.unwrap();
}

#[tokio::test]
async fn prefixed_site_is_isolated_from_root() {
    let fixture = SiteFixture::new();
    fixture.write("index.html", b"prefixed-index");
    fixture.write("assets/app.js", b"prefixed-script");
    let server = StaticServer::run(fixture.config("/zipeek/")).await.unwrap();

    for (path, expected_body) in [
        ("/zipeek/", "prefixed-index"),
        ("/zipeek/assets/app.js", "prefixed-script"),
    ] {
        let reply = request(server.address(), "GET", path).await;
        expect_status(&reply, 200);
        assert!(reply.contains(expected_body), "{path}: {reply}");
    }
    for path in ["/", "/assets/app.js", "/zipeek/missing.js"] {
        expect_status(&request(server.address(), "GET", path).await, 404);
    }

    server.stop().await.unwrap();
}

#[tokio::test]
async fn startup_requires_index_and_a_directory() {
    let fixture = SiteFixture::new();
    assert!(StaticServer::run(fixture.config("/")).await.is_err());

    fixture.write("index.html", b"ready");
    let server = StaticServer::run(fixture.config("/")).await.unwrap();
    server.stop().await.unwrap();

    let nonexistent = fixture.root.join("not-a-directory");
    assert!(
        StaticServer::run(ServerConfig {
            root: nonexistent,
            ..fixture.config("/")
        })
        .await
        .is_err()
    );
}

#[tokio::test]
async fn direct_api_rejects_invalid_base_paths() {
    let fixture = SiteFixture::new();
    fixture.write("index.html", b"ready");

    for path in ["", "zipeek/", "/../", "/a//b/", "/a/./", "/a?b/"] {
        let error = match StaticServer::run(fixture.config(path)).await {
            Ok(server) => {
                server.stop().await.unwrap();
                panic!("invalid base path accepted: {path}");
            }
            Err(error) => error,
        };
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput, "{path}");
    }
}

#[tokio::test]
async fn shutdown_releases_port_for_new_server() {
    let fixture = SiteFixture::new();
    fixture.write("index.html", b"restartable");
    let server = StaticServer::run(fixture.config("/")).await.unwrap();
    let address = server.address();
    expect_status(&request(address, "GET", "/").await, 200);
    server.stop().await.unwrap();

    let restarted = StaticServer::run(ServerConfig {
        address,
        ..fixture.config("/")
    })
    .await
    .unwrap();
    expect_status(&request(address, "GET", "/").await, 200);
    restarted.stop().await.unwrap();
}

#[tokio::test]
async fn two_independent_instances_use_distinct_ports() {
    let first = SiteFixture::new();
    let second = SiteFixture::new();
    first.write("index.html", b"site-one");
    second.write("index.html", b"site-two");

    let one = StaticServer::run(first.config("/")).await.unwrap();
    let two = StaticServer::run(second.config("/")).await.unwrap();
    assert_ne!(one.address(), two.address());
    assert!(
        request(one.address(), "GET", "/")
            .await
            .contains("site-one")
    );
    assert!(
        request(two.address(), "GET", "/")
            .await
            .contains("site-two")
    );

    one.stop().await.unwrap();
    two.stop().await.unwrap();
}

#[tokio::test]
async fn executable_serves_dist_and_can_be_stopped() {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let fixture = SiteFixture::new();
    fixture.write("index.html", b"executable-fixture-page");

    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_zipeek-static-server"))
        .arg("--root")
        .arg(&fixture.root)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let announcement = timeout(Duration::from_secs(10), lines.next_line())
        .await
        .expect("server startup timed out")
        .unwrap()
        .expect("server exited before announcing its address");
    let address: SocketAddr = announcement
        .split("http://")
        .nth(1)
        .expect("server must announce its URL")
        .parse()
        .expect("server must announce a valid socket address");
    let reply = request(address, "GET", "/").await;
    expect_status(&reply, 200);
    assert!(reply.contains("executable-fixture-page"), "{reply}");

    child.kill().await.unwrap();
    let status = child.wait().await.unwrap();
    assert!(!status.success());
}

#[tokio::test]
async fn executable_rejects_invalid_options_without_starting() {
    let output = timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(env!("CARGO_BIN_EXE_zipeek-static-server"))
            .args(["--base-path", "/../"])
            .output(),
    )
    .await
    .expect("CLI validation timed out")
    .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid --base-path"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
