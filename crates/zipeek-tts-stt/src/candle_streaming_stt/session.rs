use std::sync::atomic::{AtomicBool, Ordering};

use candle_core::Tensor;

use super::{config::TransducerConfig, transducer::CandleTransducerHead};

pub trait TokenPredictor {
    fn predict(&mut self, encoded_frame: &Tensor, context: &[u32]) -> Result<u32, String>;
}

impl TokenPredictor for CandleTransducerHead {
    fn predict(&mut self, encoded_frame: &Tensor, context: &[u32]) -> Result<u32, String> {
        self.next_token(encoded_frame, context)
            .map_err(|error| format!("Candle transducer joiner failed: {error}"))
    }
}

pub struct StreamingDecoderSession {
    config: TransducerConfig,
    tokens: Vec<u32>,
    processed_frames: u64,
    finished: bool,
}

impl StreamingDecoderSession {
    pub fn new(config: TransducerConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            config,
            tokens: Vec::new(),
            processed_frames: 0,
            finished: false,
        })
    }

    pub fn consume_encoded(
        &mut self,
        encoded: &Tensor,
        predictor: &mut impl TokenPredictor,
        cancelled: &AtomicBool,
    ) -> Result<&[u32], String> {
        if self.finished {
            return Err("streaming ASR session already finished".into());
        }
        let (frames, width) = encoded
            .dims2()
            .map_err(|error| format!("streaming encoder output rank: {error}"))?;
        if width != self.config.encoder_dim {
            return Err("streaming encoder width disagrees with transducer configuration".into());
        }
        for frame_index in 0..frames {
            if cancelled.load(Ordering::Relaxed) {
                return Err("streaming ASR inference cancelled".into());
            }
            let frame = encoded
                .get(frame_index)
                .map_err(|error| error.to_string())?;
            for _ in 0..self.config.max_symbols_per_frame {
                if cancelled.load(Ordering::Relaxed) {
                    return Err("streaming ASR inference cancelled".into());
                }
                let token = predictor.predict(&frame, &self.tokens)?;
                if token as usize >= self.config.vocabulary_size {
                    return Err("transducer returned token outside vocabulary".into());
                }
                if token == self.config.blank_id {
                    break;
                }
                if self.tokens.len() >= self.config.max_tokens_per_session {
                    return Err("streaming ASR session token limit reached".into());
                }
                self.tokens.push(token);
            }
            self.processed_frames += 1;
        }
        Ok(&self.tokens)
    }

    pub fn tokens(&self) -> &[u32] {
        &self.tokens
    }

    pub fn processed_frames(&self) -> u64 {
        self.processed_frames
    }

    pub fn finish(&mut self) -> &[u32] {
        self.finished = true;
        &self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Tensor};

    struct PredictSequence {
        sequence: Vec<u32>,
        cursor: usize,
    }

    impl TokenPredictor for PredictSequence {
        fn predict(&mut self, _: &Tensor, _: &[u32]) -> Result<u32, String> {
            let token = self.sequence[self.cursor];
            self.cursor += 1;
            Ok(token)
        }
    }

    fn config() -> TransducerConfig {
        TransducerConfig {
            vocabulary_size: 10,
            blank_id: 0,
            context_size: 2,
            encoder_dim: 4,
            decoder_dim: 4,
            joiner_dim: 4,
            max_symbols_per_frame: 3,
            max_tokens_per_session: 4,
        }
    }

    #[test]
    fn partial_tokens_survive_independent_chunks() {
        let mut session = StreamingDecoderSession::new(config()).unwrap();
        let mut predictor = PredictSequence {
            sequence: vec![1, 2, 0, 3, 0],
            cursor: 0,
        };
        let chunk = Tensor::zeros((1, 4), candle_core::DType::F32, &Device::Cpu).unwrap();
        let cancel = AtomicBool::new(false);
        assert_eq!(session.consume_encoded(&chunk, &mut predictor, &cancel).unwrap(), &[1, 2]);
        assert_eq!(session.consume_encoded(&chunk, &mut predictor, &cancel).unwrap(), &[1, 2, 3]);
        assert_eq!(session.processed_frames(), 2);
        assert_eq!(session.finish(), &[1, 2, 3]);
        assert!(session.consume_encoded(&chunk, &mut predictor, &cancel).is_err());
    }

    #[test]
    fn cancelled_session_does_not_consume_frames() {
        let mut session = StreamingDecoderSession::new(config()).unwrap();
        let mut predictor = PredictSequence { sequence: vec![1], cursor: 0 };
        let chunk = Tensor::zeros((1, 4), candle_core::DType::F32, &Device::Cpu).unwrap();
        assert!(session.consume_encoded(&chunk, &mut predictor, &AtomicBool::new(true)).is_err());
        assert_eq!(session.processed_frames(), 0);
    }
}
