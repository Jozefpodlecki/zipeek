mod common;

use common::{Server, voiced_wav};

#[test]
fn real_server_rejects_invalid_payloads_and_foreign_origins() {
    let server = Server::start_without_stt_model();
    let empty_tts = server.request(
        "POST",
        "/v1/tts",
        &[("Content-Type", "text/plain; charset=utf-8")],
        b" ",
    );
    assert_eq!(empty_tts.status, 400);
    let invalid_utf8 = server.request(
        "POST",
        "/v1/tts",
        &[("Content-Type", "text/plain; charset=utf-8")],
        &[0xff],
    );
    assert_eq!(invalid_utf8.status, 400);
    let wrong_tts_type = server.request(
        "POST",
        "/v1/tts",
        &[("Content-Type", "application/json")],
        b"hello",
    );
    assert_eq!(wrong_tts_type.status, 400);
    let invalid_wav = server.request(
        "POST",
        "/v1/stt",
        &[("Content-Type", "audio/wav")],
        b"invalid",
    );
    assert_eq!(invalid_wav.status, 400);
    let wrong_stt_type = server.request(
        "POST",
        "/v1/stt",
        &[("Content-Type", "application/octet-stream")],
        &voiced_wav(),
    );
    assert_eq!(wrong_stt_type.status, 400);
    let remote_origin = server.request(
        "GET",
        "/v1/capabilities",
        &[("Origin", "https://example.org")],
        &[],
    );
    assert_eq!(remote_origin.status, 400);
    let local_origin = server.request(
        "GET",
        "/v1/capabilities",
        &[("Origin", "http://localhost:1420")],
        &[],
    );
    assert_eq!(local_origin.status, 200);
    assert_eq!(
        local_origin.header("access-control-allow-origin"),
        Some("http://localhost:1420")
    );
    let dotted_local_origin = server.request(
        "GET",
        "/v1/capabilities",
        &[("Origin", "http://localhost.:1420")],
        &[],
    );
    assert_eq!(dotted_local_origin.status, 200);
    assert_eq!(
        dotted_local_origin.header("access-control-allow-origin"),
        Some("http://localhost.:1420")
    );
    let dotted_preflight = server.request(
        "OPTIONS",
        "/v1/tts",
        &[
            ("Origin", "http://localhost.:1420"),
            ("Access-Control-Request-Method", "POST"),
            ("Access-Control-Request-Headers", "content-type"),
        ],
        &[],
    );
    assert_eq!(dotted_preflight.status, 204);
    assert_eq!(
        dotted_preflight.header("access-control-allow-origin"),
        Some("http://localhost.:1420")
    );
    let preflight = server.request(
        "OPTIONS",
        "/v1/stt",
        &[
            ("Origin", "http://localhost:1420"),
            ("Access-Control-Request-Method", "POST"),
            ("Access-Control-Request-Headers", "content-type"),
        ],
        &[],
    );
    assert_eq!(preflight.status, 204);
    assert_eq!(
        preflight.header("access-control-allow-origin"),
        Some("http://localhost:1420")
    );
    let untrusted_preflight = server.request(
        "OPTIONS",
        "/v1/stt",
        &[
            ("Origin", "https://example.org"),
            ("Access-Control-Request-Method", "POST"),
            ("Access-Control-Request-Headers", "content-type"),
        ],
        &[],
    );
    assert_eq!(untrusted_preflight.status, 400);
}
