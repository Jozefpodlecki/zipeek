use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use serde::Serialize;
use tauri::command;

use super::{error::CommandError, native_http};

const SPEECH: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8799));
const MAX_RESPONSE: usize = 8 * 1024 * 1024;

#[derive(Serialize)]
pub struct SpeechReply {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

#[command]
pub async fn speech_request(
    operation: String,
    bytes: Vec<u8>,
) -> Result<SpeechReply, CommandError> {
    let (method, path, content_type, limit) = match operation.as_str() {
        "capabilities" => ("GET", "/v1/capabilities", "text/plain", 0),
        "tts" => ("POST", "/v1/tts", "text/plain; charset=utf-8", 4096),
        "stt" => ("POST", "/v1/stt", "audio/wav", 960_044),
        _ => return Err(CommandError::Speech("Unsupported speech operation".into())),
    };
    if bytes.len() > limit {
        return Err(CommandError::Speech(
            "Speech request exceeds size limit".into(),
        ));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let (status, response) = native_http::request(
            SPEECH,
            path,
            method,
            content_type,
            &bytes,
            &[],
            MAX_RESPONSE,
            Duration::from_secs(120),
        )?;
        if status != 200 {
            return Err(format!("Speech service responded with HTTP {status}"));
        }
        if operation == "tts" && !(44..=MAX_RESPONSE).contains(&response.len()) {
            return Err("Invalid speech audio response".into());
        }
        Ok(SpeechReply {
            bytes: response,
            content_type: if operation == "tts" {
                "audio/wav"
            } else {
                "text/plain"
            },
        })
    })
    .await
    .map_err(|_| CommandError::Speech("Speech worker task stopped".into()))?
    .map_err(CommandError::Speech)
}
