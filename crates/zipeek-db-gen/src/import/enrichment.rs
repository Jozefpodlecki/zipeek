use std::collections::BTreeSet;

use duckdb_neo::{Parameters, connection::Connection};
use sha2::{Digest, Sha256};
use zipeek_core::Word;

use super::storage::report_progress;
use super::{CharacterReading, ImportError, SourceSense};

fn insert_error(source: duckdb_neo::Error) -> ImportError {
    ImportError::Insert {
        word_id: None,
        source,
    }
}

fn digest_field(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

pub(super) fn enrich_dictionary(
    connection: &Connection,
    words: &[Word],
    senses: &[SourceSense],
    readings: &[CharacterReading],
) -> Result<(), ImportError> {
    let character_insert = connection
        .prepare(
            "INSERT INTO characters (character, unicode_scalar) VALUES ($1, $2)",
            true,
        )
        .map_err(insert_error)?;
    let occurrence_insert = connection
        .prepare(
            "INSERT INTO word_characters (word_id, character_position, character) VALUES ($1, $2, $3)",
            true,
        )
        .map_err(insert_error)?;
    let candidate_insert = connection
        .prepare(
            "INSERT INTO vocabulary_identities (word_id, reading_key, candidate_key, identity_status) VALUES ($1, $2, $3, 'unresolved')",
            true,
        )
        .map_err(insert_error)?;
    let mut characters = BTreeSet::new();
    let mut fingerprint = Sha256::new();
    for (index, word) in words.iter().enumerate() {
        let word_id = word.id.as_i32();
        digest_field(&mut fingerprint, &word.simplified);
        digest_field(&mut fingerprint, &word.pinyin);
        digest_field(
            &mut fingerprint,
            word.pinyin_numeric.as_deref().unwrap_or(""),
        );
        digest_field(&mut fingerprint, word.traditional.as_deref().unwrap_or(""));
        digest_field(&mut fingerprint, word.english.as_deref().unwrap_or(""));
        digest_field(&mut fingerprint, &format!("{:?}", word.part_of_speech));
        digest_field(&mut fingerprint, &format!("{:?}", word.hsk2_level));
        digest_field(&mut fingerprint, &format!("{:?}", word.hsk3_level));
        digest_field(&mut fingerprint, &format!("{:?}", word.frequency));
        for (index, character) in word.simplified.chars().enumerate() {
            let text = character.to_string();
            let scalar = character as u32 as i32;
            if characters.insert(character) {
                character_insert
                    .execute(Parameters::Positional(&[&text, &scalar]))
                    .and_then(|mut result| result.drain())
                    .map_err(insert_error)?;
            }
            let position = (index + 1) as i32;
            occurrence_insert
                .execute(Parameters::Positional(&[&word_id, &position, &text]))
                .and_then(|mut result| result.drain())
                .map_err(insert_error)?;
        }
        let reading = word.pinyin_numeric.as_deref().unwrap_or(&word.pinyin);
        let candidate_key = format!(
            "{}:{}:{}",
            word.simplified.chars().count(),
            word.simplified,
            reading
        );
        candidate_insert
            .execute(Parameters::Positional(&[
                &word_id,
                &reading,
                &candidate_key,
            ]))
            .and_then(|mut result| result.drain())
            .map_err(insert_error)?;
        report_progress("enrichment", index + 1, words.len());
    }
    for sense in senses {
        digest_field(&mut fingerprint, &sense.word_id.as_i32().to_string());
        digest_field(&mut fingerprint, &sense.sense_order.to_string());
        digest_field(&mut fingerprint, &sense.gloss);
        digest_field(&mut fingerprint, &sense.source);
        digest_field(&mut fingerprint, &sense.source_ref.to_string());
        digest_field(&mut fingerprint, &sense.source_gloss_order.to_string());
    }
    for reading in readings {
        digest_field(&mut fingerprint, &reading.character);
        digest_field(&mut fingerprint, &reading.traditional);
        digest_field(&mut fingerprint, &reading.pinyin_numeric);
        digest_field(&mut fingerprint, &reading.source_line.to_string());
    }
    let snapshot_fingerprint = hex::encode(fingerprint.finalize());
    let word_count = words.len() as i32;
    let sense_count = senses.len() as i32;
    connection
        .execute(
            "INSERT INTO dictionary_versions (snapshot_fingerprint, provenance, word_count, sense_count) VALUES ($1, 'generated-verified-input-manifest', $2, $3)",
            Parameters::Positional(&[&snapshot_fingerprint, &word_count, &sense_count]),
        )
        .map_err(insert_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_prefix_prevents_field_boundary_collisions() {
        let mut first = Sha256::new();
        digest_field(&mut first, "ab");
        digest_field(&mut first, "c");
        let mut second = Sha256::new();
        digest_field(&mut second, "a");
        digest_field(&mut second, "bc");
        assert_ne!(
            hex::encode(first.finalize()),
            hex::encode(second.finalize())
        );
    }
}
