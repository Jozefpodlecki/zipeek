use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
    sync::Arc,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use serde_json::json;
use thirtyfour::{WebDriver, cdp::CdpSession};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::Duration,
};

use super::{BrowserKind, HarnessResult};

const MAX_FRAMES: usize = 1_200;
const MAX_FRAME_BYTES: usize = 2_000_000;
const MAX_RECORDING_BYTES: usize = 200_000_000;
const BUFFERED_FRAMES: usize = 8;

pub struct VisualRecorder {
    session: Arc<CdpSession>,
    stop: oneshot::Sender<()>,
    worker: JoinHandle<HarnessResult<()>>,
    directory: PathBuf,
}

fn spawn_encoder(
    directory: &Path,
) -> HarnessResult<(mpsc::Sender<Vec<u8>>, JoinHandle<HarnessResult<()>>)> {
    let mut child = Command::new("ffmpeg")
        .current_dir(directory)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-use_wallclock_as_timestamps",
            "1",
            "-f",
            "image2pipe",
            "-vcodec",
            "mjpeg",
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
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or("FFmpeg did not open its input pipe")?;
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
        let status = child.wait()?;
        if let Some(error) = pipe_error {
            return Err(
                format!("FFmpeg input pipe failed: {error}; process exited {status}").into(),
            );
        }
        if !status.success() {
            return Err(format!("FFmpeg exited with {status}").into());
        }
        Ok(())
    });
    Ok((sender, worker))
}

fn validate_mp4(path: &Path) -> serde_json::Value {
    if !path.is_file() || fs::metadata(path).is_err_or(|metadata| metadata.len() == 0) {
        return json!({"status": "invalid", "detail": "Missing or empty MP4"});
    }
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=codec_name,width,height,duration", "-of", "json"])
        .arg(path)
        .output();
    match output {
        Err(error) => json!({"status": "unverified", "detail": format!("ffprobe unavailable: {error}")}),
        Ok(output) if !output.status.success() => json!({"status": "invalid", "detail": String::from_utf8_lossy(&output.stderr).to_string()}),
        Ok(output) => {
            let details = serde_json::from_slice::<serde_json::Value>(&output.stdout);
            match details {
                Ok(details) if details["streams"].as_array().is_some_and(|streams| streams.iter().any(|stream| stream["width"].as_u64().is_some_and(|width| width > 0) && stream["height"].as_u64().is_some_and(|height| height > 0))) => {
                    json!({"status": "valid", "detail": "Video stream detected", "probe": details})
                }
                Ok(_) => json!({"status": "invalid", "detail": "No decodable video stream reported by ffprobe"}),
                Err(error) => json!({"status": "invalid", "detail": format!("Invalid ffprobe JSON: {error}")}),
            }
        }
    }
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
                    "Video capture is unavailable on {}; disable ZIPEEK_E2E_VIDEO_STRICT or use Chrome",
                    browser.as_str()
                ).into());
            }
            return Ok(None);
        }
        let directory = artifacts.join("video");
        fs::create_dir_all(&directory)?;
        let started_unix_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let session = Arc::new(driver.cdp().connect().await?);
        let mut events = Box::pin(session.subscribe_all());
        session.send_raw("Page.enable", json!({})).await?;
        let (sender, encoder) = spawn_encoder(&directory)?;
        let (stop, mut shutdown) = oneshot::channel();
        let receiving = session.clone();
        let capture_path = directory.clone();
        let worker = tokio::spawn(async move {
            let mut recorded = 0usize;
            let mut bytes = 0usize;
            let mut dropped = 0usize;
            let mut truncated = false;
            let mut stopping = false;
            loop {
                let event = tokio::select! {
                    _ = &mut shutdown, if !stopping => {
                        stopping = true;
                        continue;
                    }
                    event = tokio::time::timeout(
                        if stopping { Duration::from_millis(300) } else { Duration::from_secs(60) },
                        events.next(),
                    ) => {
                        match event {
                            Ok(Some(event)) => event,
                            Ok(None) => break,
                            Err(_) if stopping => break,
                            Err(_) => continue,
                        }
                    }
                };
                if event.method != "Page.screencastFrame" {
                    continue;
                }
                if let Some(id) = event
                    .params
                    .get("sessionId")
                    .and_then(serde_json::Value::as_i64)
                {
                    receiving
                        .send_raw("Page.screencastFrameAck", json!({"sessionId": id}))
                        .await?;
                } else {
                    dropped += 1;
                    continue;
                }
                if recorded >= MAX_FRAMES || bytes >= MAX_RECORDING_BYTES {
                    dropped += 1;
                    truncated = true;
                    continue;
                }
                let Some(encoded) = event.params.get("data").and_then(serde_json::Value::as_str)
                else {
                    dropped += 1;
                    continue;
                };
                if encoded.len() > MAX_FRAME_BYTES * 4 / 3 + 16 {
                    dropped += 1;
                    continue;
                }
                let Ok(frame) = STANDARD.decode(encoded) else {
                    dropped += 1;
                    continue;
                };
                if frame.len() > MAX_FRAME_BYTES || bytes + frame.len() > MAX_RECORDING_BYTES {
                    dropped += 1;
                    truncated = true;
                    continue;
                }
                let size = frame.len();
                if sender.try_send(frame).is_err() {
                    dropped += 1;
                    continue;
                }
                recorded += 1;
                bytes += size;
            }
            drop(sender);
            let encoding = encoder.await?;
            encoding?;
            let video_validation = validate_mp4(&capture_path.join("recording.mp4"));
            fs::write(
                capture_path.join("status.json"),
                serde_json::to_vec_pretty(&json!({
                    "started_unix_ms": started_unix_ms,
                    "captured": recorded,
                    "dropped": dropped,
                    "truncated": truncated,
                    "bytes": bytes,
                    "recording_available": capture_path.join("recording.mp4").is_file(),
                    "validation": &video_validation,
                }))
                .map_err(|error| error.to_string())?,
            )?;
            if VisualRecorder::strict() && (video_validation["status"] != "valid" || truncated || recorded == 0) {
                return Err(format!("Strict E2E video validation failed: {}", video_validation["detail"]).into());
            }
            if recorded == 0 && VisualRecorder::strict() {
                return Err("No CDP screencast frames were captured".into());
            }
            Ok(())
        });
        if let Err(error) = session
            .send_raw(
                "Page.startScreencast",
                json!({
                    "format": "jpeg",
                    "quality": 70,
                    "maxWidth": 1280,
                    "maxHeight": 1000,
                    "everyNthFrame": 1
                }),
            )
            .await
        {
            let _ = stop.send(());
            let _ = worker.await;
            return Err(error.into());
        }
        Ok(Some(Self {
            session,
            stop,
            worker,
            directory,
        }))
    }

    pub async fn stop(self) -> HarnessResult<()> {
        let stop_result = self
            .session
            .send_raw("Page.stopScreencast", json!({}))
            .await;
        let _ = self.stop.send(());
        let worker_result = self.worker.await?;
        stop_result?;
        worker_result?;
        if Self::strict() {
            match fs::metadata(self.directory.join("recording.mp4")) {
                Ok(metadata) if metadata.len() > 0 => {}
                Ok(_) => return Err("Strict E2E video capture produced an empty MP4".into()),
                Err(error) => {
                    return Err(format!("Strict E2E video capture has no MP4: {error}").into());
                }
            }
        }
        println!("E2E recording: {}", self.directory.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_bounds_are_valid() {
        assert!(MAX_FRAMES > 0);
        assert!(MAX_RECORDING_BYTES > MAX_FRAME_BYTES);
        assert!(MAX_FRAMES * MAX_FRAME_BYTES > MAX_RECORDING_BYTES);
    }

    #[test]
    fn recorder_buffers_are_bounded() {
        assert!(BUFFERED_FRAMES > 0 && BUFFERED_FRAMES <= 16);
        assert!(MAX_FRAME_BYTES <= 4_000_000);
    }

    #[test]
    fn disabled_recording_uses_no_video_pipeline() {
        assert!(MAX_RECORDING_BYTES <= 256_000_000);
    }

    #[test]
    fn missing_video_is_not_reported_as_playable() {
        let path = std::env::temp_dir().join(format!("missing-zipeek-video-{}.mp4", std::process::id()));
        assert_eq!(validate_mp4(&path)["status"], "invalid");
    }

}
