use std::{
    fmt,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    time::Duration,
};

pub const DEFAULT_PORT: u16 = 8799;
pub const MAX_TEXT_BYTES: usize = 4096;
pub const MAX_AUDIO_BYTES: usize = 960_044;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 16 * 1024;

#[derive(Debug)]
pub enum SpeechClientError {
    InvalidInput(&'static str),
    Transport(std::io::Error),
    Protocol(&'static str),
    Service { status: u16, message: String },
}

impl fmt::Display for SpeechClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::Protocol(message) => f.write_str(message),
            Self::Transport(error) => fmt::Display::fmt(error, f),
            Self::Service { status, message } => {
                write!(f, "local speech service HTTP {status}: {message}")
            }
        }
    }
}

impl std::error::Error for SpeechClientError {}

impl From<std::io::Error> for SpeechClientError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeechClient {
    port: u16,
    timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechCapabilities {
    pub language: String,
    pub synthesis: bool,
    pub recognition: bool,
}

impl SpeechClient {
    pub fn new(port: u16) -> Result<Self, SpeechClientError> {
        if port == 0 {
            return Err(SpeechClientError::InvalidInput(
                "speech port must be nonzero",
            ));
        }
        Ok(Self {
            port,
            timeout: Duration::from_secs(35),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, SpeechClientError> {
        if timeout.is_zero() {
            return Err(SpeechClientError::InvalidInput("timeout must be nonzero"));
        }
        self.timeout = timeout;
        Ok(self)
    }

    pub fn health(&self) -> Result<(), SpeechClientError> {
        let body = self.request("GET", "/health", None, &[], 64, "text/plain")?;
        if body != b"ok" {
            return Err(SpeechClientError::Protocol("unexpected health response"));
        }
        Ok(())
    }

    pub fn capabilities(&self) -> Result<SpeechCapabilities, SpeechClientError> {
        let body = self.request("GET", "/v1/capabilities", None, &[], 4096, "text/plain")?;
        let body = std::str::from_utf8(&body)
            .map_err(|_| SpeechClientError::Protocol("invalid capabilities encoding"))?;
        parse_capabilities(body)
    }

    pub fn synthesize(&self, text: &str) -> Result<Vec<u8>, SpeechClientError> {
        if text.trim().is_empty()
            || text.len() > MAX_TEXT_BYTES
            || text.chars().any(char::is_control)
        {
            return Err(SpeechClientError::InvalidInput(
                "invalid speech synthesis text",
            ));
        }
        let wav = self.request(
            "POST",
            "/v1/tts",
            Some("text/plain; charset=utf-8"),
            text.as_bytes(),
            MAX_RESPONSE_BYTES,
            "audio/wav",
        )?;
        validate_pcm16_wav(&wav, None)
            .map_err(|_| SpeechClientError::Protocol("speech service returned invalid WAV"))?;
        Ok(wav)
    }

    pub fn transcribe(&self, wav: &[u8]) -> Result<String, SpeechClientError> {
        if wav.len() > MAX_AUDIO_BYTES || validate_pcm16_wav(wav, Some(16_000)).is_err() {
            return Err(SpeechClientError::InvalidInput(
                "audio must be mono 16 kHz PCM16 WAV, at most 30 seconds",
            ));
        }
        let body = self.request(
            "POST",
            "/v1/stt",
            Some("audio/wav"),
            wav,
            MAX_TEXT_BYTES,
            "text/plain",
        )?;
        let text = String::from_utf8(body)
            .map_err(|_| SpeechClientError::Protocol("invalid transcript encoding"))?;
        if text.trim().is_empty()
            || text.len() > MAX_TEXT_BYTES
            || text.chars().any(char::is_control)
        {
            return Err(SpeechClientError::Protocol("invalid speech transcript"));
        }
        Ok(text)
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        content_type: Option<&str>,
        body: &[u8],
        max_body: usize,
        expected_type: &str,
    ) -> Result<Vec<u8>, SpeechClientError> {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, self.port);
        let mut stream = TcpStream::connect_timeout(&addr.into(), self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        let mut headers = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nContent-Length: {}\r\n",
            self.port,
            body.len()
        );
        if let Some(content_type) = content_type {
            headers.push_str(&format!("Content-Type: {content_type}\r\n"));
        }
        headers.push_str("\r\n");
        stream.write_all(headers.as_bytes())?;
        stream.write_all(body)?;
        let mut response = Vec::new();
        let mut chunk = [0_u8; 8192];
        loop {
            let count = stream.read(&mut chunk)?;
            if count == 0 {
                break;
            }
            if response.len().saturating_add(count) > max_body + MAX_HEADER_BYTES + 65536 {
                return Err(SpeechClientError::Protocol(
                    "speech response exceeds allowed size",
                ));
            }
            response.extend_from_slice(&chunk[..count]);
        }
        parse_http_response(&response, max_body, expected_type)
    }
}

impl Default for SpeechClient {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            timeout: Duration::from_secs(35),
        }
    }
}

fn parse_capabilities(body: &str) -> Result<SpeechCapabilities, SpeechClientError> {
    let mut provider = None;
    let mut language = None;
    let mut synthesis = None;
    let mut recognition = None;
    for line in body.lines() {
        let (key, value) = line
            .split_once('=')
            .ok_or(SpeechClientError::Protocol("malformed capability line"))?;
        let slot = match key {
            "provider" => &mut provider,
            "language" => &mut language,
            "synthesis" => &mut synthesis,
            "recognition" => &mut recognition,
            _ => return Err(SpeechClientError::Protocol("unknown capability field")),
        };
        if slot.replace(value).is_some() {
            return Err(SpeechClientError::Protocol("duplicate capability field"));
        }
    }
    if provider != Some("local") || language != Some("zh-CN") {
        return Err(SpeechClientError::Protocol(
            "unexpected speech provider or language",
        ));
    }
    fn boolean(value: Option<&str>) -> Result<bool, SpeechClientError> {
        match value {
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            _ => Err(SpeechClientError::Protocol("invalid capability flag")),
        }
    }
    Ok(SpeechCapabilities {
        language: "zh-CN".into(),
        synthesis: boolean(synthesis)?,
        recognition: boolean(recognition)?,
    })
}

fn parse_http_response(
    bytes: &[u8],
    max_body: usize,
    expected_type: &str,
) -> Result<Vec<u8>, SpeechClientError> {
    let split = bytes
        .windows(4)
        .position(|value| value == b"\r\n\r\n")
        .ok_or(SpeechClientError::Protocol(
            "incomplete HTTP response headers",
        ))?;
    if split > MAX_HEADER_BYTES {
        return Err(SpeechClientError::Protocol("oversized HTTP headers"));
    }
    let headers = std::str::from_utf8(&bytes[..split])
        .map_err(|_| SpeechClientError::Protocol("invalid HTTP headers"))?;
    let mut lines = headers.split("\r\n");
    let status_line = lines
        .next()
        .ok_or(SpeechClientError::Protocol("missing HTTP status"))?;
    let mut status_parts = status_line.split_whitespace();
    if status_parts.next() != Some("HTTP/1.1") {
        return Err(SpeechClientError::Protocol("unsupported HTTP version"));
    }
    let status = status_parts
        .next()
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or(SpeechClientError::Protocol("invalid HTTP status"))?;
    let mut content_length = None;
    let mut chunked = false;
    let mut response_type = None;
    for line in lines {
        let (key, value) = line
            .split_once(':')
            .ok_or(SpeechClientError::Protocol("invalid HTTP header"))?;
        let value = value.trim();
        if key.eq_ignore_ascii_case("content-length") {
            if content_length
                .replace(
                    value
                        .parse::<usize>()
                        .map_err(|_| SpeechClientError::Protocol("invalid content length"))?,
                )
                .is_some()
            {
                return Err(SpeechClientError::Protocol("duplicate content length"));
            }
        } else if key.eq_ignore_ascii_case("transfer-encoding") {
            if !value.eq_ignore_ascii_case("chunked") || chunked {
                return Err(SpeechClientError::Protocol("unsupported transfer encoding"));
            }
            chunked = true;
        } else if key.eq_ignore_ascii_case("content-type") {
            if response_type.replace(value).is_some() {
                return Err(SpeechClientError::Protocol("duplicate content type"));
            }
        }
    }
    if chunked && content_length.is_some() {
        return Err(SpeechClientError::Protocol("ambiguous HTTP framing"));
    }
    let payload = &bytes[split + 4..];
    let body = if chunked {
        decode_chunks(payload, max_body)?
    } else {
        if content_length.is_some_and(|expected| expected != payload.len()) {
            return Err(SpeechClientError::Protocol("incomplete HTTP body"));
        }
        if payload.len() > max_body {
            return Err(SpeechClientError::Protocol("oversized HTTP response"));
        }
        payload.to_vec()
    };
    if !(200..300).contains(&status) {
        let message = String::from_utf8_lossy(&body)
            .chars()
            .filter(|character| !character.is_control())
            .take(256)
            .collect();
        return Err(SpeechClientError::Service { status, message });
    }
    let response_type =
        response_type.ok_or(SpeechClientError::Protocol("missing response content type"))?;
    if response_type
        .split(';')
        .next()
        .is_none_or(|value| !value.trim().eq_ignore_ascii_case(expected_type))
    {
        return Err(SpeechClientError::Protocol(
            "unexpected response content type",
        ));
    }
    Ok(body)
}

fn decode_chunks(mut input: &[u8], max_body: usize) -> Result<Vec<u8>, SpeechClientError> {
    let mut output = Vec::new();
    loop {
        let split = input
            .windows(2)
            .position(|bytes| bytes == b"\r\n")
            .ok_or(SpeechClientError::Protocol("truncated chunk header"))?;
        if split > 32 {
            return Err(SpeechClientError::Protocol("oversized chunk header"));
        }
        let header = std::str::from_utf8(&input[..split])
            .map_err(|_| SpeechClientError::Protocol("invalid chunk header"))?;
        let length = usize::from_str_radix(header, 16)
            .map_err(|_| SpeechClientError::Protocol("invalid chunk length"))?;
        input = &input[split + 2..];
        if length == 0 {
            if input != b"\r\n" {
                return Err(SpeechClientError::Protocol("unexpected chunk trailers"));
            }
            return Ok(output);
        }
        let framed_length = length
            .checked_add(2)
            .ok_or(SpeechClientError::Protocol("invalid chunk length"))?;
        if length > max_body.saturating_sub(output.len())
            || input.len() < framed_length
            || &input[length..framed_length] != b"\r\n"
        {
            return Err(SpeechClientError::Protocol(
                "invalid or oversized chunk body",
            ));
        }
        output.extend_from_slice(&input[..length]);
        input = &input[length + 2..];
    }
}

fn validate_pcm16_wav(wav: &[u8], required_rate: Option<u32>) -> Result<(), SpeechClientError> {
    let invalid = || SpeechClientError::Protocol("invalid PCM16 WAV framing");
    if wav.len() < 46
        || wav.len() > MAX_RESPONSE_BYTES
        || &wav[..4] != b"RIFF"
        || &wav[8..12] != b"WAVE"
        || u32::from_le_bytes(wav[4..8].try_into().map_err(|_| invalid())?) as usize
            != wav.len() - 8
    {
        return Err(invalid());
    }
    let mut offset = 12;
    let mut format_found = false;
    let mut data_found = false;
    while offset + 8 <= wav.len() {
        let kind = &wav[offset..offset + 4];
        let chunk_size = u32::from_le_bytes(
            wav[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| invalid())?,
        ) as usize;
        let start = offset + 8;
        let end = start.checked_add(chunk_size).ok_or_else(invalid)?;
        if end > wav.len() {
            return Err(invalid());
        }
        if kind == b"fmt " {
            if format_found || chunk_size < 16 {
                return Err(invalid());
            }
            let format =
                u16::from_le_bytes(wav[start..start + 2].try_into().map_err(|_| invalid())?);
            let channels = u16::from_le_bytes(
                wav[start + 2..start + 4]
                    .try_into()
                    .map_err(|_| invalid())?,
            );
            let rate = u32::from_le_bytes(
                wav[start + 4..start + 8]
                    .try_into()
                    .map_err(|_| invalid())?,
            );
            let byte_rate = u32::from_le_bytes(
                wav[start + 8..start + 12]
                    .try_into()
                    .map_err(|_| invalid())?,
            );
            let alignment = u16::from_le_bytes(
                wav[start + 12..start + 14]
                    .try_into()
                    .map_err(|_| invalid())?,
            );
            let depth = u16::from_le_bytes(
                wav[start + 14..start + 16]
                    .try_into()
                    .map_err(|_| invalid())?,
            );
            if format != 1
                || channels != 1
                || !(8_000..=96_000).contains(&rate)
                || required_rate.is_some_and(|required| required != rate)
                || byte_rate != rate * 2
                || alignment != 2
                || depth != 16
            {
                return Err(invalid());
            }
            format_found = true;
        } else if kind == b"data" {
            if data_found || chunk_size < 2 || chunk_size % 2 != 0 {
                return Err(invalid());
            }
            if required_rate.is_some() && chunk_size > 16_000 * 30 * 2 {
                return Err(invalid());
            }
            data_found = true;
        }
        offset = end.checked_add(chunk_size % 2).ok_or_else(invalid)?;
    }
    if offset != wav.len() || !format_found || !data_found {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_untrusted_capability_fields() {
        assert!(
            parse_capabilities(
                "provider=local\nlanguage=zh-CN\nsynthesis=true\nrecognition=false\n"
            )
            .is_ok()
        );
        assert!(parse_capabilities("provider=local\nlanguage=zh-CN\nsynthesis=true\nsynthesis=false\nrecognition=false\n").is_err());
        assert!(
            parse_capabilities(
                "provider=remote\nlanguage=zh-CN\nsynthesis=true\nrecognition=true\n"
            )
            .is_err()
        );
    }

    #[test]
    fn validates_http_framing_and_limits() {
        assert_eq!(parse_http_response(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nok", 16, "text/plain").unwrap(), b"ok");
        assert_eq!(parse_http_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n\r\n2\r\nok\r\n0\r\n\r\n", 16, "text/plain").unwrap(), b"ok");
        assert!(
            parse_http_response(
                b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nContent-Type: text/plain\r\n\r\nok",
                16,
                "text/plain"
            )
            .is_err()
        );
        assert!(parse_http_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n\r\n20\r\nok\r\n", 16, "text/plain").is_err());
    }

    #[test]
    fn talks_to_a_loopback_http_service() {
        use std::{net::TcpListener, thread};
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0_u8; 1024];
            let size = stream.read(&mut bytes).unwrap();
            let request = std::str::from_utf8(&bytes[..size]).unwrap();
            assert!(request.starts_with("GET /health HTTP/1.1\r\n"));
            assert!(request.contains(&format!("Host: 127.0.0.1:{port}")));
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\nok").unwrap();
        });
        SpeechClient::new(port).unwrap().health().unwrap();
        server.join().unwrap();
    }

    fn wav(samples: &[i16], rate: u32) -> Vec<u8> {
        let length = (samples.len() * 2) as u32;
        let mut result = Vec::new();
        result.extend_from_slice(b"RIFF");
        result.extend_from_slice(&(36 + length).to_le_bytes());
        result.extend_from_slice(b"WAVEfmt ");
        result.extend_from_slice(&16_u32.to_le_bytes());
        result.extend_from_slice(&1_u16.to_le_bytes());
        result.extend_from_slice(&1_u16.to_le_bytes());
        result.extend_from_slice(&rate.to_le_bytes());
        result.extend_from_slice(&(rate * 2).to_le_bytes());
        result.extend_from_slice(&2_u16.to_le_bytes());
        result.extend_from_slice(&16_u16.to_le_bytes());
        result.extend_from_slice(b"data");
        result.extend_from_slice(&length.to_le_bytes());
        for sample in samples {
            result.extend_from_slice(&sample.to_le_bytes());
        }
        result
    }

    #[test]
    fn rejects_malformed_wav_and_incompatible_capture_rate() {
        let valid = wav(&[1, -1], 16_000);
        assert!(validate_pcm16_wav(&valid, Some(16_000)).is_ok());
        assert!(validate_pcm16_wav(&wav(&[1, -1], 22_050), Some(16_000)).is_err());
        let mut forged = valid.clone();
        forged[40..44].copy_from_slice(&100_u32.to_le_bytes());
        assert!(validate_pcm16_wav(&forged, None).is_err());
        forged = valid.clone();
        forged[20..22].copy_from_slice(&3_u16.to_le_bytes());
        assert!(validate_pcm16_wav(&forged, None).is_err());
        assert!(validate_pcm16_wav(&wav(&[], 16_000), None).is_err());
    }

    #[test]
    fn checks_response_media_type_and_chunk_length_overflow() {
        assert!(
            parse_http_response(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\nok",
                16,
                "text/plain"
            )
            .is_err()
        );
        assert!(parse_http_response(b"HTTP/1.1 200 OK\r\n\r\nok", 16, "text/plain").is_err());
        assert!(parse_http_response(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Type: text/plain\r\n\r\nok", 16, "text/plain").is_err());
        assert!(decode_chunks(b"ffffffffffffffff\r\n", 16).is_err());
    }

    #[test]
    fn refuses_zero_port_and_invalid_input() {
        assert!(SpeechClient::new(0).is_err());
        assert!(SpeechClient::default().synthesize("\n").is_err());
        assert!(SpeechClient::default().transcribe(b"invalid").is_err());
    }
}
