use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::command;

use super::{error::CommandError, native_http};

const ADDRESS: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1431));
const MAX_CONTROL_BODY: usize = 128 * 1024;
const MAX_IMAGE_BODY: usize = 8 * 1024 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(10);
const IMAGE_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_POLLS: usize = 90;

#[derive(Deserialize)]
struct ImageJob {
    id: String,
    status: String,
    image_url: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct ImagePayload {
    bytes: Vec<u8>,
    mime: &'static str,
}

#[command]
pub async fn image_health() -> Result<String, CommandError> {
    tauri::async_runtime::spawn_blocking(|| {
        let response = request("GET", "/health", &[])?;
        let status: serde_json::Value = serde_json::from_slice(&response)
            .map_err(|error| format!("Invalid image health response: {error}"))?;
        if status.get("status").and_then(|value| value.as_str()) != Some("ok") {
            return Err("Image service is not ready".to_owned());
        }
        Ok(status
            .get("backend")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_owned())
    })
    .await
    .map_err(|error| CommandError::Image(format!("Image health task failed: {error}")))?
    .map_err(CommandError::Image)
}

#[command]
pub async fn image_generate(prompt: String) -> Result<ImagePayload, CommandError> {
    tauri::async_runtime::spawn_blocking(move || generate(&prompt))
        .await
        .map_err(|error| CommandError::Image(format!("Image generation task failed: {error}")))?
        .map_err(CommandError::Image)
}

fn generate(prompt: &str) -> Result<ImagePayload, String> {
    if prompt.trim().is_empty() || prompt.chars().count() > 500 {
        return Err("Image prompt must contain 1 to 500 characters".into());
    }
    require_inference_backend(&image_backend()?)?;
    let body = serde_json::to_vec(&serde_json::json!({
        "prompt": prompt,
        "width": 512,
        "height": 512,
    }))
    .map_err(|error| error.to_string())?;
    let submitted: ImageJob = serde_json::from_slice(&request("POST", "/v1/images", &body)?)
        .map_err(|error| format!("Invalid image job response: {error}"))?;
    if !valid_id(&submitted.id)
        || !matches!(
            submitted.status.as_str(),
            "queued" | "running" | "completed"
        )
    {
        return Err("Image service returned an invalid job".into());
    }
    for _ in 0..MAX_POLLS {
        let job: ImageJob =
            serde_json::from_slice(&request("GET", &format!("/v1/jobs/{}", submitted.id), &[])?)
                .map_err(|error| format!("Invalid image status response: {error}"))?;
        if job.id != submitted.id {
            return Err("Image service returned a mismatched job identifier".into());
        }
        match job.status.as_str() {
            "completed" => {
                if job.image_url.as_deref() != Some(format!("/v1/images/{}", job.id).as_str()) {
                    return Err("Image service returned an unexpected image URL".into());
                }
                return download_image(&job.id);
            }
            "failed" => {
                return Err(job
                    .error
                    .unwrap_or_else(|| "Image generation failed".into())
                    .chars()
                    .filter(|ch| !ch.is_control())
                    .take(300)
                    .collect());
            }
            "queued" | "running" => thread::sleep(POLL_INTERVAL),
            _ => return Err("Image service returned an unknown job status".into()),
        }
    }
    Err("Image generation exceeded the 180-second limit".into())
}

fn image_backend() -> Result<String, String> {
    let response = request("GET", "/health", &[])?;
    let status: serde_json::Value = serde_json::from_slice(&response)
        .map_err(|error| format!("Invalid image health response: {error}"))?;
    if status.get("status").and_then(|value| value.as_str()) != Some("ok") {
        return Err("Image service is not ready".into());
    }
    Ok(status
        .get("backend")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_owned())
}

fn require_inference_backend(backend: &str) -> Result<(), String> {
    match backend {
        "sdxl" => Ok(()),
        "placeholder" => Err("Image service is in preview-only mode. Start the SDXL backend with model files before generating images.".into()),
        other => Err(format!("Unsupported image generation backend: {other}")),
    }
}

fn download_image(id: &str) -> Result<ImagePayload, String> {
    if !valid_id(id) {
        return Err("Invalid image identifier".into());
    }
    let (status, bytes) = native_http::request(
        ADDRESS,
        &format!("/v1/images/{id}"),
        "GET",
        "application/octet-stream",
        &[],
        &[],
        MAX_IMAGE_BODY,
        IMAGE_TIMEOUT,
    )?;
    if status != 200 {
        return Err(format!("Image service returned HTTP {status}"));
    }
    let mime = image_mime(&bytes)?;
    Ok(ImagePayload { bytes, mime })
}

fn image_mime(bytes: &[u8]) -> Result<&'static str, String> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok("image/png");
    }
    if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        return Ok("image/svg+xml");
    }
    Err("Image server returned an unsupported format".into())
}

fn valid_id(value: &str) -> bool {
    value.len() == 36
        && value.chars().enumerate().all(|(index, value)| {
            if [8, 13, 18, 23].contains(&index) {
                value == '-'
            } else {
                value.is_ascii_hexdigit()
            }
        })
}

fn request(method: &str, path: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    if !(path == "/health" || path == "/v1/images" || path.starts_with("/v1/jobs/")) {
        return Err("Invalid native image request".into());
    }
    let (status, bytes) = native_http::request(
        ADDRESS,
        path,
        method,
        "application/json",
        body,
        &[],
        MAX_CONTROL_BODY,
        CONTROL_TIMEOUT,
    )?;
    let expected = if method == "POST" { 202 } else { 200 };
    if status != expected {
        return Err(format!("Image service returned HTTP {status}"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_backend_is_never_treated_as_ai_image_generation() {
        assert!(require_inference_backend("sdxl").is_ok());
        assert!(require_inference_backend("placeholder").is_err());
        assert!(require_inference_backend("unknown").is_err());
    }

    #[test]
    fn rejects_invalid_image_ids() {
        assert!(valid_id("01234567-89ab-cdef-0123-456789abcdef"));
        assert!(!valid_id("../unexpected"));
        assert!(!valid_id("01234567-89ab-cdef-0123-456789abcdeg"));
        assert!(download_image("../unexpected").is_err());
    }

    #[test]
    fn validates_image_bytes_before_passing_to_webview() {
        assert_eq!(
            image_mime(b"\x89PNG\r\n\x1a\nwhatever").unwrap(),
            "image/png"
        );
        assert_eq!(
            image_mime(b"<svg xmlns='test'></svg>").unwrap(),
            "image/svg+xml"
        );
        assert!(image_mime(b"<html>unexpected</html>").is_err());
        assert!(image_mime(b"GIF89a").is_err());
    }

    #[test]
    fn refuses_arbitrary_native_image_paths() {
        assert!(request("GET", "http://example.com/", &[]).is_err());
        assert!(request("GET", "/v1/images/anything", &[]).is_err());
    }
}
