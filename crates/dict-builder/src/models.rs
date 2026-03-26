use std::{collections::BTreeMap, ops::Deref, path::{Path, PathBuf}};

use cc_cedict_parser_rs::*;
use hashbrown::{hash_map, HashMap};
use prost::Message;
use anyhow::Result;
use xxhash_rust::xxh3::xxh3_64;

use crate::dict;

pub struct Lexeme {
    pub id: u64,
    pub traditional: Box<str>,
    pub simplified: Box<str>,
    pub pinyin: Vec<Box<str>>,
    pub senses: Vec<OwnedSense>,
    pub classifiers: Vec<OwnedClassifier>,
    pub references: Vec<Reference>,
}

impl From<HashToLexemeMap> for dict::HashToLexeme {
    fn from(map: HashToLexemeMap) -> Self {
        let entries = map.0.into_iter()
            .map(|(hash, lex_id)| dict::hash_to_lexeme::Entry {
                hash,
                lexeme_id: lex_id,
            })
            .collect();

        dict::HashToLexeme { entries }
    }
}

impl From<Lexeme> for dict::Lexeme {
    fn from(value: Lexeme) -> Self {
        dict::Lexeme {
            id: value.id,
            traditional: value.traditional.into(),
            simplified: value.simplified.into(),
            pinyin: value.pinyin.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Shard> for dict::Shard {
    fn from(value: Shard) -> Self {
        dict::Shard {
            shard_id: value.id,
            lexemes: value.entries.into_iter().map(|(_, lexeme)| lexeme.into()).collect(),
        }
    }
}

impl dict::Shard {
    pub fn new(shard_id: u64, entries: BTreeMap<u64, Lexeme>) -> Self {
        let lexemes = entries.into_values().map(Into::into).collect();

        Self {
            shard_id,
            lexemes
        }
    }

    pub fn encode_to_vec(self) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(self.encoded_len());
        self.encode(&mut buffer)?;
        Ok(buffer)
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let buf = std::fs::read(path)?;
        Ok(dict::Shard::decode(&buf[..])?)
    }
}

pub struct HashToLexemeMap(HashMap<u64, u64>);

impl Deref for HashToLexemeMap {
    type Target = HashMap<u64, u64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct LexemeId(u64);

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

pub struct Shard {
    id: u64,
    entries: BTreeMap<u64, Lexeme>
}

impl Shard {
    pub fn empty(id: u64) -> Self {
        Self { id, entries: Default::default() }
    }

    pub fn new(id: u64, entries: BTreeMap<u64, Lexeme>) -> Self {
        Self { id, entries }
    }

    pub fn insert(&mut self, lexeme_id: u64, lexeme: Lexeme) {
        self.entries.insert(lexeme_id, lexeme);
    }

    pub fn file_path(&self, base_dir: &Path) -> PathBuf {
        let file_path = base_dir.join(format!("shard_{}.pb", self.id));
        file_path
    }

    pub fn encode_to_vec(self) -> Result<Vec<u8>> {
        let proto: dict::Shard = self.into();

        Ok(proto.encode_to_vec()?)
    }
}

pub trait AShardsMap {
    fn get(&self, lexeme_id: &u64) -> Option<&Shard>;
}

pub struct ShardsMap {
    pub shards_count: u64,
    pub map: HashMap<u64, Shard>
}

impl IntoIterator for ShardsMap {
    type Item = Shard;
    type IntoIter = hash_map::IntoValues<u64, Shard>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_values()
    }
}

impl ShardsMap {    
    pub fn file_path(&self, base_dir: &Path) -> PathBuf {
        let file_path = base_dir.join("index.pb");
        file_path
    }

    pub fn get(&self, lexeme_id: &u64) -> Option<&Shard> {
        self.map.get(lexeme_id)
    }

    // pub fn resolve(&self, lexeme_id: u64, base_dir: &Path) -> Result<Option<dict::Lexeme>> {
    //     let lex_id = match hash_map.refer(hash) {
    //         Some(id) => id,
    //         None => return Ok(None),
    //     };
    //     let shard_id = lex_id % self.shards_count;
    //     let shard = match self.map.get(&shard_id) {
    //         Some(s) => s,
    //         None => {
    //             let shard_path = base_dir.join(format!("shard_{}.pb", shard_id));
    //             Shard::from_path(&shard_path)?
    //         }
    //     };
    //     Ok(shard.entries.get(&lex_id).cloned())
    // }
}