use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc};

use axum::{
    Router, middleware::{self, Next},
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, HeaderValue, Method, Request, StatusCode, header},
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
    if let Err(error) = validate_request(&headers) { return error_response(error); }
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8"), (header::CACHE_CONTROL, "no-store")], "ok").into_response()
}

async fn capabilities(State(speech): State<Arc<Speech>>, headers: HeaderMap) -> Response {
    if let Err(error) = validate_request(&headers) { return error_response(error); }
    let details = format!(
        "provider=local\nlanguage=zh-CN\nsynthesis={}\nrecognition={}\n",
        speech.synthesis_available(),
        speech.recognition_available()
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8"), (header::CACHE_CONTROL, "no-store")], details).into_response()
}

fn error_response(error: SpeechError) -> Response {
    let status = match error {
        SpeechError::Invalid(_) => StatusCode::BAD_REQUEST,
        SpeechError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        SpeechError::Busy => StatusCode::TOO_MANY_REQUESTS,
        SpeechError::Failed(_) | SpeechError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, [(header::CONTENT_TYPE, "text/plain; charset=utf-8"), (header::CACHE_CONTROL, "no-store")], error.to_string()).into_response()
}

const TRUSTED_BROWSER_ORIGINS: [&str; 2] = [
    "http://localhost:1420",
    "http://127.0.0.1:1420",
];

fn trusted_browser_origin(headers: &HeaderMap) -> Result<Option<HeaderValue>, SpeechError> {
    let mut origins = headers.get_all(header::ORIGIN).iter();
    let Some(origin) = origins.next() else { return Ok(None) };
    if origins.next().is_some() {
        return Err(SpeechError::Invalid("multiple Origin headers are not allowed"));
    }
    let raw = origin.to_str().map_err(|_| SpeechError::Invalid("invalid Origin"))?;
    if !TRUSTED_BROWSER_ORIGINS.contains(&raw) {
        return Err(SpeechError::Invalid("browser origin is not an approved local Zipeek origin"));
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
        if origin.is_none() || request.headers()
            .get(header::ACCESS_CONTROL_REQUEST_METHOD)
            .is_none_or(|value| value != "POST")
        {
            return StatusCode::FORBIDDEN.into_response();
        }
        if request.headers().get(header::ACCESS_CONTROL_REQUEST_HEADERS)
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| !value.eq_ignore_ascii_case("content-type"))
        {
            return StatusCode::FORBIDDEN.into_response();
        }
        let mut response = StatusCode::NO_CONTENT.into_response();
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("POST"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("content-type"));
        headers.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("600"));
        if request.headers().get("access-control-request-private-network")
            .is_some_and(|value| value == "true") {
            headers.insert("access-control-allow-private-network", HeaderValue::from_static("true"));
        }
        if let Some(origin) = origin { headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin); }
        headers.insert(header::VARY, HeaderValue::from_static("Origin"));
        return response;
    }
    let mut response = next.run(request).await;
    if let Some(origin) = origin {
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response.headers_mut().insert(header::VARY, HeaderValue::from_static("Origin"));
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

async fn synthesize(State(speech): State<Arc<Speech>>, headers: HeaderMap, body: Bytes) -> Response {
    if let Err(error) = validate_request(&headers) { return error_response(error); }
    if headers.get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()) != Some("text/plain; charset=utf-8") {
        return error_response(SpeechError::Invalid("expected text/plain; charset=utf-8"));
    }
    if body.len() > MAX_TEXT_BYTES {
        return error_response(SpeechError::Invalid("text payload too large"));
    }
    let text = match std::str::from_utf8(&body) {
        Ok(text) => text,
        Err(_) => return error_response(SpeechError::Invalid("text must be UTF-8")),
    };
    match speech.synthesize(text).await {
        Ok(wav) => ([(header::CONTENT_TYPE, "audio/wav"), (header::CACHE_CONTROL, "no-store")], wav).into_response(),
        Err(error) => error_response(error),
    }
}

async fn transcribe(State(speech): State<Arc<Speech>>, headers: HeaderMap, body: Bytes) -> Response {
    if let Err(error) = validate_request(&headers) { return error_response(error); }
    if headers.get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()) != Some("audio/wav") {
        return error_response(SpeechError::Invalid("expected audio/wav"));
    }
    match speech.transcribe(body.to_vec()).await {
        Ok(text) => ([(header::CONTENT_TYPE, "text/plain; charset=utf-8"), (header::CACHE_CONTROL, "no-store")], text).into_response(),
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
        for host in ["example.org", "example.org:8799", "127.0.0.1.evil.test", "localhost:0", "localhost:abc", "localhost:65536", "[::1]:8799", "localhost:8799:1"] {
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
            PathBuf::from("missing-espeak"), PathBuf::from("missing-data"), PathBuf::from("missing-model"),
        ));
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "127.0.0.1:8799".parse().unwrap());
        headers.insert(header::CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
        assert_eq!(synthesize(State(speech.clone()), headers.clone(), Bytes::from_static(b" ")).await.status(), StatusCode::BAD_REQUEST);
        assert_eq!(synthesize(State(speech.clone()), headers.clone(), Bytes::from_static(b"hello")).await.status(), StatusCode::SERVICE_UNAVAILABLE);
        headers.insert(header::ORIGIN, "https://example.org".parse().unwrap());
        assert_eq!(synthesize(State(speech.clone()), headers, Bytes::from_static(b"hello")).await.status(), StatusCode::BAD_REQUEST);
        let mut audio_headers = HeaderMap::new();
        audio_headers.insert(header::HOST, "localhost:8799".parse().unwrap());
        audio_headers.insert(header::CONTENT_TYPE, "audio/wav".parse().unwrap());
        assert_eq!(transcribe(State(speech), audio_headers, Bytes::from_static(b"not a wav")).await.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn status_endpoints_reject_nonlocal_hosts_and_browser_origins() {
        use std::path::PathBuf;
        let speech = Arc::new(Speech::with_paths(
            PathBuf::from("missing-espeak"), PathBuf::from("missing-data"), PathBuf::from("missing-model"),
        ));
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "127.0.0.1:8799".parse().unwrap());
        let response = health(headers.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(capabilities(State(speech.clone()), headers.clone()).await.status(), StatusCode::OK);
        headers.insert(header::ORIGIN, "https://example.org".parse().unwrap());
        assert_eq!(health(headers.clone()).await.status(), StatusCode::BAD_REQUEST);
        assert_eq!(capabilities(State(speech.clone()), headers).await.status(), StatusCode::BAD_REQUEST);
        let mut remote = HeaderMap::new();
        remote.insert(header::HOST, "remote.example".parse().unwrap());
        assert_eq!(health(remote).await.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn hosted_and_other_local_origins_cannot_access_native_audio() {
        for origin in ["https://example.com", "https://localhost:1420", "http://localhost:1421", "http://127.0.0.1:8799", "http://localhost:1420.evil.test", "null"] {
            let mut headers = HeaderMap::new();
            headers.insert(header::HOST, "localhost:8799".parse().unwrap());
            headers.insert(header::ORIGIN, origin.parse().unwrap());
            assert!(validate_request(&headers).is_err(), "{origin}");
        }
    }

    #[test]
    fn invalid_port_is_rejected() {
        assert!("invalid".parse::<u16>().is_err());
    }
}
