mod common;

use std::{fs, path::PathBuf, time::Duration};

use common::{REFERENCE_TEXT, Server, check_non_silent_wav, normalize_transcript};

fn generate_mandarin_audio(server: &Server) -> Vec<u8> {
    let capabilities = server.request("GET", "/v1/capabilities", &[], &[]);
    assert_eq!(capabilities.status, 200);
    assert!(
        capabilities.text().contains("synthesis=true\n"),
        "Mandarin TTS is unavailable; place Qwen3-TTS weights under models/speech or configure eSpeak"
    );
    let response = server
        .request_timeout(
            "POST",
            "/v1/tts",
            &[
                ("Content-Type", "text/plain; charset=utf-8"),
                ("Accept", "audio/wav"),
            ],
            REFERENCE_TEXT.as_bytes(),
            Duration::from_secs(360),
        )
        .expect("request Mandarin speech from the running TTS API");
    assert_eq!(
        response.status,
        200,
        "Mandarin TTS failed: {}",
        String::from_utf8_lossy(&response.body)
    );
    assert_eq!(response.header("content-type"), Some("audio/wav"));
    check_non_silent_wav(&response.body);
    response.body
}

fn save_generated_audio(wav: &[u8]) -> PathBuf {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/speech-e2e/mandarin_nihao_shijie.wav");
    fs::create_dir_all(output.parent().expect("fixture directory"))
        .expect("create generated fixture output directory");
    fs::write(&output, wav).expect("save the TTS-generated WAV bytes");
    eprintln!("Generated Mandarin TTS WAV: {}", output.display());
    output
}

#[test]
#[ignore = "requires Qwen3-TTS model or working eSpeak; generates a WAV on disk"]
fn generates_mandarin_nihao_shijie_wav_from_tts_api() {
    let server = Server::start_without_stt_model();
    let wav = generate_mandarin_audio(&server);
    let output = save_generated_audio(&wav);
    assert_eq!(fs::read(output).expect("read generated WAV"), wav);
}

#[test]
#[ignore = "requires working Mandarin TTS and Qwen3-ASR checkpoint; runs real TTS -> STT inference"]
fn generated_mandarin_tts_audio_transcribes_over_stt_api() {
    let server = Server::start();
    let wav = generate_mandarin_audio(&server);
    let output = save_generated_audio(&wav);
    let capabilities = server.request("GET", "/v1/capabilities", &[], &[]);
    assert!(
        capabilities.text().contains("recognition=true\n"),
        "Qwen3-ASR is unavailable; put the checkpoint under models/speech/Qwen3-ASR-0.6B"
    );
    let response = server
        .request_timeout(
            "POST",
            "/v1/stt",
            &[("Content-Type", "audio/wav")],
            &wav,
            Duration::from_secs(240),
        )
        .expect("transcribe the generated WAV through the live STT HTTP API");
    assert_eq!(
        response.status,
        200,
        "STT failed for {}: {}",
        output.display(),
        String::from_utf8_lossy(&response.body)
    );
    assert_eq!(
        response.header("content-type"),
        Some("text/plain; charset=utf-8")
    );
    let actual = response.text();
    assert!(!actual.trim().is_empty(), "STT returned empty transcript");
    assert_eq!(
        normalize_transcript(&actual),
        normalize_transcript(REFERENCE_TEXT),
        "TTS -> STT mismatch; listen to {} to verify Mandarin pronunciation",
        output.display()
    );
}
