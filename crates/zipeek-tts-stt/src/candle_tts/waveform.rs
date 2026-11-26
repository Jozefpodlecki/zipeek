#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformError {
    InvalidSampleRate,
    Empty,
    TooLong,
    NonFinite,
}

pub fn encode_mono_pcm16_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, WaveformError> {
    if !(8_000..=96_000).contains(&sample_rate) {
        return Err(WaveformError::InvalidSampleRate);
    }
    if samples.is_empty() {
        return Err(WaveformError::Empty);
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(WaveformError::NonFinite);
    }
    if samples.len() > sample_rate as usize * 30 {
        return Err(WaveformError::TooLong);
    }
    let data_size = samples.len() * 2;
    let mut wav = Vec::with_capacity(44 + data_size);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&((36 + data_size) as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());
    for sample in samples {
        let quantized = (sample.clamp(-1.0, 1.0) * 32768.0).round();
        let pcm = (quantized as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        wav.extend_from_slice(&pcm.to_le_bytes());
    }
    Ok(wav)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waveform_encodes_standard_mono_pcm16_wav() {
        let wav = encode_mono_pcm16_wav(&[-1.0, 0.0, 0.5, 1.0], 24_000).unwrap();
        assert_eq!(wav.len(), 52);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24_000);
        assert_eq!(
            i16::from_le_bytes(wav[44..46].try_into().unwrap()),
            i16::MIN
        );
        assert_eq!(
            i16::from_le_bytes(wav[50..52].try_into().unwrap()),
            i16::MAX
        );
    }

    #[test]
    fn rejects_invalid_neural_waveform() {
        assert_eq!(
            encode_mono_pcm16_wav(&[], 24_000),
            Err(WaveformError::Empty)
        );
        assert_eq!(
            encode_mono_pcm16_wav(&[f32::NAN], 24_000),
            Err(WaveformError::NonFinite)
        );
        assert_eq!(
            encode_mono_pcm16_wav(&[0.0], 0),
            Err(WaveformError::InvalidSampleRate)
        );
        assert_eq!(
            encode_mono_pcm16_wav(&vec![0.0; 24_000 * 30 + 1], 24_000),
            Err(WaveformError::TooLong)
        );
    }
}
