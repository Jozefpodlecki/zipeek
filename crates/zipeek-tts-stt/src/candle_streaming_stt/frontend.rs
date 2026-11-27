use std::f32::consts::PI;

use super::pcm::SAMPLE_RATE;

pub const FEATURE_BINS: usize = 80;
pub const FRAME_SAMPLES: usize = 400;
pub const FRAME_SHIFT: usize = 160;
const FFT_SIZE: usize = 512;
const SPECTRUM_BINS: usize = FFT_SIZE / 2 + 1;
const LOW_FREQUENCY: f32 = 20.0;
const HIGH_FREQUENCY: f32 = 7600.0;

pub struct StreamingFbank {
    pending: Vec<f32>,
    start: usize,
    window: Vec<f32>,
    filters: Vec<Vec<(usize, f32)>>,
    closed: bool,
    produced: u64,
}

impl StreamingFbank {
    pub fn new() -> Self {
        let window = (0..FRAME_SAMPLES)
            .map(|index| ((PI * (index as f32 + 0.5) / FRAME_SAMPLES as f32).sin()).powf(0.85))
            .collect();
        let mel_low = mel(LOW_FREQUENCY);
        let mel_high = mel(HIGH_FREQUENCY);
        let edges = (0..FEATURE_BINS + 2)
            .map(|index| mel_low + (mel_high - mel_low) * index as f32 / (FEATURE_BINS + 1) as f32)
            .collect::<Vec<_>>();
        let filters = (0..FEATURE_BINS)
            .map(|band| {
                (0..SPECTRUM_BINS)
                    .filter_map(|index| {
                        let frequency = index as f32 * SAMPLE_RATE as f32 / FFT_SIZE as f32;
                        let value = mel(frequency);
                        let weight = if value >= edges[band] && value < edges[band + 1] {
                            (value - edges[band]) / (edges[band + 1] - edges[band])
                        } else if value >= edges[band + 1] && value < edges[band + 2] {
                            (edges[band + 2] - value) / (edges[band + 2] - edges[band + 1])
                        } else {
                            0.0
                        };
                        (weight > 0.0).then_some((index, weight))
                    })
                    .collect()
            })
            .collect();
        Self {
            pending: Vec::new(),
            start: 0,
            window,
            filters,
            closed: false,
            produced: 0,
        }
    }

    pub fn accept(&mut self, samples: &[f32]) -> Result<Vec<[f32; FEATURE_BINS]>, String> {
        if self.closed {
            return Err("streaming feature extractor already finalized".into());
        }
        if samples
            .iter()
            .any(|sample| !sample.is_finite() || sample.abs() > 1.0)
        {
            return Err("streaming fbank samples must be finite normalized PCM".into());
        }
        self.pending.extend_from_slice(samples);
        let mut output = Vec::new();
        while self.pending.len().saturating_sub(self.start) >= FRAME_SAMPLES {
            output.push(self.extract(&self.pending[self.start..self.start + FRAME_SAMPLES]));
            self.start += FRAME_SHIFT;
            self.produced += 1;
        }
        if self.start > 0 {
            self.pending.drain(..self.start);
            self.start = 0;
        }
        Ok(output)
    }

    pub fn finish(&mut self) -> Result<Vec<[f32; FEATURE_BINS]>, String> {
        if self.closed {
            return Err("streaming feature extractor already finalized".into());
        }
        self.closed = true;
        self.pending.clear();
        Ok(Vec::new())
    }

    pub fn produced_frames(&self) -> u64 {
        self.produced
    }

    pub fn buffered_samples(&self) -> usize {
        self.pending.len()
    }

    fn extract(&self, input: &[f32]) -> [f32; FEATURE_BINS] {
        let mean = input.iter().sum::<f32>() / FRAME_SAMPLES as f32;
        let mut real = [0f32; FFT_SIZE];
        let mut imaginary = [0f32; FFT_SIZE];
        let mut previous = input[0] - mean;
        for (index, sample) in input.iter().enumerate() {
            let centered = sample - mean;
            let emphasized = if index == 0 {
                centered * 0.03
            } else {
                centered - 0.97 * previous
            };
            real[index] = emphasized * self.window[index];
            previous = centered;
        }
        fft(&mut real, &mut imaginary);
        let mut power = [0f32; SPECTRUM_BINS];
        for index in 0..SPECTRUM_BINS {
            power[index] = real[index] * real[index] + imaginary[index] * imaginary[index];
        }
        std::array::from_fn(|band| {
            let energy = self.filters[band]
                .iter()
                .map(|(index, weight)| power[*index] * weight)
                .sum::<f32>();
            energy.max(1e-10).ln()
        })
    }
}

impl Default for StreamingFbank {
    fn default() -> Self {
        Self::new()
    }
}

fn mel(hz: f32) -> f32 {
    1127.0 * (1.0 + hz / 700.0).ln()
}

fn fft(real: &mut [f32; FFT_SIZE], imaginary: &mut [f32; FFT_SIZE]) {
    let mut j = 0;
    for i in 1..FFT_SIZE {
        let mut bit = FFT_SIZE >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            real.swap(i, j);
            imaginary.swap(i, j);
        }
    }
    let mut width = 2;
    while width <= FFT_SIZE {
        let angle = -2.0 * PI / width as f32;
        let cosine = angle.cos();
        let sine = angle.sin();
        for base in (0..FFT_SIZE).step_by(width) {
            let mut wr = 1.0f32;
            let mut wi = 0.0f32;
            for offset in 0..width / 2 {
                let left = base + offset;
                let right = left + width / 2;
                let tr = wr * real[right] - wi * imaginary[right];
                let ti = wr * imaginary[right] + wi * real[right];
                real[right] = real[left] - tr;
                imaginary[right] = imaginary[left] - ti;
                real[left] += tr;
                imaginary[left] += ti;
                let next_wr = wr * cosine - wi * sine;
                wi = wr * sine + wi * cosine;
                wr = next_wr;
            }
        }
        width *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Vec<f32> {
        (0..SAMPLE_RATE)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / SAMPLE_RATE as f32).sin() * 0.35)
            .collect()
    }

    #[test]
    fn arbitrary_chunks_match_one_shot_features() {
        let samples = example();
        let mut batch = StreamingFbank::new();
        let expected = batch.accept(&samples).unwrap();
        let mut streamed = StreamingFbank::new();
        let mut actual = Vec::new();
        let mut offset = 0;
        for size in [1, 23, 400, 17, 3000, 11, 571, 128, 6667, 10000] {
            if offset >= samples.len() {
                break;
            }
            let end = (offset + size).min(samples.len());
            actual.extend(streamed.accept(&samples[offset..end]).unwrap());
            offset = end;
        }
        actual.extend(streamed.accept(&samples[offset..]).unwrap());
        assert_eq!(actual, expected);
        assert_eq!(streamed.produced_frames() as usize, expected.len());
        assert!(streamed.buffered_samples() < FRAME_SAMPLES);
        assert!(streamed.finish().unwrap().is_empty());
        assert!(streamed.accept(&[0.0]).is_err());
    }

    #[test]
    fn features_are_finite_and_reject_invalid_input() {
        let mut features = StreamingFbank::new();
        assert!(features.accept(&[f32::NAN]).is_err());
        assert!(features.accept(&[1.1]).is_err());
        let frames = features.accept(&example()).unwrap();
        assert!(!frames.is_empty());
        assert!(frames.iter().flatten().all(|value| value.is_finite()));
    }

    #[test]
    fn fragmented_short_audio_never_produces_a_partial_frame() {
        let mut features = StreamingFbank::new();
        assert!(
            features
                .accept(&vec![0.0; FRAME_SAMPLES - 1])
                .unwrap()
                .is_empty()
        );
        assert_eq!(features.accept(&[0.0]).unwrap().len(), 1);
        assert!(features.finish().unwrap().is_empty());
    }
}
