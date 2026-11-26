use std::sync::atomic::AtomicBool;

use candle_core::Tensor;

use super::{
    config::TransducerConfig,
    frontend::{FEATURE_BINS, StreamingFbank},
    pcm::PcmChunkBuffer,
    session::{StreamingDecoderSession, TokenPredictor},
};

pub trait StreamingFeatureEncoder {
    fn accept_features(&mut self, frames: &[[f32; FEATURE_BINS]]) -> Result<Option<Tensor>, String>;
}

pub struct StreamingAsrPipeline<E, P> {
    pcm: PcmChunkBuffer,
    features: StreamingFbank,
    encoder: E,
    predictor: P,
    decoder: StreamingDecoderSession,
    finalized: bool,
}

impl<E: StreamingFeatureEncoder, P: TokenPredictor> StreamingAsrPipeline<E, P> {
    pub fn new(
        config: TransducerConfig,
        encoder: E,
        predictor: P,
        pcm_chunk_samples: usize,
        max_queued_samples: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            pcm: PcmChunkBuffer::new(pcm_chunk_samples, max_queued_samples)?,
            features: StreamingFbank::new(),
            encoder,
            predictor,
            decoder: StreamingDecoderSession::new(config)?,
            finalized: false,
        })
    }

    pub fn push_pcm(&mut self, packet: &[f32], cancelled: &AtomicBool) -> Result<&[u32], String> {
        if self.finalized {
            return Err("streaming ASR pipeline already finalized".into());
        }
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("streaming ASR inference cancelled".into());
        }
        self.pcm.push(packet)?;
        while let Some(chunk) = self.pcm.take_chunk() {
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                return Err("streaming ASR inference cancelled".into());
            }
            let frames = self.features.accept(&chunk)?;
            if !frames.is_empty() {
                if let Some(encoded) = self.encoder.accept_features(&frames)? {
                    self.decoder.consume_encoded(&encoded, &mut self.predictor, cancelled)?;
                }
            }
        }
        Ok(self.decoder.tokens())
    }

    pub fn finish(&mut self, cancelled: &AtomicBool) -> Result<&[u32], String> {
        if self.finalized {
            return Err("streaming ASR pipeline already finalized".into());
        }
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("streaming ASR inference cancelled".into());
        }
        let last_samples = self.pcm.finish();
        let last_features = self.features.accept(&last_samples)?;
        if !last_features.is_empty() {
            if let Some(encoded) = self.encoder.accept_features(&last_features)? {
                self.decoder.consume_encoded(&encoded, &mut self.predictor, cancelled)?;
            }
        }
        self.features.finish()?;
        self.finalized = true;
        Ok(self.decoder.finish())
    }

    pub fn queued_samples(&self) -> usize {
        self.pcm.pending_samples()
    }

    pub fn received_samples(&self) -> u64 {
        self.pcm.total_received()
    }

    pub fn encoded_frames(&self) -> u64 {
        self.decoder.processed_frames()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use std::sync::atomic::AtomicBool;

    struct ShapeOnlyEncoder;

    impl StreamingFeatureEncoder for ShapeOnlyEncoder {
        fn accept_features(&mut self, frames: &[[f32; FEATURE_BINS]]) -> Result<Option<Tensor>, String> {
            let data = vec![0f32; frames.len() * 4];
            Tensor::from_vec(data, (frames.len(), 4), &Device::Cpu)
                .map(Some)
                .map_err(|error| error.to_string())
        }
    }

    struct BlankPredictor;

    impl TokenPredictor for BlankPredictor {
        fn predict(&mut self, _: &Tensor, _: &[u32]) -> Result<u32, String> {
            Ok(0)
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
            max_symbols_per_frame: 2,
            max_tokens_per_session: 100,
        }
    }

    #[test]
    fn packet_splitting_does_not_change_encoder_frame_counts() {
        let cancel = AtomicBool::new(false);
        let mut stream = StreamingAsrPipeline::new(config(), ShapeOnlyEncoder, BlankPredictor, 320, 640).unwrap();
        let pcm = vec![0.0; 1600];
        for packet in pcm.chunks(97) {
            assert!(stream.push_pcm(packet, &cancel).unwrap().is_empty());
        }
        assert!(stream.finish(&cancel).unwrap().is_empty());
        assert_eq!(stream.received_samples(), 1600);
        assert_eq!(stream.encoded_frames(), 8);
        assert!(stream.push_pcm(&[0.0], &cancel).is_err());
    }

    #[test]
    fn cancelled_packet_cannot_enter_buffer() {
        let cancelled = AtomicBool::new(true);
        let mut stream = StreamingAsrPipeline::new(config(), ShapeOnlyEncoder, BlankPredictor, 320, 640).unwrap();
        assert!(stream.push_pcm(&[0.0; 320], &cancelled).is_err());
        assert_eq!(stream.received_samples(), 0);
    }
}
