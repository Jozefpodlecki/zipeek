use db_gen::{
    database::GeneratorDatabase,
    import::{ImportDiagnosticKind as Kind, ImportError, insert_words, parse_hsk},
    import_primary, prepare,
    schema::create_schema,
};
use duckdb_neo::{Parameters, connection::Connection};
use zipeek_core::{Hsk2Level, Hsk3Level, PartOfSpeech};

const GROUPED: &[u8] = include_bytes!("fixtures/hsk/grouped.json");
const REJECTED: &[u8] = include_bytes!("fixtures/hsk/rejections.json");

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

#[test]
fn first_forms_and_exact_transcriptions_survive_grouping_and_gloss_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    assert_eq!(imported.words.len(), 4);
    let word = imported
        .words
        .iter()
        .find(|word| word.simplified == "学习")
        .unwrap();
    assert_eq!(word.traditional.as_deref(), Some("學習"));
    assert_eq!(word.pinyin, "xué xí");
    assert_eq!(word.pinyin_numeric.as_deref(), Some("xue2 xi2"));
    assert_eq!(word.english.as_deref(), Some("to Study|he/she ｜ learner"));
    assert_eq!(
        word.part_of_speech,
        vec![PartOfSpeech::Noun, PartOfSpeech::Verb]
    );
    assert_eq!(word.hsk2_level, Some(Hsk2Level::One));
    assert_eq!(word.hsk3_level, Some(Hsk3Level::SevenToNine));
    assert_eq!(word.frequency, Some(100));
    for kind in [
        Kind::LevelConflict,
        Kind::UnknownLevel,
        Kind::RepeatedReading,
        Kind::DisplayDrift,
        Kind::GlossPipeReplacement,
        Kind::MissingEnglish,
    ] {
        assert!(
            imported.diagnostics.iter().any(|event| event.kind == kind),
            "missing {kind:?}"
        );
    }
    let replacement = imported
        .diagnostics
        .iter()
        .find(|event| event.kind == Kind::GlossPipeReplacement)
        .unwrap();
    assert!(replacement.detail.contains("he/she | learner"));
    assert!(replacement.detail.contains("he/she ｜ learner"));
    Ok(())
}

#[test]
fn ids_are_assigned_after_rejection_and_exact_reading_key_sorting()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    assert_eq!(imported, parse_hsk(GROUPED)?);
    let keys: Vec<_> = imported
        .words
        .iter()
        .map(|word| (word.simplified.as_str(), word.pinyin_numeric.as_deref()))
        .collect();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    for (index, word) in imported.words.iter().enumerate() {
        assert_eq!(word.id.as_i32() as usize, index + 1);
    }
    let accepted = parse_hsk(REJECTED)?;
    assert_eq!(accepted.source_entries, 6);
    assert_eq!(accepted.rejected_entries, 5);
    assert_eq!(accepted.words.len(), 1);
    assert_eq!(accepted.words[0].simplified, "valid");
    assert_eq!(accepted.words[0].id.as_i32(), 1);
    assert_eq!(accepted.words[0].hsk3_level, Some(Hsk3Level::SevenToNine));
    let unknown = accepted
        .diagnostics
        .iter()
        .find(|event| event.simplified == "unknown")
        .unwrap();
    assert!(unknown.detail.contains("CC"));
    assert!(unknown.detail.contains("QT"));
    assert!(unknown.detail.contains("all 2 forms"));
    Ok(())
}

#[test]
fn display_conflicts_include_both_original_source_rows_verbatim() {
    let bytes = include_bytes!("fixtures/hsk/conflict.json");
    let error = parse_hsk(bytes).unwrap_err();
    let ImportError::DisplayKeyConflict { first, second, .. } = error else {
        panic!("wrong error")
    };
    let source = std::str::from_utf8(bytes).unwrap();
    assert!(source.contains(&first));
    assert!(source.contains(&second));
    assert_ne!(first, second);
    assert!(first.contains("a1"));
    assert!(second.contains("a2"));
}

#[test]
fn malformed_json_and_wrong_field_types_fail_clearly() {
    for bytes in [
        b"{".as_slice(),
        b"null",
        br#"[{"simplified":12}]"#,
        br#"[{"simplified":"x","pos":false}]"#,
    ] {
        assert!(matches!(parse_hsk(bytes), Err(ImportError::Json(_))));
    }
    assert!(parse_hsk(b"[]").unwrap().words.is_empty());
}

#[test]
fn duplicate_entries_merge_memberships_and_keep_first_nonempty_gloss_and_rank()
-> Result<(), Box<dyn std::error::Error>> {
    let mut entries: serde_json::Value = serde_json::from_slice(GROUPED)?;
    let mut first = entries[0].clone();
    first["forms"] = serde_json::json!([first["forms"][1].clone()]);
    first["level"] = serde_json::json!(["old-3"]);
    let mut second = first.clone();
    second["level"] = serde_json::json!(["old-2", "new-1"]);
    second["frequency"] = serde_json::json!(200);
    second["forms"][0]["meanings"] = serde_json::json!(["ignored later gloss"]);
    entries = serde_json::json!([first, second]);
    let imported = parse_hsk(&serde_json::to_vec(&entries)?)?;
    assert_eq!(imported.words.len(), 1);
    let word = &imported.words[0];
    assert_eq!(word.hsk2_level, Some(Hsk2Level::Two));
    assert_eq!(word.hsk3_level, Some(Hsk3Level::One));
    assert_eq!(word.frequency, Some(100));
    assert_eq!(word.english.as_deref(), Some("to Study|he/she ｜ learner"));
    assert!(
        imported
            .diagnostics
            .iter()
            .any(|event| event.kind == Kind::FrequencyConflict)
    );
    Ok(())
}

#[test]
fn exact_numeric_keys_and_source_case_are_not_normalized() -> Result<(), Box<dyn std::error::Error>>
{
    let mut entries: serde_json::Value = serde_json::from_slice(GROUPED)?;
    entries[0]["forms"][1]["transcriptions"]["numeric"] = serde_json::json!("Xue2 xi2");
    let imported = parse_hsk(&serde_json::to_vec(&entries)?)?;
    assert_eq!(
        imported
            .words
            .iter()
            .filter(|word| word.simplified == "学习")
            .count(),
        2
    );
    assert!(
        imported
            .words
            .iter()
            .any(|word| word.pinyin == "DISPLAY DRIFT"
                && word.pinyin_numeric.as_deref() == Some("Xue2 xi2"))
    );
    Ok(())
}

#[test]
fn sql_parameters_preserve_quotes_unicode_nulls_and_distinct_readings()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    insert_words(connection, &imported.words)?;
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM words")?,
        "4"
    );
    assert_eq!(
        scalar(connection, "SELECT simplified FROM words WHERE id = 1")?,
        imported.words[0].simplified
    );
    assert_eq!(
        scalar(connection, "SELECT english FROM words WHERE id = 1")?,
        "a'b"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words WHERE simplified = '行' AND hsk2_level IS NULL AND hsk3_level = 2 AND frequency IS NULL"
        )?,
        "2"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words WHERE simplified = '行' AND pinyin_numeric = 'hang2' AND english IS NULL AND traditional IS NULL"
        )?,
        "1"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words WHERE id = 1 AND part_of_speech IS NULL"
        )?,
        "1"
    );
    Ok(())
}

#[test]
fn failed_batch_rolls_back_earlier_inserts_and_preserves_existing_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    create_schema(database.connection())?;
    insert_words(database.connection(), &imported.words[1..2])?;
    let error = insert_words(database.connection(), &imported.words).unwrap_err();
    assert!(matches!(
        error,
        ImportError::Insert {
            word_id: Some(2),
            ..
        }
    ));
    assert_eq!(
        scalar(
            database.connection(),
            "SELECT string_agg(id::VARCHAR, ',' ORDER BY id) FROM words"
        )?,
        "2"
    );
    insert_words(database.connection(), &imported.words[..1])?;
    assert_eq!(
        scalar(database.connection(), "SELECT count(*)::VARCHAR FROM words")?,
        "2"
    );
    Ok(())
}

#[test]
fn missing_primary_source_is_a_clear_pipeline_error() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/inputs/MANIFEST.toml");
    let prepared = prepare(&manifest)?;
    assert!(matches!(
        import_primary(&prepared),
        Err(db_gen::GeneratorError::Import(ImportError::MissingSource {
            name: "hsk-primary"
        }))
    ));
    Ok(())
}

#[test]
fn preserved_snapshot_reproduces_rows_ids_diagnostics_and_expected_audit_counts()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/MANIFEST.toml");
    let prepared = prepare(&manifest)?;
    let imported = import_primary(&prepared)?;
    let primary = prepared
        .inputs()
        .sources()
        .iter()
        .find(|source| source.name() == "hsk-primary")
        .unwrap();
    assert_eq!(imported, parse_hsk(primary.file().bytes())?);
    assert_eq!(
        (
            imported.source_entries,
            imported.rejected_entries,
            imported.words.len()
        ),
        (11494, 64, 12100)
    );
    assert_eq!(
        imported
            .diagnostics
            .iter()
            .filter(|event| event.kind == Kind::RejectedEntry)
            .count(),
        64
    );
    assert_eq!(
        scalar(
            prepared.database().connection(),
            "SELECT count(*)::VARCHAR FROM words"
        )?,
        "12100"
    );
    assert_eq!(
        scalar(
            prepared.database().connection(),
            "SELECT count(*)::VARCHAR FROM (SELECT id, row_number() OVER (ORDER BY simplified, pinyin_numeric) AS expected FROM words) WHERE id != expected"
        )?,
        "0"
    );
    assert_eq!(
        scalar(
            prepared.database().connection(),
            "SELECT count(*)::VARCHAR FROM words WHERE hsk2_level IS NULL AND hsk3_level IS NULL"
        )?,
        "0"
    );
    Ok(())
}
