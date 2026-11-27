use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

const MAX_HEADERS: usize = 16 * 1024;

pub(super) fn request(
    address: SocketAddr,
    path: &str,
    method: &str,
    content_type: &str,
    payload: &[u8],
    extra_headers: &[(&str, &str)],
    max_body: usize,
    timeout: Duration,
) -> Result<(u16, Vec<u8>), String> {
    if !matches!(method, "GET" | "POST")
        || !path.starts_with('/')
        || path
            .chars()
            .any(|character| character == '\r' || character == '\n')
        || payload.len() > max_body
    {
        return Err("Invalid native service request".into());
    }
    let mut socket = TcpStream::connect_timeout(&address, Duration::from_secs(5))
        .map_err(|_| "Cannot connect to native service".to_owned())?;
    socket
        .set_read_timeout(Some(timeout))
        .map_err(|error| error.to_string())?;
    socket
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut headers = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n",
        payload.len()
    );
    for (name, value) in extra_headers {
        if !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || value
                .chars()
                .any(|character| character == '\r' || character == '\n')
        {
            return Err("Invalid native service header".into());
        }
        headers.push_str(name);
        headers.push_str(": ");
        headers.push_str(value);
        headers.push_str("\r\n");
    }
    headers.push_str("\r\n");
    socket
        .write_all(headers.as_bytes())
        .map_err(|error| error.to_string())?;
    socket
        .write_all(payload)
        .map_err(|error| error.to_string())?;

    let mut received = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let count = socket
            .read(&mut chunk)
            .map_err(|_| "Native service response timed out".to_owned())?;
        if count == 0 {
            break;
        }
        if received.len().saturating_add(count) > max_body + MAX_HEADERS + 64 * 1024 {
            return Err("Native service response exceeds the size limit".into());
        }
        received.extend_from_slice(&chunk[..count]);
    }
    parse_response(&received, max_body)
}

fn parse_response(bytes: &[u8], max_body: usize) -> Result<(u16, Vec<u8>), String> {
    let split = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .ok_or("Incomplete native HTTP response")?;
    if split > MAX_HEADERS {
        return Err("Native response headers too large".into());
    }
    let headers =
        std::str::from_utf8(&bytes[..split]).map_err(|_| "Invalid native response headers")?;
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|part| part.parse::<u16>().ok())
        .ok_or("Invalid native response status")?;
    let payload = &bytes[split + 4..];
    let chunked = headers.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.eq_ignore_ascii_case("transfer-encoding")
                && value.trim().eq_ignore_ascii_case("chunked")
        })
    });
    let body = if chunked {
        let mut remaining = payload;
        let mut result = Vec::new();
        loop {
            let end = remaining
                .windows(2)
                .position(|part| part == b"\r\n")
                .ok_or("Malformed chunked response")?;
            let size = std::str::from_utf8(&remaining[..end])
                .ok()
                .and_then(|part| usize::from_str_radix(part.split(';').next()?.trim(), 16).ok())
                .ok_or("Malformed chunk length")?;
            remaining = &remaining[end + 2..];
            if size == 0 {
                break;
            }
            if size > max_body.saturating_sub(result.len())
                || remaining.len() < size + 2
                || &remaining[size..size + 2] != b"\r\n"
            {
                return Err("Malformed or oversized chunk".into());
            }
            result.extend_from_slice(&remaining[..size]);
            remaining = &remaining[size + 2..];
        }
        result
    } else {
        if let Some(length) = headers.lines().find_map(|line| {
            line.split_once(':')
                .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        }) {
            if length != payload.len() {
                return Err("Incomplete native HTTP body".into());
            }
        }
        payload.to_vec()
    };
    if body.len() > max_body {
        return Err("Native response exceeds size limit".into());
    }
    Ok((status, body))
}

#[cfg(test)]
mod tests {
    use super::parse_response;

    #[test]
    fn handles_content_length_and_chunked_bodies() {
        assert_eq!(
            parse_response(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok", 10)
                .unwrap()
                .1,
            b"ok"
        );
        assert_eq!(
            parse_response(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nok\r\n0\r\n\r\n",
                10
            )
            .unwrap()
            .1,
            b"ok"
        );
        assert!(parse_response(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nok", 10).is_err());
        assert!(parse_response(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok", 1).is_err());
    }
}
