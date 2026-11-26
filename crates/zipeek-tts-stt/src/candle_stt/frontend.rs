use std::f32::consts::PI;

pub const SAMPLE_RATE: usize = 16_000;
pub const MAX_SECONDS: usize = 30;
pub const FFT_SIZE: usize = 400;
pub const HOP_SIZE: usize = 160;
pub const MEL_BINS: usize = 128;
pub const MAX_FRAMES: usize = 3000;
pub const MIN_SAMPLES: usize = 8_000;
pub const CHUNK_FRAMES: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct MelFeatures {
    pub values: Vec<f32>,
    pub valid_frames: usize,
    pub padded_frames: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct Complex {
    real: f32,
    imag: f32,
}

impl Complex {
    fn multiply(self, other: Self) -> Self {
        Self {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.real * other.imag + self.imag * other.real,
        }
    }
}

fn fft(samples: &[Complex]) -> Vec<Complex> {
    let size = samples.len();
    if size == 1 {
        return samples.to_vec();
    }
    let radix = if size.is_multiple_of(2) {
        2
    } else if size.is_multiple_of(5) {
        5
    } else {
        size
    };
    let inner = size / radix;
    let columns: Vec<Vec<Complex>> = (0..radix)
        .map(|offset| {
            fft(&(0..inner)
                .map(|i| samples[i * radix + offset])
                .collect::<Vec<_>>())
        })
        .collect();
    (0..size)
        .map(|frequency| {
            let mut sum = Complex::default();
            for (column, transformed) in columns.iter().enumerate() {
                let angle = -2.0 * PI * (column * frequency) as f32 / size as f32;
                let twiddle = Complex {
                    real: angle.cos(),
                    imag: angle.sin(),
                };
                let term = transformed[frequency % inner].multiply(twiddle);
                sum.real += term.real;
                sum.imag += term.imag;
            }
            sum
        })
        .collect()
}

fn hertz_to_mel(hertz: f32) -> f32 {
    let linear = hertz / (200.0 / 3.0);
    if hertz < 1000.0 {
        linear
    } else {
        15.0 + (hertz / 1000.0).ln() / (6.4_f32.ln() / 27.0)
    }
}

fn mel_to_hertz(mel: f32) -> f32 {
    if mel < 15.0 {
        mel * (200.0 / 3.0)
    } else {
        1000.0 * ((mel - 15.0) * (6.4_f32.ln() / 27.0)).exp()
    }
}

fn mel_filters() -> Vec<Vec<f32>> {
    let top = hertz_to_mel(8000.0);
    let edges: Vec<f32> = (0..MEL_BINS + 2)
        .map(|index| mel_to_hertz(top * index as f32 / (MEL_BINS + 1) as f32))
        .collect();
    (0..MEL_BINS)
        .map(|band| {
            let low = edges[band];
            let middle = edges[band + 1];
            let high = edges[band + 2];
            let area = 2.0 / (high - low);
            (0..=FFT_SIZE / 2)
                .map(|bin| {
                    let frequency = bin as f32 * SAMPLE_RATE as f32 / FFT_SIZE as f32;
                    let ascending = (frequency - low) / (middle - low);
                    let descending = (high - frequency) / (high - middle);
                    area * ascending.min(descending).max(0.0)
                })
                .collect()
        })
        .collect()
}

fn reflected_sample(samples: &[f32], index: isize) -> f32 {
    if samples.len() == 1 {
        return samples[0];
    }
    let edge = (samples.len() - 1) as isize;
    let period = 2 * edge;
    let position = index.rem_euclid(period);
    let reflected = if position > edge {
        period - position
    } else {
        position
    };
    samples[reflected as usize]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendError {
    Empty,
    TooLong,
    NonFinite,
}

pub fn extract_qwen_features(samples: &[f32]) -> Result<MelFeatures, FrontendError> {
    if samples.is_empty() {
        return Err(FrontendError::Empty);
    }
    if samples.len() > SAMPLE_RATE * MAX_SECONDS {
        return Err(FrontendError::TooLong);
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(FrontendError::NonFinite);
    }
    let mut waveform = samples.to_vec();
    waveform.resize(waveform.len().max(MIN_SAMPLES), 0.0);
    let valid_frames = waveform.len() / HOP_SIZE;
    let padded_frames = valid_frames.div_ceil(CHUNK_FRAMES) * CHUNK_FRAMES;
    let filters = mel_filters();
    let window: Vec<f32> = (0..FFT_SIZE)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / FFT_SIZE as f32).cos())
        .collect();
    let mut values = vec![0.0_f32; MEL_BINS * padded_frames];
    for frame in 0..valid_frames {
        let offset = (frame * HOP_SIZE) as isize - (FFT_SIZE / 2) as isize;
        let input: Vec<Complex> = (0..FFT_SIZE)
            .map(|position| Complex {
                real: reflected_sample(&waveform, offset + position as isize) * window[position],
                imag: 0.0,
            })
            .collect();
        let spectrum = fft(&input);
        let power: Vec<f32> = spectrum[..=FFT_SIZE / 2]
            .iter()
            .map(|item| item.real * item.real + item.imag * item.imag)
            .collect();
        for (band, filter) in filters.iter().enumerate() {
            let energy = filter
                .iter()
                .zip(&power)
                .map(|(weight, value)| weight * value)
                .sum::<f32>();
            values[band * padded_frames + frame] = energy.max(1e-10).log10();
        }
    }
    let peak = (0..MEL_BINS)
        .flat_map(|band| {
            values[band * padded_frames..band * padded_frames + valid_frames]
                .iter()
                .copied()
        })
        .fold(f32::NEG_INFINITY, f32::max);
    for band in 0..MEL_BINS {
        for frame in 0..valid_frames {
            let value = &mut values[band * padded_frames + frame];
            *value = (value.max(peak - 8.0) + 4.0) / 4.0;
        }
    }
    Ok(MelFeatures {
        values,
        valid_frames,
        padded_frames,
    })
}

pub fn log_mel_spectrogram(samples: &[f32]) -> Result<Vec<f32>, FrontendError> {
    extract_qwen_features(samples).map(|features| features.values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_matches_simple_dft() {
        let data: Vec<Complex> = (0..FFT_SIZE)
            .map(|i| Complex {
                real: (i as f32 * 0.13).sin(),
                imag: 0.0,
            })
            .collect();
        let spectrum = fft(&data);
        for bin in [0, 1, 17, 129, 200] {
            let (mut real, mut imag) = (0.0_f32, 0.0_f32);
            for (index, sample) in data.iter().enumerate() {
                let angle = -2.0 * PI * (index * bin) as f32 / FFT_SIZE as f32;
                real += sample.real * angle.cos();
                imag += sample.real * angle.sin();
            }
            assert!((spectrum[bin].real - real).abs() < 0.03);
            assert!((spectrum[bin].imag - imag).abs() < 0.03);
        }
    }

    #[test]
    fn mel_output_is_finite_and_has_qwen3_asr_shape() {
        let data = vec![0.0; 1600];
        let spectrogram = log_mel_spectrogram(&data).unwrap();
        assert_eq!(spectrogram.len(), MEL_BINS * CHUNK_FRAMES);
        assert!(spectrogram.iter().all(|value| value.is_finite()));
        assert!(spectrogram.iter().enumerate().all(
            |(index, value)| if index % CHUNK_FRAMES < 50 {
                (*value + 1.5).abs() < 0.01
            } else {
                *value == 0.0
            }
        ));
    }

    #[test]
    fn qwen3_asr_has_128_mel_bands_and_3000_frame_capacity() {
        assert_eq!(MEL_BINS, 128);
        assert_eq!(MAX_FRAMES, 3000);
        assert_eq!(CHUNK_FRAMES, 100);
        assert_eq!(HOP_SIZE, 160);
        assert_eq!(FFT_SIZE, 400);
    }

    #[test]
    fn short_audio_uses_100_frame_chunk_with_maskable_tail() {
        let features = extract_qwen_features(&vec![0.0; 320]).unwrap();
        assert_eq!(features.valid_frames, 50);
        assert_eq!(features.padded_frames, 100);
        assert_eq!(features.values.len(), 12_800);
        assert!(features.values[50..100].iter().all(|value| *value == 0.0));
    }

    #[test]
    fn longer_audio_pads_to_next_complete_encoder_chunk() {
        let features = extract_qwen_features(&vec![0.0; 16_160]).unwrap();
        assert_eq!(features.valid_frames, 101);
        assert_eq!(features.padded_frames, 200);
        assert_eq!(features.values.len(), 25_600);
        assert_eq!(features.values[101], 0.0);
    }

    #[test]
    fn frontend_rejects_invalid_input() {
        assert_eq!(log_mel_spectrogram(&[]), Err(FrontendError::Empty));
        assert_eq!(
            log_mel_spectrogram(&[f32::NAN]),
            Err(FrontendError::NonFinite)
        );
        assert_eq!(
            log_mel_spectrogram(&vec![0.0; SAMPLE_RATE * MAX_SECONDS + 1]),
            Err(FrontendError::TooLong)
        );
    }

    #[test]
    fn mel_filter_bank_has_nonnegative_coefficients() {
        let bank = mel_filters();
        assert_eq!(bank.len(), MEL_BINS);
        assert!(
            bank.iter()
                .all(|filter| filter.len() == FFT_SIZE / 2 + 1 && filter.iter().any(|v| *v > 0.0))
        );
    }
}
