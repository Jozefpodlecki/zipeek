use std::collections::VecDeque;

pub const SAMPLE_RATE: usize = 16_000;
pub const DEFAULT_MAX_BUFFER_SAMPLES: usize = SAMPLE_RATE * 2;

pub struct PcmChunkBuffer {
    queued: VecDeque<f32>,
    total_received: u64,
    chunk_samples: usize,
    max_buffer_samples: usize,
    finished: bool,
}

impl PcmChunkBuffer {
    pub fn new(chunk_samples: usize, max_buffer_samples: usize) -> Result<Self, String> {
        if chunk_samples == 0
            || chunk_samples > max_buffer_samples
            || max_buffer_samples > SAMPLE_RATE * 30
        {
            return Err("invalid streaming PCM buffer configuration".into());
        }
        Ok(Self {
            queued: VecDeque::new(),
            total_received: 0,
            chunk_samples,
            max_buffer_samples,
            finished: false,
        })
    }

    pub fn push(&mut self, samples: &[f32]) -> Result<(), String> {
        if self.finished {
            return Err("stream is already finalized".into());
        }
        if samples
            .iter()
            .any(|value| !value.is_finite() || value.abs() > 1.0)
        {
            return Err("streaming PCM must contain finite normalized samples".into());
        }
        if self.queued.len().saturating_add(samples.len()) > self.max_buffer_samples {
            return Err("streaming PCM backpressure limit exceeded".into());
        }
        self.total_received = self.total_received.saturating_add(samples.len() as u64);
        self.queued.extend(samples.iter().copied());
        Ok(())
    }

    pub fn take_chunk(&mut self) -> Option<Vec<f32>> {
        if self.queued.len() < self.chunk_samples {
            return None;
        }
        Some(self.queued.drain(..self.chunk_samples).collect())
    }

    pub fn finish(&mut self) -> Vec<f32> {
        self.finished = true;
        self.queued.drain(..).collect()
    }

    pub fn total_received(&self) -> u64 {
        self.total_received
    }

    pub fn pending_samples(&self) -> usize {
        self.queued.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary_packet_sizes_preserve_every_sample_in_order() {
        let mut buffer = PcmChunkBuffer::new(4, 16).unwrap();
        buffer.push(&[0.1, 0.2]).unwrap();
        assert!(buffer.take_chunk().is_none());
        buffer.push(&[0.3, 0.4, 0.5, 0.6, 0.7]).unwrap();
        assert_eq!(buffer.take_chunk().unwrap(), vec![0.1, 0.2, 0.3, 0.4]);
        assert_eq!(buffer.finish(), vec![0.5, 0.6, 0.7]);
        assert_eq!(buffer.total_received(), 7);
        assert!(buffer.push(&[0.2]).is_err());
    }

    #[test]
    fn backpressure_rejects_without_corrupting_buffer() {
        let mut buffer = PcmChunkBuffer::new(4, 5).unwrap();
        buffer.push(&[0.0; 4]).unwrap();
        assert!(buffer.push(&[0.0; 2]).is_err());
        assert_eq!(buffer.pending_samples(), 4);
        assert_eq!(buffer.take_chunk().unwrap(), vec![0.0; 4]);
        assert!(buffer.push(&[f32::NAN]).is_err());
    }
}
