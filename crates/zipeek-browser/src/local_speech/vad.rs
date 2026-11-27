#[derive(Clone, Copy)]
pub(super) struct VadConfig {
    pub(super) silence_ms: f64,
}

pub(super) struct SpeechStopDetector {
    config: VadConfig,
    noise_floor: f32,
    speech_started: bool,
    voice_ms: f64,
    silence_ms: f64,
    last_sample_ms: Option<f64>,
}

impl SpeechStopDetector {
    pub(super) fn new(config: VadConfig) -> Self {
        Self {
            config,
            noise_floor: 0.006,
            speech_started: false,
            voice_ms: 0.0,
            silence_ms: 0.0,
            last_sample_ms: None,
        }
    }

    pub(super) fn heard_speech(&self) -> bool {
        self.speech_started
    }

    pub(super) fn observe(&mut self, samples: &[f32], now_ms: f64) -> bool {
        if samples.is_empty() || !now_ms.is_finite() {
            return false;
        }
        let delta_ms = match self.last_sample_ms {
            Some(previous) if now_ms >= previous => (now_ms - previous).min(160.0),
            _ => 0.0,
        };
        self.last_sample_ms = Some(now_ms);
        let power = samples
            .iter()
            .filter(|value| value.is_finite())
            .map(|sample| sample * sample)
            .sum::<f32>()
            / samples.len() as f32;
        let rms = power.sqrt();
        let start_threshold = (self.noise_floor * 3.0).max(0.012);
        let continue_threshold = (self.noise_floor * 1.7).max(0.007);
        let voiced = rms
            >= if self.speech_started {
                continue_threshold
            } else {
                start_threshold
            };

        if !self.speech_started {
            if voiced {
                self.voice_ms += delta_ms;
                if self.voice_ms >= 200.0 {
                    self.speech_started = true;
                    self.silence_ms = 0.0;
                }
            } else {
                self.voice_ms = 0.0;
                self.noise_floor = (self.noise_floor * 0.97 + rms * 0.03).clamp(0.002, 0.05);
            }
            return false;
        }
        if voiced {
            self.silence_ms = 0.0;
        } else {
            self.silence_ms += delta_ms;
        }
        self.silence_ms >= self.config.silence_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(amplitude: f32) -> Vec<f32> {
        (0..1024)
            .map(|index| {
                if index % 2 == 0 {
                    amplitude
                } else {
                    -amplitude
                }
            })
            .collect()
    }

    #[test]
    fn silence_without_speech_does_not_stop() {
        let mut vad = SpeechStopDetector::new(VadConfig { silence_ms: 900.0 });
        for tick in 0..100 {
            assert!(!vad.observe(&frame(0.001), tick as f64 * 80.0));
        }
    }

    #[test]
    fn stops_after_confirmed_speech_and_continuous_silence() {
        let mut vad = SpeechStopDetector::new(VadConfig { silence_ms: 900.0 });
        for tick in 0..8 {
            assert!(!vad.observe(&frame(0.001), tick as f64 * 80.0));
        }
        for tick in 8..15 {
            assert!(!vad.observe(&frame(0.09), tick as f64 * 80.0));
        }
        for tick in 15..26 {
            assert!(!vad.observe(&frame(0.001), tick as f64 * 80.0));
        }
        assert!(vad.observe(&frame(0.001), 26.0 * 80.0));
    }

    #[test]
    fn brief_noise_and_natural_pause_do_not_stop() {
        let mut vad = SpeechStopDetector::new(VadConfig { silence_ms: 1500.0 });
        assert!(!vad.observe(&frame(0.09), 0.0));
        assert!(!vad.observe(&frame(0.001), 80.0));
        for tick in 2..9 {
            assert!(!vad.observe(&frame(0.09), tick as f64 * 80.0));
        }
        for tick in 9..24 {
            assert!(!vad.observe(&frame(0.001), tick as f64 * 80.0));
        }
        for tick in 24..28 {
            assert!(!vad.observe(&frame(0.09), tick as f64 * 80.0));
        }
        for tick in 28..46 {
            assert!(!vad.observe(&frame(0.001), tick as f64 * 80.0));
        }
        assert!(vad.observe(&frame(0.001), 46.0 * 80.0));
    }

    #[test]
    fn adapts_to_steady_background_noise() {
        let mut vad = SpeechStopDetector::new(VadConfig { silence_ms: 900.0 });
        for tick in 0..40 {
            assert!(!vad.observe(&frame(0.009), tick as f64 * 80.0));
        }
        assert!(!vad.heard_speech());
        for tick in 40..45 {
            assert!(!vad.observe(&frame(0.07), tick as f64 * 80.0));
        }
        assert!(vad.heard_speech());
    }

    #[test]
    fn invalid_and_delayed_samples_do_not_trigger_instant_stop() {
        let mut vad = SpeechStopDetector::new(VadConfig { silence_ms: 900.0 });
        assert!(!vad.observe(&[], 0.0));
        assert!(!vad.observe(&frame(0.1), 0.0));
        assert!(!vad.observe(&frame(0.1), 2000.0));
        assert!(!vad.observe(&frame(0.1), 2080.0));
        assert!(!vad.observe(&frame(0.001), 9000.0));
    }
}
