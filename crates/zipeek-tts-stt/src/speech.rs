use std::{
    fmt,
    io,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub const MAX_TEXT_BYTES: usize = 4096;
pub const MAX_AUDIO_BYTES: usize = 960_044;
pub const MAX_AUDIO_SAMPLES: usize = 16_000 * 30;
const MAX_WAV_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub enum SpeechError {
    Invalid(&'static str),
    Unavailable(&'static str),
    Busy,
    Failed(String),
    Io(io::Error),
}

impl fmt::Display for SpeechError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Unavailable(message) => f.write_str(message),
            Self::Busy => f.write_str("speech engine is busy"),
            Self::Failed(message) => f.write_str(message),
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

#[derive(Clone)]
pub struct Speech {
    espeak: PathBuf,
    espeak_data: PathBuf,
    whisper_model: PathBuf,
    context: Arc<Mutex<Option<WhisperContext>>>,
    synthesis: Arc<Semaphore>,
    recognition: Arc<Semaphore>,
}

impl Speech {
    pub fn new() -> io::Result<Self> {
        let resources = std::env::current_exe()?
            .parent()
            .unwrap_or(Path::new("."))
            .join("resources");
        Ok(Self::with_paths(
            std::env::var_os("ZIPEEK_ESPEAK_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| resources.join(if cfg!(windows) { "espeak-ng.exe" } else { "espeak-ng" })),
            std::env::var_os("ZIPEEK_ESPEAK_DATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| resources.clone()),
            std::env::var_os("ZIPEEK_WHISPER_MODEL")
                .map(PathBuf::from)
                .unwrap_or_else(|| resources.join("ggml-model.bin")),
        ))
    }

    pub fn with_paths(espeak: PathBuf, espeak_data: PathBuf, whisper_model: PathBuf) -> Self {
        Self {
            espeak,
            espeak_data,
            whisper_model,
            context: Arc::new(Mutex::new(None)),
            synthesis: Arc::new(Semaphore::new(1)),
            recognition: Arc::new(Semaphore::new(1)),
        }
    }

    pub fn synthesis_available(&self) -> bool {
        self.espeak.is_file() && self.espeak_data.join("espeak-ng-data").is_dir()
    }

    pub fn recognition_available(&self) -> bool {
        self.whisper_model.is_file()
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, SpeechError> {
        validate_text(text)?;
        if !self.synthesis_available() {
            return Err(SpeechError::Unavailable("eSpeak NG executable or voice data missing"));
        }
        let _permit = self.synthesis.clone().try_acquire_owned().map_err(|_| SpeechError::Busy)?;
        let mut command = Command::new(&self.espeak);
        command.args(["--stdin", "--stdout", "-v", "cmn", "-s", "145"])
            .arg(format!("--path={}", self.espeak_data.display()))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        let mut child = command.spawn()?;
        let mut input = child.stdin.take().ok_or(SpeechError::Failed("missing speech stdin".into()))?;
        let output = child.stdout.take().ok_or(SpeechError::Failed("missing speech stdout".into()))?;
        let errors = child.stderr.take().ok_or(SpeechError::Failed("missing speech stderr".into()))?;
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
                return Err(SpeechError::Failed(format!("speech synthesis failed: {}", String::from_utf8_lossy(&errors))));
            }
            finalize_synthesis_wav(output)
        }).await;
        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(error)) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                Err(error)
            }
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                Err(SpeechError::Failed("speech synthesis timed out".into()))
            }
        }
    }

    pub async fn transcribe(&self, audio: Vec<u8>) -> Result<String, SpeechError> {
        let samples = decode_pcm16_wav(&audio)?;
        if !self.recognition_available() {
            return Err(SpeechError::Unavailable("multilingual Whisper model missing"));
        }
        let permit = self.recognition.clone().try_acquire_owned().map_err(|_| SpeechError::Busy)?;
        let model = self.whisper_model.clone();
        let context = self.context.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut guard = context.lock().map_err(|_| SpeechError::Failed("Whisper lock poisoned".into()))?;
            if guard.is_none() {
                let model = model.to_str().ok_or(SpeechError::Invalid("model path must be UTF-8"))?;
                let loaded = WhisperContext::new_with_params(model, WhisperContextParameters::default())
                    .map_err(|error| SpeechError::Failed(error.to_string()))?;
                if !loaded.is_multilingual() {
                    return Err(SpeechError::Failed("Mandarin requires a multilingual Whisper model".into()));
                }
                *guard = Some(loaded);
            }
            let context = guard.as_ref().ok_or(SpeechError::Failed("Whisper context unavailable".into()))?;
            let mut state = context.create_state().map_err(|error| SpeechError::Failed(error.to_string()))?;
            let mut parameters = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            parameters.set_language(Some("zh"));
            parameters.set_translate(false);
            parameters.set_no_context(true);
            parameters.set_print_special(false);
            parameters.set_print_progress(false);
            parameters.set_print_realtime(false);
            parameters.set_print_timestamps(false);
            state.full(parameters, &samples).map_err(|error| SpeechError::Failed(error.to_string()))?;
            validate_transcript(
                state.as_iter().map(|segment| segment.to_string()).collect::<String>(),
            )
        }).await.map_err(|error| SpeechError::Failed(error.to_string()))?
    }
}

pub fn validate_text(text: &str) -> Result<(), SpeechError> {
    if text.trim().is_empty() || text.len() > MAX_TEXT_BYTES || text.chars().any(char::is_control) {
        return Err(SpeechError::Invalid("text must be nonempty, at most 4096 bytes, and contain no control characters"));
    }
    Ok(())
}

pub fn validate_transcript(transcript: String) -> Result<String, SpeechError> {
    let transcript = transcript.trim();
    if transcript.is_empty() {
        return Err(SpeechError::Failed("speech recognition produced no transcript".into()));
    }
    if transcript.len() > MAX_TEXT_BYTES || transcript.chars().any(char::is_control) {
        return Err(SpeechError::Failed("speech recognition produced an invalid transcript".into()));
    }
    Ok(transcript.to_owned())
}

pub fn finalize_synthesis_wav(mut wav: Vec<u8>) -> Result<Vec<u8>, SpeechError> {
    let invalid = || SpeechError::Failed("speech engine returned invalid PCM16 WAV".into());
    if wav.len() < 46 || wav.len() > MAX_WAV_BYTES || wav.len() - 8 > u32::MAX as usize
        || &wav[..4] != b"RIFF" || &wav[8..12] != b"WAVE"
        || &wav[12..16] != b"fmt " || le_u32(&wav[16..20]) != 16
        || le_u16(&wav[20..22]) != 1 || le_u16(&wav[22..24]) != 1
        || &wav[36..40] != b"data" || le_u16(&wav[32..34]) != 2
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
    wav[4..8].copy_from_slice(&u32::try_from(wav.len() - 8).map_err(|_| invalid())?.to_le_bytes());
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
    let invalid = || SpeechError::Invalid("audio must be mono 16 kHz PCM16 WAV, at most 30 seconds");
    if audio.len() < 44 || audio.len() > MAX_AUDIO_BYTES || &audio[..4] != b"RIFF"
        || &audio[8..12] != b"WAVE" || le_u32(&audio[4..8]) as usize != audio.len() - 8 {
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
                if format_found || chunk_size < 16 || le_u16(&audio[start..start + 2]) != 1
                    || le_u16(&audio[start + 2..start + 4]) != 1
                    || le_u32(&audio[start + 4..start + 8]) != 16_000
                    || le_u32(&audio[start + 8..start + 12]) != 32_000
                    || le_u16(&audio[start + 12..start + 14]) != 2
                    || le_u16(&audio[start + 14..start + 16]) != 16 {
                    return Err(invalid());
                }
                format_found = true;
            }
            b"data" => {
                if data.is_some() { return Err(invalid()); }
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
    Ok(data.chunks_exact(2).map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0).collect())
}

async fn read_bounded(reader: impl AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>, SpeechError> {
    let mut output = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut output).await?;
    if output.len() > limit {
        return Err(SpeechError::Failed("speech engine output exceeded its limit".into()));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for sample in samples { result.extend_from_slice(&sample.to_le_bytes()); }
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
    fn empty_resources_report_unavailable() {
        let speech = Speech::with_paths(PathBuf::from("not-an-executable"), PathBuf::from("absent"), PathBuf::from("absent-model"));
        assert!(!speech.synthesis_available());
        assert!(!speech.recognition_available());
    }
}
