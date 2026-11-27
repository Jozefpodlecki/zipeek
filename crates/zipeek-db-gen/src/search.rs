use zipeek_db::native::{Parameters, Connection};

use crate::artifact::ArtifactError;

const POPULATE_DOCUMENTS: &str = r#"
INSERT INTO word_search_documents (word_id, english_text)
WITH lexical_definitions AS (
    SELECT senses.word_id,
           string_agg(coalesce(display.definition, senses.gloss), ' ' ORDER BY senses.sense_order) AS definitions
    FROM word_senses AS senses
    LEFT JOIN sense_display AS display
      ON display.word_id = senses.word_id AND display.sense_order = senses.sense_order
    WHERE display.sense_kind IS NULL OR display.sense_kind = 'lexical'
    GROUP BY senses.word_id
)
SELECT words.id,
       trim(concat_ws(' ',
           nullif(trim(coalesce(words.english, '')), ''),
           nullif(trim(coalesce(lexical_definitions.definitions, '')), '')
       ))
FROM words
LEFT JOIN lexical_definitions ON lexical_definitions.word_id = words.id
ORDER BY words.id
"#;

const POPULATE_KEYS: &str = r#"
INSERT INTO word_search_keys (word_id, simplified, traditional, pinyin_plain, pinyin_numeric)
SELECT id,
       simplified,
       coalesce(traditional, ''),
       regexp_replace(
           translate(lower(pinyin), 'āáǎàēéěèīíǐìōóǒòūúǔùǖǘǚǜü', 'aaaaeeeeiiiioooouuuuvvvvv'),
           '[^a-z]', '', 'g'
       ),
       regexp_replace(replace(replace(lower(coalesce(pinyin_numeric, '')), 'u:', 'v'), 'ü', 'v'), '[^a-z0-9]', '', 'g')
FROM words
ORDER BY id
"#;

fn execute(connection: &Connection, statement: &'static str) -> Result<(), ArtifactError> {
    connection.execute(statement, Parameters::None).map_err(|source| ArtifactError::Query {
        query: statement.to_owned(),
        source,
    })?;
    Ok(())
}

pub(crate) fn populate(connection: &Connection) -> Result<(), ArtifactError> {
    execute(connection, POPULATE_DOCUMENTS)?;
    execute(connection, POPULATE_KEYS)?;
    verify(connection)
}

pub(crate) fn count(connection: &Connection, query: &'static str) -> Result<i64, ArtifactError> {
    let failure = |source| ArtifactError::Query { query: query.to_owned(), source };
    let mut result = connection.query(query, Parameters::None).map_err(failure)?;
    let chunk = result.next_chunk().map_err(failure)?
        .ok_or(ArtifactError::LogicalMismatch { stage: "dictionary search count" })?;
    let vector = chunk.get_vector_at::<i64>(0).map_err(failure)?;
    let reader = vector.reader().map_err(failure)?;
    reader.iter().next().flatten().copied()
        .ok_or(ArtifactError::LogicalMismatch { stage: "dictionary search count" })
}

pub(crate) fn verify(connection: &Connection) -> Result<(), ArtifactError> {
    for query in [
        "SELECT (SELECT count(*) FROM words) - (SELECT count(*) FROM word_search_documents)",
        "SELECT (SELECT count(*) FROM words) - (SELECT count(*) FROM word_search_keys)",
        "SELECT count(*) FROM word_search_keys WHERE length(simplified) = 0",
        "SELECT count(*) FROM word_search_documents AS d LEFT JOIN words AS w ON w.id = d.word_id WHERE w.id IS NULL",
        "SELECT count(*) FROM word_search_keys AS k LEFT JOIN words AS w ON w.id = k.word_id WHERE w.id IS NULL",
    ] {
        if count(connection, query)? != 0 {
            return Err(ArtifactError::LogicalMismatch { stage: "dictionary search index coverage" });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{database::GeneratorDatabase, schema::create_schema};

    #[test]
    fn populated_keys_normalize_pinyin_and_keep_original_word_ids() -> Result<(), Box<dyn std::error::Error>> {
        let database = GeneratorDatabase::open()?;
        let connection = database.connection();
        create_schema(connection)?;
        connection.execute(
            "INSERT INTO words (id, simplified, traditional, pinyin, pinyin_numeric, english) VALUES (5, '商店', '商店', 'shāng diàn', 'shang1 dian4', 'shop')",
            Parameters::None,
        )?;
        connection.execute("INSERT INTO words (id, simplified, pinyin, pinyin_numeric) VALUES (6, '女', 'nǚ', 'nü3')", Parameters::None)?;
        populate(connection)?;
        assert_eq!(count(connection, "SELECT count(*) FROM word_search_keys WHERE word_id = 6 AND pinyin_plain = 'nv' AND pinyin_numeric = 'nv3'")?, 1);
        assert_eq!(count(connection, "SELECT count(*) FROM word_search_keys WHERE word_id = 5 AND pinyin_plain = 'shangdian' AND pinyin_numeric = 'shang1dian4'")?, 1);
        assert_eq!(count(connection, "SELECT count(*) FROM word_search_documents WHERE word_id = 5 AND english_text = 'shop'")?, 1);
        Ok(())
    }

    #[test]
    fn search_documents_include_real_senses() -> Result<(), Box<dyn std::error::Error>> {
        let database = GeneratorDatabase::open()?;
        let connection = database.connection();
        create_schema(connection)?;
        connection.execute("INSERT INTO words (id, simplified, pinyin) VALUES (7, '商店', 'shāngdiàn')", Parameters::None)?;
        connection.execute("INSERT INTO word_senses (word_id, sense_order, gloss, source, source_ref, source_gloss_order) VALUES (7, 1, 'shop', 'hsk-primary', 1, 1)", Parameters::None)?;
        connection.execute("INSERT INTO sense_display (word_id, sense_order, definition, derivation, sense_kind) VALUES (7, 1, 'retail store', 'source_verbatim', 'lexical')", Parameters::None)?;
        populate(connection)?;
        assert_eq!(count(connection, "SELECT count(*) FROM word_search_documents WHERE word_id = 7 AND english_text = 'retail store'")?, 1);
        Ok(())
    }
}
