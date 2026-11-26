#![allow(dead_code)]

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct Server {
    child: Child,
    address: SocketAddr,
}

pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Reply {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn text(&self) -> String {
        String::from_utf8(self.body.clone()).expect("HTTP response is not valid UTF-8")
    }
}

impl Server {
    pub fn start() -> Self {
        Self::start_with_checkpoint(None)
    }

    pub fn start_without_stt_model() -> Self {
        Self::start_with_checkpoint(Some("missing-stt-checkpoint-for-http-tests.safetensors"))
    }

    fn start_with_checkpoint(checkpoint: Option<&str>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral loopback port");
        let address = listener.local_addr().expect("ephemeral port address");
        drop(listener);
        let mut command = Command::new(env!("CARGO_BIN_EXE_zipeek-tts-stt"));
        command
            .env("ZIPEEK_SPEECH_PORT", address.port().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(checkpoint) = checkpoint {
            command.env("ZIPEEK_CANDLE_STT_MODEL", checkpoint);
            command.env(
                "ZIPEEK_QWEN_TTS_MODEL",
                "missing-tts-checkpoint-for-http-tests.safetensors",
            );
        }
        let child = command.spawn().expect("spawn actual speech server binary");
        let mut server = Self { child, address };
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            if let Some(status) = server.child.try_wait().expect("poll speech server") {
                panic!("speech server exited before readiness: {status}");
            }
            if let Ok(reply) =
                server.request_timeout("GET", "/health", &[], &[], Duration::from_millis(700))
            {
                if reply.status == 200 && reply.body == b"ok" {
                    return server;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        panic!("speech server did not become healthy within 15 seconds");
    }

    pub fn request(
        &self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> Reply {
        self.request_timeout(method, path, headers, body, Duration::from_secs(15))
            .expect("HTTP request to speech server")
    }

    pub fn request_timeout(
        &self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &[u8],
        timeout: Duration,
    ) -> std::io::Result<Reply> {
        let mut stream = TcpStream::connect_timeout(&self.address, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nContent-Length: {}\r\n",
            self.address.port(),
            body.len()
        )?;
        for (name, value) in headers {
            write!(stream, "{name}: {value}\r\n")?;
        }
        stream.write_all(b"\r\n")?;
        stream.write_all(body)?;
        stream.flush()?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        parse_http(&response)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn parse_http(raw: &[u8]) -> std::io::Result<Reply> {
    let invalid = || std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid HTTP response");
    let end = raw
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .ok_or_else(invalid)?;
    let head = std::str::from_utf8(&raw[..end]).map_err(|_| invalid())?;
    let mut lines = head.split("\r\n");
    let first = lines.next().ok_or_else(invalid)?;
    let status = first
        .split_whitespace()
        .nth(1)
        .ok_or_else(invalid)?
        .parse::<u16>()
        .map_err(|_| invalid())?;
    let mut headers = Vec::new();
    for line in lines {
        let (name, value) = line.split_once(':').ok_or_else(invalid)?;
        headers.push((name.to_owned(), value.trim().to_owned()));
    }
    let body = &raw[end + 4..];
    let chunked = headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("transfer-encoding") && value.eq_ignore_ascii_case("chunked")
    });
    let body = if chunked {
        parse_chunks(body)?
    } else {
        if let Some((_, length)) = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        {
            if length.parse::<usize>().map_err(|_| invalid())? != body.len() {
                return Err(invalid());
            }
        }
        body.to_vec()
    };
    Ok(Reply {
        status,
        headers,
        body,
    })
}

fn parse_chunks(mut raw: &[u8]) -> std::io::Result<Vec<u8>> {
    let invalid =
        || std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid chunked response");
    let mut output = Vec::new();
    loop {
        let end = raw
            .windows(2)
            .position(|part| part == b"\r\n")
            .ok_or_else(invalid)?;
        let length = std::str::from_utf8(&raw[..end])
            .map_err(|_| invalid())?
            .split(';')
            .next()
            .ok_or_else(invalid)?;
        let length = usize::from_str_radix(length, 16).map_err(|_| invalid())?;
        raw = &raw[end + 2..];
        if length == 0 {
            return Ok(output);
        }
        let after = length.checked_add(2).ok_or_else(invalid)?;
        if raw.len() < after || &raw[length..after] != b"\r\n" {
            return Err(invalid());
        }
        output.extend_from_slice(&raw[..length]);
        raw = &raw[after..];
    }
}

pub fn voiced_wav() -> Vec<u8> {
    let sample_rate = 16_000usize;
    let samples = sample_rate / 2;
    let mut wav = Vec::with_capacity(44 + samples * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&u32::try_from(36 + samples * 2).unwrap().to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&(sample_rate as u32).to_le_bytes());
    wav.extend_from_slice(&(sample_rate as u32 * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&u32::try_from(samples * 2).unwrap().to_le_bytes());
    for sample in 0..samples {
        let value = (6000.0
            * (2.0 * std::f64::consts::PI * 440.0 * sample as f64 / sample_rate as f64).sin())
            as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

pub fn check_non_silent_wav(wav: &[u8]) {
    assert!(
        wav.len() > 44 && wav.len() % 2 == 0,
        "WAV has no 16-bit PCM samples"
    );
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(
        u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize,
        wav.len() - 8
    );
    assert_eq!(&wav[8..16], b"WAVEfmt ");
    assert_eq!(u32::from_le_bytes(wav[16..20].try_into().unwrap()), 16);
    assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
    let rate = u32::from_le_bytes(wav[24..28].try_into().unwrap());
    assert!((8_000..=96_000).contains(&rate));
    assert_eq!(
        u32::from_le_bytes(wav[28..32].try_into().unwrap()),
        rate * 2
    );
    assert_eq!(u16::from_le_bytes(wav[32..34].try_into().unwrap()), 2);
    assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
    assert_eq!(&wav[36..40], b"data");
    assert_eq!(
        u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
        wav.len() - 44
    );
    assert!(
        wav[44..]
            .chunks_exact(2)
            .any(|sample| i16::from_le_bytes([sample[0], sample[1]]).unsigned_abs() > 100),
        "WAV PCM data is silent"
    );
}

pub const REFERENCE_TEXT: &str = "你好世界";
pub const REFERENCE_WAV: &[u8] = include_bytes!("../fixtures/mandarin_nihao_shijie.wav");

pub fn read_reference_wav() -> Vec<u8> {
    check_non_silent_wav(REFERENCE_WAV);
    assert_eq!(
        u32::from_le_bytes(REFERENCE_WAV[24..28].try_into().unwrap()),
        16_000
    );
    REFERENCE_WAV.to_vec()
}

pub fn normalize_transcript(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !"，。！？、,.!?".contains(*c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_parser_handles_content_length() {
        let reply = parse_http(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok").unwrap();
        assert_eq!(reply.status, 200);
        assert_eq!(reply.body, b"ok");
    }

    #[test]
    fn http_parser_handles_chunked() {
        let reply = parse_http(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nok\r\n0\r\n\r\n",
        )
        .unwrap();
        assert_eq!(reply.body, b"ok");
    }

    #[test]
    fn check_generated_fixture() {
        check_non_silent_wav(&voiced_wav());
    }
}
