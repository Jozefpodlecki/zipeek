use reqwest::Url;

use crate::AiClientError;

pub(super) fn chat_url(endpoint: &str, api_key: &str) -> Result<Url, AiClientError> {
    let mut url = Url::parse(endpoint.trim())
        .map_err(|_| AiClientError("Enter a valid AI endpoint URL in Settings".to_owned()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AiClientError(
            "AI endpoint must use HTTP or HTTPS".to_owned(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AiClientError(
            "Do not place credentials in the endpoint URL".to_owned(),
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(AiClientError(
            "Use the API base URL without query parameters or a fragment".to_owned(),
        ));
    }
    if !api_key.is_empty() && url.scheme() == "http" {
        let local = matches!(
            url.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        );
        if !local {
            return Err(AiClientError(
                "Refusing to send an API key over unencrypted remote HTTP".to_owned(),
            ));
        }
    }
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    url.join("chat/completions")
        .map_err(|_| AiClientError("Invalid AI endpoint path".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::chat_url;

    #[test]
    fn preserves_base_path_and_rejects_credential_leaks() {
        assert_eq!(
            chat_url("http://localhost:1234/v1", "key")
                .unwrap()
                .as_str(),
            "http://localhost:1234/v1/chat/completions"
        );
        for url in [
            "file:///tmp",
            "https://user:password@example.test/v1/",
            "https://example.test/v1?key=bad",
            "https://example.test/v1#fragment",
        ] {
            assert!(chat_url(url, "").is_err(), "must reject: {url}");
        }
        assert!(chat_url("http://example.test/v1/", "secret").is_err());
        assert!(chat_url("https://example.test/v1/", "secret").is_ok());
    }
}
