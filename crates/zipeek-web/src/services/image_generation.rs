use web_sys::Storage;

use crate::preferences::PreferencesStore;
pub(crate) const DEFAULT_URL: &str = "http://127.0.0.1:1431";

pub(crate) fn read_api_url(storage: &Storage) -> String {
    PreferencesStore::new(storage)
        .read()
        .image_server_url
        .unwrap_or_else(|| DEFAULT_URL.to_owned())
}

pub(crate) fn validate_api_url(url: &str) -> Result<&str, &'static str> {
    let url = url.trim().trim_end_matches('/');
    if !(url.starts_with("http://") || url.starts_with("https://"))
        || url.contains(char::is_whitespace)
        || url.contains('?')
        || url.contains('#')
    {
        return Err("Enter an HTTP or HTTPS URL without query parameters");
    }
    if url.split_once("://").is_some_and(|(_, address)| {
        address
            .split('/')
            .next()
            .is_some_and(|authority| authority.contains('@'))
    }) {
        return Err("Image server URLs must not contain embedded credentials");
    }
    if url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .is_empty()
    {
        return Err("An image server hostname is required");
    }
    Ok(url)
}

pub(crate) fn save_api_url(storage: &Storage, url: &str) -> Result<(), String> {
    let url = validate_api_url(url).map_err(str::to_owned)?;
    PreferencesStore::new(storage).update(|settings| {
        settings.image_server_url = Some(url.to_owned());
    })
}

pub(crate) fn reset_api_url(storage: &Storage) -> Result<(), String> {
    PreferencesStore::new(storage).update(|settings| settings.image_server_url = None)
}

mod client;
pub(crate) use client::generate;

mod tauri_image;
#[derive(serde::Deserialize)]
struct ImageHealth {
    status: String,
    backend: String,
}

pub(crate) fn describe_backend(backend: &str) -> Result<String, String> {
    match backend {
        "sdxl" => Ok("SDXL ready".to_owned()),
        "placeholder" => Err("Preview-only backend: no AI images are generated. Start the SDXL service with its model files.".into()),
        other => Err(format!("Unsupported image backend: {other}")),
    }
}

pub(crate) async fn require_image_backend(api_url: &str) -> Result<String, String> {
    let backend = if tauri_image::desktop_mode()? {
        tauri_image::health().await?
    } else {
        let base = validate_api_url(api_url).map_err(str::to_owned)?;
        let response = gloo::net::http::Request::get(&format!("{base}/health"))
            .send()
            .await
            .map_err(|error| error.to_string())?;
        if !response.ok() {
            return Err(format!(
                "Image service health check returned HTTP {}",
                response.status()
            ));
        }
        let health: ImageHealth = response
            .json()
            .await
            .map_err(|error| format!("Invalid image service health response: {error}"))?;
        if health.status != "ok" {
            return Err("Image service is not ready".into());
        }
        health.backend
    };
    describe_backend(&backend)
}

pub(crate) async fn check_health(api_url: &str) -> Result<String, String> {
    require_image_backend(api_url).await
}

#[cfg(test)]
mod backend_tests {
    use super::describe_backend;

    #[test]
    fn rejects_preview_or_unknown_backend_as_generation_ready() {
        assert_eq!(describe_backend("sdxl").unwrap(), "SDXL ready");
        assert!(
            describe_backend("placeholder")
                .unwrap_err()
                .contains("Preview-only")
        );
        assert!(describe_backend("unknown").is_err());
    }
}

pub(crate) fn release_preview(url: &str) {
    tauri_image::release_blob_url(url);
}
