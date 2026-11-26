use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use serde_json::{Value, json};
use thirtyfour::{WebDriver, cdp::CdpSession};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{Duration, timeout},
};

use super::{BrowserKind, HarnessResult};

const MAX_FRAMES: usize = 1_200;
const MAX_FRAME_BYTES: usize = 2_000_000;
const MAX_RECORDING_BYTES: usize = 200_000_000;
const BUFFERED_FRAMES: usize = 8;
const FRAME_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const CDP_COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

pub struct VisualRecorder {
    session: Arc<CdpSession>,
    shutdown: oneshot::Sender<()>,
    worker: JoinHandle<HarnessResult<()>>,
    directory: PathBuf,
}

#[derive(Default)]
struct CaptureStats {
    captured: usize,
    dropped: usize,
    bytes: usize,
    truncated: bool,
    invalid_frames: usize,
    backpressure_drops: usize,
    acknowledgment_failures: usize,
}

enum FrameDisposition {
    Accepted,
    Invalid,
    Backpressure,
    LimitReached,
}

impl CaptureStats {
    fn record(&mut self, disposition: FrameDisposition, size: usize) {
        match disposition {
            FrameDisposition::Accepted => {
                self.captured += 1;
                self.bytes += size;
            }
            FrameDisposition::Invalid => {
                self.dropped += 1;
                self.invalid_frames += 1;
            }
            FrameDisposition::Backpressure => {
                self.dropped += 1;
                self.backpressure_drops += 1;
            }
            FrameDisposition::LimitReached => {
                self.dropped += 1;
                self.truncated = true;
            }
        }
    }

    fn accept(&mut self, encoded: &str, frames: &mpsc::Sender<Vec<u8>>) {
        if self.captured >= MAX_FRAMES || self.bytes >= MAX_RECORDING_BYTES {
            self.record(FrameDisposition::LimitReached, 0);
            return;
        }

        if encoded.len() > MAX_FRAME_BYTES.div_ceil(3) * 4 + 16 {
            self.record(FrameDisposition::Invalid, 0);
            return;
        }

        let Ok(frame) = STANDARD.decode(encoded) else {
            self.record(FrameDisposition::Invalid, 0);
            return;
        };

        if frame.len() > MAX_FRAME_BYTES || frame.is_empty() {
            self.record(FrameDisposition::Invalid, 0);
            return;
        }

        if self.bytes.saturating_add(frame.len()) > MAX_RECORDING_BYTES {
            self.record(FrameDisposition::LimitReached, 0);
            return;
        }

        if !is_jpeg(&frame) {
            self.record(FrameDisposition::Invalid, 0);
            return;
        }

        let size = frame.len();
        match frames.try_send(frame) {
            Ok(()) => self.record(FrameDisposition::Accepted, size),
            Err(_) => self.record(FrameDisposition::Backpressure, 0),
        }
    }

    fn status(
        &self,
        started_unix_ms: u128,
        video: &Path,
        validation: Value,
        capture_error: Option<&str>,
        encoder_error: Option<&str>,
    ) -> Value {
        let recording_available =
            fs::metadata(video).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0);

        json!({
            "started_unix_ms": started_unix_ms,
            "captured": self.captured,
            "dropped": self.dropped,
            "truncated": self.truncated,
            "bytes": self.bytes,
            "invalid_frames": self.invalid_frames,
            "backpressure_drops": self.backpressure_drops,
            "acknowledgment_failures": self.acknowledgment_failures,
            "recording_available": recording_available,
            "validation": validation,
            "capture_error": capture_error,
            "encoder_error": encoder_error,
        })
    }
}

fn is_jpeg(data: &[u8]) -> bool {
    data.len() >= 4 && data.starts_with(&[0xff, 0xd8]) && data.ends_with(&[0xff, 0xd9])
}

fn start_encoder(directory: &Path) -> HarnessResult<Child> {
    let stderr = File::create(directory.join("encoder-stderr.log"))?;
    let process = Command::new("ffmpeg")
        .current_dir(directory)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "image2pipe",
            "-vcodec",
            "mjpeg",
            "-use_wallclock_as_timestamps",
            "1",
            "-i",
            "pipe:0",
            "-fps_mode",
            "vfr",
            "-vf",
            "pad=ceil(iw/2)*2:ceil(ih/2)*2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "recording.mp4",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()?;
    Ok(process)
}

fn spawn_encoder(
    directory: &Path,
) -> HarnessResult<(mpsc::Sender<Vec<u8>>, JoinHandle<HarnessResult<()>>)> {
    let mut process = start_encoder(directory)?;
    let mut stdin = process
        .stdin
        .take()
        .ok_or("FFmpeg did not expose an input pipe")?;
    let (sender, mut receiver) = mpsc::channel::<Vec<u8>>(BUFFERED_FRAMES);

    let worker = tokio::task::spawn_blocking(move || -> HarnessResult<()> {
        let mut pipe_error = None;

        while let Some(frame) = receiver.blocking_recv() {
            if let Err(error) = stdin.write_all(&frame) {
                pipe_error = Some(error);
                break;
            }
        }

        drop(stdin);
        let exit = process.wait()?;

        if let Some(error) = pipe_error {
            return Err(format!("FFmpeg input failed: {error}; process exited {exit}").into());
        }
        if !exit.success() {
            return Err(format!("FFmpeg exited with {exit}; see encoder-stderr.log").into());
        }
        Ok(())
    });

    Ok((sender, worker))
}

fn validate_mp4(path: &Path) -> Value {
    if !fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0) {
        return json!({"status": "invalid", "detail": "Missing, unreadable or empty MP4"});
    }

    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,width,height,duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output();

    let probe = match probe {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            return json!({
                "status": "invalid",
                "detail": format!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr)),
            });
        }
        Err(error) => {
            return json!({"status": "unverified", "detail": format!("ffprobe unavailable: {error}")});
        }
    };

    let details: Value = match serde_json::from_slice(&probe.stdout) {
        Ok(details) => details,
        Err(error) => {
            return json!({"status": "invalid", "detail": format!("Invalid ffprobe output: {error}")});
        }
    };

    let has_video = details["streams"].as_array().is_some_and(|streams| {
        streams.iter().any(|stream| {
            stream["width"].as_u64().is_some_and(|width| width > 0)
                && stream["height"].as_u64().is_some_and(|height| height > 0)
                && stream["codec_name"].as_str().is_some()
        })
    });

    if !has_video {
        return json!({"status": "invalid", "detail": "No usable video stream", "probe": details});
    }

    let decode = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-xerror", "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-frames:v", "1", "-f", "null", "-"])
        .output();

    match decode {
        Ok(output) if output.status.success() => {
            json!({"status": "valid", "detail": "Video stream and first frame decoded", "probe": details})
        }
        Ok(output) => json!({
            "status": "invalid",
            "detail": format!("Video decoding failed: {}", String::from_utf8_lossy(&output.stderr)),
            "probe": details,
        }),
        Err(error) => json!({
            "status": "unverified",
            "detail": format!("FFmpeg decoder unavailable: {error}"),
            "probe": details,
        }),
    }
}

fn write_status(directory: &Path, status: &Value) -> HarnessResult<()> {
    let temporary = directory.join("status.json.tmp");
    let target = directory.join("status.json");
    let contents = serde_json::to_vec_pretty(status).map_err(|error| error.to_string())?;
    fs::write(&temporary, contents)?;
    if target.exists() {
        fs::remove_file(&target)?;
    }
    fs::rename(temporary, target)?;
    Ok(())
}

async fn send_cdp(
    session: &CdpSession,
    method: &str,
    params: Value,
    duration: Duration,
) -> HarnessResult<()> {
    match timeout(duration, session.send_raw(method, params)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(format!("{method} failed: {error}").into()),
        Err(_) => Err(format!("{method} timed out after {duration:?}").into()),
    }
}

async fn record_frames(
    session: Arc<CdpSession>,
    mut shutdown: oneshot::Receiver<()>,
    directory: PathBuf,
    started_unix_ms: u128,
    sender: mpsc::Sender<Vec<u8>>,
    encoder: JoinHandle<HarnessResult<()>>,
) -> HarnessResult<()> {
    let mut events = Box::pin(session.subscribe_all());
    let mut counters = CaptureStats::default();
    let mut capture_error = None;

    loop {
        let event = tokio::select! {
            biased;
            _ = &mut shutdown => break,
            next = timeout(FRAME_IDLE_TIMEOUT, events.next()) => match next {
                Ok(Some(event)) => event,
                Ok(None) => {
                    capture_error = Some("CDP event stream closed during recording".to_owned());
                    break;
                }
                Err(_) => continue,
            },
        };

        if event.method != "Page.screencastFrame" {
            continue;
        }

        let Some(id) = event.params.get("sessionId").and_then(Value::as_i64) else {
            counters.record(FrameDisposition::Invalid, 0);
            continue;
        };

        if let Err(error) = send_cdp(
            &session,
            "Page.screencastFrameAck",
            json!({"sessionId": id}),
            CDP_COMMAND_TIMEOUT,
        )
        .await
        {
            counters.acknowledgment_failures += 1;
            capture_error = Some(error.to_string());
            break;
        }

        match event.params.get("data").and_then(Value::as_str) {
            Some(data) => counters.accept(data, &sender),
            None => counters.record(FrameDisposition::Invalid, 0),
        }
    }

    drop(sender);

    let encoder_error = match encoder.await {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error.to_string()),
        Err(error) => Some(format!("FFmpeg encoder task failed: {error}")),
    };

    let video = directory.join("recording.mp4");
    let validation = validate_mp4(&video);
    let status = counters.status(
        started_unix_ms,
        &video,
        validation,
        capture_error.as_deref(),
        encoder_error.as_deref(),
    );
    write_status(&directory, &status)?;

    if let Some(error) = capture_error {
        return Err(error.into());
    }
    if let Some(error) = encoder_error {
        return Err(error.into());
    }
    if VisualRecorder::strict()
        && (status["validation"]["status"] != "valid"
            || counters.truncated
            || counters.captured == 0)
    {
        return Err(format!(
            "Strict E2E recording failed: {} (captured {}, dropped {}, truncated {})",
            status["validation"]["detail"], counters.captured, counters.dropped, counters.truncated
        )
        .into());
    }
    Ok(())
}

impl VisualRecorder {
    pub fn enabled() -> bool {
        std::env::var("ZIPEEK_E2E_VIDEO").as_deref() != Ok("0")
    }

    pub fn strict() -> bool {
        std::env::var("ZIPEEK_E2E_VIDEO_STRICT").as_deref() == Ok("1")
    }

    pub async fn start(
        driver: &WebDriver,
        artifacts: &Path,
        browser: BrowserKind,
    ) -> HarnessResult<Option<Self>> {
        if !Self::enabled() {
            return Ok(None);
        }

        if !browser.supports_cdp_video() {
            if Self::strict() {
                return Err(format!(
                    "Video capture is unavailable on {}; use a CDP-capable browser",
                    browser.as_str()
                )
                .into());
            }
            return Ok(None);
        }

        let directory = artifacts.join("video");
        fs::create_dir_all(&directory)?;
        let started_unix_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let session = Arc::new(driver.cdp().connect().await?);
        send_cdp(&session, "Page.enable", json!({}), CDP_COMMAND_TIMEOUT).await?;

        let (sender, encoder) = spawn_encoder(&directory)?;
        let (shutdown, receiver) = oneshot::channel();
        let worker = tokio::spawn(record_frames(
            Arc::clone(&session),
            receiver,
            directory.clone(),
            started_unix_ms,
            sender,
            encoder,
        ));

        let start_result = send_cdp(
            &session,
            "Page.startScreencast",
            json!({
                "format": "jpeg",
                "quality": 70,
                "maxWidth": 1280,
                "maxHeight": 1000,
                "everyNthFrame": 1,
            }),
            CDP_COMMAND_TIMEOUT,
        )
        .await;

        if let Err(error) = start_result {
            let _ = shutdown.send(());
            let cleanup = worker.await;
            if let Err(cleanup_error) = cleanup {
                return Err(format!("{error}; recorder cleanup failed: {cleanup_error}").into());
            }
            return Err(error);
        }

        Ok(Some(Self {
            session,
            shutdown,
            worker,
            directory,
        }))
    }

    pub async fn stop(self) -> HarnessResult<()> {
        let command_result = send_cdp(
            &self.session,
            "Page.stopScreencast",
            json!({}),
            STOP_COMMAND_TIMEOUT,
        )
        .await;

        let _ = self.shutdown.send(());
        let worker_result = self.worker.await?;

        if let Err(error) = worker_result {
            return Err(error);
        }
        command_result?;
        println!("E2E recording: {}", self.directory.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_validation_rejects_empty_and_malformed_bytes() {
        assert!(!is_jpeg(&[]));
        assert!(!is_jpeg(&[0xff, 0xd8, 0x00, 0x00]));
        assert!(is_jpeg(&[0xff, 0xd8, 0xff, 0xd9]));
    }

    #[test]
    fn invalid_frames_are_counted_separately() {
        let (sender, _receiver) = mpsc::channel(BUFFERED_FRAMES);
        let mut stats = CaptureStats::default();
        stats.accept("not base64", &sender);
        stats.accept(&STANDARD.encode([1_u8, 2, 3, 4]), &sender);
        assert_eq!(stats.captured, 0);
        assert_eq!(stats.dropped, 2);
        assert_eq!(stats.invalid_frames, 2);
    }

    #[test]
    fn accepted_frames_are_forwarded_and_counted() {
        let (sender, mut receiver) = mpsc::channel(BUFFERED_FRAMES);
        let mut stats = CaptureStats::default();
        let jpeg = [0xff, 0xd8, 0xff, 0xd9];
        stats.accept(&STANDARD.encode(jpeg), &sender);
        assert_eq!(stats.captured, 1);
        assert_eq!(stats.bytes, jpeg.len());
        assert_eq!(receiver.try_recv().unwrap(), jpeg.to_vec());
    }

    #[test]
    fn full_queue_is_reported_as_backpressure() {
        let (sender, _receiver) = mpsc::channel(1);
        let mut stats = CaptureStats::default();
        let jpeg = STANDARD.encode([0xff, 0xd8, 0xff, 0xd9]);
        stats.accept(&jpeg, &sender);
        stats.accept(&jpeg, &sender);
        assert_eq!(stats.captured, 1);
        assert_eq!(stats.dropped, 1);
        assert_eq!(stats.backpressure_drops, 1);
    }

    #[test]
    fn missing_mp4_is_invalid() {
        let file = std::env::temp_dir().join(format!(
            "zipeek-missing-video-{}-{}.mp4",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert_eq!(validate_mp4(&file)["status"], "invalid");
    }

    #[test]
    fn exhausted_capture_limit_drops_frames_without_exceeding_budget() {
        let (sender, _receiver) = mpsc::channel(BUFFERED_FRAMES);
        let mut stats = CaptureStats {
            captured: MAX_FRAMES,
            ..CaptureStats::default()
        };
        let jpeg = STANDARD.encode([0xff, 0xd8, 0xff, 0xd9]);
        stats.accept(&jpeg, &sender);
        assert_eq!(stats.captured, MAX_FRAMES);
        assert_eq!(stats.dropped, 1);
        assert!(stats.truncated);
    }

    #[test]
    fn status_keeps_encoder_failure_separate_from_video_availability() {
        let stats = CaptureStats::default();
        let nonexistent = Path::new("missing-zipeek-recording.mp4");
        let status = stats.status(
            42,
            nonexistent,
            json!({"status": "invalid"}),
            None,
            Some("encoder exited unexpectedly"),
        );
        assert_eq!(status["recording_available"], false);
        assert_eq!(status["encoder_error"], "encoder exited unexpectedly");
    }
}
