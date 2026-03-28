use std::{fs::File, io::BufReader, path::Path};

use cc_cedict_parser_rs::*;
use hashbrown::HashMap;
use anyhow::{anyhow, Result};
use zipseek_core::*;

pub struct BuildOutput {
    pub search_index: SearchIndex,
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

        let file_path = self.search_index.file_path(base_dir);
        let buffer = self.search_index.encode_to_vec()?;
        std::fs::write(file_path, buffer)?;        

        Ok(())
    }
}

pub fn build(reader: LineReader<BufReader<File>>, shards_count: u64) -> Result<BuildOutput> {
    let mut lexeme_id = 0;
    let mut map: HashMap<u64, Shard> = HashMap::new();
    let mut hash_to_lexeme = HashToLexemeMap::new(shards_count);
    let mut search_index = SearchIndex::new();

    for line in reader {
        let line = line?;
        let ccdict_entry = RichEntry::new(&line).ok_or_else(|| anyhow!("Could not parse entry"))?;

        for sense in &ccdict_entry.senses {
            for gloss in &sense.glosses {
                search_index.insert(lexeme_id, gloss);
            }
        }

        let trad_hash = LexemeHash::new(ccdict_entry.traditional);
        let simp_hash = LexemeHash::new(ccdict_entry.simplified);

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

    search_index.finalize();

    Ok(BuildOutput {
        search_index,
        shards: shards_map,
        hash_to_lexeme,
    })
}