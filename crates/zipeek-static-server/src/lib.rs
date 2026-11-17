#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub root: PathBuf,
    pub address: SocketAddr,
    pub base_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("crates/zipeek-web/dist"),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1420),
            base_path: "/".to_owned(),
        }
    }
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<ServerConfig, String> {
    let mut config = ServerConfig::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                config.root = PathBuf::from(args.next().ok_or("--root requires a directory")?);
            }
            "--host" => {
                let host = args.next().ok_or("--host requires an IP address")?;
                let host: IpAddr = host
                    .parse()
                    .map_err(|_| format!("invalid --host: {host}"))?;
                config.address.set_ip(host);
            }
            "--port" => {
                let port = args.next().ok_or("--port requires a number")?;
                let port = port
                    .parse()
                    .map_err(|_| format!("invalid --port: {port}"))?;
                config.address.set_port(port);
            }
            "--base-path" => {
                let path = args.next().ok_or("--base-path requires an absolute path")?;
                if !valid_base_path(&path) {
                    return Err(format!("invalid --base-path: {path}"));
                }
                config.base_path = path;
            }
            "--help" | "-h" => {
                return Err("usage: zipeek-static-server [--root DIST_DIR] [--host IP] [--port PORT] [--base-path /PREFIX/]".to_owned());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    Ok(config)
}

fn valid_base_path(path: &str) -> bool {
    path == "/"
        || (path.starts_with('/')
            && path.ends_with('/')
            && path[1..path.len() - 1].split('/').all(|part| {
                !part.is_empty()
                    && part != "."
                    && part != ".."
                    && part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            }))
}

fn validate_base_path(path: &str) -> std::io::Result<()> {
    if valid_base_path(path) {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid base path: {path}"),
        ))
    }
}

pub fn validate_root(root: &Path) -> Result<PathBuf, std::io::Error> {
    let root = root.canonicalize()?;
    let index = root.join("index.html");
    if !index.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("missing Trunk build index: {}", index.display()),
        ));
    }
    Ok(root)
}

pub fn app(root: PathBuf, base_path: &str) -> Router {
    let files = ServeDir::new(root).append_index_html_on_directories(true);
    if base_path == "/" {
        Router::new().fallback_service(files)
    } else {
        Router::new().nest_service(base_path.trim_end_matches('/'), files)
    }
}

pub async fn serve(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    validate_base_path(&config.base_path)?;
    let root = validate_root(&config.root)?;
    let listener = TcpListener::bind(config.address).await?;
    println!(
        "Serving {} on http://{}",
        root.display(),
        listener.local_addr()?
    );
    axum::serve(listener, app(root, &config.base_path))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

pub struct StaticServer {
    address: SocketAddr,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
}

impl StaticServer {
    pub async fn run(config: ServerConfig) -> std::io::Result<Self> {
        validate_base_path(&config.base_path)?;
        let root = validate_root(&config.root)?;
        let listener = TcpListener::bind(config.address).await?;
        let address = listener.local_addr()?;
        let (shutdown, receiver) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, app(root, &config.base_path))
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await
        });
        Ok(Self {
            address,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub async fn stop(mut self) -> std::io::Result<()> {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
        if let Some(task) = self.task.take() {
            task.await.map_err(std::io::Error::other)??;
        }
        Ok(())
    }
}

impl Drop for StaticServer {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[test]
    fn parses_arguments_and_rejects_invalid_options() {
        let args = ["--root", "dist", "--host", "127.0.0.1", "--port", "0"];
        let config = parse_args(args.into_iter().map(str::to_owned)).unwrap();
        assert_eq!(config.root, PathBuf::from("dist"));
        assert_eq!(config.address, "127.0.0.1:0".parse().unwrap());
        let config = parse_args(["--base-path", "/zipeek/"].map(str::to_owned)).unwrap();
        assert_eq!(config.base_path, "/zipeek/");
        for args in [
            vec!["--port", "invalid"],
            vec!["--host", "invalid"],
            vec!["--root"],
            vec!["--unknown"],
            vec!["--base-path", "/../"],
            vec!["--base-path", "invalid"],
            vec!["--base-path", "/double//slash/"],
        ] {
            assert!(parse_args(args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[tokio::test]
    async fn owned_server_rejects_missing_assets() {
        let config = ServerConfig {
            root: std::env::temp_dir().join("zipeek-no-such-dist-root"),
            address: "127.0.0.1:0".parse().unwrap(),
            base_path: "/".to_owned(),
        };
        assert!(StaticServer::run(config).await.is_err());
    }

    #[test]
    fn requires_an_existing_built_index() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-static-server-missing-{}",
            std::process::id()
        ));
        assert!(validate_root(&root).is_err());
    }

    #[tokio::test]
    async fn serves_index_and_static_assets_without_spa_fallback() {
        let root =
            std::env::temp_dir().join(format!("zipeek-static-server-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("index.html"), "zipeek-test-index").unwrap();
        std::fs::write(root.join("sample.wasm"), b"wasm-test").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let serving_root = root.clone();
        let handle =
            tokio::spawn(
                async move { axum::serve(listener, app(serving_root, "/")).await.unwrap() },
            );
        for (path, status, body) in [
            ("/", "200 OK", "zipeek-test-index"),
            ("/sample.wasm", "200 OK", "wasm-test"),
            ("/missing", "404 Not Found", ""),
        ] {
            let mut stream = TcpStream::connect(address).await.unwrap();
            stream
                .write_all(
                    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                        .as_bytes(),
                )
                .await
                .unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).await.unwrap();
            let response = String::from_utf8_lossy(&response);
            assert!(response.contains(status), "{path}: {response}");
            assert!(response.contains(body), "{path}: {response}");
        }
        handle.abort();
        let _ = handle.await;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let serving_root = root.clone();
        let pages = tokio::spawn(async move {
            axum::serve(listener, app(serving_root, "/zipeek/"))
                .await
                .unwrap()
        });
        for (path, status) in [
            ("/zipeek/", "200 OK"),
            ("/zipeek/sample.wasm", "200 OK"),
            ("/", "404 Not Found"),
        ] {
            let mut stream = TcpStream::connect(address).await.unwrap();
            stream
                .write_all(
                    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                        .as_bytes(),
                )
                .await
                .unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).await.unwrap();
            let response = String::from_utf8_lossy(&response);
            assert!(response.contains(status), "{path}: {response}");
        }
        pages.abort();
        let _ = pages.await;
        std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("zipeek-static-server-test-{}", std::process::id())),
        )
        .unwrap();
    }
}
