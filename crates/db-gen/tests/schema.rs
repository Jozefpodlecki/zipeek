use db_gen::{database::GeneratorDatabase, schema::create_schema};
use duckdb_neo::{Parameters, connection::Connection};

fn scalar(connection: &Connection, query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = connection.query(query, Parameters::None)?;
    let chunk = result.next_chunk()?.ok_or("missing result chunk")?;
    let reader = chunk.get_vector_at::<String>(0)?.reader()?;
    Ok(reader
        .iter()
        .next()
        .flatten()
        .ok_or("missing value")?
        .to_owned())
}

fn fixture() -> Result<GeneratorDatabase, Box<dyn std::error::Error>> {
    let database = GeneratorDatabase::open()?;
    create_schema(database.connection())?;
    database
        .connection()
        .execute(include_str!("fixtures/words.sql"), Parameters::None)?;
    Ok(database)
}

#[test]
fn schema_has_exact_columns_types_nullability_and_only_two_explicit_indexes()
-> Result<(), Box<dyn std::error::Error>> {
    let database = fixture()?;
    let connection = database.connection();
    assert_eq!(
        scalar(
            connection,
            "SELECT string_agg(column_name || ':' || data_type || ':' || is_nullable, ',' ORDER BY ordinal_position) FROM information_schema.columns WHERE table_name = 'words'"
        )?,
        "id:INTEGER:NO,simplified:VARCHAR:NO,traditional:VARCHAR:YES,pinyin:VARCHAR:NO,pinyin_numeric:VARCHAR:YES,english:VARCHAR:YES,part_of_speech:VARCHAR:YES,hsk2_level:INTEGER:YES,hsk3_level:INTEGER:YES,frequency:INTEGER:YES"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT string_agg(index_name || ':' || expressions, ',' ORDER BY index_name) FROM duckdb_indexes() WHERE table_name = 'words'"
        )?,
        "idx_words_hsk2:[hsk2_level],idx_words_hsk3:[hsk3_level]"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT string_agg(constraint_type, ',' ORDER BY constraint_type) FROM duckdb_constraints() WHERE table_name = 'words'"
        )?,
        "NOT NULL,NOT NULL,NOT NULL,PRIMARY KEY,UNIQUE"
    );
    Ok(())
}

#[test]
fn required_fields_primary_key_and_display_key_are_enforced()
-> Result<(), Box<dyn std::error::Error>> {
    let database = fixture()?;
    for query in [
        "INSERT INTO words (id,simplified,pinyin) VALUES (1,'另','lìng')",
        "INSERT INTO words (id,simplified,pinyin) VALUES (5,'行','háng')",
        "INSERT INTO words (id,simplified,pinyin) VALUES (NULL,'另','lìng')",
        "INSERT INTO words (id,simplified,pinyin) VALUES (5,NULL,'lìng')",
        "INSERT INTO words (id,simplified,pinyin) VALUES (5,'另',NULL)",
    ] {
        assert!(
            database
                .connection()
                .execute(query, Parameters::None)
                .is_err(),
            "accepted: {query}"
        );
    }
    assert_eq!(
        scalar(database.connection(), "SELECT count(*)::VARCHAR FROM words")?,
        "4"
    );
    Ok(())
}

#[test]
fn source_text_independent_memberships_and_missing_metadata_survive_sql_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let database = fixture()?;
    let connection = database.connection();
    assert_eq!(
        scalar(
            connection,
            "SELECT simplified || ':' || traditional || ':' || pinyin || ':' || pinyin_numeric || ':' || english || ':' || part_of_speech FROM words WHERE id = 1"
        )?,
        "学习:學習:xué xí:xue2 xi2:to study|to learn:noun,verb"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words WHERE simplified = '行'"
        )?,
        "2"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words WHERE id = 2 AND traditional IS NULL AND hsk2_level IS NULL AND hsk3_level = 7 AND frequency IS NULL"
        )?,
        "1"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words WHERE id = 3 AND hsk2_level = 3 AND hsk3_level IS NULL AND english IS NULL"
        )?,
        "1"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT pinyin || ':' || pinyin_numeric FROM words WHERE id = 4 AND part_of_speech IS NULL"
        )?,
        "nǚ:nü3"
    );
    Ok(())
}

#[test]
fn preparation_includes_the_empty_words_schema() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/inputs/MANIFEST.toml");
    let prepared = db_gen::prepare(&manifest)?;
    assert_eq!(
        scalar(
            prepared.database().connection(),
            "SELECT count(*)::VARCHAR FROM words"
        )?,
        "0"
    );
    Ok(())
}
