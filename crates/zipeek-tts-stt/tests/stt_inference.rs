mod common;

use std::time::Duration;

use common::{REFERENCE_TEXT, Server, normalize_transcript, read_reference_wav};

#[test]
#[ignore = "requires downloaded Qwen3-ASR safetensors; transcribes embedded Mandarin WAV with full neural inference"]
fn transcribes_actual_mandarin_speech_over_http() {
    assert!(
        std::env::var_os("ZIPEEK_CANDLE_STT_MODEL").is_some(),
        "set ZIPEEK_CANDLE_STT_MODEL"
    );
    let wav = read_reference_wav();
    let server = Server::start();
    let capabilities = server.request("GET", "/v1/capabilities", &[], &[]);
    assert_eq!(capabilities.status, 200);
    assert!(
        capabilities.text().contains("recognition=true\n"),
        "Qwen3-ASR checkpoint not available"
    );
    let reply = server
        .request_timeout(
            "POST",
            "/v1/stt",
            &[("Content-Type", "audio/wav")],
            &wav,
            Duration::from_secs(240),
        )
        .expect("real checkpoint STT HTTP request");
    assert_eq!(
        reply.status,
        200,
        "STT must transcribe rather than return an error: {}",
        String::from_utf8_lossy(&reply.body)
    );
    assert_eq!(
        reply.header("content-type"),
        Some("text/plain; charset=utf-8")
    );
    let actual = reply.text();
    assert!(
        !actual.trim().is_empty(),
        "STT returned empty transcription"
    );
    assert_eq!(
        normalize_transcript(&actual),
        normalize_transcript(REFERENCE_TEXT),
        "Mandarin transcription differs from actual reference speech"
    );
}
