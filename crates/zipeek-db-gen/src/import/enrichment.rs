use std::collections::BTreeSet;

use duckdb_neo::{Parameters, QueryParameter, connection::Connection};
use sha2::{Digest, Sha256};
use zipeek_core::Word;

use super::storage::{BULK_ROWS, bulk_sql, report_progress};
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
    let mut occurrences: Vec<(i32, i32, String)> = Vec::new();
    let mut candidates: Vec<(i32, String, String)> = Vec::with_capacity(words.len());
    let mut characters = BTreeSet::new();
    let mut fingerprint = Sha256::new();
    for word in words {
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
            characters.insert(character);
            let position = (index + 1) as i32;
            occurrences.push((word_id, position, text));
        }
        let reading = word.pinyin_numeric.as_deref().unwrap_or(&word.pinyin);
        let candidate_key = format!(
            "{}:{}:{}",
            word.simplified.chars().count(),
            word.simplified,
            reading
        );
        candidates.push((word_id, reading.to_owned(), candidate_key));
    }
    let chars: Vec<(String, i32)> = characters
        .iter()
        .map(|character| (character.to_string(), *character as u32 as i32))
        .collect();
    for chunk in chars.chunks(BULK_ROWS) {
        let sql = bulk_sql(
            "INSERT INTO characters (character, unicode_scalar)",
            2,
            chunk.len(),
        );
        let stmt = connection.prepare(&sql, true).map_err(insert_error)?;
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 2);
        for row in chunk {
            params.extend_from_slice(&[&row.0, &row.1]);
        }
        stmt.execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(insert_error)?;
    }
    for (index, chunk) in occurrences.chunks(BULK_ROWS).enumerate() {
        let sql = bulk_sql(
            "INSERT INTO word_characters (word_id, character_position, character)",
            3,
            chunk.len(),
        );
        let stmt = connection.prepare(&sql, true).map_err(insert_error)?;
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 3);
        for row in chunk {
            params.extend_from_slice(&[&row.0, &row.1, &row.2]);
        }
        stmt.execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(insert_error)?;
        report_progress(
            "character occurrences",
            ((index + 1) * BULK_ROWS).min(occurrences.len()),
            occurrences.len(),
        );
    }
    for (index, chunk) in candidates.chunks(BULK_ROWS).enumerate() {
        let sql = bulk_sql(
            "INSERT INTO vocabulary_identities (word_id, reading_key, candidate_key, identity_status)",
            4,
            chunk.len(),
        );
        let status = "unresolved";
        let stmt = connection.prepare(&sql, true).map_err(insert_error)?;
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 4);
        for row in chunk {
            params.extend_from_slice(&[&row.0, &row.1, &row.2, &status]);
        }
        stmt.execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(insert_error)?;
        report_progress(
            "enrichment",
            ((index + 1) * BULK_ROWS).min(candidates.len()),
            candidates.len(),
        );
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
    connection
        .execute(
            "INSERT INTO pinyin_syllable_candidates (pinyin_numeric, attested_character_count) \
         SELECT pinyin_numeric, COUNT(DISTINCT character) \
         FROM character_readings WHERE trim(pinyin_numeric) <> '' GROUP BY pinyin_numeric",
            Parameters::None,
        )
        .map_err(insert_error)?;
    connection.execute(
        "INSERT INTO character_gloss_candidates (character, source_word_id, sense_order, gloss) \
         SELECT w.simplified, w.id, s.sense_order, s.gloss \
         FROM words w JOIN word_senses s ON s.word_id = w.id \
         JOIN characters c ON c.character = w.simplified \
         WHERE length(w.simplified) = 1",
        Parameters::None,
    ).map_err(insert_error)?;
    digest_field(
        &mut fingerprint,
        include_str!("../../data/relationships.csv"),
    );
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
