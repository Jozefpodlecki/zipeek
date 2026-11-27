use crate::{
    candle_stt::{
        QwenAsrAssets,
        frontend::{self, FrontendError},
    },
    candle_tts::{KokoroAssets, waveform},
    runtime::models::QwenTtsAssets,
    speech_cache::TtsAudioCache,
    speech_workers::{ComputeDevice, EngineStatus, SttWorker, TtsWorker},
};

use std::{
    fmt, io,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
};

pub const MAX_TEXT_BYTES: usize = 4096;
pub const MAX_AUDIO_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_AUDIO_SAMPLES: usize = 16_000 * 30;
const MAX_WAV_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub enum SpeechError {
    Invalid(&'static str),
    Unavailable(&'static str),
    Busy,
    Timeout(&'static str),
    ModelLoad(String),
    Inference(String),
    Codec(String),
    Cancelled,
    Io(io::Error),
}

impl fmt::Display for SpeechError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Unavailable(message) => f.write_str(message),
            Self::Busy => f.write_str("speech engine is busy"),
            Self::Timeout(message) => f.write_str(message),
            Self::ModelLoad(message) | Self::Inference(message) | Self::Codec(message) => {
                f.write_str(message)
            }
            Self::Cancelled => f.write_str("speech request cancelled"),
            Self::Io(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl std::error::Error for SpeechError {}

impl From<io::Error> for SpeechError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn find_speech_models_root<'a>(roots: impl IntoIterator<Item = &'a Path>) -> Option<PathBuf> {
    roots
        .into_iter()
        .flat_map(Path::ancestors)
        .map(|parent| parent.join("models").join("speech"))
        .find(|candidate| candidate.is_dir())
}

fn default_speech_models_root() -> PathBuf {
    let working_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let executable = std::env::current_exe().ok();
    let roots = std::iter::once(working_directory.as_path())
        .chain(executable.as_deref().and_then(Path::parent));
    find_speech_models_root(roots).unwrap_or_else(|| working_directory.join("models/speech"))
}

fn read_device(key: &str, default: &str) -> io::Result<ComputeDevice> {
    let value = std::env::var(key).unwrap_or_else(|_| default.to_owned());
    match value.as_str() {
        "cpu" => Ok(ComputeDevice::Cpu),
        "cuda" => Ok(ComputeDevice::Cuda),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{key} must be cpu or cuda"),
        )),
    }
}

struct CancelOnDrop(Arc<AtomicBool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[derive(Clone)]
pub struct Speech {
    espeak: PathBuf,
    espeak_data: PathBuf,
    candle_stt_model: PathBuf,
    candle_tts_model: PathBuf,
    qwen_tts_model: PathBuf,
    synthesis: Arc<Semaphore>,
    recognition: Arc<Semaphore>,
    tts_worker: TtsWorker,
    stt_worker: SttWorker,
    tts_cache: Arc<Mutex<TtsAudioCache>>,
}

impl Speech {
    pub fn new() -> io::Result<Self> {
        let resources = std::env::current_exe()?
            .parent()
            .unwrap_or(Path::new("."))
            .join("resources");
        let speech_models = default_speech_models_root();
        let espeak = std::env::var_os("ZIPEEK_ESPEAK_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                resources.join(if cfg!(windows) {
                    "espeak-ng.exe"
                } else {
                    "espeak-ng"
                })
            });
        let espeak_data = std::env::var_os("ZIPEEK_ESPEAK_DATA")
            .map(PathBuf::from)
            .unwrap_or(resources);
        let candle_stt_model = std::env::var_os("ZIPEEK_CANDLE_STT_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|| speech_models.join("Qwen3-ASR-0.6B/model.safetensors"));
        let qwen_tts_model = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                speech_models.join("Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
            });
        let candle_tts_model = std::env::var_os("ZIPEEK_CANDLE_TTS_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|| speech_models.join("Kokoro-82M-v1.1-zh/kokoro-v1_1-zh.pth"));
        let tts_device = read_device("ZIPEEK_TTS_DEVICE", "cuda")?;
        let stt_device = read_device("ZIPEEK_STT_DEVICE", "cpu")?;
        Ok(Self::build(
            espeak,
            espeak_data,
            candle_stt_model,
            qwen_tts_model,
            candle_tts_model,
            tts_device,
            stt_device,
        ))
    }

    pub fn with_paths(espeak: PathBuf, espeak_data: PathBuf, candle_stt_model: PathBuf) -> Self {
        let qwen_tts_model = PathBuf::from("zipeek-model-not-configured/model.safetensors");
        let candle_tts_model = std::env::var_os("ZIPEEK_CANDLE_TTS_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                default_speech_models_root().join("Kokoro-82M-v1.1-zh/kokoro-v1_1-zh.pth")
            });
        Self::build(
            espeak,
            espeak_data,
            candle_stt_model,
            qwen_tts_model,
            candle_tts_model,
            ComputeDevice::Cuda,
            ComputeDevice::Cpu,
        )
    }

    fn build(
        espeak: PathBuf,
        espeak_data: PathBuf,
        candle_stt_model: PathBuf,
        qwen_tts_model: PathBuf,
        candle_tts_model: PathBuf,
        tts_device: ComputeDevice,
        stt_device: ComputeDevice,
    ) -> Self {
        let tts_installed = QwenTtsAssets::from_checkpoint(&qwen_tts_model)
            .validate()
            .is_ok();
        let stt_installed = QwenAsrAssets::from_checkpoint(&candle_stt_model)
            .validate()
            .is_ok();
        let tts_worker = TtsWorker::start(
            qwen_tts_model
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf(),
            tts_device,
            tts_installed,
        );
        let stt_worker = SttWorker::start(candle_stt_model.clone(), stt_device, stt_installed);
        Self {
            espeak,
            espeak_data,
            candle_stt_model,
            qwen_tts_model,
            candle_tts_model,
            synthesis: Arc::new(Semaphore::new(1)),
            recognition: Arc::new(Semaphore::new(1)),
            tts_worker,
            stt_worker,
            tts_cache: Arc::new(Mutex::new(TtsAudioCache::new())),
        }
    }

    pub async fn shutdown_workers(&self) -> Result<(), SpeechError> {
        let (tts, stt) = tokio::join!(self.tts_worker.shutdown(), self.stt_worker.shutdown());
        tts?;
        stt?;
        Ok(())
    }

    pub async fn initialize_neural_engines(&self) -> Result<(), SpeechError> {
        if self.qwen_tts_model.is_file() {
            let permit = self
                .synthesis
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| SpeechError::Unavailable("Qwen3-TTS worker is shutting down"))?;
            let result = self.tts_worker.warm().await;
            drop(permit);
            result?;
        }
        if self.candle_stt_model.is_file() {
            let permit = self
                .recognition
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| SpeechError::Unavailable("Qwen3-ASR worker is shutting down"))?;
            let result = self.stt_worker.warm().await;
            drop(permit);
            result?;
        }
        Ok(())
    }

    pub fn neural_tts_engine_loaded(&self) -> bool {
        self.tts_worker.status() == EngineStatus::Ready
    }

    pub fn neural_stt_engine_loaded(&self) -> bool {
        self.stt_worker.status() == EngineStatus::Ready
    }

    pub fn neural_tts_status(&self) -> &'static str {
        self.tts_worker.status().as_str()
    }

    pub fn neural_stt_status(&self) -> &'static str {
        self.stt_worker.status().as_str()
    }

    pub fn neural_tts_failure(&self) -> &'static str {
        self.tts_worker.failure_code()
    }

    pub fn neural_stt_failure(&self) -> &'static str {
        self.stt_worker.failure_code()
    }

    pub fn synthesis_available(&self) -> bool {
        if self.qwen_tts_checkpoint_present()
            && crate::candle_tts::qwen::NEURAL_WAVEFORM_DECODER_READY
        {
            self.neural_tts_engine_loaded()
        } else {
            self.espeak_available()
        }
    }

    fn espeak_available(&self) -> bool {
        self.espeak.is_file() && self.espeak_data.join("espeak-ng-data").is_dir()
    }

    pub fn qwen_tts_checkpoint_present(&self) -> bool {
        QwenTtsAssets::from_checkpoint(&self.qwen_tts_model)
            .validate()
            .is_ok()
    }

    pub fn recognition_available(&self) -> bool {
        self.neural_stt_engine_loaded()
    }

    pub fn candle_stt_checkpoint_present(&self) -> bool {
        QwenAsrAssets::from_checkpoint(&self.candle_stt_model)
            .validate()
            .is_ok()
    }

    pub fn probe_candle_stt_weights(
        &self,
        device: &candle_core::Device,
    ) -> Result<(), SpeechError> {
        crate::candle_stt::weights::probe_qwen_asr_checkpoint(&self.candle_stt_model, device)
            .map_err(SpeechError::ModelLoad)
    }

    pub fn candle_tts_checkpoint_present(&self) -> bool {
        KokoroAssets::from_checkpoint(&self.candle_tts_model)
            .validate()
            .is_ok()
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, SpeechError> {
        validate_text(text)?;
        if !self.synthesis_available() && !self.qwen_tts_checkpoint_present() {
            return Err(SpeechError::Unavailable(
                "Qwen3-TTS checkpoint or eSpeak NG backend is unavailable",
            ));
        }
        let cache_key = if self.qwen_tts_checkpoint_present()
            && crate::candle_tts::qwen::NEURAL_WAVEFORM_DECODER_READY
        {
            format!("qwen:{text}")
        } else {
            format!("espeak:{text}")
        };
        if let Some(cached) = self
            .tts_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&cache_key)
        {
            return Ok(cached);
        }
        let permit = self
            .synthesis
            .clone()
            .try_acquire_owned()
            .map_err(|_| SpeechError::Busy)?;
        if self.qwen_tts_checkpoint_present()
            && crate::candle_tts::qwen::NEURAL_WAVEFORM_DECODER_READY
        {
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancellation_guard = CancelOnDrop(Arc::clone(&cancelled));
            let result = tokio::time::timeout(
                Duration::from_secs(180),
                self.tts_worker.generate(text.to_owned(), cancelled, permit),
            )
            .await
            .map_err(|_| SpeechError::Timeout("Qwen3-TTS inference timed out"))?;
            drop(cancellation_guard);
            if let Ok(ref wav) = result {
                self.tts_cache
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(cache_key, wav.clone());
            }
            return result;
        }
        let _permit = permit;
        let mut command = Command::new(&self.espeak);
        command
            .args(["--stdin", "--stdout", "-v", "cmn", "-s", "145"])
            .arg(format!("--path={}", self.espeak_data.display()))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        let mut child = command.spawn()?;
        let mut input = child
            .stdin
            .take()
            .ok_or(SpeechError::Inference("missing speech stdin".into()))?;
        let output = child
            .stdout
            .take()
            .ok_or(SpeechError::Inference("missing speech stdout".into()))?;
        let errors = child
            .stderr
            .take()
            .ok_or(SpeechError::Inference("missing speech stderr".into()))?;
        let result = tokio::time::timeout(Duration::from_secs(30), async {
            input.write_all(text.as_bytes()).await?;
            input.shutdown().await?;
            drop(input);
            let (output, errors, status) = tokio::try_join!(
                read_bounded(output, MAX_WAV_BYTES),
                read_bounded(errors, 8192),
                async { child.wait().await.map_err(SpeechError::Io) }
            )?;
            if !status.success() {
                return Err(SpeechError::Inference(format!(
                    "speech synthesis failed: {}",
                    String::from_utf8_lossy(&errors)
                )));
            }
            finalize_synthesis_wav(output)
        })
        .await;
        match result {
            Ok(Ok(output)) => {
                self.tts_cache
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(cache_key, output.clone());
                Ok(output)
            }
            Ok(Err(error)) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                Err(error)
            }
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                Err(SpeechError::Timeout("speech synthesis timed out"))
            }
        }
    }

    pub async fn transcribe(&self, audio: Vec<u8>) -> Result<String, SpeechError> {
        self.transcribe_audio(audio, crate::audio_codec::InputFormat::Wav)
            .await
    }

    pub async fn transcribe_audio(
        &self,
        audio: Vec<u8>,
        format: crate::audio_codec::InputFormat,
    ) -> Result<String, SpeechError> {
        let permit = self
            .recognition
            .clone()
            .try_acquire_owned()
            .map_err(|_| SpeechError::Busy)?;
        let samples =
            tokio::task::spawn_blocking(move || crate::audio_codec::decode_for_stt(&audio, format))
                .await
                .map_err(|error| {
                    SpeechError::Codec(format!("audio decode worker failed: {error}"))
                })??;
        if samples.iter().all(|value| value.abs() <= 1e-6) {
            return Err(SpeechError::Invalid(
                "audio contains no audible speech signal",
            ));
        }
        if !self.candle_stt_checkpoint_present() {
            return Err(SpeechError::Unavailable(
                "Qwen3-ASR checkpoint assets are missing",
            ));
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancellation_guard = CancelOnDrop(Arc::clone(&cancelled));
        let result = tokio::time::timeout(
            Duration::from_secs(180),
            self.stt_worker.recognize(samples, cancelled, permit),
        )
        .await
        .map_err(|_| SpeechError::Timeout("Qwen3-ASR inference timed out"))?;
        drop(cancellation_guard);
        let transcript = result?;
        validate_transcript(transcript)
    }
}

pub fn prepare_stt_features(audio: &[u8]) -> Result<Vec<f32>, SpeechError> {
    let samples = decode_pcm16_wav(audio)?;
    if samples.iter().all(|sample| sample.abs() <= 1e-6) {
        return Err(SpeechError::Invalid(
            "audio contains no audible speech signal",
        ));
    }
    frontend::log_mel_spectrogram(&samples).map_err(|error| match error {
        FrontendError::Empty => SpeechError::Invalid("audio has no PCM samples"),
        FrontendError::TooLong => SpeechError::Invalid("audio exceeds 30 seconds"),
        FrontendError::NonFinite => SpeechError::Invalid("invalid PCM samples"),
    })
}

pub fn encode_native_tts_waveform(
    samples: &[f32],
    sample_rate: u32,
) -> Result<Vec<u8>, SpeechError> {
    waveform::encode_mono_pcm16_wav(samples, sample_rate)
        .map_err(|error| SpeechError::Codec(format!("invalid neural speech waveform: {error:?}")))
}

pub fn prepare_stt_audio_input(
    audio: &[u8],
    device: &candle_core::Device,
) -> Result<crate::candle_stt::tensor::QwenAudioInput, SpeechError> {
    let samples = decode_pcm16_wav(audio)?;
    if samples.iter().all(|sample| sample.abs() <= 1e-6) {
        return Err(SpeechError::Invalid(
            "audio contains no audible speech signal",
        ));
    }
    crate::candle_stt::tensor::pcm_to_qwen_audio_input(&samples, device)
        .map_err(|error| SpeechError::Inference(error.to_string()))
}

pub fn prepare_stt_tensor(
    audio: &[u8],
    device: &candle_core::Device,
) -> Result<candle_core::Tensor, SpeechError> {
    prepare_stt_audio_input(audio, device).map(|input| input.features)
}

pub fn validate_text(text: &str) -> Result<(), SpeechError> {
    if text.trim().is_empty() || text.len() > MAX_TEXT_BYTES || text.chars().any(char::is_control) {
        return Err(SpeechError::Invalid(
            "text must be nonempty, at most 4096 bytes, and contain no control characters",
        ));
    }
    Ok(())
}

pub fn validate_transcript(transcript: String) -> Result<String, SpeechError> {
    let transcript = transcript.trim();
    if transcript.is_empty() {
        return Err(SpeechError::Inference(
            "speech recognition produced no transcript".into(),
        ));
    }
    if transcript.len() > MAX_TEXT_BYTES || transcript.chars().any(char::is_control) {
        return Err(SpeechError::Inference(
            "speech recognition produced an invalid transcript".into(),
        ));
    }
    Ok(transcript.to_owned())
}

pub fn finalize_synthesis_wav(mut wav: Vec<u8>) -> Result<Vec<u8>, SpeechError> {
    let invalid = || SpeechError::Codec("speech engine returned invalid PCM16 WAV".into());
    if wav.len() < 46
        || wav.len() > MAX_WAV_BYTES
        || wav.len() - 8 > u32::MAX as usize
        || &wav[..4] != b"RIFF"
        || &wav[8..12] != b"WAVE"
        || &wav[12..16] != b"fmt "
        || le_u32(&wav[16..20]) != 16
        || le_u16(&wav[20..22]) != 1
        || le_u16(&wav[22..24]) != 1
        || &wav[36..40] != b"data"
        || le_u16(&wav[32..34]) != 2
        || le_u16(&wav[34..36]) != 16
    {
        return Err(invalid());
    }
    let rate = le_u32(&wav[24..28]);
    if !(8_000..=96_000).contains(&rate)
        || le_u32(&wav[28..32]) != rate * 2
        || (wav.len() - 44) % 2 != 0
    {
        return Err(invalid());
    }
    let data_size = u32::try_from(wav.len() - 44).map_err(|_| invalid())?;
    let riff_size = u32::try_from(wav.len() - 8).map_err(|_| invalid())?;
    wav[4..8].copy_from_slice(&riff_size.to_le_bytes());
    wav[40..44].copy_from_slice(&data_size.to_le_bytes());
    Ok(wav)
}

fn le_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub fn decode_pcm16_wav(audio: &[u8]) -> Result<Vec<f32>, SpeechError> {
    let invalid =
        || SpeechError::Invalid("audio must be mono 16 kHz PCM16 WAV, at most 30 seconds");
    if audio.len() < 44
        || audio.len() > MAX_AUDIO_BYTES
        || &audio[..4] != b"RIFF"
        || &audio[8..12] != b"WAVE"
        || le_u32(&audio[4..8]) as usize != audio.len() - 8
    {
        return Err(invalid());
    }
    let mut offset = 12;
    let mut format_found = false;
    let mut data: Option<&[u8]> = None;
    while offset + 8 <= audio.len() {
        let chunk_size = le_u32(&audio[offset + 4..offset + 8]) as usize;
        let start = offset + 8;
        let end = start.checked_add(chunk_size).ok_or_else(invalid)?;
        if end > audio.len() {
            return Err(invalid());
        }
        match &audio[offset..offset + 4] {
            b"fmt " => {
                if format_found
                    || chunk_size < 16
                    || le_u16(&audio[start..start + 2]) != 1
                    || le_u16(&audio[start + 2..start + 4]) != 1
                    || le_u32(&audio[start + 4..start + 8]) != 16_000
                    || le_u32(&audio[start + 8..start + 12]) != 32_000
                    || le_u16(&audio[start + 12..start + 14]) != 2
                    || le_u16(&audio[start + 14..start + 16]) != 16
                {
                    return Err(invalid());
                }
                format_found = true;
            }
            b"data" => {
                if data.is_some() {
                    return Err(invalid());
                }
                data = Some(&audio[start..end]);
            }
            _ => {}
        }
        offset = end.checked_add(chunk_size % 2).ok_or_else(invalid)?;
    }
    if !format_found || offset != audio.len() {
        return Err(invalid());
    }
    let data = data.ok_or_else(invalid)?;
    if data.len() < 2 || data.len() % 2 != 0 || data.len() / 2 > MAX_AUDIO_SAMPLES {
        return Err(invalid());
    }
    Ok(data
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0)
        .collect())
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
        return Err(SpeechError::Inference(
            "speech engine output exceeded its limit".into(),
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_request_guard_signals_cancellation() {
        let flag = Arc::new(AtomicBool::new(false));
        {
            let _guard = CancelOnDrop(Arc::clone(&flag));
            assert!(!flag.load(Ordering::Relaxed));
        }
        assert!(flag.load(Ordering::Relaxed));
    }

    fn wav(samples: &[i16]) -> Vec<u8> {
        let length = (samples.len() * 2) as u32;
        let mut result = Vec::new();
        result.extend_from_slice(b"RIFF");
        result.extend_from_slice(&(36 + length).to_le_bytes());
        result.extend_from_slice(b"WAVEfmt ");
        result.extend_from_slice(&16_u32.to_le_bytes());
        result.extend_from_slice(&1_u16.to_le_bytes());
        result.extend_from_slice(&1_u16.to_le_bytes());
        result.extend_from_slice(&16_000_u32.to_le_bytes());
        result.extend_from_slice(&32_000_u32.to_le_bytes());
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
    fn decodes_valid_pcm_audio() {
        let samples = decode_pcm16_wav(&wav(&[0, -32768, 16384])).unwrap();
        assert_eq!(samples, vec![0.0, -1.0, 0.5]);
    }

    #[test]
    fn rejects_malformed_and_wrong_format() {
        assert!(decode_pcm16_wav(b"hello").is_err());
        let mut wrong_rate = wav(&[1]);
        wrong_rate[24..28].copy_from_slice(&8_000_u32.to_le_bytes());
        assert!(decode_pcm16_wav(&wrong_rate).is_err());
        let mut truncated = wav(&[1]);
        truncated.pop();
        assert!(decode_pcm16_wav(&truncated).is_err());
        assert!(decode_pcm16_wav(&wav(&[])).is_err());
        assert!(decode_pcm16_wav(&wav(&vec![0; MAX_AUDIO_SAMPLES + 1])).is_err());
    }

    #[test]
    fn rejects_ambiguous_wav_chunks_and_unsupported_encoding() {
        let mut duplicate_data = wav(&[1]);
        duplicate_data.extend_from_slice(b"data");
        duplicate_data.extend_from_slice(&2_u32.to_le_bytes());
        duplicate_data.extend_from_slice(&1_i16.to_le_bytes());
        let size = (duplicate_data.len() - 8) as u32;
        duplicate_data[4..8].copy_from_slice(&size.to_le_bytes());
        assert!(decode_pcm16_wav(&duplicate_data).is_err());

        let mut stereo = wav(&[1]);
        stereo[22..24].copy_from_slice(&2_u16.to_le_bytes());
        assert!(decode_pcm16_wav(&stereo).is_err());

        let mut wrong_encoding = wav(&[1]);
        wrong_encoding[20..22].copy_from_slice(&3_u16.to_le_bytes());
        assert!(decode_pcm16_wav(&wrong_encoding).is_err());

        let mut oversized_chunk = wav(&[1]);
        oversized_chunk[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_pcm16_wav(&oversized_chunk).is_err());
    }

    #[test]
    fn accepts_wav_with_padded_unknown_chunk() {
        let mut audio = wav(&[1, -1]);
        audio.splice(36..36, [b'J', b'U', b'N', b'K', 1, 0, 0, 0, 42, 0]);
        let size = (audio.len() - 8) as u32;
        audio[4..8].copy_from_slice(&size.to_le_bytes());
        assert_eq!(decode_pcm16_wav(&audio).unwrap().len(), 2);
    }

    #[test]
    fn validates_synthesis_text() {
        assert!(validate_text("你好，世界").is_ok());
        assert!(validate_text("").is_err());
        assert!(validate_text("hello\nworld").is_err());
        assert!(validate_text(&"好".repeat(1400)).is_err());
    }

    #[test]
    fn finalizes_espeak_stdout_placeholder_lengths() {
        let mut audio = wav(&[1, -2, 3]);
        audio[24..28].copy_from_slice(&22_050_u32.to_le_bytes());
        audio[28..32].copy_from_slice(&44_100_u32.to_le_bytes());
        audio[4..8].copy_from_slice(&0_u32.to_le_bytes());
        audio[40..44].copy_from_slice(&0_u32.to_le_bytes());
        let finalized = finalize_synthesis_wav(audio).unwrap();
        assert_eq!(le_u32(&finalized[4..8]) as usize, finalized.len() - 8);
        assert_eq!(le_u32(&finalized[40..44]) as usize, finalized.len() - 44);
        assert_eq!(&finalized[44..], &[1, 0, 254, 255, 3, 0]);
    }

    #[test]
    fn rejects_invalid_synthesized_audio() {
        let valid = wav(&[1]);
        let mut wrong_format = valid.clone();
        wrong_format[20..22].copy_from_slice(&3_u16.to_le_bytes());
        assert!(finalize_synthesis_wav(wrong_format).is_err());
        let mut wrong_channels = valid.clone();
        wrong_channels[22..24].copy_from_slice(&2_u16.to_le_bytes());
        assert!(finalize_synthesis_wav(wrong_channels).is_err());
        let mut wrong_bitrate = valid.clone();
        wrong_bitrate[28..32].copy_from_slice(&1_u32.to_le_bytes());
        assert!(finalize_synthesis_wav(wrong_bitrate).is_err());
        assert!(finalize_synthesis_wav(valid[..45].to_vec()).is_err());
        assert!(finalize_synthesis_wav(wav(&[])).is_err());
    }

    #[test]
    fn rejects_empty_or_unsafe_recognition_output() {
        assert_eq!(validate_transcript(" 你好。 ".into()).unwrap(), "你好。");
        assert!(validate_transcript("  ".into()).is_err());
        assert!(validate_transcript("你好\n世界".into()).is_err());
        assert!(validate_transcript("好".repeat(1400)).is_err());
    }

    #[test]
    fn speech_models_are_found_in_checkout_from_nested_executable_directory() {
        let root = std::env::temp_dir().join(format!(
            "zipeek-speech-model-path-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let models = root.join("models/speech");
        let nested_executable = root.join("target/debug/deps/zipeek-tts-stt.exe");
        std::fs::create_dir_all(&models).unwrap();
        let found = find_speech_models_root([nested_executable.as_path()]);
        assert_eq!(found.as_deref(), Some(models.as_path()));
        assert_eq!(
            models.join("Qwen3-ASR-0.6B/model.safetensors"),
            root.join("models/speech/Qwen3-ASR-0.6B/model.safetensors")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn readiness_queries_are_nonblocking_and_shared() {
        let speech = Speech::with_paths(
            PathBuf::from("missing-executable"),
            PathBuf::from("missing-data"),
            PathBuf::from("missing-model"),
        );

        assert!(!speech.neural_tts_engine_loaded());
        assert!(!speech.neural_stt_engine_loaded());
        let clone = speech.clone();
        assert_eq!(clone.neural_tts_status(), "not_installed");
        assert!(!speech.neural_stt_engine_loaded());
    }

    #[test]
    fn empty_resources_report_unavailable() {
        let speech = Speech::with_paths(
            PathBuf::from("not-an-executable"),
            PathBuf::from("absent"),
            PathBuf::from("absent-model"),
        );
        assert!(!speech.synthesis_available());
        assert!(!speech.recognition_available());
    }
}
