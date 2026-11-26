use std::path::Path;

use duckdb_neo::{Parameters, connection::Connection};

use super::ImportError;

const BUNDLED_RELATIONSHIPS: &str = include_str!("../../data/relationships.csv");

pub fn import_relationship_csv(connection: &Connection) -> Result<(), ImportError> {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/relationships.csv");
    let expected = BUNDLED_RELATIONSHIPS
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .count();
    import_from_path(connection, &file, expected)
}

fn import_from_path(
    connection: &Connection,
    file: &Path,
    expected: usize,
) -> Result<(), ImportError> {
    let path = file
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    let sql = format!(
        r#"
        INSERT INTO word_relations (
            relation_id, source_spelling, relation_type, target_spelling,
            source_word_id, target_word_id, source_resolution, target_resolution
        )
        WITH source_entries AS (
            SELECT trim("word") AS source_spelling, trim("relation") AS relation_type,
                   trim("related_word") AS target_spelling
            FROM read_csv('{path}', header = false, skip = 1, auto_detect = false,
                          delim = ',', quote = '"', escape = '"',
                          columns = {{'word': 'VARCHAR', 'relation': 'VARCHAR', 'related_word': 'VARCHAR'}})
        ), spellings AS (
            SELECT simplified AS spelling, MIN(id) AS unique_id, COUNT(*) AS matches
            FROM words GROUP BY simplified
        ), resolved AS (
            SELECT e.source_spelling, e.relation_type, e.target_spelling,
                   CASE WHEN s.matches = 1 THEN s.unique_id ELSE NULL END AS source_word_id,
                   CASE WHEN t.matches = 1 THEN t.unique_id ELSE NULL END AS target_word_id,
                   CASE WHEN s.matches IS NULL THEN 'missing'
                        WHEN s.matches = 1 THEN 'resolved' ELSE 'ambiguous' END AS source_resolution,
                   CASE WHEN t.matches IS NULL THEN 'missing'
                        WHEN t.matches = 1 THEN 'resolved' ELSE 'ambiguous' END AS target_resolution
            FROM source_entries e
            LEFT JOIN spellings s ON s.spelling = e.source_spelling
            LEFT JOIN spellings t ON t.spelling = e.target_spelling
        )
        SELECT row_number() OVER (ORDER BY source_spelling, relation_type, target_spelling),
               source_spelling, relation_type, target_spelling,
               source_word_id, target_word_id, source_resolution, target_resolution
        FROM resolved
        ORDER BY source_spelling, relation_type, target_spelling
    "#
    );
    connection
        .execute(&sql, Parameters::None)
        .map_err(|source| ImportError::Insert {
            word_id: None,
            source,
        })?;

    let mut result = connection.query(
        "SELECT COUNT(*)::VARCHAR, \
                COUNT(*) FILTER (WHERE source_resolution = 'resolved' AND target_resolution = 'resolved')::VARCHAR, \
                COUNT(*) FILTER (WHERE (source_resolution = 'ambiguous' OR target_resolution = 'ambiguous') AND source_resolution <> 'missing' AND target_resolution <> 'missing')::VARCHAR, \
                COUNT(*) FILTER (WHERE (source_resolution = 'missing' OR target_resolution = 'missing') AND source_resolution <> 'ambiguous' AND target_resolution <> 'ambiguous')::VARCHAR, \
                COUNT(*) FILTER (WHERE (source_resolution = 'missing' AND target_resolution = 'ambiguous') OR (source_resolution = 'ambiguous' AND target_resolution = 'missing'))::VARCHAR \
         FROM word_relations",
        Parameters::None,
    ).map_err(|source| ImportError::Insert { word_id: None, source })?;
    let chunk = result
        .next_chunk()
        .map_err(|source| ImportError::Insert {
            word_id: None,
            source,
        })?
        .ok_or(ImportError::RelationshipVerification)?;
    let mut values = Vec::with_capacity(5);
    for index in 0..5 {
        let vector =
            chunk
                .get_vector_at::<String>(index)
                .map_err(|source| ImportError::Insert {
                    word_id: None,
                    source,
                })?;
        let reader = vector.reader().map_err(|source| ImportError::Insert {
            word_id: None,
            source,
        })?;
        let value = reader
            .iter()
            .next()
            .flatten()
            .ok_or(ImportError::RelationshipVerification)?;
        values.push(
            value
                .parse::<usize>()
                .map_err(|_| ImportError::RelationshipVerification)?,
        );
    }
    if values[0] != expected {
        return Err(ImportError::RelationshipCountMismatch {
            expected,
            actual: values[0],
        });
    }
    if values[1..].iter().sum::<usize>() != values[0] {
        return Err(ImportError::RelationshipVerification);
    }
    tracing::info!(
        total = values[0],
        fully_resolved = values[1],
        ambiguous_only = values[2],
        missing_only = values[3],
        ambiguous_and_missing = values[4],
        "Word relationships imported and mutually exclusive resolution totals verified"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use duckdb_neo::{Parameters, connection::Connection};

    use crate::{database::GeneratorDatabase, schema::create_schema};

    use super::*;

    #[test]
    fn relationship_source_is_bundled_with_generator() {
        assert!(
            BUNDLED_RELATIONSHIPS
                .trim_start_matches('\u{feff}')
                .starts_with("word,relation,related_word")
        );
        assert!(BUNDLED_RELATIONSHIPS.lines().count() > 200_000);
    }

    fn first_string(
        connection: &Connection,
        sql: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = connection.query(sql, Parameters::None)?;
        let chunk = result.next_chunk()?.ok_or("no query result")?;
        let vector = chunk.get_vector_at::<String>(0)?;
        let reader = vector.reader()?;
        Ok(reader
            .iter()
            .next()
            .flatten()
            .ok_or("null result")?
            .to_owned())
    }

    #[test]
    fn relationship_import_preserves_ambiguous_and_missing_endpoints()
    -> Result<(), Box<dyn std::error::Error>> {
        let db = GeneratorDatabase::open()?;
        let connection = db.connection();
        create_schema(connection)?;
        connection.execute(
            "INSERT INTO words (id, simplified, pinyin) VALUES \
             (1, '行', 'xíng'), (2, '行', 'háng'), (3, '银行', 'yínháng'), (4, '猫', 'māo')",
            Parameters::None,
        )?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zipeek-relations-{}-{nonce}.csv",
            std::process::id()
        ));
        fs::write(
            &path,
            "\u{feff}word,relation,related_word\n行,synonym,银行\n银行,synonym,猫\n猫,cross_reference,未收录\n行,antonym,未收录\n",
        )?;
        let imported = import_from_path(connection, &path, 4);
        let _ = fs::remove_file(&path);
        imported?;
        assert_eq!(
            first_string(connection, "SELECT COUNT(*)::VARCHAR FROM word_relations")?,
            "4"
        );
        assert_eq!(
            first_string(
                connection,
                "SELECT source_resolution FROM word_relations WHERE source_spelling = '行'"
            )?,
            "ambiguous"
        );
        assert_eq!(
            first_string(
                connection,
                "SELECT (source_word_id IS NULL)::VARCHAR FROM word_relations WHERE source_spelling = '行'"
            )?,
            "true"
        );
        assert_eq!(
            first_string(
                connection,
                "SELECT (target_word_id = 3)::VARCHAR FROM word_relations WHERE source_spelling = '行'"
            )?,
            "true"
        );
        assert_eq!(
            first_string(
                connection,
                "SELECT target_resolution FROM word_relations WHERE target_spelling = '未收录'"
            )?,
            "missing"
        );
        assert_eq!(
            first_string(
                connection,
                "SELECT COUNT(*)::VARCHAR FROM word_relations WHERE source_resolution = 'resolved' AND target_resolution = 'resolved'"
            )?,
            "1"
        );
        Ok(())
    }

    #[test]
    fn relationship_import_rejects_truncated_inputs() -> Result<(), Box<dyn std::error::Error>> {
        let db = GeneratorDatabase::open()?;
        let connection = db.connection();
        create_schema(connection)?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zipeek-relations-truncated-{}-{nonce}.csv",
            std::process::id()
        ));
        fs::write(&path, "word,relation,related_word\n猫,synonym,狗\n")?;
        let result = import_from_path(connection, &path, 2);
        let _ = fs::remove_file(&path);
        assert!(matches!(
            result,
            Err(ImportError::RelationshipCountMismatch {
                expected: 2,
                actual: 1
            })
        ));
        Ok(())
    }
}
