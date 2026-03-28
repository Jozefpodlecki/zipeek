use std::path::{Path, PathBuf};

use hashbrown::HashMap;
use prost::Message;
use anyhow::Result;
use xxhash_rust::xxh3::xxh3_64;

use crate::storage;

pub struct LexemeHash(u64);

impl LexemeHash {
    pub fn new(lexeme: &str) -> Self {
        Self(xxh3_64(lexeme.as_bytes()))
    }
}

pub struct HashToLexemeMap2 {
    shard_count: u64,
    map: HashMap<u64, u64>
}

pub struct LexemeId {
    id: u64,
    shard_count: u64,
}

impl LexemeId {
    pub fn shard_id(&self) -> u64 {
        self.id % self.shard_count
    }
}

pub struct HashToLexemeMap(pub HashMap<u64, u64>);

impl HashToLexemeMap {
    pub fn new(map: HashMap<u64, u64>) -> Self {
        Self(map)
    }

    pub fn file_path(base_dir: &Path) -> PathBuf {
        base_dir.join("index.pb")
    }

    pub fn get(&self, hash: LexemeHash) -> Option<&u64> {
        self.0.get(&hash.0)
    }

    pub fn decode_from_slice(buffer: &[u8]) -> Result<Self> {
        let proto = storage::HashToLexeme::decode(buffer)?;
        let map: HashMap<u64, u64> = proto.entries.into_iter()
            .map(|e| (e.hash, e.lexeme_id))
            .collect();

        Ok(Self(map))
    }

    pub fn decode_from_file(path: &Path) -> Result<Self> {
        let buffer: Vec<u8> = std::fs::read(path)?;
        let map = Self::decode_from_slice(&buffer)?;
        
        Ok(map)
    }

    pub fn encode_to_vec(self) -> Result<Vec<u8>> {
        let proto: storage::HashToLexeme = self.into();
        let mut buffer = Vec::with_capacity(proto.encoded_len());
        proto.encode(&mut buffer)?;

        Ok(buffer)
    }
}