use std::{fs::create_dir_all, path::Path};

use anyhow::Result;
use cc_cedict_parser_rs::LineReader;
use dict_builder::{build, clear_dir, BuildOutput, HashToLexemeMap, Hsk20, Hsk30, Refiner, ShardResolver};

fn main() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut file_path = current_dir.join("crates/dict-builder/cedict_ts.u8");
    
    if !file_path.exists() {
        file_path = current_dir.join("cedict_ts.u8");
    }
    println!("{file_path:?}");
    let reader = LineReader::from_file(&file_path)?;

    let shards_count = 50;
    let mut output = build(reader, shards_count)?;

    // Refiner::generate_blank(shards.all_lexemes())?;
    let hsk20 = Hsk20::new()?;
    let hsk30 = Hsk30::new()?;
    let mut refiner = Refiner::new()?;
    for (_, shard) in &mut output.shards {
        for (_, lexeme) in shard {
            // refiner.refine(lexeme);

            if let Some(level) = hsk20.get_level(&lexeme.simplified) {
                lexeme.standards.push(level.to_standard());
            }

            if let Some(level) = hsk30.get_level(&lexeme.simplified) {
                lexeme.standards.push(level.to_standard());
            }
        }
    }


    let base_dir = Path::new("public/lexicon");
    
    if base_dir.exists() {
        // clear_dir(base_dir)?;
        // output.save(base_dir)?;

        let BuildOutput {
            mut shards,
            hash_to_lexeme,
        } = output;

        let file_path = HashToLexemeMap::file_path(base_dir);
        let hash_to_lexeme = HashToLexemeMap::decode(&file_path)?;

        // let lexeme_id = hash_to_lexeme.get("海外").unwrap();
        
    }
    else {
        create_dir_all(base_dir)?;
        output.save(base_dir);
    }   

    Ok(())
}