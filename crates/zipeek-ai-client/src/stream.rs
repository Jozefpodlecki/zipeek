use serde::Deserialize;

use crate::AiClientError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamUpdate {
    Content(String),
    Reasoning { received_chars: usize },
}

#[derive(Deserialize)]
struct StreamEvent {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: Option<StreamDelta>,
    #[serde(default)]
    message: Option<StreamDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Default)]
struct StreamDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
    reasoning: Option<String>,
}

const MAX_EVENT_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Default)]
pub(super) struct SseDecoder {
    pending: Vec<u8>,
    data_lines: Vec<String>,
    data_bytes: usize,
    content: String,
    done: bool,
    completed: bool,
    reasoning_chars: usize,
}

impl SseDecoder {
    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn push<F: FnMut(StreamUpdate)>(
        &mut self,
        bytes: &[u8],
        on_delta: &mut F,
    ) -> Result<(), AiClientError> {
        let mut remaining = bytes;
        while let Some(position) = remaining.iter().position(|byte| *byte == b'\n') {
            self.pending.extend_from_slice(&remaining[..=position]);
            if self.pending.len() > MAX_EVENT_BYTES {
                return Err(AiClientError(
                    "AI streamed an excessively large event".to_owned(),
                ));
            }
            let mut line = std::mem::take(&mut self.pending);
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let line = std::str::from_utf8(&line)
                .map_err(|_| AiClientError("AI stream contains invalid UTF-8".to_owned()))?;
            if line.is_empty() {
                self.dispatch(on_delta)?;
            } else if let Some(data) = line.strip_prefix("data:") {
                let data = data.strip_prefix(' ').unwrap_or(data);
                self.data_bytes = self.data_bytes.saturating_add(data.len() + 1);
                if self.data_bytes > MAX_EVENT_BYTES {
                    return Err(AiClientError(
                        "AI streamed an excessively large event".to_owned(),
                    ));
                }
                self.data_lines.push(data.to_owned());
            }
            if self.done {
                return Ok(());
            }
            remaining = &remaining[position + 1..];
        }
        self.pending.extend_from_slice(remaining);
        if self.pending.len() > MAX_EVENT_BYTES {
            return Err(AiClientError(
                "AI streamed an excessively large event".to_owned(),
            ));
        }
        Ok(())
    }

    fn dispatch<F: FnMut(StreamUpdate)>(&mut self, on_delta: &mut F) -> Result<(), AiClientError> {
        if self.data_lines.is_empty() {
            return Ok(());
        }
        let data = self.data_lines.join("\n");
        self.data_lines.clear();
        self.data_bytes = 0;
        if data.trim() == "[DONE]" {
            self.done = true;
            return Ok(());
        }
        let event: StreamEvent = serde_json::from_str(&data)
            .map_err(|_| AiClientError("AI returned malformed streaming JSON".to_owned()))?;
        if event.error.is_some() {
            return Err(AiClientError(
                "AI provider reported an error during generation".to_owned(),
            ));
        }
        let Some(choice) = event.choices.into_iter().next() else {
            return Ok(());
        };
        if let Some(reason) = choice.finish_reason {
            if reason == "length" {
                return Err(AiClientError(
                    "AI stopped because its output token limit was reached".to_owned(),
                ));
            }
            self.completed = true;
        }
        let delta = choice.delta.or(choice.message).unwrap_or_default();
        if let Some(text) = delta.content.filter(|text| !text.is_empty()) {
            if self.content.len().saturating_add(text.len()) > MAX_OUTPUT_BYTES {
                return Err(AiClientError(
                    "AI response exceeded the maximum supported size".to_owned(),
                ));
            }
            self.content.push_str(&text);
            on_delta(StreamUpdate::Content(text));
        }
        if let Some(reasoning) = delta
            .reasoning_content
            .or(delta.reasoning)
            .filter(|text| !text.is_empty())
        {
            self.reasoning_chars = self
                .reasoning_chars
                .saturating_add(reasoning.chars().count());
            on_delta(StreamUpdate::Reasoning {
                received_chars: self.reasoning_chars,
            });
        }
        Ok(())
    }

    pub fn finish<F: FnMut(StreamUpdate)>(
        mut self,
        on_delta: &mut F,
    ) -> Result<String, AiClientError> {
        if !self.done {
            if !self.pending.is_empty() {
                self.pending.push(b'\n');
                self.push(&[], on_delta)?;
            }
            self.dispatch(on_delta)?;
        }
        if !self.done && !self.completed {
            return Err(AiClientError(
                "AI stream ended before completion".to_owned(),
            ));
        }
        if self.content.trim().is_empty() {
            return Err(AiClientError(
                "AI returned no text response. Check whether the model supports chat completions."
                    .to_owned(),
            ));
        }
        Ok(self.content.trim().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{SseDecoder, StreamUpdate};

    #[test]
    fn rejects_oversized_event_across_short_lines() {
        let mut decoder = SseDecoder::default();
        let line = b"data: abcdefghijklmnopqrstuvwxyz\n";
        let mut rejected = false;
        for _ in 0..(super::MAX_EVENT_BYTES / 24 + 1) {
            if decoder.push(line, &mut |_| {}).is_err() {
                rejected = true;
                break;
            }
        }
        assert!(rejected);
    }

    #[test]
    fn fragmented_unicode_and_multiline_events() {
        let mut reader = SseDecoder::default();
        let mut deltas = Vec::new();
        let bytes = "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\r\n\r\ndata: {\"choices\":[{\"delta\":{\"content\":\"，世界\"}}]}\n\ndata: [DONE]\n\n".as_bytes();
        for chunk in bytes.chunks(2) {
            reader.push(chunk, &mut |delta| deltas.push(delta)).unwrap();
        }
        assert!(reader.is_done());
        assert_eq!(reader.finish(&mut |_| {}).unwrap(), "你好，世界");
        assert_eq!(
            deltas,
            [
                StreamUpdate::Content("你好".to_owned()),
                StreamUpdate::Content("，世界".to_owned())
            ]
        );
    }

    #[test]
    fn accepts_many_small_events_in_one_large_transport_chunk() {
        let mut decoder = SseDecoder::default();
        let event = "data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\n\n";
        let data = event.repeat(super::MAX_EVENT_BYTES / event.len() + 1);
        decoder.push(data.as_bytes(), &mut |_| {}).unwrap();
        decoder.push(b"data: [DONE]\n\n", &mut |_| {}).unwrap();
        assert!(!decoder.finish(&mut |_| {}).unwrap().is_empty());
    }

    #[test]
    fn reasoning_counts_do_not_leak_reasoning_text() {
        let mut reader = SseDecoder::default();
        let mut updates = Vec::new();
        let data = "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"thinking\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"答案\"}}]}\n\ndata: [DONE]\n\n";
        reader
            .push(data.as_bytes(), &mut |event| updates.push(event))
            .unwrap();
        assert_eq!(reader.finish(&mut |_| {}).unwrap(), "答案");
        assert_eq!(
            updates,
            vec![
                StreamUpdate::Reasoning { received_chars: 8 },
                StreamUpdate::Content("答案".to_owned()),
            ]
        );
    }

    #[test]
    fn catches_truncation_and_bad_json() {
        let mut reader = SseDecoder::default();
        reader
            .push(
                b"data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
                &mut |_| {},
            )
            .unwrap();
        assert!(
            reader
                .finish(&mut |_| {})
                .unwrap_err()
                .0
                .contains("before completion")
        );
        let mut reader = SseDecoder::default();
        assert!(
            reader
                .push(b"data: {bad}\n\n", &mut |_| {})
                .unwrap_err()
                .0
                .contains("malformed")
        );
    }

    #[test]
    fn accepts_finish_reason_without_done_marker() {
        let mut reader = SseDecoder::default();
        reader.push(b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n", &mut |_| {}).unwrap();
        assert_eq!(reader.finish(&mut |_| {}).unwrap(), "ok");
    }
}
