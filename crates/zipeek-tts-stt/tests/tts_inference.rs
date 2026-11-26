mod common;

use common::{Server, check_non_silent_wav};

#[test]
#[ignore = "requires Qwen3-TTS model weights or configured eSpeak resources; produces real audio"]
fn produces_real_mandarin_audio_over_http() {
    let server = Server::start();
    let capabilities = server.request("GET", "/v1/capabilities", &[], &[]);
    assert_eq!(capabilities.status, 200);
    assert!(
        capabilities.text().contains("synthesis=true\n"),
        "TTS resources unavailable"
    );
    let result = server.request(
        "POST",
        "/v1/tts",
        &[("Content-Type", "text/plain; charset=utf-8")],
        "你好，世界。".as_bytes(),
    );
    assert_eq!(
        result.status,
        200,
        "TTS must work, not merely return unavailable: {}",
        String::from_utf8_lossy(&result.body)
    );
    assert_eq!(result.header("content-type"), Some("audio/wav"));
    check_non_silent_wav(&result.body);
}
