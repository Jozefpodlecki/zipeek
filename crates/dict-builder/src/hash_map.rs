use std::{collections::BTreeMap, ops::Deref, path::{Path, PathBuf}};

use cc_cedict_parser_rs::*;
use hashbrown::{hash_map, HashMap};
use prost::Message;
use anyhow::Result;
use xxhash_rust::xxh3::xxh3_64;

use crate::dict;

pub struct HashToLexemeMap(pub HashMap<u64, u64>);

impl HashToLexemeMap {
    pub fn new(map: HashMap<u64, u64>) -> Self {
        Self(map)
    }

    pub fn file_path(base_dir: &Path) -> PathBuf {
        base_dir.join("index.pb")
    }

    pub fn get(&self, key: &str) -> Option<&u64> {
        let hash = xxh3_64(key.as_bytes());
        self.0.get(&hash)
    }

    pub fn decode(path: &Path) -> Result<Self> {
        let buf = std::fs::read(path)?;
        let proto = dict::HashToLexeme::decode(&buf[..])?;
        let map: HashMap<u64, u64> = proto.entries.into_iter()
            .map(|e| (e.hash, e.lexeme_id))
            .collect();
        
        Ok(Self(map))
    }

    pub fn encode_to_vec(self) -> Result<Vec<u8>> {
        let proto: dict::HashToLexeme = self.into();
        let mut buffer = Vec::with_capacity(proto.encoded_len());
        proto.encode(&mut buffer)?;

        Ok(buffer)
    }
}