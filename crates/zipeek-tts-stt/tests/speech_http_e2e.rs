mod common;

use common::{Server, voiced_wav};

#[test]
fn real_server_health_capabilities_and_api_contracts() {
    let server = Server::start_without_stt_model();
    let health = server.request("GET", "/health", &[], &[]);
    assert_eq!(health.status, 200);
    assert_eq!(health.body, b"ok");
    let capabilities = server.request("GET", "/v1/capabilities", &[], &[]);
    assert_eq!(capabilities.status, 200);
    let text = capabilities.text();
    assert!(text.contains("provider=local\n"));
    assert!(text.contains("language=zh-CN\n"));
    assert!(text.contains("synthesis=true\n") || text.contains("synthesis=false\n"));
    assert!(
        text.contains("recognition=false\n"),
        "STT missing-checkpoint test server must report unavailable"
    );

    let tts = server.request(
        "POST",
        "/v1/tts",
        &[("Content-Type", "text/plain; charset=utf-8")],
        "你好世界".as_bytes(),
    );
    if text.contains("synthesis=true\n") {
        assert_eq!(
            tts.status,
            200,
            "TTS advertised available: {}",
            tts.text_if_error()
        );
        assert_eq!(tts.header("content-type"), Some("audio/wav"));
        common::check_non_silent_wav(&tts.body);
    } else {
        assert_eq!(
            tts.status, 503,
            "unavailable TTS must explicitly return 503"
        );
    }

    let stt = server.request(
        "POST",
        "/v1/stt",
        &[("Content-Type", "audio/wav")],
        &voiced_wav(),
    );
    assert_eq!(stt.status, 503, "missing-checkpoint STT must return 503");
}

trait ErrorBody {
    fn text_if_error(&self) -> String;
}

impl ErrorBody for common::Reply {
    fn text_if_error(&self) -> String {
        if self.status == 200 {
            String::new()
        } else {
            String::from_utf8_lossy(&self.body).into_owned()
        }
    }
}
