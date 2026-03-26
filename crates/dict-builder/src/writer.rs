use std::{collections::BTreeMap, fs::{create_dir_all, File}, io::{BufWriter, Write}, path::Path};
use anyhow::Result;
use hashbrown::HashMap;
use prost::Message;

use crate::{dict, models::Lexeme};


// pub fn write_shards(base_dir: &Path, shards: HashMap<u64, BTreeMap<u64, Lexeme>>) -> Result<()> {
//     // create_dir_all(base_dir)?;

//     for (shard_id, entries) in shards {
//         let file_path = base_dir.join(format!("shard_{}.pb", shard_id));
//         let mut writer = BufWriter::new(File::create(file_path)?);
//         let shard_proto = dict::Shard::new(shard_id, entries);
//         let buffer = shard_proto.encode_to_vec()?;
//         writer.write_all(&buffer)?;
//     }

//     Ok(())
// }
