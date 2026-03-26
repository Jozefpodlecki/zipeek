use std::{fs::create_dir_all, path::Path};

use anyhow::Result;
use cc_cedict_parser_rs::LineReader;
use dict_builder::{build, clear_dir, BuildOutput, HashToLexemeMap, ShardResolver};

fn main() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let file_path = current_dir.join("crates/dict-builder/cedict_ts.u8");
    let reader = LineReader::from_file(&file_path)?;

    let shards_count = 50;
    let BuildOutput {
        shards,
        hash_to_lexeme,
    } = build(reader, shards_count)?;

    let base_dir = Path::new("public/lexicon");
    
    if base_dir.exists() {
        // clear_dir(base_dir)?;
        
        let file_path = HashToLexemeMap::file_path(base_dir);
        let hash_to_lexeme = HashToLexemeMap::decode(&file_path)?;

        // let lexeme_id = hash_to_lexeme.get("海外").unwrap();
        let lexeme_id = hash_to_lexeme.get("学习").unwrap();
        // 
        let shard = shards.get(lexeme_id).unwrap();
        println!("{:?} {}", shard.get(lexeme_id), shard.len());
    }
    else {
        create_dir_all(base_dir)?;

        let file_path = HashToLexemeMap::file_path(base_dir);
        let buffer = hash_to_lexeme.encode_to_vec()?;
        std::fs::write(file_path, buffer)?;

        for shard in shards {
            let file_path = shard.file_path(base_dir);
            let buffer = shard.encode_to_vec()?;
            std::fs::write(file_path, buffer)?;        
        }
    }   

    Ok(())
}