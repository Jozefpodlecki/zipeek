use zipeek_db::native::{Parameters, QueryParameter, Connection};
use zipeek_core::{PartOfSpeech, Word};

use crate::database::DatabaseSession;

use super::{
    CharacterReading, ImportError, SourceSense, enrichment::enrich_dictionary,
    qualifiers::extract_qualifiers, relations::lexical_reference, sense_kind::classify,
};

const INSERT_WORD: &str = "INSERT INTO words (id, simplified, traditional, pinyin, pinyin_numeric, english, part_of_speech, hsk2_level, hsk3_level, frequency) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)";

pub fn insert_words(connection: &Connection, words: &[Word]) -> Result<(), ImportError> {
    insert_dictionary(connection, words, &[])
}

pub fn insert_dictionary(
    connection: &Connection,
    words: &[Word],
    senses: &[SourceSense],
) -> Result<(), ImportError> {
    insert_full_dictionary(connection, words, senses, &[])
}

pub fn insert_full_dictionary(
    connection: &Connection,
    words: &[Word],
    senses: &[SourceSense],
    readings: &[CharacterReading],
) -> Result<(), ImportError> {
    let mut transaction = DatabaseSession::new(connection)
        .transaction()
        .map_err(|source| ImportError::Insert {
            word_id: None,
            source,
        })?;
    tracing::info!(
        words = words.len(),
        senses = senses.len(),
        readings = readings.len(),
        "Starting dictionary insertion"
    );
    let result = insert_batch(transaction.connection(), words)
        .and_then(|()| {
            tracing::info!("Populating word memberships");
            populate_memberships(transaction.connection())
        })
        .and_then(|()| {
            tracing::info!("Inserting dictionary senses");
            insert_senses(transaction.connection(), senses)
        })
        .and_then(|()| {
            tracing::info!("Inserting character readings");
            insert_character_readings(transaction.connection(), readings)
        })
        .and_then(|()| {
            tracing::info!("Enriching character and vocabulary relationships");
            enrich_dictionary(transaction.connection(), words, senses, readings)
        });
    match result {
        Ok(()) => match transaction.commit() {
            Ok(()) => Ok(()),
            Err(source) => rollback(
                &mut transaction,
                ImportError::Insert { word_id: None, source },
            ),
        },
        Err(error) => rollback(&mut transaction, error),
    }
}

fn serialize_part_of_speech(values: &[PartOfSpeech]) -> Option<String> {
    (!values.is_empty()).then(|| {
        values
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(",")
    })
}

pub(super) const BULK_ROWS: usize = 256;

pub(super) fn bulk_sql(prefix: &str, columns: usize, rows: usize) -> String {
    let mut sql = String::from(prefix);
    sql.push_str(" VALUES ");
    for row in 0..rows {
        if row != 0 {
            sql.push(',');
        }
        sql.push('(');
        for column in 0..columns {
            if column != 0 {
                sql.push(',');
            }
            sql.push('$');
            sql.push_str(&(row * columns + column + 1).to_string());
        }
        sql.push(')');
    }
    sql
}

fn insert_batch(connection: &Connection, words: &[Word]) -> Result<(), ImportError> {
    if words.len() < BULK_ROWS {
        return insert_batch_single_row(connection, words);
    }
    let prefix = INSERT_WORD.split_once(" VALUES (").unwrap().0;
    for (batch_index, chunk) in words.chunks(BULK_ROWS).enumerate() {
        let ids: Vec<i32> = chunk.iter().map(|word| word.id.as_i32()).collect();
        let pos: Vec<Option<String>> = chunk
            .iter()
            .map(|word| serialize_part_of_speech(&word.part_of_speech))
            .collect();
        let old: Vec<Option<i32>> = chunk
            .iter()
            .map(|word| word.hsk2_level.map(|level| level.as_i32()))
            .collect();
        let new: Vec<Option<i32>> = chunk
            .iter()
            .map(|word| word.hsk3_level.map(|level| level.as_i32()))
            .collect();
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 10);
        for (index, word) in chunk.iter().enumerate() {
            params.extend_from_slice(&[
                &ids[index],
                &word.simplified,
                &word.traditional,
                &word.pinyin,
                &word.pinyin_numeric,
                &word.english,
                &pos[index],
                &old[index],
                &new[index],
                &word.frequency,
            ]);
        }
        let sql = bulk_sql(prefix, 10, chunk.len());
        let statement = connection
            .prepare(&sql, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        statement
            .execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: (chunk.len() == 1).then_some(ids[0]),
                source,
            })?;
        report_progress(
            "words",
            ((batch_index + 1) * BULK_ROWS).min(words.len()),
            words.len(),
        );
    }
    Ok(())
}

fn insert_batch_single_row(connection: &Connection, words: &[Word]) -> Result<(), ImportError> {
    let statement =
        connection
            .prepare(INSERT_WORD, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
    for (index, word) in words.iter().enumerate() {
        let pos = serialize_part_of_speech(&word.part_of_speech);
        let old = word.hsk2_level.map(|level| level.as_i32());
        let new = word.hsk3_level.map(|level| level.as_i32());
        let result = statement.execute(Parameters::Positional(&[
            &word.id.as_i32(),
            &word.simplified,
            &word.traditional,
            &word.pinyin,
            &word.pinyin_numeric,
            &word.english,
            &pos,
            &old,
            &new,
            &word.frequency,
        ]));
        result
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: Some(word.id.as_i32()),
                source,
            })?;
        report_progress("words", index + 1, words.len());
    }
    Ok(())
}

fn populate_memberships(connection: &Connection) -> Result<(), ImportError> {
    for query in [
        "INSERT INTO word_hsk_membership (word_id, system, level) SELECT id, 'hsk2', hsk2_level FROM words WHERE hsk2_level IS NOT NULL AND id NOT IN (SELECT word_id FROM word_hsk_membership WHERE system = 'hsk2')",
        "INSERT INTO word_hsk_membership (word_id, system, level) SELECT id, 'hsk3', hsk3_level FROM words WHERE hsk3_level IS NOT NULL AND id NOT IN (SELECT word_id FROM word_hsk_membership WHERE system = 'hsk3')",
        "INSERT INTO word_pos_membership (word_id, pos) SELECT DISTINCT w.id, trim(p.pos) FROM words AS w CROSS JOIN UNNEST(string_split(w.part_of_speech, ',')) AS p(pos) WHERE w.part_of_speech IS NOT NULL AND trim(p.pos) <> '' AND NOT EXISTS (SELECT 1 FROM word_pos_membership AS existing WHERE existing.word_id = w.id AND existing.pos = trim(p.pos))",
    ] {
        connection
            .execute(query, Parameters::None)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
    }
    Ok(())
}

fn insert_senses(connection: &Connection, senses: &[SourceSense]) -> Result<(), ImportError> {
    let prefix = "INSERT INTO word_senses (word_id, sense_order, gloss, source, source_ref, source_gloss_order)";
    for (batch_index, chunk) in senses.chunks(BULK_ROWS).enumerate() {
        let ids: Vec<i32> = chunk.iter().map(|sense| sense.word_id.as_i32()).collect();
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 6);
        for (index, sense) in chunk.iter().enumerate() {
            params.extend_from_slice(&[
                &ids[index],
                &sense.sense_order,
                &sense.gloss,
                &sense.source,
                &sense.source_ref,
                &sense.source_gloss_order,
            ]);
        }
        let sql = bulk_sql(prefix, 6, chunk.len());
        let statement = connection
            .prepare(&sql, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        statement
            .execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: (chunk.len() == 1).then_some(ids[0]),
                source,
            })?;
        report_progress(
            "raw senses",
            ((batch_index + 1) * BULK_ROWS).min(senses.len()),
            senses.len(),
        );
    }
    let mut display_rows: Vec<(i32, i32, Option<String>, &'static str, &'static str)> =
        Vec::with_capacity(senses.len());
    let mut relation_rows: Vec<(
        i32,
        i32,
        &'static str,
        String,
        Option<String>,
        Option<String>,
        String,
    )> = Vec::new();
    let mut qualifier_rows: Vec<(i32, i32, &'static str, &'static str, String)> = Vec::new();
    for sense in senses {
        let reference = lexical_reference(&sense.gloss);
        let (definition, derivation): (Option<&str>, &'static str) = match reference.as_ref() {
            Some(value) if value.definition.is_empty() => (None, "relation_only"),
            Some(value) => (
                Some(value.definition.as_str()),
                "explicit_annotation_removed",
            ),
            None => (Some(sense.gloss.as_str()), "source_verbatim"),
        };
        let (clean_definition, qualifiers) = definition
            .map(extract_qualifiers)
            .unwrap_or_else(|| (String::new(), Vec::new()));
        let derivation = if !qualifiers.is_empty() && derivation == "source_verbatim" {
            "explicit_annotation_removed"
        } else {
            derivation
        };
        let kind = classify(&sense.gloss, derivation == "relation_only").as_str();
        let word_id = sense.word_id.as_i32();
        display_rows.push((
            word_id,
            sense.sense_order,
            definition.map(|_| clean_definition),
            derivation,
            kind,
        ));
        for qualifier in qualifiers {
            qualifier_rows.push((
                word_id,
                sense.sense_order,
                qualifier.kind,
                qualifier.code,
                qualifier.annotation,
            ));
        }
        if let Some(reference) = reference {
            relation_rows.push((
                word_id,
                sense.sense_order,
                reference.relation_type,
                reference.target_simplified,
                reference.target_traditional,
                reference.target_pinyin,
                reference.source_annotation,
            ));
        }
    }
    for (index, chunk) in display_rows.chunks(BULK_ROWS).enumerate() {
        let sql = bulk_sql(
            "INSERT INTO sense_display (word_id, sense_order, definition, derivation, sense_kind)",
            5,
            chunk.len(),
        );
        let stmt = connection
            .prepare(&sql, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 5);
        for row in chunk {
            params.extend_from_slice(&[&row.0, &row.1, &row.2, &row.3, &row.4]);
        }
        stmt.execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        report_progress(
            "senses",
            ((index + 1) * BULK_ROWS).min(display_rows.len()),
            display_rows.len(),
        );
    }
    for chunk in qualifier_rows.chunks(BULK_ROWS) {
        let sql = bulk_sql(
            "INSERT INTO sense_qualifiers (word_id, sense_order, qualifier_kind, qualifier_code, source_annotation, extraction_method)",
            6,
            chunk.len(),
        );
        let method = "explicit_source_annotation";
        let stmt = connection
            .prepare(&sql, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 6);
        for row in chunk {
            params.extend_from_slice(&[&row.0, &row.1, &row.2, &row.3, &row.4, &method]);
        }
        stmt.execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
    }
    for chunk in relation_rows.chunks(BULK_ROWS) {
        let sql = bulk_sql(
            "INSERT INTO sense_relations (word_id, sense_order, relation_order, relation_type, target_simplified, target_traditional, target_pinyin, source_annotation, extraction_method)",
            9,
            chunk.len(),
        );
        let relation_order = 1_i32;
        let method = "explicit_source_annotation";
        let stmt = connection
            .prepare(&sql, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 9);
        for row in chunk {
            params.extend_from_slice(&[
                &row.0,
                &row.1,
                &relation_order,
                &row.2,
                &row.3,
                &row.4,
                &row.5,
                &row.6,
                &method,
            ]);
        }
        stmt.execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
    }
    Ok(())
}

fn rollback(transaction: &mut crate::database::Transaction<'_>, original: ImportError) -> Result<(), ImportError> {
    match transaction.rollback() {
        Ok(()) => Err(original),
        Err(rollback) => Err(ImportError::Rollback {
            original: Box::new(original),
            rollback,
        }),
    }
}

pub fn insert_character_readings(
    connection: &Connection,
    readings: &[CharacterReading],
) -> Result<(), ImportError> {
    let prefix =
        "INSERT INTO character_readings (character, traditional, pinyin_numeric, source_line)";
    for (batch_index, chunk) in readings.chunks(BULK_ROWS).enumerate() {
        let mut params: Vec<&dyn QueryParameter> = Vec::with_capacity(chunk.len() * 4);
        for reading in chunk {
            params.extend_from_slice(&[
                &reading.character,
                &reading.traditional,
                &reading.pinyin_numeric,
                &reading.source_line,
            ]);
        }
        let sql = bulk_sql(prefix, 4, chunk.len());
        let statement = connection
            .prepare(&sql, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        statement
            .execute(Parameters::Positional(&params))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
        report_progress(
            "readings",
            ((batch_index + 1) * BULK_ROWS).min(readings.len()),
            readings.len(),
        );
    }
    Ok(())
}

const PROGRESS_INTERVAL: usize = 2_000;

pub(super) fn report_progress(stage: &str, completed: usize, total: usize) {
    if completed % PROGRESS_INTERVAL == 0 || completed == total {
        tracing::info!(stage, completed, total, "Dictionary insertion progress");
    }
}

#[cfg(test)]
mod progress_tests {
    use super::*;

    #[test]
    fn part_of_speech_serialization_matches_database_values() {
        assert_eq!(serialize_part_of_speech(&[]), None);
        assert_eq!(
            serialize_part_of_speech(&[PartOfSpeech::Noun]),
            Some("noun".to_owned()),
        );
        assert_eq!(
            serialize_part_of_speech(&[PartOfSpeech::Noun, PartOfSpeech::Verb]),
            Some("noun,verb".to_owned()),
        );
    }

    #[test]
    fn progress_interval_is_nonzero() {
        assert!(PROGRESS_INTERVAL > 0);
    }

    #[test]
    fn bulk_statement_uses_unique_positional_parameters() {
        assert_eq!(
            bulk_sql("INSERT INTO t (a, b)", 2, 2),
            "INSERT INTO t (a, b) VALUES ($1,$2),($3,$4)"
        );
        assert!(BULK_ROWS > 1);
    }
}
