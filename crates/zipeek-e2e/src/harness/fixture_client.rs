use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

use serde_json::{Value, json};

use super::HarnessResult;

const MAX_RESPONSE_BYTES: usize = 1_048_576;
const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureScenario {
    Success,
    MalformedGeneration,
    ServerError,
    SlowEvaluation,
    TranslatorRejectedAnalysis,
    TranslatorValidAnalysis,
    TranslatorStreaming,
}

impl FixtureScenario {
    pub fn parse(value: &str) -> HarnessResult<Self> {
        match value {
            "success" => Ok(Self::Success),
            "malformed-generation" => Ok(Self::MalformedGeneration),
            "server-error" => Ok(Self::ServerError),
            "slow-evaluation" => Ok(Self::SlowEvaluation),
            "translator-rejected-analysis" => Ok(Self::TranslatorRejectedAnalysis),
            "translator-valid-analysis" => Ok(Self::TranslatorValidAnalysis),
            "translator-streaming" => Ok(Self::TranslatorStreaming),
            _ => Err(format!("Unsupported fixture scenario: {value}").into()),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::MalformedGeneration => "malformed-generation",
            Self::ServerError => "server-error",
            Self::SlowEvaluation => "slow-evaluation",
            Self::TranslatorRejectedAnalysis => "translator-rejected-analysis",
            Self::TranslatorValidAnalysis => "translator-valid-analysis",
            Self::TranslatorStreaming => "translator-streaming",
        }
    }
}

#[derive(Clone)]
pub struct FixtureClient {
    address: SocketAddr,
}

impl FixtureClient {
    pub fn from_ai_url(url: &str) -> HarnessResult<Self> {
        let authority = url
            .strip_prefix("http://127.0.0.1:")
            .ok_or("AI fixture URL must use loopback HTTP")?;
        let port = authority
            .strip_suffix("/v1/")
            .ok_or("AI fixture URL must end in /v1/")?
            .parse::<u16>()
            .map_err(|error| format!("Invalid AI fixture port: {error}"))?;
        if port == 0 {
            return Err("AI fixture port must be nonzero".into());
        }
        Ok(Self {
            address: SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
        })
    }

    pub async fn reset(&self, scenario: FixtureScenario) -> HarnessResult<()> {
        let response = self
            .request(
                "POST",
                "/__e2e/reset",
                Some(json!({"scenario": scenario.as_str()})),
            )
            .await?;
        if response["scenario"].as_str() != Some(scenario.as_str()) {
            return Err(
                format!("AI fixture did not confirm reset to {}", scenario.as_str()).into(),
            );
        }
        Ok(())
    }

    pub async fn health(&self) -> HarnessResult<()> {
        let response = self.request("GET", "/__e2e/health", None).await?;
        if response["ready"] != true {
            return Err("AI fixture health endpoint is not ready".into());
        }
        Ok(())
    }

    pub async fn requests(&self) -> HarnessResult<Value> {
        self.request("GET", "/__e2e/requests", None).await
    }

    async fn request(
        &self,
        method: &'static str,
        path: &'static str,
        body: Option<Value>,
    ) -> HarnessResult<Value> {
        let address = self.address;
        tokio::task::spawn_blocking(move || request_blocking(address, method, path, body)).await?
    }
}

fn request_blocking(
    address: SocketAddr,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> HarnessResult<Value> {
    let mut socket = TcpStream::connect_timeout(&address, SOCKET_TIMEOUT)?;
    socket.set_read_timeout(Some(SOCKET_TIMEOUT))?;
    socket.set_write_timeout(Some(SOCKET_TIMEOUT))?;
    let payload = body.map_or_else(String::new, |value| value.to_string());
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{payload}",
        payload.len()
    );
    socket.write_all(request.as_bytes())?;
    let mut bytes = Vec::new();
    Read::take(&mut socket, (MAX_RESPONSE_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err("AI fixture response exceeds 1 MiB".into());
    }
    let separator = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .ok_or("AI fixture response has no HTTP header boundary")?;
    let header = std::str::from_utf8(&bytes[..separator])
        .map_err(|error| format!("Invalid AI fixture response headers: {error}"))?;
    let status = header
        .lines()
        .next()
        .ok_or("AI fixture response has no status line")?;
    if !status.starts_with("HTTP/1.1 200 ") {
        return Err(format!("AI fixture returned {status}").into());
    }
    serde_json::from_slice(&bytes[separator + 4..])
        .map_err(|error| format!("Invalid AI fixture response JSON: {error}").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::ai_fixture::AiFixtureServer;

    #[test]
    fn fixture_scenarios_are_explicit() {
        for scenario in [
            FixtureScenario::Success,
            FixtureScenario::MalformedGeneration,
            FixtureScenario::ServerError,
            FixtureScenario::SlowEvaluation,
            FixtureScenario::TranslatorRejectedAnalysis,
            FixtureScenario::TranslatorValidAnalysis,
            FixtureScenario::TranslatorStreaming,
        ] {
            assert_eq!(FixtureScenario::parse(scenario.as_str()).unwrap(), scenario);
        }
        assert!(FixtureScenario::parse("unknown").is_err());
    }

    #[test]
    fn fixture_address_rejects_non_loopback_and_incorrect_paths() {
        assert!(FixtureClient::from_ai_url("https://example.com/v1/").is_err());
        assert!(FixtureClient::from_ai_url("http://127.0.0.1:1234/").is_err());
        assert!(FixtureClient::from_ai_url("http://127.0.0.1:0/v1/").is_err());
    }

    #[tokio::test]
    async fn native_fixture_reset_and_request_capture_are_independent_of_browser() {
        let mut server = AiFixtureServer::start().unwrap();
        let client = FixtureClient::from_ai_url(&server.url()).unwrap();
        client.health().await.unwrap();
        client
            .reset(FixtureScenario::MalformedGeneration)
            .await
            .unwrap();
        let records = client.requests().await.unwrap();
        assert_eq!(records["scenario"], "malformed-generation");
        client.reset(FixtureScenario::Success).await.unwrap();
        assert_eq!(client.requests().await.unwrap()["scenario"], "success");
        server.stop().unwrap();
    }
}
