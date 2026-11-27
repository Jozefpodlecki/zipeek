use std::{collections::HashMap, fs, path::Path};

pub struct TransducerTokens {
    pieces: Vec<String>,
    blank_id: u32,
}

impl TransducerTokens {
    pub fn from_file(path: &Path, blank_id: u32) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("cannot read streaming token list: {error}"))?;
        Self::parse(&text, blank_id)
    }

    pub fn parse(text: &str, blank_id: u32) -> Result<Self, String> {
        let mut by_id = HashMap::new();
        for (line_no, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let separator = line
                .rfind(char::is_whitespace)
                .ok_or_else(|| format!("missing token id at line {}", line_no + 1))?;
            let (piece, id) = line.split_at(separator);
            let index = id
                .trim()
                .parse::<u32>()
                .map_err(|_| format!("invalid token id at line {}", line_no + 1))?;
            if piece.is_empty() || piece.trim() != piece || index > 1_000_000 {
                return Err(format!("invalid token entry at line {}", line_no + 1));
            }
            if by_id.insert(index, piece.to_owned()).is_some() {
                return Err(format!("duplicate token id at line {}", line_no + 1));
            }
        }
        let size = by_id.len();
        if size == 0 || size > 1_000_000 || blank_id as usize >= size {
            return Err("invalid streaming token vocabulary or blank id".into());
        }
        let pieces = (0..size)
            .map(|index| {
                by_id
                    .remove(&(index as u32))
                    .ok_or_else(|| "sparse token ids".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { pieces, blank_id })
    }

    pub fn len(&self) -> usize {
        self.pieces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub fn piece(&self, index: u32) -> Option<&str> {
        self.pieces.get(index as usize).map(String::as_str)
    }

    pub fn raw_pieces(&self, tokens: &[u32]) -> Result<Vec<&str>, String> {
        tokens
            .iter()
            .filter(|id| **id != self.blank_id)
            .map(|id| {
                self.piece(*id)
                    .ok_or_else(|| "transducer emitted unknown token id".into())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chinese_and_sentencepiece_markers_without_guessing_decoding() {
        let list = TransducerTokens::parse("<blk> 0\n▁ 1\n你 2\n好 3\n", 0).unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list.raw_pieces(&[0, 2, 3]).unwrap(), vec!["你", "好"]);
        assert_eq!(list.raw_pieces(&[1]).unwrap(), vec!["▁"]);
    }

    #[test]
    fn rejects_duplicate_sparse_and_invalid_vocabularies() {
        assert!(TransducerTokens::parse("x 0\ny 0", 0).is_err());
        assert!(TransducerTokens::parse("x 0\ny 2", 0).is_err());
        assert!(TransducerTokens::parse("x 0", 1).is_err());
    }
}
