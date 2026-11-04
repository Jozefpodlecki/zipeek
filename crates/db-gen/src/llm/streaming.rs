use std::io::{self, Write};

use aimux_core::error::AiMuxError;
use aimux_core::generate::{GenerateTextResult, StreamTextResult};
use aimux_core::stream_part::TextStreamPart;
use futures_util::StreamExt;

use super::error::LlmError;

pub async fn consume_silent(stream_result: StreamTextResult) -> Result<GenerateTextResult, LlmError> {
    stream_result.consume().await.map_err(classify)
}

pub fn classify(error: AiMuxError) -> LlmError {
    match &error {
        AiMuxError::Timeout(_) => LlmError::Timeout { millis: 0 },
        AiMuxError::Aborted(_) => LlmError::Aborted,
        _ => LlmError::Provider(error),
    }
}

pub async fn consume_progress(
    mut stream_result: StreamTextResult,
) -> Result<GenerateTextResult, LlmError> {
    stream_result.stream = Box::pin(stream_result.stream.inspect(|event| {
        if let Ok(TextStreamPart::TextDelta { text, .. }) = event {
            print!("{text}");
            let _ = io::stdout().flush();
        }
    }));

    let result = stream_result.consume().await.map_err(classify)?;
    if !result.text().is_empty() {
        println!();
    }
    Ok(result)
}
