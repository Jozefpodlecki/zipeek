use encoding_rs::{CoderResult, UTF_8};

pub(super) struct TokenTextDecoder {
    decoder: encoding_rs::Decoder,
}

impl TokenTextDecoder {
    pub(super) fn new() -> Self {
        Self {
            decoder: UTF_8.new_decoder(),
        }
    }

    pub(super) fn decode(&mut self, bytes: &[u8], last: bool) -> String {
        let mut output = String::with_capacity(bytes.len().saturating_mul(4).saturating_add(16));
        let mut remaining = bytes;
        loop {
            let (status, read, _) = self.decoder.decode_to_string(remaining, &mut output, last);
            remaining = &remaining[read..];
            if status == CoderResult::InputEmpty {
                return output;
            }
            output.reserve(remaining.len().saturating_mul(4).saturating_add(16));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_utf8_split_across_tokens() {
        let mut decoder = TokenTextDecoder::new();
        assert_eq!(decoder.decode(&[0xF0, 0x9F], false), "");
        assert_eq!(decoder.decode(&[0x8C, 0xB8], false), "🌸");
        assert_eq!(decoder.decode(&[], true), "");
    }

    #[test]
    fn flushes_incomplete_sequence() {
        let mut decoder = TokenTextDecoder::new();
        assert_eq!(decoder.decode(&[0xE2, 0x82], false), "");
        assert_eq!(decoder.decode(&[], true), "�");
    }

    #[test]
    fn decodes_large_pieces_without_truncation() {
        let mut decoder = TokenTextDecoder::new();
        let input = "é🌸hello".repeat(1024);
        assert_eq!(decoder.decode(input.as_bytes(), false), input);
    }
}
