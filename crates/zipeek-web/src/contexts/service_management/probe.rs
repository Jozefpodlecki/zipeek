use futures::{
    channel::oneshot,
    future::{Either, select},
    pin_mut,
};
use gloo::{net::http::Request, timers::callback::Timeout};

const TIMEOUT_MS: u32 = 5_000;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ProbeError {
    Unreachable,
    Failed(String),
}

pub(super) fn health_url(service: &str, endpoint: &str) -> Result<String, ProbeError> {
    let endpoint = endpoint.trim().trim_end_matches('/');
    if !(endpoint.starts_with("http://") || endpoint.starts_with("https://"))
        || endpoint.chars().any(char::is_whitespace)
        || (endpoint.contains('?') || endpoint.contains('#'))
        || endpoint.split_once("://").is_some_and(|(_, tail)| {
            tail.split('/')
                .next()
                .is_some_and(|authority| authority.contains('@') || authority.is_empty())
        })
    {
        return Err(ProbeError::Failed(
            "Invalid service URL in Settings".to_owned(),
        ));
    }
    let path = match service {
        "ai" => "models",
        "image" => "health",
        _ => return Err(ProbeError::Failed("Unknown service".to_owned())),
    };
    Ok(format!("{endpoint}/{path}"))
}

pub(super) async fn with_timeout<T>(
    operation: impl Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let (send_timeout, timeout) = oneshot::channel::<()>();
    let timer = Timeout::new(TIMEOUT_MS, move || {
        let _ = send_timeout.send(());
    });
    pin_mut!(operation, timeout);
    let result = match select(operation, timeout).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err("Service check timed out".to_owned()),
    };
    drop(timer);
    result
}

pub(super) async fn check(service: &str, endpoint: &str) -> Result<(), ProbeError> {
    let url = health_url(service, endpoint)?;
    match with_timeout(async {
        Request::get(&url)
            .send()
            .await
            .map_err(|error| error.to_string())
    })
    .await
    {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => Err(ProbeError::Failed(format!(
            "Health check returned HTTP {}",
            response.status()
        ))),
        Err(message) if message == "Service check timed out" => Err(ProbeError::Failed(message)),
        Err(_) => Err(ProbeError::Unreachable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_routes_preserve_api_prefixes() {
        assert_eq!(
            health_url("ai", "http://localhost:1234/v1/").unwrap(),
            "http://localhost:1234/v1/models"
        );
        assert_eq!(
            health_url("image", "http://localhost:1431").unwrap(),
            "http://localhost:1431/health"
        );
    }

    #[test]
    fn rejects_unsafe_or_unknown_endpoints() {
        assert!(health_url("ai", "http://user:secret@localhost/v1").is_err());
        assert!(health_url("image", "file:///etc").is_err());
        assert!(health_url("speech", "http://localhost").is_err());
    }
}
