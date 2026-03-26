use std::{fs::File, io::BufReader};

use cc_cedict_parser_rs::*;
use hashbrown::HashMap;
use xxhash_rust::xxh3::xxh3_64;
use anyhow::{anyhow, Result};

use crate::{models::ChineseLexeme, HashToLexemeMap, Shard, InMemoryShardsMap};


pub struct BuildOutput {
    pub shards: InMemoryShardsMap,
    pub hash_to_lexeme: HashToLexemeMap,
}

pub fn build(reader: LineReader<BufReader<File>>, shards_count: u64) -> Result<BuildOutput> {
    let mut lexeme_id = 0;
    let mut map: HashMap<u64, Shard> = HashMap::new();
    let mut hash_to_lexeme = HashMap::new();

    for line in reader {
        let line = line?;
        let entry = RichEntry::new(&line).ok_or_else(|| anyhow!("Could not parse entry"))?;

        let trad_hash = xxh3_64(entry.traditional.as_bytes());
        let simp_hash = xxh3_64(entry.simplified.as_bytes());

        hash_to_lexeme.insert(trad_hash, lexeme_id);
        hash_to_lexeme.insert(simp_hash, lexeme_id);

        let shard_id = lexeme_id % shards_count;

        let lexeme = ChineseLexeme {
            id: lexeme_id,
            traditional: entry.traditional.into(),
            simplified: entry.simplified.into(),
            pinyin: entry.pinyin.into_iter().map(Into::into).collect(),
            senses: entry.senses.into_iter().map(|s| s.to_owned()).collect(),
            classifiers: entry.classifiers.into_iter().map(|c| c.to_owned()).collect(),
            references: entry.references,
            part_of_speech: vec![]
        };

        map.entry(shard_id).or_insert_with(|| Shard::empty(shard_id)).insert(lexeme_id, lexeme);

        lexeme_id += 1;
    }

    let shards_map = InMemoryShardsMap {
        shards_count,
        map,
    };

    Ok(BuildOutput {
        shards: shards_map,
        hash_to_lexeme: HashToLexemeMap::new(hash_to_lexeme),
    })
}