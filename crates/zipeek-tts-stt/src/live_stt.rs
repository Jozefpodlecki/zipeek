use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    extract::{
        State,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinHandle,
};

use crate::{
    candle_tts::waveform::encode_mono_pcm16_wav,
    speech::{MAX_AUDIO_SAMPLES, Speech},
};

const MAX_PACKET_BYTES: usize = 6_401;
const MAX_PCM_BYTES: usize = 6_400;
const START: u8 = 1;
const FINISH: u8 = 2;
const AUDIO: u8 = 3;
const CONNECTED: u8 = 0x81;
const READY: u8 = 0x82;
const PARTIAL: u8 = 0x83;
const FINAL: u8 = 0x84;
const ERROR: u8 = 0x85;
const SNAPSHOT_INTERVAL_SAMPLES: usize = 16_000;
const MIN_SPEECH_SAMPLES: usize = 3_200;
const SESSION_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_CONCURRENT_SESSIONS: usize = 2;
static LIVE_SESSIONS: OnceLock<Arc<Semaphore>> = OnceLock::new();

struct Inflight(JoinHandle<Result<String, String>>);

impl Drop for Inflight {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Default)]
struct LivePcm {
    samples: Vec<f32>,
    finished: bool,
}

impl LivePcm {
    fn append(&mut self, packet: &[u8]) -> Result<(), &'static str> {
        if self.finished {
            return Err("session is finalized");
        }
        if packet.is_empty() || packet.len() > MAX_PCM_BYTES || packet.len() % 2 != 0 {
            return Err("invalid PCM packet size");
        }
        let count = packet.len() / 2;
        if self.samples.len() + count > MAX_AUDIO_SAMPLES {
            return Err("session exceeds 30 seconds");
        }
        self.samples.reserve(count);
        for pair in packet.chunks_exact(2) {
            self.samples
                .push(i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0);
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), &'static str> {
        if self.finished {
            return Err("session already finalized");
        }
        self.finished = true;
        Ok(())
    }

    fn speech_present(&self) -> bool {
        self.samples.len() >= MIN_SPEECH_SAMPLES && self.samples.iter().any(|x| x.abs() > 1e-3)
    }

    fn wav(&self) -> Result<Vec<u8>, &'static str> {
        encode_mono_pcm16_wav(&self.samples, 16_000).map_err(|_| "failed to encode audio")
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ClientFrame<'a> {
    Start,
    Finish,
    Audio(&'a [u8]),
}

fn client_frame(data: &[u8]) -> Result<ClientFrame<'_>, &'static str> {
    match data {
        [START, rate @ ..]
            if rate.len() == 4
                && u32::from_le_bytes(rate.try_into().map_err(|_| "invalid sample rate")?)
                    == 16_000 =>
        {
            Ok(ClientFrame::Start)
        }
        [FINISH] => Ok(ClientFrame::Finish),
        [AUDIO, pcm @ ..]
            if !pcm.is_empty() && pcm.len() <= MAX_PCM_BYTES && pcm.len() % 2 == 0 =>
        {
            Ok(ClientFrame::Audio(pcm))
        }
        _ => Err("invalid_frame"),
    }
}

fn transcript_frame(kind: u8, text: &str, samples: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(5 + text.len());
    result.push(kind);
    result.extend_from_slice(&(samples as u32).to_le_bytes());
    result.extend_from_slice(text.as_bytes());
    result
}

fn error_code(code: &str) -> u8 {
    match code {
        "no_speech" => 1,
        "recognition_unavailable" => 2,
        "recognition_task_failed" => 3,
        "session_timeout" => 4,
        "invalid_session_transition" => 5,
        "unexpected_websocket_message" => 6,
        "session exceeds 30 seconds" => 7,
        "invalid PCM packet size" | "invalid_frame" => 8,
        _ => 9,
    }
}

async fn report(socket: &mut WebSocket, data: Vec<u8>) -> Result<(), ()> {
    socket
        .send(Message::Binary(data.into()))
        .await
        .map_err(|_| ())
}

async fn failure(socket: &mut WebSocket, code: &str) {
    let _ = report(socket, vec![ERROR, error_code(code)]).await;
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: 1008,
            reason: "stream rejected".into(),
        })))
        .await;
}

fn recognize(
    speech: &Arc<Speech>,
    pcm: &LivePcm,
) -> Result<JoinHandle<Result<String, String>>, &'static str> {
    let wav = pcm.wav()?;
    let speech = Arc::clone(speech);
    Ok(tokio::spawn(async move {
        speech.transcribe(wav).await.map_err(|err| {
            tracing::warn!(error = %err, "live ASR snapshot failed");
            "recognition_unavailable".to_owned()
        })
    }))
}

pub async fn upgrade(
    State(speech): State<Arc<Speech>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if headers.get(header::ORIGIN).is_none() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let permits = LIVE_SESSIONS
        .get_or_init(|| Arc::new(Semaphore::new(MAX_CONCURRENT_SESSIONS)))
        .clone();
    let Ok(permit) = permits.try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    ws.max_message_size(MAX_PACKET_BYTES)
        .max_frame_size(MAX_PACKET_BYTES)
        .on_upgrade(move |socket| serve(socket, speech, permit))
        .into_response()
}

async fn serve(mut socket: WebSocket, speech: Arc<Speech>, _permit: OwnedSemaphorePermit) {
    let task = async {
        let mut pcm = LivePcm::default();
        let mut started = false;
        let mut inflight: Option<Inflight> = None;
        let mut last_requested = 0usize;
        let mut last_completed = 0usize;
        let mut last_text = String::new();
        if report(&mut socket, {
            let mut header = vec![CONNECTED];
            header.extend_from_slice(&16_000_u32.to_le_bytes());
            header.extend_from_slice(&30_u16.to_le_bytes());
            header.push(1);
            header
        })
        .await
        .is_err()
        {
            return;
        }
        loop {
            let has_pending = inflight.is_some();
            tokio::select! {
                biased;
                outcome = async { (&mut inflight.as_mut().expect("guarded").0).await }, if has_pending => {
                    inflight = None;
                    match outcome {
                        Ok(Ok(text)) => {
                            last_completed = last_requested;
                            if text != last_text && !pcm.finished {
                                last_text = text.clone();
                                if report(&mut socket, transcript_frame(PARTIAL, &text, last_completed)).await.is_err() { return; }
                            }
                            if pcm.finished && last_completed == pcm.samples.len() {
                                let _ = report(&mut socket, transcript_frame(FINAL, &text, last_completed)).await;
                                return;
                            }
                        }
                        Ok(Err(code)) => {
                            if pcm.finished || code != "recognition_unavailable" {
                                failure(&mut socket, &code).await;
                                return;
                            }
                        }
                        Err(_) => { failure(&mut socket, "recognition_task_failed").await; return; }
                    }
                    if pcm.finished {
                        if !pcm.speech_present() { failure(&mut socket, "no_speech").await; return; }
                        last_requested = pcm.samples.len();
                        match recognize(&speech, &pcm) { Ok(handle) => inflight = Some(Inflight(handle)), Err(code) => { failure(&mut socket, code).await; return; } }
                    } else if pcm.samples.len().saturating_sub(last_requested) >= SNAPSHOT_INTERVAL_SAMPLES && pcm.speech_present() {
                        last_requested = pcm.samples.len();
                        match recognize(&speech, &pcm) { Ok(handle) => inflight = Some(Inflight(handle)), Err(code) => { failure(&mut socket, code).await; return; } }
                    }
                }
                message = socket.recv() => {
                    match message {
                        Some(Ok(Message::Binary(bytes))) => {
                            match client_frame(&bytes) {
                                Ok(ClientFrame::Start) if !started => {
                                    started = true;
                                    if report(&mut socket, vec![READY]).await.is_err() { return; }
                                }
                                Ok(ClientFrame::Finish) if started && !pcm.finished => {
                                    if pcm.finish().is_err() || !pcm.speech_present() { failure(&mut socket, "no_speech").await; return; }
                                    if inflight.is_none() {
                                        last_requested = pcm.samples.len();
                                        match recognize(&speech, &pcm) { Ok(handle) => inflight = Some(Inflight(handle)), Err(code) => { failure(&mut socket, code).await; return; } }
                                    }
                                }
                                Ok(ClientFrame::Audio(data)) if started && !pcm.finished => {
                                    if let Err(code) = pcm.append(data) { failure(&mut socket, code).await; return; }
                                    if inflight.is_none() && pcm.samples.len().saturating_sub(last_requested) >= SNAPSHOT_INTERVAL_SAMPLES && pcm.speech_present() {
                                        last_requested = pcm.samples.len();
                                        match recognize(&speech, &pcm) { Ok(handle) => inflight = Some(Inflight(handle)), Err(code) => { failure(&mut socket, code).await; return; } }
                                    }
                                }
                                _ => { failure(&mut socket, "invalid_session_transition").await; return; }
                            }
                        }
                        Some(Ok(Message::Text(_))) => { failure(&mut socket, "unexpected_websocket_message").await; return; }
                        Some(Ok(Message::Ping(data))) => {
                            if socket.send(Message::Pong(data)).await.is_err() { return; }
                        }
                        Some(Ok(Message::Pong(_))) => {}
                        Some(Ok(Message::Close(_))) | None | Some(Err(_)) => { return; }
                        _ => { failure(&mut socket, "unexpected_websocket_message").await; return; }
                    }
                }
            }
        }
    };
    if tokio::time::timeout(SESSION_TIMEOUT, task).await.is_err() {
        failure(&mut socket, "session_timeout").await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pcm_maintains_signed_little_endian_audio_and_bounds() {
        let mut pcm = LivePcm::default();
        pcm.append(&[0, 128, 255, 127]).unwrap();
        assert_eq!(pcm.samples, vec![-1.0, 32767.0 / 32768.0]);
        assert!(pcm.append(&[1]).is_err());
        assert!(pcm.append(&vec![0; MAX_PACKET_BYTES + 2]).is_err());
        assert_eq!(pcm.samples.len(), 2);
        pcm.finish().unwrap();
        assert!(pcm.append(&[0, 0]).is_err());
    }
    #[test]
    fn binary_protocol_is_strict() {
        assert_eq!(client_frame(&[1, 0x80, 0x3e, 0, 0]), Ok(ClientFrame::Start));
        assert_eq!(client_frame(&[2]), Ok(ClientFrame::Finish));
        assert_eq!(
            client_frame(&[3, 0, 0xff]),
            Ok(ClientFrame::Audio(&[0, 0xff]))
        );
        assert!(client_frame(&[1, 0x80, 0xbb, 0, 0]).is_err());
        assert!(client_frame(&[3, 0]).is_err());
        assert!(client_frame(&[3]).is_err());
        assert!(client_frame(&[4]).is_err());
        assert_eq!(
            transcript_frame(PARTIAL, "你好", 16000),
            [vec![PARTIAL, 0x80, 0x3e, 0, 0], "你好".as_bytes().to_vec()].concat()
        );
    }
    #[test]
    fn no_speech_is_rejected() {
        let mut pcm = LivePcm::default();
        pcm.append(&vec![0; 6400]).unwrap();
        assert!(!pcm.speech_present());
    }
}
