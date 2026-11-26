#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransducerConfig {
    pub vocabulary_size: usize,
    pub blank_id: u32,
    pub context_size: usize,
    pub encoder_dim: usize,
    pub decoder_dim: usize,
    pub joiner_dim: usize,
    pub max_symbols_per_frame: usize,
    pub max_tokens_per_session: usize,
}

impl TransducerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.vocabulary_size < 2 || self.vocabulary_size > u32::MAX as usize {
            return Err("invalid transducer vocabulary size".into());
        }
        if self.blank_id as usize >= self.vocabulary_size {
            return Err("transducer blank id exceeds vocabulary".into());
        }
        if !(1..=2).contains(&self.context_size) {
            return Err("only 1- and 2-token stateless transducer contexts are implemented".into());
        }
        if [self.encoder_dim, self.decoder_dim, self.joiner_dim]
            .iter()
            .any(|dim| *dim == 0 || *dim > 8192)
        {
            return Err("invalid transducer projection dimensions".into());
        }
        if self.max_symbols_per_frame == 0
            || self.max_symbols_per_frame > 64
            || self.max_tokens_per_session == 0
            || self.max_tokens_per_session > 65536
        {
            return Err("invalid transducer decoding budget".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TransducerConfig {
        TransducerConfig {
            vocabulary_size: 128,
            blank_id: 0,
            context_size: 2,
            encoder_dim: 192,
            decoder_dim: 64,
            joiner_dim: 128,
            max_symbols_per_frame: 4,
            max_tokens_per_session: 256,
        }
    }

    #[test]
    fn rejects_invalid_blank_or_unbounded_generation() {
        let mut config = config();
        config.validate().unwrap();
        config.blank_id = 128;
        assert!(config.validate().is_err());
        config.blank_id = 0;
        config.max_symbols_per_frame = 0;
        assert!(config.validate().is_err());
    }
}
