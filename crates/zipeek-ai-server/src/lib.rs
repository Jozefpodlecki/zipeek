#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

use std::{convert::Infallible, net::SocketAddr, sync::Arc};

pub mod fixture_backend;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::{get, post},
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{Semaphore, mpsc};
use zipeek_ai_core::{
    ChatMessage, ChatRole, GenerationBackend, GenerationCancellation, GenerationError,
    GenerationEvent, GenerationOptions, GenerationRequest,
};

const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone)]
pub struct AiServer {
    backend: Arc<dyn GenerationBackend>,
    model: String,
    slots: Arc<Semaphore>,
    allowed_origins: Arc<Vec<String>>,
}

impl AiServer {
    pub fn new(
        backend: Arc<dyn GenerationBackend>,
        model: impl Into<String>,
        max_requests: usize,
    ) -> Result<Self, String> {
        let model = model.into();
        if model.trim().is_empty() || max_requests == 0 {
            return Err("server needs a model identifier and a positive concurrency limit".into());
        }
        Ok(Self {
            backend,
            model,
            slots: Arc::new(Semaphore::new(max_requests)),
            allowed_origins: Arc::new(vec![
                "http://localhost:1420".into(),
                "http://127.0.0.1:1420".into(),
            ]),
        })
    }

    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Result<Self, String> {
        if origins.iter().any(|origin| !valid_origin(origin)) {
            return Err("allowed origins must be exact http(s) scheme://host[:port] values".into());
        }
        self.allowed_origins = Arc::new(origins);
        Ok(self)
    }

    pub async fn run(self, address: SocketAddr) -> std::io::Result<RunningAiServer> {
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let (shutdown, receiver) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, self.router())
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await
        });
        Ok(RunningAiServer {
            address,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/health", get(health))
            .route("/v1/models", get(models))
            .route("/v1/chat/completions", post(completions))
            .layer(middleware::from_fn_with_state(self.clone(), cors))
            .with_state(self)
    }
}

pub struct RunningAiServer {
    address: SocketAddr,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
}

impl RunningAiServer {
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub async fn stop(mut self) -> std::io::Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.task.take() {
            task.await.map_err(std::io::Error::other)??;
        }
        Ok(())
    }
}

impl Drop for RunningAiServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

fn valid_origin(origin: &str) -> bool {
    let Some((scheme, authority)) = origin.split_once("://") else {
        return false;
    };
    (scheme == "http" || scheme == "https")
        && !authority.is_empty()
        && !authority.chars().any(|character| {
            matches!(character, '/' | '?' | '#' | '@' | '*' | '\n' | '\r')
                || character.is_whitespace()
        })
}

async fn cors(State(server): State<AiServer>, request: Request, next: Next) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    let Some(origin_text) = origin.as_ref().map(|origin| origin.to_str().unwrap_or("")) else {
        return if request.method() == Method::OPTIONS {
            StatusCode::NO_CONTENT.into_response()
        } else {
            next.run(request).await
        };
    };
    if !server
        .allowed_origins
        .iter()
        .any(|allowed| allowed == origin_text)
    {
        return rejection(StatusCode::FORBIDDEN, "request origin is not allowed");
    }
    let mut response = if request.method() == Method::OPTIONS {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(request).await
    };
    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.unwrap());
    headers.insert(header::VARY, HeaderValue::from_static("Origin"));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("authorization, content-type"),
    );
    response
}

#[derive(Deserialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<RequestMessage>,
    #[serde(default)]
    stream: bool,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    response_format: Option<Value>,
}

#[derive(Deserialize)]
struct RequestMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ApiError {
    error: ApiErrorBody,
}

#[derive(Serialize)]
struct ApiErrorBody {
    message: String,
    #[serde(rename = "type")]
    kind: &'static str,
}

fn rejection(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ApiError {
            error: ApiErrorBody {
                message: message.into(),
                kind: "invalid_request_error",
            },
        }),
    )
        .into_response()
}

fn validate_request(
    raw: CompletionRequest,
    model: &str,
) -> Result<(GenerationRequest, bool), String> {
    if raw.model != model {
        return Err(format!("model unavailable: {}", raw.model));
    }
    if raw
        .response_format
        .as_ref()
        .is_some_and(|format| format.get("type").and_then(Value::as_str) != Some("text"))
    {
        return Err("structured response_format is not yet supported by this server".into());
    }
    let messages = raw
        .messages
        .into_iter()
        .map(|message| {
            let role = match message.role.as_str() {
                "system" => ChatRole::System,
                "user" => ChatRole::User,
                "assistant" => ChatRole::Assistant,
                _ => return Err(format!("unsupported message role: {}", message.role)),
            };
            Ok(ChatMessage {
                role,
                content: message.content,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let options = GenerationOptions {
        max_output_tokens: raw.max_completion_tokens.or(raw.max_tokens).unwrap_or(512),
        temperature: raw.temperature.unwrap_or(0.2),
    };
    let request = GenerationRequest {
        model: raw.model,
        messages,
        options,
    };
    request.validate().map_err(|error| error.to_string())?;
    Ok((request, raw.stream))
}

struct CancelOnDrop(GenerationCancellation);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn models(State(state): State<AiServer>) -> Json<Value> {
    Json(json!({"object":"list","data":[{"id":state.model,"object":"model","owned_by":"zipeek"}]}))
}

fn backend_status(error: &GenerationError) -> StatusCode {
    match error {
        GenerationError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
        GenerationError::ModelUnavailable(_) => StatusCode::NOT_FOUND,
        GenerationError::Cancelled => StatusCode::SERVICE_UNAVAILABLE,
        GenerationError::Backend(_) => StatusCode::BAD_GATEWAY,
    }
}

async fn completions(
    State(state): State<AiServer>,
    Json(raw): Json<CompletionRequest>,
) -> Response {
    let (request, streaming) = match validate_request(raw, &state.model) {
        Ok(request) => request,
        Err(error) => return rejection(StatusCode::BAD_REQUEST, error),
    };
    let permit = match state.slots.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => return rejection(StatusCode::TOO_MANY_REQUESTS, "model is busy"),
    };
    if !streaming {
        let mut output = String::new();
        let mut overflow = false;
        let mut on_event = |event: GenerationEvent| {
            if let GenerationEvent::Content(chunk) = event {
                if output.len().saturating_add(chunk.len()) > MAX_OUTPUT_BYTES {
                    overflow = true;
                } else if !overflow {
                    output.push_str(&chunk);
                }
            }
        };
        let result = state
            .backend
            .generate(request, GenerationCancellation::default(), &mut on_event)
            .await;
        drop(permit);
        if overflow {
            return rejection(
                StatusCode::BAD_GATEWAY,
                "model output exceeded the response limit",
            );
        }
        return match result {
            Ok(()) if !output.trim().is_empty() => Json(json!({"object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":output},"finish_reason":"stop"}]})).into_response(),
            Ok(()) => rejection(StatusCode::BAD_GATEWAY, "model returned an empty response"),
            Err(error) => rejection(backend_status(&error), error.to_string()),
        };
    }

    let (sender, receiver) = mpsc::unbounded_channel::<Event>();
    let cancellation = GenerationCancellation::default();
    let worker_cancellation = cancellation.clone();
    tokio::spawn(async move {
        let _permit = permit;
        let mut bytes = 0usize;
        let mut completed = false;
        let mut on_event = |event: GenerationEvent| {
            if worker_cancellation.is_cancelled() {
                return;
            }
            let payload = match event {
                GenerationEvent::Content(content) => {
                    bytes = bytes.saturating_add(content.len());
                    if bytes > MAX_OUTPUT_BYTES {
                        worker_cancellation.cancel();
                        return;
                    }
                    json!({"choices":[{"index":0,"delta":{"content":content},"finish_reason":null}]})
                }
                GenerationEvent::Complete => {
                    completed = true;
                    json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]})
                }
            };
            if sender
                .send(Event::default().data(payload.to_string()))
                .is_err()
            {
                worker_cancellation.cancel();
            }
        };
        let result = state
            .backend
            .generate(request, worker_cancellation.clone(), &mut on_event)
            .await;
        if bytes > MAX_OUTPUT_BYTES {
            let _ = sender.send(Event::default().data(
                json!({"error":{"message":"model output exceeded response limit"}}).to_string(),
            ));
        } else if let Err(error) = result {
            let _ = sender.send(
                Event::default().data(json!({"error":{"message":error.to_string()}}).to_string()),
            );
        } else if !completed {
            let _ = sender.send(Event::default().data(
                json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}).to_string(),
            ));
        }
        let _ = sender.send(Event::default().data("[DONE]"));
    });
    let output = stream::unfold(
        (receiver, CancelOnDrop(cancellation)),
        |(mut receiver, cancellation)| async move {
            receiver
                .recv()
                .await
                .map(|event| (Ok::<Event, Infallible>(event), (receiver, cancellation)))
        },
    );
    Sse::new(output).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn completion(model: &str) -> CompletionRequest {
        CompletionRequest {
            model: model.into(),
            messages: vec![RequestMessage {
                role: "user".into(),
                content: "你好".into(),
            }],
            stream: true,
            temperature: None,
            max_tokens: None,
            max_completion_tokens: None,
            response_format: None,
        }
    }
    #[test]
    fn rejects_unsupported_model_roles_and_structured_output() {
        assert!(validate_request(completion("different"), "loaded").is_err());
        let mut request = completion("loaded");
        request.messages[0].role = "tool".into();
        assert!(validate_request(request, "loaded").is_err());
        let mut request = completion("loaded");
        request.response_format = Some(json!({"type":"json_schema"}));
        assert!(validate_request(request, "loaded").is_err());
    }
    #[test]
    fn only_accepts_explicit_origins() {
        for origin in [
            "*",
            "http://localhost:1420/",
            "file://",
            "https://site.example/path",
            "https://user@site.example",
        ] {
            assert!(!valid_origin(origin), "accepted {origin}");
        }
        assert!(valid_origin("https://example.org"));
        assert!(valid_origin("http://127.0.0.1:1420"));
    }

    #[test]
    fn validates_client_request_defaults() {
        let (request, streaming) = validate_request(completion("loaded"), "loaded").unwrap();
        assert!(streaming);
        assert_eq!(request.options.max_output_tokens, 512);
        assert_eq!(request.messages[0].role, ChatRole::User);
    }
}

#[cfg(test)]
mod http_tests;
