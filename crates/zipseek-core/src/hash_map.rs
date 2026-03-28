use std::path::{Path, PathBuf};

use hashbrown::HashMap;
use prost::Message;
use anyhow::Result;
use xxhash_rust::xxh3::xxh3_64;

use crate::{storage, ProtobufDeserialize};

pub struct LexemeHash(u64);

impl LexemeHash {
    pub fn new(lexeme: &str) -> Self {
        Self(xxh3_64(lexeme.as_bytes()))
    }
}

pub struct LexemeId {
    pub id: u64,
    pub shard_id: u64,
}

pub struct HashToLexemeMap {
    pub(crate) shard_count: u64,
    pub(crate) map: HashMap<u64, u64>
}

impl HashToLexemeMap {
    pub fn new(shard_count: u64) -> Self {
        Self {
            map: Default::default(),
            shard_count
        }
    }

    pub fn file_path(base_dir: &Path) -> PathBuf {
        base_dir.join("index.pb")
    }

    pub fn insert(&mut self, hash: LexemeHash, lexeme_id: u64) {
        self.map.insert(hash.0, lexeme_id);
    }

    pub fn get_shard_id(&self, lexeme_id: u64) -> u64 {
        lexeme_id % self.shard_count
    }

    pub fn get(&self, hash: LexemeHash) -> Option<LexemeId> {
        self.map.get(&hash.0).map(|&id| LexemeId {
            id,
            shard_id: id % self.shard_count
        })
    }

    pub fn decode_from_slice(buffer: &[u8]) -> Result<Self> {
        let proto = storage::HashToLexeme::decode(buffer)?;
        let map: HashMap<u64, u64> = proto.entries.into_iter()
            .map(|e| (e.hash, e.lexeme_id))
            .collect();

        Ok(Self {
            shard_count: proto.shard_count,
            map
        })
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

impl ProtobufDeserialize for HashToLexemeMap {
    fn decode_from_slice(data: &[u8]) -> Result<Self> {
        HashToLexemeMap::decode_from_slice(data)
    }
}