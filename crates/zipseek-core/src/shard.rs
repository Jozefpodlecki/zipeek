use std::{collections::BTreeMap, path::{Path, PathBuf}};

use hashbrown::{hash_map, HashMap};
use prost::Message;
use anyhow::Result;

use crate::{storage, ChineseLexeme, LexemeId, ProtobufDeserialize};

#[derive(Debug, Clone)]
pub struct Shard {
    pub id: u64,
    pub entries: BTreeMap<u64, ChineseLexeme>
}

#[derive(Debug, Clone)]
pub struct OwnedLexemeNeighbors {
    pub prev: Option<ChineseLexeme>,
    pub current: ChineseLexeme,
    pub next: Option<ChineseLexeme>,
}

pub struct LexemeNeighbors<'a> {
    pub prev: Option<&'a ChineseLexeme>,
    pub current: &'a ChineseLexeme,
    pub next: Option<&'a ChineseLexeme>,
}

impl<'a> LexemeNeighbors<'a> {
    pub fn to_owned(self) -> OwnedLexemeNeighbors {
        OwnedLexemeNeighbors {
            prev: self.prev.cloned(),
            current: self.current.clone(),
            next: self.next.cloned(),
        }
    }
}

impl Shard {
    pub fn empty(id: u64) -> Self {
        Self { id, entries: Default::default() }
    }

    pub fn new(id: u64, entries: BTreeMap<u64, ChineseLexeme>) -> Self {
        Self { id, entries }
    }

    pub fn insert(&mut self, lexeme_id: u64, lexeme: ChineseLexeme) {
        self.entries.insert(lexeme_id, lexeme);
    }

    pub fn get(&self, id: &u64) -> Option<&ChineseLexeme> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: &u64) -> Option<&mut ChineseLexeme> {
        self.entries.get_mut(&id)
    }

    pub fn get_with_neighbors<'a>(&'a self, id: LexemeId) -> Option<LexemeNeighbors<'a>> {
        let current = self.entries.get(&id.shard_id)?;

        let prev = self.entries.range(..id.id).next_back();
        let next = self.entries.range((id.id + 1)..).next();

        Some(LexemeNeighbors {
            prev: prev.map(|(_, v)| v),
            current: current,
            next: next.map(|(_, v)| v),
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn file_path(&self, base_dir: &Path) -> PathBuf {
        let file_path = base_dir.join(format!("shard_{}.pb", self.id));
        file_path
    }

    pub fn decode_from_slice<R: AsRef<[u8]>>(bytes: R) -> Result<Self> {
        Ok(storage::Shard::decode(bytes.as_ref())?.into())
    }

    pub fn encode_to_vec(self) -> Result<Vec<u8>> {
        let proto: storage::Shard = self.into();

        Ok(proto.encode_to_vec()?)
    }
}

impl ProtobufDeserialize for Shard {
    fn decode_from_slice(data: &[u8]) -> Result<Self> {
        Shard::decode_from_slice(data)
    }
}

impl IntoIterator for Shard {
    type Item = ChineseLexeme;
    type IntoIter = std::collections::btree_map::IntoValues<u64, ChineseLexeme>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_values()
    }
}

impl<'a> IntoIterator for &'a mut Shard {
    type Item = (&'a u64, &'a mut ChineseLexeme);
    type IntoIter = std::collections::btree_map::IterMut<'a, u64, ChineseLexeme>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter_mut()
    }
}

// for example wasm would implement network shard resolver
pub trait ShardResolver {
    fn get(&self, lexeme_id: &u64) -> Option<Shard>;
}

impl ShardResolver for InMemoryShardsMap {
    fn get(&self, lexeme_id: &u64) -> Option<Shard> {
        let shard_id = lexeme_id % self.shards_count;
        self.map.get(&shard_id).cloned()
    }
}

pub struct InMemoryShardsMap {
    pub shards_count: u64,
    pub map: HashMap<u64, Shard>
}

impl IntoIterator for InMemoryShardsMap {
    type Item = Shard;
    type IntoIter = hash_map::IntoValues<u64, Shard>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_values()
    }
}

impl<'a> IntoIterator for &'a mut InMemoryShardsMap {
    type Item = (&'a u64, &'a mut Shard);
    type IntoIter = hashbrown::hash_map::IterMut<'a, u64, Shard>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

impl InMemoryShardsMap {
    pub fn iter(&self) -> impl Iterator<Item = &Shard> {
        self.map.values()
    }

    pub fn all_lexemes(&self) -> impl Iterator<Item = &ChineseLexeme> {
        self.map.values().flat_map(|shard| shard.entries.values())
    }

    pub fn file_path(&self, base_dir: &Path) -> PathBuf {
        let file_path = base_dir.join("index.pb");
        file_path
    }
}