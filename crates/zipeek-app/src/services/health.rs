use super::registry::ServiceKey;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

pub(super) const HEALTH_TIMEOUT: Duration = Duration::from_millis(500);

pub(super) fn health_ready(kind: ServiceKey) -> bool {
    health_ready_at(kind, kind.address())
}

pub(super) fn health_ready_at(kind: ServiceKey, address: SocketAddr) -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&address, HEALTH_TIMEOUT) else {
        return false;
    };
    if stream.set_read_timeout(Some(HEALTH_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(HEALTH_TIMEOUT)).is_err()
    {
        return false;
    }
    let request = b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request).is_err() {
        return false;
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 1024];
    while bytes.len() < 16_384 {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(_) => break,
        }
    }
    let Ok(response) = std::str::from_utf8(&bytes) else {
        return false;
    };
    let Some((headers, body)) = response.split_once("\r\n\r\n") else {
        return false;
    };
    if !headers
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("HTTP/1.1 200 ") || line.starts_with("HTTP/1.0 200 "))
    {
        return false;
    }
    if kind.name() == "speech" {
        return body.trim() == "ok";
    }
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json.get("status")
                .and_then(|status| status.as_str())
                .map(|s| s == "ok")
        })
        .unwrap_or(false)
}
