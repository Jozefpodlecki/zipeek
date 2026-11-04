use aimux_core::generate::{GenerateTextOptions, GenerateTextResult, stream_text};
use aimux_core::language_model::LanguageModel;
use aimux_core::message::ModelMessage;
use aimux_providers::{PresetSettings, create_provider};
use std::sync::Arc;
use tracing::debug;

use super::error::{LlmError, LlmResult};
use super::streaming::{consume_progress, consume_silent};

#[derive(Clone)]
pub struct LlmClient {
    base_url: String,
    timeout: Option<aimux_core::options::TimeoutConfiguration>,
    max_retries: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub request_timeout_ms: u64,
    pub first_chunk_timeout_ms: u64,
    pub max_retries: u32,
}

impl LlmClient {
    pub fn new(config: &Config) -> Self {
        let base = config.base_url.trim_end_matches('/');
        let base_url = if base.ends_with("/v1") {
            base.to_owned()
        } else {
            format!("{base}/v1")
        };
        Self {
            base_url,
            timeout: Some(aimux_core::options::TimeoutConfiguration {
                total_ms: Some(config.request_timeout_ms),
                first_chunk_ms: Some(config.first_chunk_timeout_ms),
                step_ms: None,
                chunk_ms: None,
            }),
            max_retries: config.max_retries,
        }
    }

    fn model(&self, model_id: &str) -> LlmResult<Arc<dyn LanguageModel>> {
        let provider = create_provider(
            "openai",
            PresetSettings {
                api_key: Some("lmstudio".to_owned().into()),
                base_url: Some(self.base_url.clone()),
                ..Default::default()
            },
        )?;
        provider.language_model(model_id).map_err(LlmError::Provider)
    }

    fn options(
        &self,
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
    ) -> GenerateTextOptions {
        GenerateTextOptions {
            tools,
            timeout: self.timeout.clone(),
            instructions,
            max_retries: Some(self.max_retries),
            ..Default::default()
        }
    }

    pub async fn chat(
        &self,
        model_id: &str,
        messages: &[ModelMessage],
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
    ) -> LlmResult<GenerateTextResult> {
        let model = self.model(model_id)?;
        let stream = stream_text(model.as_ref(), messages.to_vec(), self.options(tools, instructions))
            .await
            .map_err(LlmError::Provider)?;
        consume_progress(stream).await
    }

    pub async fn chat_silent(
        &self,
        model_id: &str,
        messages: &[ModelMessage],
        tools: Option<Vec<aimux_core::tool::Tool>>,
        instructions: Option<String>,
    ) -> LlmResult<GenerateTextResult> {
        let model = self.model(model_id)?;
        let stream = stream_text(model.as_ref(), messages.to_vec(), self.options(tools, instructions))
            .await
            .map_err(LlmError::Provider)?;
        consume_silent(stream).await
    }

    pub async fn complete_with_instructions(
        &self,
        model_id: &str,
        prompt: &str,
        instructions: &str,
    ) -> LlmResult<GenerateTextResult> {
        let model = self.model(model_id)?;
        debug!(model = model_id, prompt_chars = prompt.len(), "classification completion");
        let stream = stream_text(
            model.as_ref(),
            prompt.to_owned(),
            self.options(None, Some(instructions.to_owned())),
        )
        .await
        .map_err(LlmError::Provider)?;
        consume_silent(stream).await
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    use super::{Config, LlmClient};
    use crate::llm::classification::parse_model_response;

    fn mock_lm_studio(deltas: &[&str]) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let chunks = deltas.iter().map(|text| text.to_string()).collect::<Vec<_>>();
        let thread = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0, "client closed before sending request");
                request.extend_from_slice(&buffer[..count]);
                let Some(boundary) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&request[..boundary]).to_ascii_lowercase();
                let length = headers.lines().find_map(|line| {
                    line.strip_prefix("content-length:")?.trim().parse::<usize>().ok()
                }).unwrap_or(0);
                if request.len() >= boundary + 4 + length {
                    break;
                }
            }
            let body = String::from_utf8_lossy(&request).into_owned();
            let full_text = chunks.concat();
            let sequence = std::cell::Cell::new(1u64);
            let mut events = String::new();
            let mut emit = |event: serde_json::Value| {
                let event_type = event["type"].as_str().unwrap();
                events.push_str(&format!("event: {event_type}\ndata: {event}\n\n"));
                sequence.set(sequence.get() + 1);
            };
            let base_response = serde_json::json!({
                "id": "resp_test",
                "object": "response",
                "created_at": 1,
                "status": "in_progress",
                "model": "mock-classifier",
                "output": [],
                "usage": null,
            });
            emit(serde_json::json!({
                "type": "response.created",
                "sequence_number": sequence.get(),
                "response": base_response,
            }));
            emit(serde_json::json!({
                "type": "response.output_item.added",
                "sequence_number": sequence.get(),
                "output_index": 0,
                "item": {
                    "id": "msg_test", "type": "message",
                    "status": "in_progress", "role": "assistant", "content": [],
                },
            }));
            emit(serde_json::json!({
                "type": "response.content_part.added",
                "sequence_number": sequence.get(),
                "item_id": "msg_test", "output_index": 0, "content_index": 0,
                "part": { "type": "output_text", "text": "", "annotations": [] },
            }));
            for text in &chunks {
                emit(serde_json::json!({
                    "type": "response.output_text.delta",
                    "sequence_number": sequence.get(),
                    "item_id": "msg_test", "output_index": 0, "content_index": 0,
                    "delta": text,
                }));
            }
            emit(serde_json::json!({
                "type": "response.output_text.done",
                "sequence_number": sequence.get(),
                "item_id": "msg_test", "output_index": 0, "content_index": 0,
                "text": full_text,
            }));
            emit(serde_json::json!({
                "type": "response.content_part.done",
                "sequence_number": sequence.get(),
                "item_id": "msg_test", "output_index": 0, "content_index": 0,
                "part": { "type": "output_text", "text": full_text, "annotations": [] },
            }));
            let output_item = serde_json::json!({
                "id": "msg_test", "type": "message", "status": "completed",
                "role": "assistant", "content": [
                    { "type": "output_text", "text": full_text, "annotations": [] }
                ],
            });
            emit(serde_json::json!({
                "type": "response.output_item.done",
                "sequence_number": sequence.get(),
                "output_index": 0,
                "item": output_item,
            }));
            emit(serde_json::json!({
                "type": "response.completed",
                "sequence_number": sequence.get(),
                "response": {
                    "id": "resp_test", "object": "response", "created_at": 1,
                    "status": "completed", "model": "mock-classifier",
                    "output": [output_item],
                    "usage": { "input_tokens": 10, "output_tokens": 10, "total_tokens": 20 },
                },
            }));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                events.len(), events,
            );
            socket.write_all(response.as_bytes()).unwrap();
            socket.flush().unwrap();
            body
        });
        (endpoint, thread)
    }

    fn run_streaming_fixture(deltas: &[&str]) -> (String, String) {
        let (endpoint, server) = mock_lm_studio(deltas);
        let client = LlmClient::new(&Config {
            base_url: endpoint,
            request_timeout_ms: 10_000,
            first_chunk_timeout_ms: 10_000,
            max_retries: 0,
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = runtime.block_on(client.complete_with_instructions(
            "mock-classifier",
            "Simplified: 苹果\nDictionary sense: apple (fruit)",
            "Return only JSON with pos and categories",
        )).unwrap();
        (result.text(), server.join().unwrap())
    }

    #[test]
    fn streaming_http_aggregates_split_json_and_passes_system_instructions() {
        let (text, request) = run_streaming_fixture(&[
            "{\"pos\":[\"noun\"],",
            "\"categories\":[\"food\"]}",
        ]);
        let parsed = parse_model_response(&text).unwrap();
        assert_eq!(parsed.pos, vec!["noun".to_owned()]);
        assert_eq!(parsed.categories, vec!["food".to_owned()]);
        assert!(request.contains("/v1/responses"));
        assert!(request.contains("Return only JSON with pos and categories"));
        assert!(request.contains("apple (fruit)"));
    }

    #[test]
    fn streaming_http_without_final_text_is_rejected_before_json_parsing() {
        let (text, _) = run_streaming_fixture(&[]);
        let error = parse_model_response(&text).unwrap_err().to_string();
        assert!(error.contains("no final text"), "{error}");
    }
}
