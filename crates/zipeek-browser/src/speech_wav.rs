const TARGET_SAMPLE_RATE: usize = 16_000;
const MAX_DURATION_SECONDS: usize = 30;

pub fn encode_mono_pcm16(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, &'static str> {
    if !(8_000..=192_000).contains(&sample_rate) {
        return Err("Unsupported microphone sample rate");
    }
    if samples.is_empty() || samples.len() > sample_rate as usize * MAX_DURATION_SECONDS {
        return Err("Recording is empty or exceeds thirty seconds");
    }
    let count = samples.len() as u64 * TARGET_SAMPLE_RATE as u64 / sample_rate as u64;
    if count == 0 || count > 4_000_000 {
        return Err("Recording sample count is invalid");
    }
    let count = count as usize;
    let mut bytes = vec![0_u8; 44 + count * 2];
    let size = (bytes.len() - 8) as u32;
    bytes[0..4].copy_from_slice(b"RIFF");
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    bytes[8..16].copy_from_slice(b"WAVEfmt ");
    bytes[16..20].copy_from_slice(&16_u32.to_le_bytes());
    bytes[20..22].copy_from_slice(&1_u16.to_le_bytes());
    bytes[22..24].copy_from_slice(&1_u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&(TARGET_SAMPLE_RATE as u32).to_le_bytes());
    bytes[28..32].copy_from_slice(&(TARGET_SAMPLE_RATE as u32 * 2).to_le_bytes());
    bytes[32..34].copy_from_slice(&2_u16.to_le_bytes());
    bytes[34..36].copy_from_slice(&16_u16.to_le_bytes());
    bytes[36..40].copy_from_slice(b"data");
    bytes[40..44].copy_from_slice(&((count * 2) as u32).to_le_bytes());
    for index in 0..count {
        let position = index as f64 * sample_rate as f64 / TARGET_SAMPLE_RATE as f64;
        let before = (position.floor() as usize).min(samples.len() - 1);
        let after = (before + 1).min(samples.len() - 1);
        let fraction = (position - before as f64) as f32;
        let sample =
            (samples[before] * (1.0 - fraction) + samples[after] * fraction).clamp(-1.0, 1.0);
        let pcm = if sample < 0.0 {
            (sample * 32768.0).round() as i16
        } else {
            (sample * 32767.0).round() as i16
        };
        bytes[44 + index * 2..46 + index * 2].copy_from_slice(&pcm.to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_audio_at_sixteen_kilohertz() {
        let wav = encode_mono_pcm16(&vec![0.5; 48_000], 48_000).unwrap();
        assert_eq!(wav.len(), 32_044);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 32_036);
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 32_000);
        assert_eq!(i16::from_le_bytes(wav[44..46].try_into().unwrap()), 16_384);
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(encode_mono_pcm16(&[], 16_000).is_err());
        assert!(encode_mono_pcm16(&[0.0], 0).is_err());
        assert!(encode_mono_pcm16(&[0.0], 200_000).is_err());
        assert!(encode_mono_pcm16(&vec![0.0; 31 * 8_000], 8_000).is_err());
    }
}
