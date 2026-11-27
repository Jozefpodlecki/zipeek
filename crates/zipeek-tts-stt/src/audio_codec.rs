use std::io::Cursor;

use opus_pure::{
    Application, MAX_PACKET_BYTES, MAX_PACKET_SAMPLES, OggOpusReader, OggOpusWriter, OpusEncoder,
    OpusHead, Trim,
};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error as SymphoniaError,
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};
use symphonia::default::{get_codecs, get_probe};

use crate::speech::{MAX_AUDIO_BYTES, MAX_AUDIO_SAMPLES, SpeechError, decode_pcm16_wav};

const INPUT_SAMPLE_LIMIT: usize = 96_000 * 30;
const OPUS_SAMPLE_RATE: u32 = 48_000;
const MAX_TTS_SAMPLES: usize = 96_000 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFormat {
    Wav,
    Mp3,
    Flac,
    OggOpus,
}

impl InputFormat {
    pub fn from_content_type(value: &str) -> Option<Self> {
        match value {
            "audio/wav" | "audio/x-wav" => Some(Self::Wav),
            "audio/mpeg" => Some(Self::Mp3),
            "audio/flac" | "audio/x-flac" => Some(Self::Flac),
            "audio/ogg" | "audio/opus" => Some(Self::OggOpus),
            _ => None,
        }
    }
}

fn invalid() -> SpeechError {
    SpeechError::Invalid("unsupported, malformed or oversized audio; maximum duration 30 seconds")
}

pub fn decode_for_stt(bytes: &[u8], format: InputFormat) -> Result<Vec<f32>, SpeechError> {
    if bytes.is_empty() || bytes.len() > MAX_AUDIO_BYTES {
        return Err(invalid());
    }
    let pcm = match format {
        InputFormat::Wav => return decode_pcm16_wav(bytes),
        InputFormat::OggOpus => decode_opus(bytes)?,
        InputFormat::Mp3 | InputFormat::Flac => {
            decode_symphonia(bytes, format, INPUT_SAMPLE_LIMIT)?
        }
    };
    let samples = resample_mono(&pcm.0, pcm.1, 16_000, MAX_AUDIO_SAMPLES)?;
    if samples.is_empty() || samples.iter().any(|sample| !sample.is_finite()) {
        return Err(invalid());
    }
    Ok(samples)
}

fn decode_opus(bytes: &[u8]) -> Result<(Vec<f32>, u32), SpeechError> {
    if !bytes.starts_with(b"OggS") {
        return Err(invalid());
    }
    let mut reader = OggOpusReader::new(Cursor::new(bytes)).map_err(|_| invalid())?;
    let channels = reader.head().channel_count as usize;
    if !(1..=2).contains(&channels) {
        return Err(invalid());
    }
    let mut decoder = reader.head().decoder(48_000).map_err(|_| invalid())?;
    let mut trim = Trim::new(reader.head(), 48_000, channels).map_err(|_| invalid())?;
    let mut block = vec![0.0f32; MAX_PACKET_SAMPLES * channels];
    let mut mono = Vec::new();
    for packet in reader.packets() {
        let packet = packet.map_err(|_| invalid())?;
        let count = decoder
            .decode(&packet.data, MAX_PACKET_SAMPLES, &mut block)
            .map_err(|_| invalid())?;
        let kept = trim.keep(&packet, &block[..count * channels]);
        append_mono(&mut mono, kept, channels, INPUT_SAMPLE_LIMIT)?;
    }
    Ok((mono, 48_000))
}

fn decode_symphonia(
    bytes: &[u8],
    format: InputFormat,
    sample_limit: usize,
) -> Result<(Vec<f32>, u32), SpeechError> {
    let valid_magic = match format {
        InputFormat::Flac => bytes.starts_with(b"fLaC"),
        InputFormat::Mp3 => {
            bytes.starts_with(b"ID3")
                || bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0
        }
        _ => false,
    };
    if !valid_magic {
        return Err(invalid());
    }
    let source = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(match format {
        InputFormat::Mp3 => "mp3",
        InputFormat::Flac => "flac",
        _ => return Err(invalid()),
    });
    let mut probed = get_probe()
        .format(
            &hint,
            source,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|error| {
            tracing::debug!(%error, "audio container probe failed");
            invalid()
        })?
        .format;
    let track = probed.default_track().ok_or_else(invalid)?;
    let id = track.id;
    let rate = track.codec_params.sample_rate.ok_or_else(invalid)?;
    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|error| {
            tracing::debug!(%error, "audio codec initialization failed");
            invalid()
        })?;
    if !(8_000..=96_000).contains(&rate) {
        return Err(invalid());
    }
    let mut mono = Vec::new();
    let mut recovered_packets = 0usize;
    loop {
        let packet = match probed.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(error) => {
                tracing::debug!(%error, "audio packet demux failed");
                return Err(invalid());
            }
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(error))
                if format == InputFormat::Mp3 && recovered_packets < 2 && !mono.is_empty() =>
            {
                recovered_packets += 1;
                tracing::warn!(%error, recovered_packets, "skipping recoverable MP3 frame error");
                continue;
            }
            Err(error) => {
                tracing::warn!(%error, "audio packet decoding failed");
                return Err(invalid());
            }
        };
        let channels = decoded.spec().channels.count();
        if !(1..=2).contains(&channels) {
            return Err(invalid());
        }
        if decoded.capacity() > sample_limit * channels {
            return Err(invalid());
        }
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        append_mono(&mut mono, buffer.samples(), channels, sample_limit)?;
    }
    Ok((mono, rate))
}

fn append_mono(
    mono: &mut Vec<f32>,
    samples: &[f32],
    channels: usize,
    limit: usize,
) -> Result<(), SpeechError> {
    if samples.len() % channels != 0 || samples.len() / channels > limit.saturating_sub(mono.len())
    {
        return Err(invalid());
    }
    for frame in samples.chunks_exact(channels) {
        let sample = frame.iter().copied().sum::<f32>() / channels as f32;
        if !sample.is_finite() {
            return Err(invalid());
        }
        mono.push(sample);
    }
    Ok(())
}

fn greatest_common_divisor(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn downsample_taps(cutoff: f64, radius: isize, fraction: f64) -> Vec<f64> {
    (-radius..=radius)
        .map(|offset| {
            let distance = fraction - offset as f64;
            let argument = std::f64::consts::PI * distance * cutoff;
            let sinc = if argument.abs() < 1e-12 {
                1.0
            } else {
                argument.sin() / argument
            };
            let phase = distance / radius as f64;
            let window = 0.42
                + 0.5 * (std::f64::consts::PI * phase).cos()
                + 0.08 * (2.0 * std::f64::consts::PI * phase).cos();
            cutoff * sinc * window
        })
        .collect()
}

pub fn resample_mono(
    samples: &[f32],
    input_rate: u32,
    output_rate: u32,
    limit: usize,
) -> Result<Vec<f32>, SpeechError> {
    if samples.is_empty()
        || input_rate == 0
        || output_rate == 0
        || samples.iter().any(|s| !s.is_finite())
    {
        return Err(invalid());
    }
    let output_len = (samples.len() as u64 * output_rate as u64).div_ceil(input_rate as u64);
    if output_len == 0 || output_len > limit as u64 {
        return Err(invalid());
    }
    if input_rate == output_rate {
        return Ok(samples.to_vec());
    }
    let ratio = input_rate as f64 / output_rate as f64;
    let mut result = Vec::with_capacity(output_len as usize);
    if input_rate > output_rate {
        let cutoff = output_rate as f64 / input_rate as f64;
        if cutoff < 1.0 / 32.0 {
            return Err(invalid());
        }
        let radius = (24.0 / cutoff).ceil() as isize;
        let phases = (output_rate / greatest_common_divisor(input_rate, output_rate)) as usize;
        let cached_taps = if phases <= 256 {
            Some(
                (0..phases)
                    .map(|phase| {
                        downsample_taps(
                            cutoff,
                            radius,
                            ((phase as u64 * input_rate as u64) % output_rate as u64) as f64
                                / output_rate as f64,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        for index in 0..output_len as usize {
            let numerator = index as u64 * input_rate as u64;
            let center = (numerator / output_rate as u64) as isize;
            let fraction = (numerator % output_rate as u64) as f64 / output_rate as f64;
            let phase = index % phases;
            let taps;
            let weights = if let Some(cached) = &cached_taps {
                &cached[phase]
            } else {
                taps = downsample_taps(cutoff, radius, fraction);
                &taps
            };
            let mut weighted = 0.0_f64;
            let mut sum = 0.0_f64;
            for (tap, weight) in weights.iter().enumerate() {
                let source = center + tap as isize - radius;
                if !(0..samples.len() as isize).contains(&source) {
                    continue;
                }
                weighted += f64::from(samples[source as usize]) * weight;
                sum += weight;
            }
            result.push(if sum.abs() > 1e-12 {
                (weighted / sum) as f32
            } else {
                0.0
            });
        }
    } else {
        for index in 0..output_len as usize {
            let position = index as f64 * ratio;
            let before = (position as usize).min(samples.len() - 1);
            let after = (before + 1).min(samples.len() - 1);
            let weight = (position - before as f64) as f32;
            result.push(samples[before] * (1.0 - weight) + samples[after] * weight);
        }
    }
    Ok(result)
}

pub fn wav_to_ogg_opus(wav: &[u8]) -> Result<Vec<u8>, SpeechError> {
    let (source, rate) = decode_tts_wav(wav)?;
    let pcm = resample_mono(&source, rate, OPUS_SAMPLE_RATE, MAX_TTS_SAMPLES * 2)?;
    let mut encoder = OpusEncoder::new(48_000, 1, Application::Voip)
        .map_err(|error| SpeechError::Codec(format!("Opus encoder: {error}")))?;
    encoder.bitrate_bps = 24_000;
    let head = OpusHead::for_encoder(&encoder, OPUS_SAMPLE_RATE);
    let frames = (pcm.len() + usize::from(head.pre_skip)).div_ceil(960);
    let final_granule = u64::from(head.pre_skip) + pcm.len() as u64;
    let mut writer = OggOpusWriter::new(Vec::new(), head)
        .map_err(|error| SpeechError::Codec(format!("Opus container: {error}")))?;
    let mut packet = vec![0u8; MAX_PACKET_BYTES];
    let mut block = [0.0f32; 960];
    for frame_index in 0..frames {
        let start = (frame_index * 960).min(pcm.len());
        let end = (start + 960).min(pcm.len());
        block.fill(0.0);
        block[..end - start].copy_from_slice(&pcm[start..end]);
        let size = encoder
            .encode(&block, 960, &mut packet)
            .map_err(|error| SpeechError::Codec(format!("Opus encoding: {error}")))?;
        if frame_index + 1 == frames {
            let duration = final_granule - writer.granule() as u64;
            writer
                .write_packet_with_duration(&packet[..size], duration as u32)
                .map_err(|error| SpeechError::Codec(format!("Opus final packet: {error}")))?;
        } else {
            writer
                .write_packet(&packet[..size])
                .map_err(|error| SpeechError::Codec(format!("Opus packet: {error}")))?;
        }
    }
    writer
        .finish()
        .map_err(|error| SpeechError::Codec(format!("Opus finish: {error}")))
}

fn encode_mp3_frames(
    config: shine_rs::Mp3EncoderConfig,
    pcm: &[i16],
) -> Result<Vec<u8>, shine_rs::EncoderError> {
    let mut encoder = shine_rs::Mp3Encoder::new(config)?;
    let frame_samples = encoder.samples_per_frame();
    let mut output = Vec::new();
    for frame in pcm.chunks(frame_samples) {
        let encoded_frames = if frame.len() == frame_samples {
            encoder.encode_interleaved(frame)?
        } else {
            let mut padded = vec![0i16; frame_samples];
            padded[..frame.len()].copy_from_slice(frame);
            encoder.encode_interleaved(&padded)?
        };
        for encoded_frame in encoded_frames {
            output.extend_from_slice(&encoded_frame);
        }
    }
    output.extend_from_slice(&encoder.finish()?);
    Ok(output)
}

pub fn wav_to_mp3(wav: &[u8]) -> Result<Vec<u8>, SpeechError> {
    let (samples, source_rate) = decode_tts_wav(wav)?;
    let mut failures = Vec::new();
    for target_rate in [48_000, 44_100, 32_000, 24_000] {
        let pcm = resample_mono(&samples, source_rate, target_rate, MAX_TTS_SAMPLES)?;
        let pcm: Vec<i16> = pcm
            .iter()
            .map(|sample| {
                (sample * 32768.0)
                    .round()
                    .clamp(i16::MIN as f32, i16::MAX as f32) as i16
            })
            .collect();
        let config = shine_rs::Mp3EncoderConfig::new()
            .sample_rate(target_rate)
            .bitrate(128)
            .channels(1)
            .stereo_mode(shine_rs::StereoMode::Mono);
        match encode_mp3_frames(config, &pcm) {
            Ok(mp3) => match decode_symphonia(&mp3, InputFormat::Mp3, MAX_TTS_SAMPLES) {
                Ok((decoded, decoded_rate)) if decoded.len() > 0 && decoded_rate == target_rate => {
                    let expected_seconds = samples.len() as f64 / f64::from(source_rate);
                    let decoded_seconds = decoded.len() as f64 / f64::from(decoded_rate);
                    if (expected_seconds - decoded_seconds).abs() <= 0.15 {
                        return Ok(mp3);
                    }
                    failures.push(format!("{target_rate} Hz: decoded duration {decoded_seconds:.3}s, expected {expected_seconds:.3}s"));
                }
                Ok((_, decoded_rate)) => failures.push(format!("{target_rate} Hz: decoded stream has unexpected rate {decoded_rate} or no samples")),
                Err(error) => failures.push(format!("{target_rate} Hz: {error}")),
            },
            Err(error) => failures.push(format!("{target_rate} Hz: {error}")),
        }
    }
    Err(SpeechError::Codec(format!(
        "MP3 encoding produced no independently decodable output: {}",
        failures.join("; ")
    )))
}

fn decode_tts_wav(wav: &[u8]) -> Result<(Vec<f32>, u32), SpeechError> {
    if wav.len() < 44
        || wav.len() > 16 * 1024 * 1024
        || &wav[0..4] != b"RIFF"
        || &wav[8..12] != b"WAVE"
    {
        return Err(invalid());
    }
    let mut pos = 12;
    let mut rate = None;
    let mut data = None;
    while pos + 8 <= wav.len() {
        let size =
            u32::from_le_bytes(wav[pos + 4..pos + 8].try_into().map_err(|_| invalid())?) as usize;
        let start = pos + 8;
        let end = start.checked_add(size).ok_or_else(invalid)?;
        if end > wav.len() {
            return Err(invalid());
        }
        match &wav[pos..pos + 4] {
            b"fmt " if size >= 16 => {
                let fmt = &wav[start..start + 16];
                let channels = u16::from_le_bytes([fmt[2], fmt[3]]);
                let encoding = u16::from_le_bytes([fmt[0], fmt[1]]);
                let bits = u16::from_le_bytes([fmt[14], fmt[15]]);
                if rate.is_some() || channels != 1 || encoding != 1 || bits != 16 {
                    return Err(invalid());
                }
                rate = Some(u32::from_le_bytes(
                    fmt[4..8].try_into().map_err(|_| invalid())?,
                ));
            }
            b"data" => {
                if data.replace(&wav[start..end]).is_some() {
                    return Err(invalid());
                }
            }
            _ => {}
        }
        pos = end.checked_add(size % 2).ok_or_else(invalid)?;
    }
    let rate = rate.ok_or_else(invalid)?;
    let data = data.ok_or_else(invalid)?;
    if !(8_000..=96_000).contains(&rate)
        || data.is_empty()
        || data.len() % 2 != 0
        || data.len() / 2 > MAX_TTS_SAMPLES
    {
        return Err(invalid());
    }
    let samples = data
        .chunks_exact(2)
        .map(|v| i16::from_le_bytes([v[0], v[1]]) as f32 / 32768.0)
        .collect();
    Ok((samples, rate))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codec_content_types_are_explicit() {
        assert_eq!(
            InputFormat::from_content_type("audio/mpeg"),
            Some(InputFormat::Mp3)
        );
        assert_eq!(
            InputFormat::from_content_type("audio/ogg"),
            Some(InputFormat::OggOpus)
        );
        assert_eq!(
            InputFormat::from_content_type("audio/opus"),
            Some(InputFormat::OggOpus)
        );
        assert_eq!(
            InputFormat::from_content_type("application/octet-stream"),
            None
        );
    }
    #[test]
    fn resampling_is_bounded_and_deterministic() {
        assert_eq!(
            resample_mono(&[0.0, 1.0], 8_000, 16_000, 4).unwrap(),
            vec![0.0, 0.5, 1.0, 1.0]
        );
        assert!(resample_mono(&[0.0, 1.0], 8_000, 16_000, 3).is_err());
        assert!(resample_mono(&[f32::NAN], 16_000, 16_000, 10).is_err());
    }
    #[test]
    fn downsampling_preserves_speech_band_and_suppresses_aliases() {
        fn tone(frequency: f64) -> Vec<f32> {
            (0..48_000)
                .map(|index| {
                    (2.0 * std::f64::consts::PI * frequency * index as f64 / 48_000.0).sin() as f32
                })
                .collect()
        }
        fn rms(samples: &[f32]) -> f64 {
            (samples
                .iter()
                .map(|value| f64::from(*value).powi(2))
                .sum::<f64>()
                / samples.len() as f64)
                .sqrt()
        }
        let voice = resample_mono(&tone(1_000.0), 48_000, 16_000, 16_000).unwrap();
        let alias = resample_mono(&tone(12_000.0), 48_000, 16_000, 16_000).unwrap();
        assert_eq!(voice.len(), 16_000);
        assert!(rms(&voice[100..]) > 0.65);
        assert!(rms(&alias[100..]) < 0.05);
    }

    #[test]
    fn downsampling_keeps_constant_signal_and_respects_limits() {
        let input = vec![0.25f32; 4_800];
        let reduced = resample_mono(&input, 48_000, 16_000, 1_600).unwrap();
        assert_eq!(reduced.len(), 1_600);
        assert!(reduced.iter().all(|sample| (*sample - 0.25).abs() < 1e-4));
        assert!(resample_mono(&input, 48_000, 16_000, 1_599).is_err());
    }

    #[test]
    fn polyphase_downsampling_matches_direct_tap_calculation() {
        let input = (0..4_410)
            .map(|index| (index as f32 * 0.017).sin() * 0.4)
            .collect::<Vec<_>>();
        let output = resample_mono(&input, 44_100, 16_000, 1_600).unwrap();
        let cutoff = 16_000.0_f64 / 44_100.0_f64;
        let radius = (24.0 / cutoff).ceil() as isize;
        for index in [0usize, 1, 17, 83, 555, 1599] {
            let numerator = index as u64 * 44_100;
            let center = (numerator / 16_000) as isize;
            let fraction = (numerator % 16_000) as f64 / 16_000.0;
            let taps = downsample_taps(cutoff, radius, fraction);
            let (weighted, sum) =
                taps.iter()
                    .enumerate()
                    .fold((0.0, 0.0), |(weighted, sum), (tap, weight)| {
                        let source = center + tap as isize - radius;
                        if (0..input.len() as isize).contains(&source) {
                            (
                                weighted + f64::from(input[source as usize]) * weight,
                                sum + weight,
                            )
                        } else {
                            (weighted, sum)
                        }
                    });
            assert!((output[index] - (weighted / sum) as f32).abs() < 1e-5);
        }
    }

    #[test]
    fn ogg_opus_roundtrip_preserves_duration() {
        let samples: Vec<i16> = (0..16_000)
            .map(|i| ((i as f32 * 0.025).sin() * 9_000.0) as i16)
            .collect();
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(samples.len() as u32 * 2 + 36).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&16_000u32.to_le_bytes());
        wav.extend_from_slice(&32_000u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(samples.len() as u32 * 2).to_le_bytes());
        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        let opus = wav_to_ogg_opus(&wav).unwrap();
        assert!(opus.starts_with(b"OggS"));
        let decoded = decode_for_stt(&opus, InputFormat::OggOpus).unwrap();
        assert_eq!(decoded.len(), 16_000);
        assert!(decoded.iter().any(|v| v.abs() > 0.02));
    }
}
