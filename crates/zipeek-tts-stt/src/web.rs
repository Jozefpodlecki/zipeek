use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::speech::{MAX_AUDIO_BYTES, MAX_TEXT_BYTES, Speech, SpeechError};

pub fn router(speech: Speech) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/capabilities", get(capabilities))
        .route("/v1/tts", post(synthesize).options(preflight))
        .route("/v1/stt", post(transcribe).options(preflight))
        .route("/v1/stt/live", get(crate::live_stt::upgrade))
        .layer(DefaultBodyLimit::max(MAX_AUDIO_BYTES.max(MAX_TEXT_BYTES)))
        .layer(middleware::from_fn(browser_bridge))
        .with_state(Arc::new(speech))
}

pub fn bind_address() -> Result<SocketAddr, String> {
    let port = std::env::var("ZIPEEK_SPEECH_PORT")
        .unwrap_or_else(|_| "8799".into())
        .parse::<u16>()
        .map_err(|_| "ZIPEEK_SPEECH_PORT must be a valid TCP port".to_owned())?;
    if port == 0 {
        return Err("ZIPEEK_SPEECH_PORT must not be zero".into());
    }
    Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
}

async fn health(headers: HeaderMap) -> Response {
    if let Err(error) = validate_request(&headers) {
        return error_response(error);
    }
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        "ok",
    )
        .into_response()
}

async fn capabilities(State(speech): State<Arc<Speech>>, headers: HeaderMap) -> Response {
    if let Err(error) = validate_request(&headers) {
        return error_response(error);
    }
    let details = format!(
        "provider=local\nlanguage=zh-CN\nsynthesis={}\nrecognition={}\nneural_tts_loaded={}\nneural_stt_loaded={}\nneural_tts_status={}\nneural_stt_status={}\nneural_tts_failure={}\nneural_stt_failure={}\n",
        speech.synthesis_available(),
        speech.recognition_available(),
        speech.neural_tts_engine_loaded(),
        speech.neural_stt_engine_loaded(),
        speech.neural_tts_status(),
        speech.neural_stt_status(),
        speech.neural_tts_failure(),
        speech.neural_stt_failure()
    );
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        details,
    )
        .into_response()
}

fn error_response(error: SpeechError) -> Response {
    let status = match &error {
        SpeechError::Invalid(_) => StatusCode::BAD_REQUEST,
        SpeechError::Unavailable(_) | SpeechError::ModelLoad(_) => StatusCode::SERVICE_UNAVAILABLE,
        SpeechError::Busy => StatusCode::TOO_MANY_REQUESTS,
        SpeechError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
        SpeechError::Cancelled => StatusCode::REQUEST_TIMEOUT,
        SpeechError::Inference(_) | SpeechError::Codec(_) | SpeechError::Io(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    let public_message = match &error {
        SpeechError::Invalid(_)
        | SpeechError::Unavailable(_)
        | SpeechError::Busy
        | SpeechError::Timeout(_)
        | SpeechError::Cancelled => error.to_string(),
        SpeechError::ModelLoad(_) => "speech model unavailable".to_owned(),
        SpeechError::Inference(_) => "speech inference failed".to_owned(),
        SpeechError::Codec(_) => "speech audio conversion failed".to_owned(),
        SpeechError::Io(_) => "speech service failed".to_owned(),
    };
    if matches!(
        &error,
        SpeechError::ModelLoad(_)
            | SpeechError::Inference(_)
            | SpeechError::Codec(_)
            | SpeechError::Io(_)
    ) {
        tracing::error!(error = %error, "speech request failed");
    }
    (
        status,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        public_message,
    )
        .into_response()
}

const TRUSTED_BROWSER_ORIGINS: [&str; 7] = [
    "http://localhost:1420",
    "http://localhost.:1420",
    "http://127.0.0.1:1420",
    "http://localhost:1421",
    "http://127.0.0.1:1421",
    "http://tauri.localhost",
    "tauri://localhost",
];

fn trusted_browser_origin(headers: &HeaderMap) -> Result<Option<HeaderValue>, SpeechError> {
    let mut origins = headers.get_all(header::ORIGIN).iter();
    let Some(origin) = origins.next() else {
        return Ok(None);
    };
    if origins.next().is_some() {
        return Err(SpeechError::Invalid(
            "multiple Origin headers are not allowed",
        ));
    }
    let raw = origin
        .to_str()
        .map_err(|_| SpeechError::Invalid("invalid Origin"))?;
    if !TRUSTED_BROWSER_ORIGINS.contains(&raw) {
        return Err(SpeechError::Invalid(
            "browser origin is not an approved local Zipeek origin",
        ));
    }
    Ok(Some(origin.clone()))
}

fn validate_origin(headers: &HeaderMap) -> Result<(), SpeechError> {
    trusted_browser_origin(headers).map(|_| ())
}

async fn browser_bridge(request: Request<Body>, next: Next) -> Response {
    if let Err(error) = validate_request(request.headers()) {
        return error_response(error);
    }
    let origin = trusted_browser_origin(request.headers()).ok().flatten();
    if request.method() == Method::OPTIONS {
        if request.uri().path() != "/v1/tts" && request.uri().path() != "/v1/stt" {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }
        if origin.is_none()
            || request
                .headers()
                .get(header::ACCESS_CONTROL_REQUEST_METHOD)
                .is_none_or(|value| value != "POST")
        {
            return StatusCode::FORBIDDEN.into_response();
        }
        if request
            .headers()
            .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| !value.eq_ignore_ascii_case("content-type"))
        {
            return StatusCode::FORBIDDEN.into_response();
        }
        let mut response = StatusCode::NO_CONTENT.into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("POST"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("content-type"),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            HeaderValue::from_static("600"),
        );
        if request
            .headers()
            .get("access-control-request-private-network")
            .is_some_and(|value| value == "true")
        {
            headers.insert(
                "access-control-allow-private-network",
                HeaderValue::from_static("true"),
            );
        }
        if let Some(origin) = origin {
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        }
        headers.insert(header::VARY, HeaderValue::from_static("Origin"));
        return response;
    }
    let mut response = next.run(request).await;
    if let Some(origin) = origin {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    response
}

async fn preflight() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

fn validate_loopback_host(headers: &HeaderMap) -> Result<(), SpeechError> {
    let invalid = || SpeechError::Invalid("request Host must be localhost or 127.0.0.1");
    let mut hosts = headers.get_all(header::HOST).iter();
    let host = hosts.next().ok_or_else(invalid)?;
    if hosts.next().is_some() {
        return Err(invalid());
    }
    let host = host.to_str().map_err(|_| invalid())?;
    let (hostname, port) = match host.split_once(':') {
        Some((hostname, port)) => (hostname, Some(port)),
        None => (host, None),
    };
    if hostname != "localhost" && hostname != "127.0.0.1" {
        return Err(invalid());
    }
    if let Some(port) = port {
        if port.is_empty() || port.parse::<u16>().ok().filter(|port| *port != 0).is_none() {
            return Err(invalid());
        }
    }
    Ok(())
}

fn validate_request(headers: &HeaderMap) -> Result<(), SpeechError> {
    validate_origin(headers)?;
    validate_loopback_host(headers)
}

fn choose_audio_format(accept: Option<&str>) -> Option<&'static str> {
    let Some(value) = accept else {
        return Some("wav");
    };
    let mut best: Option<(&'static str, u16, u8)> = None;
    for range in value.split(',') {
        let mut parts = range.trim().split(';');
        let mime = parts.next().unwrap_or("").trim().to_ascii_lowercase();
        let mut quality = 1000u16;
        let mut invalid = false;
        for parameter in parts {
            let Some((name, value)) = parameter.trim().split_once('=') else {
                continue;
            };
            if name.trim().eq_ignore_ascii_case("q") {
                match value.trim().parse::<f64>() {
                    Ok(q) if (0.0..=1.0).contains(&q) => {
                        quality = (q * 1000.0).round() as u16;
                    }
                    _ => invalid = true,
                }
            }
        }
        if invalid || quality == 0 {
            continue;
        }
        let (format, specificity) = match mime.as_str() {
            "audio/wav" | "audio/x-wav" => ("wav", 2),
            "audio/ogg" | "audio/opus" => ("opus", 2),
            "audio/mpeg" | "audio/mp3" => ("mp3", 2),
            "audio/*" => ("wav", 1),
            "*/*" => ("wav", 0),
            _ => continue,
        };
        if best.is_none_or(|(_, best_quality, best_specificity)| {
            quality > best_quality || quality == best_quality && specificity > best_specificity
        }) {
            best = Some((format, quality, specificity));
        }
    }
    best.map(|(format, _, _)| format)
}

async fn synthesize(
    State(speech): State<Arc<Speech>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(error) = validate_request(&headers) {
        return error_response(error);
    }
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("text/plain; charset=utf-8")
    {
        return error_response(SpeechError::Invalid("expected text/plain; charset=utf-8"));
    }
    if body.len() > MAX_TEXT_BYTES {
        return error_response(SpeechError::Invalid("text payload too large"));
    }
    let text = match std::str::from_utf8(&body) {
        Ok(text) => text,
        Err(_) => return error_response(SpeechError::Invalid("text must be UTF-8")),
    };
    let Some(requested_format) = choose_audio_format(
        headers
            .get(header::ACCEPT)
            .and_then(|value| value.to_str().ok()),
    ) else {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    };
    match speech.synthesize(text).await {
        Ok(wav) => match requested_format {
            "opus" => {
                let encoded =
                    tokio::task::spawn_blocking(move || crate::audio_codec::wav_to_ogg_opus(&wav))
                        .await;
                match encoded {
                    Ok(Ok(opus)) => (
                        [
                            (header::CONTENT_TYPE, "audio/ogg"),
                            (header::CACHE_CONTROL, "no-store"),
                        ],
                        opus,
                    )
                        .into_response(),
                    Ok(Err(error)) => error_response(error),
                    Err(error) => {
                        error_response(SpeechError::Codec(format!("Opus worker failed: {error}")))
                    }
                }
            }
            "mp3" => {
                let encoded =
                    tokio::task::spawn_blocking(move || crate::audio_codec::wav_to_mp3(&wav)).await;
                match encoded {
                    Ok(Ok(mp3)) => (
                        [
                            (header::CONTENT_TYPE, "audio/mpeg"),
                            (header::CACHE_CONTROL, "no-store"),
                        ],
                        mp3,
                    )
                        .into_response(),
                    Ok(Err(error)) => error_response(error),
                    Err(error) => {
                        error_response(SpeechError::Codec(format!("MP3 worker failed: {error}")))
                    }
                }
            }
            _ => (
                [
                    (header::CONTENT_TYPE, "audio/wav"),
                    (header::CACHE_CONTROL, "no-store"),
                ],
                wav,
            )
                .into_response(),
        },
        Err(error) => error_response(error),
    }
}

async fn transcribe(
    State(speech): State<Arc<Speech>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(error) = validate_request(&headers) {
        return error_response(error);
    }
    let format = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(crate::audio_codec::InputFormat::from_content_type);
    let Some(format) = format else {
        return error_response(SpeechError::Invalid(
            "expected audio/wav, audio/mpeg, audio/flac or audio/ogg",
        ));
    };
    match speech.transcribe_audio(body.to_vec(), format).await {
        Ok(text) => (
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            text,
        )
            .into_response(),
        Err(error) => error_response(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_local_development_browser_can_access_audio() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "https://example.org".parse().unwrap());
        assert!(validate_origin(&headers).is_err());
        headers.insert(header::ORIGIN, "http://localhost:1420".parse().unwrap());
        assert!(validate_origin(&headers).is_ok());
        headers.insert(header::ORIGIN, "http://localhost.:1420".parse().unwrap());
        assert!(validate_origin(&headers).is_ok());
        headers.insert(header::ORIGIN, "http://127.0.0.1:1420".parse().unwrap());
        assert!(validate_origin(&headers).is_ok());
        headers.insert(header::ORIGIN, "https://localhost:1420".parse().unwrap());
        assert!(validate_origin(&headers).is_err());
        headers.remove(header::ORIGIN);
        assert!(validate_origin(&headers).is_ok());
    }

    #[test]
    fn only_local_loopback_host_is_accepted() {
        for host in ["localhost", "localhost:8799", "127.0.0.1", "127.0.0.1:8799"] {
            let mut headers = HeaderMap::new();
            headers.insert(header::HOST, host.parse().unwrap());
            assert!(validate_request(&headers).is_ok(), "{host}");
        }
        for host in [
            "example.org",
            "example.org:8799",
            "127.0.0.1.evil.test",
            "localhost:0",
            "localhost:abc",
            "localhost:65536",
            "[::1]:8799",
            "localhost:8799:1",
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(header::HOST, host.parse().unwrap());
            assert!(validate_request(&headers).is_err(), "{host}");
        }
        let mut headers = HeaderMap::new();
        assert!(validate_request(&headers).is_err());
        headers.insert(header::HOST, "localhost:8799".parse().unwrap());
        headers.insert(header::ORIGIN, "http://localhost:8799".parse().unwrap());
        assert!(validate_request(&headers).is_err());
    }

    #[tokio::test]
    async fn local_speech_routes_reject_malformed_requests_before_invoking_engines() {
        use std::path::PathBuf;
        let speech = Arc::new(Speech::with_paths(
            PathBuf::from("missing-espeak"),
            PathBuf::from("missing-data"),
            PathBuf::from("missing-model"),
        ));
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "127.0.0.1:8799".parse().unwrap());
        headers.insert(
            header::CONTENT_TYPE,
            "text/plain; charset=utf-8".parse().unwrap(),
        );
        assert_eq!(
            synthesize(
                State(speech.clone()),
                headers.clone(),
                Bytes::from_static(b" ")
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            synthesize(
                State(speech.clone()),
                headers.clone(),
                Bytes::from_static(b"hello")
            )
            .await
            .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        headers.insert(header::ORIGIN, "https://example.org".parse().unwrap());
        assert_eq!(
            synthesize(State(speech.clone()), headers, Bytes::from_static(b"hello"))
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
        let mut audio_headers = HeaderMap::new();
        audio_headers.insert(header::HOST, "localhost:8799".parse().unwrap());
        audio_headers.insert(header::CONTENT_TYPE, "audio/wav".parse().unwrap());
        assert_eq!(
            transcribe(
                State(speech),
                audio_headers,
                Bytes::from_static(b"not a wav")
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn status_endpoints_reject_nonlocal_hosts_and_browser_origins() {
        use std::path::PathBuf;
        let speech = Arc::new(Speech::with_paths(
            PathBuf::from("missing-espeak"),
            PathBuf::from("missing-data"),
            PathBuf::from("missing-model"),
        ));
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "127.0.0.1:8799".parse().unwrap());
        let response = health(headers.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            capabilities(State(speech.clone()), headers.clone())
                .await
                .status(),
            StatusCode::OK
        );
        headers.insert(header::ORIGIN, "https://example.org".parse().unwrap());
        assert_eq!(
            health(headers.clone()).await.status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            capabilities(State(speech.clone()), headers).await.status(),
            StatusCode::BAD_REQUEST
        );
        let mut remote = HeaderMap::new();
        remote.insert(header::HOST, "remote.example".parse().unwrap());
        assert_eq!(health(remote).await.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn hosted_and_other_local_origins_cannot_access_native_audio() {
        for origin in [
            "https://example.com",
            "https://localhost:1420",
            "http://localhost.:1421",
            "http://localhost..:1420",
            "http://localhost.:1420.evil.test",
            "http://127.0.0.1:8799",
            "http://localhost:1420.evil.test",
            "null",
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(header::HOST, "localhost:8799".parse().unwrap());
            headers.insert(header::ORIGIN, origin.parse().unwrap());
            assert!(validate_request(&headers).is_err(), "{origin}");
        }
    }

    #[tokio::test]
    async fn public_errors_do_not_reveal_internal_inference_or_model_paths() {
        for (error, status, public_message) in [
            (
                SpeechError::ModelLoad("C:/secret/model.safetensors".into()),
                StatusCode::SERVICE_UNAVAILABLE,
                "speech model unavailable",
            ),
            (
                SpeechError::Inference("private tensor layout".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "speech inference failed",
            ),
            (
                SpeechError::Codec("private codec exception".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "speech audio conversion failed",
            ),
        ] {
            let response = error_response(error);
            assert_eq!(response.status(), status);
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            assert_eq!(body.as_ref(), public_message.as_bytes());
        }
    }

    #[test]
    fn cancellation_has_distinct_http_status() {
        assert_eq!(
            error_response(SpeechError::Cancelled).status(),
            StatusCode::REQUEST_TIMEOUT
        );
    }

    #[test]
    fn timeouts_have_distinct_http_status() {
        let response = error_response(SpeechError::Timeout("inference deadline exceeded"));
        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    }

    #[test]
    fn accepts_weighted_audio_content_negotiation() {
        assert_eq!(choose_audio_format(None), Some("wav"));
        assert_eq!(choose_audio_format(Some("audio/mpeg")), Some("mp3"));
        assert_eq!(
            choose_audio_format(Some("audio/ogg; q=0.9, audio/mpeg; q=0.4")),
            Some("opus")
        );
        assert_eq!(
            choose_audio_format(Some("audio/ogg;q=0, audio/wav;q=1")),
            Some("wav")
        );
        assert_eq!(choose_audio_format(Some("audio/*")), Some("wav"));
        assert_eq!(choose_audio_format(Some("audio/aac")), None);
        assert_eq!(choose_audio_format(Some("audio/mpeg;q=0")), None);
        assert_eq!(choose_audio_format(Some("audio/mpeg;q=nan")), None);
    }

    #[test]
    fn invalid_port_is_rejected() {
        assert!("invalid".parse::<u16>().is_err());
    }
}
