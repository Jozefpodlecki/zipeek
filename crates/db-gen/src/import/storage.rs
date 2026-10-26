use duckdb_neo::{Parameters, connection::Connection};
use zipeek_core::Word;

use super::ImportError;

const INSERT_WORD: &str = "INSERT INTO words (id, simplified, traditional, pinyin, pinyin_numeric, english, part_of_speech, hsk2_level, hsk3_level, frequency) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)";

pub fn insert_words(connection: &Connection, words: &[Word]) -> Result<(), ImportError> {
    connection
        .execute("BEGIN TRANSACTION", Parameters::None)
        .map_err(|source| ImportError::Insert {
            word_id: None,
            source,
        })?;
    let result = insert_batch(connection, words);
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

fn rollback(connection: &Connection, original: ImportError) -> Result<(), ImportError> {
    match connection.execute("ROLLBACK", Parameters::None) {
        Ok(_) => Err(original),
        Err(rollback) => Err(ImportError::Rollback {
            original: Box::new(original),
            rollback,
        }),
    }
}
