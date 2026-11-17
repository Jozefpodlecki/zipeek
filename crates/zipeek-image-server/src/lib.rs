#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

mod api;
mod config;
mod error;
mod inference;
mod jobs;
mod storage;

use std::{net::SocketAddr, sync::Arc};

use config::{BackendKind, Config};
use inference::{ImageBackend, PlaceholderBackend, SdxlBackend};
use jobs::JobCoordinator;
use storage::ImageStore;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

pub struct ImageServerOptions {
    pub address: SocketAddr,
    pub data_dir: std::path::PathBuf,
    pub max_pending: usize,
    pub backend: ImageBackendOptions,
    pub allowed_origins: Vec<String>,
}

pub enum ImageBackendOptions {
    Placeholder,
    Sdxl {
        model_dir: std::path::PathBuf,
        steps: usize,
    },
}

impl ImageServerOptions {
    pub fn placeholder(address: SocketAddr, data_dir: std::path::PathBuf) -> Self {
        Self {
            address,
            data_dir,
            max_pending: 4,
            backend: ImageBackendOptions::Placeholder,
            allowed_origins: Vec::new(),
        }
    }
}

impl From<Config> for ImageServerOptions {
    fn from(config: Config) -> Self {
        Self {
            address: config.bind,
            data_dir: config.data_dir,
            max_pending: config.max_pending,
            allowed_origins: Vec::new(),
            backend: match config.backend {
                BackendKind::Placeholder => ImageBackendOptions::Placeholder,
                BackendKind::Sdxl => ImageBackendOptions::Sdxl {
                    model_dir: config.model_dir,
                    steps: config.steps,
                },
            },
        }
    }
}

fn is_loopback_origin(origin: &str) -> bool {
    let Some(authority) = origin.strip_prefix("http://") else {
        return false;
    };
    let Some((host, port)) = authority.rsplit_once(':') else {
        return false;
    };
    matches!(host, "localhost" | "localhost." | "127.0.0.1")
        && port.parse::<u16>().is_ok_and(|port| port > 0)
}

pub(crate) fn is_default_dev_origin(origin: &str) -> bool {
    is_loopback_origin(origin)
        && origin
            .rsplit_once(':')
            .is_some_and(|(_, port)| port == "1420")
}

pub struct ImageServer {
    address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<std::io::Result<()>>>,
}

impl ImageServer {
    pub async fn run(options: ImageServerOptions) -> Result<Self, Box<dyn std::error::Error>> {
        if !options.address.ip().is_loopback() {
            return Err("Image server requires a loopback address".into());
        }
        if options.max_pending == 0 {
            return Err("Image server queue capacity must be positive".into());
        }
        for origin in &options.allowed_origins {
            if !is_loopback_origin(origin) {
                return Err(
                    format!("Image server CORS origin must be loopback HTTP: {origin}").into(),
                );
            }
        }
        let origins = options.allowed_origins;
        let (backend, backend_name): (Arc<dyn ImageBackend>, &'static str) = match options.backend {
            ImageBackendOptions::Placeholder => (Arc::new(PlaceholderBackend), "placeholder"),
            ImageBackendOptions::Sdxl { model_dir, steps } => {
                (Arc::new(SdxlBackend::new(model_dir, steps)?), "sdxl")
            }
        };
        let store = ImageStore::new(options.data_dir).await?;
        let coordinator = Arc::new(JobCoordinator::new(
            backend,
            store,
            options.max_pending,
            backend_name,
        ));
        let listener = TcpListener::bind(options.address).await?;
        let address = listener.local_addr()?;
        let (shutdown, receiver) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, api::router(coordinator, origins))
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

    pub async fn stop(mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
        if let Some(task) = self.task.take() {
            task.await??;
        }
        Ok(())
    }
}

impl Drop for ImageServer {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
    }
}

pub fn options_from_env() -> Result<ImageServerOptions, Box<dyn std::error::Error>> {
    Ok(Config::from_env()?.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cors_origin_validation_rejects_remote_and_malformed_origins() {
        assert!(is_loopback_origin("http://127.0.0.1:1420"));
        assert!(is_loopback_origin("http://localhost:49152"));
        assert!(is_loopback_origin("http://localhost.:1420"));
        for origin in [
            "http://localhost:1420",
            "http://localhost.:1420",
            "http://127.0.0.1:1420",
        ] {
            assert!(is_default_dev_origin(origin));
        }
        for origin in [
            "http://remote:1420",
            "https://localhost:1420",
            "http://localhost.:1421",
            "http://localhost:1420@remote",
        ] {
            assert!(!is_default_dev_origin(origin));
        }
        for origin in [
            "https://localhost:1420",
            "http://remote:1420",
            "http://127.0.0.1:1420/path",
            "http://localhost:0",
            "http://localhost:1420@remote",
        ] {
            assert!(
                !is_loopback_origin(origin),
                "unexpected valid origin: {origin}"
            );
        }
    }

    #[tokio::test]
    async fn rejects_non_loopback_bind_and_zero_queue_capacity() {
        let data_dir = std::env::temp_dir().join("zipeek-image-server-rejected-options");
        let options =
            ImageServerOptions::placeholder("0.0.0.0:0".parse().unwrap(), data_dir.clone());
        assert!(ImageServer::run(options).await.is_err());
        let mut options = ImageServerOptions::placeholder("127.0.0.1:0".parse().unwrap(), data_dir);
        options.max_pending = 0;
        assert!(ImageServer::run(options).await.is_err());
    }
}
