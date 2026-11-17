use std::{
    io::Cursor,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::catalogue::TARGET_ID;

pub struct Speech {
    executable: PathBuf,
    data: PathBuf,
    model: PathBuf,
    context: Arc<Mutex<Option<WhisperContext>>>,
    synthesis: Arc<Semaphore>,
    recognition: Arc<Semaphore>,
}

impl Speech {
    pub fn new() -> Result<Self, std::io::Error> {
        let resources = std::env::current_exe()?
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("resources");
        let executable = std::env::var_os("LINGUA_PROVIDER_ESPEAK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                resources.join(if cfg!(windows) {
                    "espeak-ng.exe"
                } else {
                    "espeak-ng"
                })
            });
        let data = std::env::var_os("LINGUA_PROVIDER_ESPEAK_DATA")
            .map(PathBuf::from)
            .unwrap_or(resources.clone());
        let model = std::env::var_os("LINGUA_PROVIDER_WHISPER_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|| resources.join("ggml-model.bin"));

        Ok(Self {
            executable,
            data,
            model,
            context: Arc::new(Mutex::new(None)),
            synthesis: Arc::new(Semaphore::new(1)),
            recognition: Arc::new(Semaphore::new(1)),
        })
    }

    pub fn synthesis_available(&self) -> bool {
        self.executable.is_file() && self.data.join("espeak-ng-data").is_dir()
    }

    pub fn recognition_available(&self) -> bool {
        self.model.is_file()
    }

    pub async fn synthesize(
        &self,
        request: SpeechSynthesisRequest,
    ) -> Result<Vec<u8>, SpeechError> {
        request.validate().map_err(SpeechError::Invalid)?;

        if !self.synthesis_available() {
            return Err(SpeechError::Unavailable(
                "eSpeak NG executable and voice data are missing",
            ));
        }

        let _permit = self
            .synthesis
            .clone()
            .try_acquire_owned()
            .map_err(|_| SpeechError::Busy)?;
        let mut command = Command::new(&self.executable);
        command
            .args(["--stdin", "--stdout", "-v", "sr", "-s"])
            .arg(request.words_per_minute.to_string())
            .arg(format!("--path={}", self.data.display()))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        #[cfg(windows)]
        command.creation_flags(0x08000000);

        let mut child = command.spawn()?;
        let mut input = child
            .stdin
            .take()
            .ok_or(SpeechError::Failed("missing speech engine stdin".into()))?;
        let output = child
            .stdout
            .take()
            .ok_or(SpeechError::Failed("missing speech engine stdout".into()))?;
        let errors = child
            .stderr
            .take()
            .ok_or(SpeechError::Failed("missing speech engine stderr".into()))?;
        let result = tokio::time::timeout(Duration::from_secs(30), async {
            input.write_all(request.text.as_bytes()).await?;
            input.shutdown().await?;
            drop(input);

            let (output, errors, status) = tokio::join!(
                read_bounded(output, 16 * 1024 * 1024),
                read_bounded(errors, 8192),
                child.wait()
            );
            let output = output?;
            let errors = errors?;

            if !status?.success() {
                return Err(SpeechError::Failed(
                    String::from_utf8_lossy(&errors).into_owned(),
                ));
            }

            if output.len() < 44 || &output[..4] != b"RIFF" || &output[8..12] != b"WAVE" {
                return Err(SpeechError::Failed(
                    "speech engine returned invalid WAV audio".into(),
                ));
            }

            Ok(output)
        })
        .await;

        match result {
            Ok(result) => result,
            Err(_) => {
                child.kill().await?;
                child.wait().await?;
                Err(SpeechError::Failed("speech synthesis timed out".into()))
            }
        }
    }

    pub async fn transcribe(&self, audio: Vec<u8>) -> Result<TranscriptionResponse, SpeechError> {
        if !self.recognition_available() {
            return Err(SpeechError::Unavailable(
                "multilingual Whisper model is missing",
            ));
        }

        let permit = self
            .recognition
            .clone()
            .try_acquire_owned()
            .map_err(|_| SpeechError::Busy)?;
        let context = self.context.clone();
        let model = self.model.clone();

        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut reader = hound::WavReader::new(Cursor::new(audio))
                .map_err(|_| SpeechError::Invalid("invalid WAV audio"))?;
            let spec = reader.spec();

            if spec.channels != 1
                || spec.sample_rate != 16000
                || spec.bits_per_sample != 16
                || spec.sample_format != hound::SampleFormat::Int
            {
                return Err(SpeechError::Invalid("audio must be mono 16 kHz PCM16 WAV"));
            }

            if reader.duration() == 0 || reader.duration() > 30 * 16000 {
                return Err(SpeechError::Invalid(
                    "audio must contain between one sample and 30 seconds",
                ));
            }

            let samples = reader
                .samples::<i16>()
                .map(|sample| sample.map(|sample| sample as f32 / 32768.0))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| SpeechError::Invalid("invalid PCM samples"))?;
            let mut context = context
                .lock()
                .map_err(|_| SpeechError::Failed("Whisper context lock poisoned".into()))?;

            if context.is_none() {
                let model = model
                    .to_str()
                    .ok_or(SpeechError::Invalid("model path must be UTF-8"))?;
                let loaded =
                    WhisperContext::new_with_params(model, WhisperContextParameters::default())
                        .map_err(|error| SpeechError::Failed(error.to_string()))?;

                if !loaded.is_multilingual() {
                    return Err(SpeechError::Failed(
                        "Serbian requires a multilingual Whisper model".into(),
                    ));
                }

                *context = Some(loaded);
            }

            let context = context
                .as_ref()
                .ok_or(SpeechError::Failed("Whisper context unavailable".into()))?;
            let mut state = context
                .create_state()
                .map_err(|error| SpeechError::Failed(error.to_string()))?;
            let mut parameters = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            parameters.set_language(Some("sr"));
            parameters.set_translate(false);
            parameters.set_no_context(true);
            parameters.set_print_special(false);
            parameters.set_print_progress(false);
            parameters.set_print_realtime(false);
            parameters.set_print_timestamps(false);
            state
                .full(parameters, &samples)
                .map_err(|error| SpeechError::Failed(error.to_string()))?;
            let text = state
                .as_iter()
                .map(|segment| segment.to_string())
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_owned();

            Ok(TranscriptionResponse {
                target_id: TARGET_ID.into(),
                text,
            })
        })
        .await
        .map_err(|error| SpeechError::Failed(error.to_string()))?
    }
}

async fn read_bounded(
    reader: impl AsyncRead + Unpin,
    limit: usize,
) -> Result<Vec<u8>, SpeechError> {
    let mut output = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut output)
        .await?;

    if output.len() > limit {
        return Err(SpeechError::Failed(
            "speech engine output exceeded its limit".into(),
        ));
    }

    Ok(output)
}

#[derive(Debug, thiserror::Error)]
pub enum SpeechError {
    #[error("{0}")]
    Invalid(&'static str),
    #[error("{0}")]
    Unavailable(&'static str),
    #[error("speech engine is busy")]
    Busy,
    #[error("{0}")]
    Failed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
