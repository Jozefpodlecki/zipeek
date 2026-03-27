use std::{collections::BTreeMap, path::Path};

use cc_cedict_parser_rs::*;
use prost::Message;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Debug, Clone)]
pub struct ChineseLexeme {
    pub id: u64,
    pub simplified: Box<str>,
    pub variants: Vec<LexicalVariant>,
    pub part_of_speech: Vec<PartOfSpeech>,
    pub standards: Vec<ReferenceStandard>,
}

#[derive(Debug, Clone)]
pub struct LexicalVariant {
    pub traditional: Box<str>,
    pub pinyin: Vec<Box<str>>,
    pub senses: Vec<ChineseSense>,
    pub classifiers: Vec<OwnedClassifier>,
    pub references: Vec<Reference>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferenceStandard {
    pub name: Box<str>,
    pub kind: Box<str>,
    pub value: Box<str>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChineseSense {
    pub glosses: Vec<Box<str>>,  
    pub tags: Vec<Box<str>>, 
    pub qualifier: Option<Box<str>>,
    pub part_of_speech: Vec<PartOfSpeech>, 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Numeral,
    MeasureWord,
    Conjunction,
    Preposition,
    Particle,
    Interjection,
    Unknown,
}

impl storage::Shard {
    pub fn new(shard_id: u64, entries: BTreeMap<u64, ChineseLexeme>) -> Self {
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
        Ok(storage::Shard::decode(&buf[..])?)
    }
}

