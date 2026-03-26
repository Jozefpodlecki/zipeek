use std::{collections::BTreeMap, path::Path};

use cc_cedict_parser_rs::*;
use prost::Message;
use anyhow::Result;

use crate::{dict, HashToLexemeMap, Shard};

#[derive(Debug, Clone)]
pub struct ChineseLexeme {
    pub id: u64,
    pub traditional: Box<str>,
    pub simplified: Box<str>,
    pub pinyin: Vec<Box<str>>,
    pub senses: Vec<OwnedSense>,
    pub classifiers: Vec<OwnedClassifier>,
    pub references: Vec<Reference>,
    pub part_of_speech: Vec<PartOfSpeech>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl dict::Shard {
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
        Ok(dict::Shard::decode(&buf[..])?)
    }
}

