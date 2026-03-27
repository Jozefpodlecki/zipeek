use std::{fs::File, io::BufReader, path::Path};

use cc_cedict_parser_rs::*;
use hashbrown::HashMap;
use xxhash_rust::xxh3::xxh3_64;
use anyhow::{anyhow, Result};

use crate::{models::ChineseLexeme, HashToLexemeMap, InMemoryShardsMap, LexicalVariant, Shard};

pub struct BuildOutput {
    pub shards: InMemoryShardsMap,
    pub hash_to_lexeme: HashToLexemeMap,
}

impl BuildOutput {
    pub fn save(self, base_dir: &Path) -> Result<()> {
        let file_path = HashToLexemeMap::file_path(base_dir);
        let buffer = self.hash_to_lexeme.encode_to_vec()?;
        std::fs::write(file_path, buffer)?;

        for shard in self.shards {
            let file_path = shard.file_path(base_dir);
            let buffer = shard.encode_to_vec()?;
            std::fs::write(file_path, buffer)?;        
        }

        Ok(())
    }
}

pub fn build(reader: LineReader<BufReader<File>>, shards_count: u64) -> Result<BuildOutput> {
    let mut lexeme_id = 0;
    let mut map: HashMap<u64, Shard> = HashMap::new();
    let mut hash_to_lexeme = HashMap::new();

    for line in reader {
        let line = line?;
        let ccdict_entry = RichEntry::new(&line).ok_or_else(|| anyhow!("Could not parse entry"))?;

        let trad_hash = xxh3_64(ccdict_entry.traditional.as_bytes());
        let simp_hash = xxh3_64(ccdict_entry.simplified.as_bytes());

        hash_to_lexeme.insert(trad_hash, lexeme_id);
        hash_to_lexeme.insert(simp_hash, lexeme_id);

        let shard_id = lexeme_id % shards_count;

        match map.entry(shard_id) {
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                let shard = entry.get_mut();

                let variant = LexicalVariant {
                    traditional: ccdict_entry.traditional.into(),
                    pinyin: ccdict_entry.pinyin.into_iter().map(Into::into).collect(),
                    senses: ccdict_entry.senses.into_iter().map(Into::into).collect(),
                    classifiers: ccdict_entry.classifiers.into_iter().map(|c| c.to_owned()).collect(),
                    references: ccdict_entry.references,
                };

                if let Some(lexeme) = shard.get_mut(&lexeme_id) {
                    lexeme.variants.push(variant);
                }
                else {
                    let lexeme = ChineseLexeme {
                        id: lexeme_id,
                        simplified: ccdict_entry.simplified.into(),
                        variants: vec![variant],
                        part_of_speech: vec![],
                        standards: vec![]
                    };

                    shard.insert(lexeme_id, lexeme);
                }
            },
            hashbrown::hash_map::Entry::Vacant(entry) => {
                let shard = entry.insert(Shard::empty(shard_id));

                let lexeme = ChineseLexeme {
                    id: lexeme_id,
                    simplified: ccdict_entry.simplified.into(),
                    variants: vec![
                        LexicalVariant {
                            traditional: ccdict_entry.traditional.into(),
                            pinyin: ccdict_entry.pinyin.into_iter().map(Into::into).collect(),
                            senses: ccdict_entry.senses.into_iter().map(Into::into).collect(),
                            classifiers: ccdict_entry.classifiers.into_iter().map(|c| c.to_owned()).collect(),
                            references: ccdict_entry.references,
                        }
                    ],
                    part_of_speech: vec![],
                    standards: vec![]
                };

                shard.insert(lexeme_id, lexeme);
            },
        }

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