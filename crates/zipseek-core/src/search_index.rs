use std::path::{Path, PathBuf};

use hashbrown::HashMap;
use anyhow::Result;
use prost::Message;

use crate::{storage, ProtobufDeserialize};

type LexemeId = u64;

#[derive(Debug)]
pub struct SearchIndex {
    pub inverted: HashMap<Box<str>, Vec<LexemeId>>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            inverted: HashMap::new(),
        }
    }

    pub fn insert(&mut self, lexeme_id: LexemeId, text: &str) {
        let normalized = normalize(text);

        for token in normalized.split_whitespace() {
            let token = token.into();

            self.inverted
                .entry(token)
                .or_default()
                .push(lexeme_id);
        }
    }

    pub fn finalize(&mut self) {

        for ids in self.inverted.values_mut() {
            ids.sort_unstable();
            ids.dedup();
        }
    }

    pub fn file_path(&self, base_dir: &Path) -> PathBuf {
        let file_path = base_dir.join("search_index.pb");
        file_path
    }

    pub fn encode_to_vec(self) -> Result<Vec<u8>> {
        let proto: storage::SearchIndex = self.into();
        let mut buffer = Vec::with_capacity(proto.encoded_len());
        proto.encode(&mut buffer)?;
        Ok(buffer)
    }

     pub fn decode_from_slice(buffer: &[u8]) -> Result<Self> {
        let proto = storage::SearchIndex::decode(buffer)?;
        Ok(Self::from(proto))
    }

    pub fn search(&self, query: &str) -> Vec<LexemeId> {
        let normalized = normalize(query);

        let mut results: Option<Vec<LexemeId>> = None;

        for token in normalized.split_whitespace() {
            let ids = match self.inverted.get(token) {
                Some(v) => v,
                None => return vec![],
            };

            results = Some(match results {
                None => ids.clone(),
                Some(prev) => intersect_two(&prev, ids),
            });
        }

        results.unwrap_or_default()
    }
}

fn normalize(text: &str) -> String {
    text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect()
}

fn intersect_two(a: &[LexemeId], b: &[LexemeId]) -> Vec<LexemeId> {
    let mut i = 0;
    let mut j = 0;
    let mut out = Vec::new();

    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                out.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }

    out
}

impl ProtobufDeserialize for SearchIndex {
    fn decode_from_slice(data: &[u8]) -> Result<Self> {
        SearchIndex::decode_from_slice(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_inverted_index_search() {
        let mut index = SearchIndex::new();

        index.insert(1, "to search");
        index.insert(2, "to ask");
        index.insert(3, "to seek");
        index.insert(4, "to explore");
        index.insert(5, "search and explore");
        
        index.finalize();

        let r1 = index.search("search");
        assert_eq!(r1, vec![1, 5]);

    }
}