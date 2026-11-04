use duckdb_neo::{Parameters, connection::Connection};
use zipeek_core::Word;

use super::{ImportError, SourceSense, relations::lexical_reference};

const INSERT_WORD: &str = "INSERT INTO words (id, simplified, traditional, pinyin, pinyin_numeric, english, part_of_speech, hsk2_level, hsk3_level, frequency) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)";

pub fn insert_words(connection: &Connection, words: &[Word]) -> Result<(), ImportError> {
    insert_dictionary(connection, words, &[])
}

pub fn insert_dictionary(
    connection: &Connection,
    words: &[Word],
    senses: &[SourceSense],
) -> Result<(), ImportError> {
    connection
        .execute("BEGIN TRANSACTION", Parameters::None)
        .map_err(|source| ImportError::Insert {
            word_id: None,
            source,
        })?;
    let result = insert_batch(connection, words)
        .and_then(|()| populate_memberships(connection))
        .and_then(|()| insert_senses(connection, senses));
    match result {
        Ok(()) => match connection.execute("COMMIT", Parameters::None) {
            Ok(_) => Ok(()),
            Err(source) => rollback(
                connection,
                ImportError::Insert {
                    word_id: None,
                    source,
                },
            ),
        },
        Err(error) => rollback(connection, error),
    }
}

fn insert_batch(connection: &Connection, words: &[Word]) -> Result<(), ImportError> {
    let statement =
        connection
            .prepare(INSERT_WORD, true)
            .map_err(|source| ImportError::Insert {
                word_id: None,
                source,
            })?;
    for word in words {
        let pos = (!word.part_of_speech.is_empty()).then(|| {
            word.part_of_speech
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(",")
        });
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
    const INSERT: &str = "INSERT INTO word_senses (word_id, sense_order, gloss, source, source_ref, source_gloss_order) VALUES ($1,$2,$3,$4,$5,$6)";
    let statement = connection.prepare(INSERT, true).map_err(|source| ImportError::Insert {
        word_id: None,
        source,
    })?;
    let display_statement = connection.prepare(
        "INSERT INTO sense_display (word_id, sense_order, definition, derivation) VALUES ($1,$2,$3,$4)", true,
    ).map_err(|source| ImportError::Insert { word_id: None, source })?;
    let relation_statement = connection.prepare(
        "INSERT INTO sense_relations (word_id, sense_order, relation_order, relation_type, target_simplified, target_traditional, target_pinyin, source_annotation, extraction_method) VALUES ($1,$2,1,$3,$4,$5,$6,$7,'explicit_source_annotation')", true,
    ).map_err(|source| ImportError::Insert { word_id: None, source })?;
    for sense in senses {
        statement.execute(Parameters::Positional(&[
            &sense.word_id.as_i32(),
            &sense.sense_order,
            &sense.gloss,
            &sense.source,
            &sense.source_ref,
            &sense.source_gloss_order,
        ]))
        .and_then(|mut result| result.drain())
        .map_err(|source| ImportError::Insert {
            word_id: Some(sense.word_id.as_i32()),
            source,
        })?;
        let reference = lexical_reference(&sense.gloss);
        let (definition, derivation): (Option<&str>, &str) = match reference.as_ref() {
            Some(value) if value.definition.is_empty() => (None, "relation_only"),
            Some(value) => (Some(value.definition.as_str()), "explicit_annotation_removed"),
            None => (Some(sense.gloss.as_str()), "source_verbatim"),
        };
        display_statement.execute(Parameters::Positional(&[
            &sense.word_id.as_i32(), &sense.sense_order, &definition, &derivation,
        ]))
        .and_then(|mut result| result.drain())
        .map_err(|source| ImportError::Insert {
            word_id: Some(sense.word_id.as_i32()), source,
        })?;
        if let Some(reference) = reference {
            relation_statement.execute(Parameters::Positional(&[
                &sense.word_id.as_i32(), &sense.sense_order, &reference.relation_type,
                &reference.target_simplified, &reference.target_traditional, &reference.target_pinyin,
                &reference.source_annotation,
            ]))
            .and_then(|mut result| result.drain())
            .map_err(|source| ImportError::Insert {
                word_id: Some(sense.word_id.as_i32()), source,
            })?;
        }
    }
    Ok(())
}

fn rollback(connection: &Connection, original: ImportError) -> Result<(), ImportError> {
    match connection.execute("ROLLBACK", Parameters::None) {
        Ok(_) => Err(original),
        Err(rollback) => Err(ImportError::Rollback {
            original: Box::new(original),
            rollback,
        }),
    }
}
