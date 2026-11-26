use std::{io, net::SocketAddr};

use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

use crate::{speech::Speech, web};

pub struct SpeechServer {
    address: SocketAddr,
    speech: Speech,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<io::Result<()>>,
}

impl SpeechServer {
    pub async fn start(listener: TcpListener, speech: Speech) -> io::Result<Self> {
        let address = listener.local_addr()?;
        if !address.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "speech server listener must bind to loopback",
            ));
        }
        speech
            .initialize_neural_engines()
            .await
            .map_err(io::Error::other)?;
        let (shutdown, receiver) = oneshot::channel();
        let server_speech = speech.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, web::router(server_speech))
                .with_graceful_shutdown(async move {
                    let _ = receiver.await;
                })
                .await
        });
        Ok(Self {
            address,
            speech,
            shutdown: Some(shutdown),
            task,
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub async fn shutdown(mut self) -> io::Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), &mut self.task).await;
        match result {
            Ok(Ok(Ok(()))) => {
                tokio::time::timeout(std::time::Duration::from_secs(5), self.speech.shutdown_workers())
                    .await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "speech workers did not stop in five seconds"))?
                    .map_err(io::Error::other)
            },
            Ok(Ok(outcome)) => outcome,
            Ok(Err(error)) => Err(io::Error::other(format!(
                "speech server task failed: {error}"
            ))),
            Err(_) => {
                self.task.abort();
                let _ = (&mut self.task).await;
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "speech server graceful shutdown exceeded five seconds",
                ))
            }
        }
    }
}

impl Drop for SpeechServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn starts_without_optional_models_and_reports_no_loaded_engines() {
        let speech = Speech::with_paths(
            PathBuf::from("zipeek-warmup-test-missing-espeak"),
            PathBuf::from("zipeek-warmup-test-missing-espeak-data"),
            PathBuf::from("zipeek-warmup-test-missing-asr/model.safetensors"),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server = SpeechServer::start(listener, speech.clone()).await.unwrap();
        assert!(!speech.neural_tts_engine_loaded());
        assert!(!speech.neural_stt_engine_loaded());
        server.shutdown().await.unwrap();
        assert_eq!(speech.neural_tts_status(), "stopped");
        assert_eq!(speech.neural_stt_status(), "stopped");
    }

    #[tokio::test]
    async fn starts_on_a_loopback_listener() {
        let speech = Speech::with_paths(
            PathBuf::from("zipeek-warmup-test-missing-espeak"),
            PathBuf::from("zipeek-warmup-test-missing-espeak-data"),
            PathBuf::from("zipeek-warmup-test-missing-asr/model.safetensors"),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server = SpeechServer::start(listener, speech).await.unwrap();
        assert!(server.address().ip().is_loopback());
        server.shutdown().await.unwrap();
    }
}
