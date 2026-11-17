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
