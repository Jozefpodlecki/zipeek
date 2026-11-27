use zipeek_db::native::{Parameters, Connection};
use zipeek_core::{Hsk2Level, Hsk3Level, HskLevel, HskSystem, PartOfSpeech};
use zipeek_db_gen::{
    GeneratorError,
    database::GeneratorDatabase,
    import::{
        ImportDiagnosticKind as Kind, ImportError, insert_dictionary, insert_words, parse_hsk,
    },
    import_primary, prepare,
    schema::create_schema,
};

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
    let senses: Vec<_> = imported
        .senses
        .iter()
        .filter(|sense| sense.word_id == word.id)
        .collect();
    assert_eq!(
        senses
            .iter()
            .map(|sense| sense.gloss.as_str())
            .collect::<Vec<_>>(),
        vec!["to Study", "he/she | learner"]
    );
    assert!(senses.iter().all(|sense| sense.source == "hsk-primary"));
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
fn imported_hsk_memberships_retain_the_correct_system() -> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    for word in imported.words {
        if let Some(level) = word.hsk2_level {
            let selected = HskLevel::Hsk2(level);
            assert_eq!(selected.system(), HskSystem::Hsk2);
            assert!(HskSystem::Hsk2.levels().contains(&selected));
        }
        if let Some(level) = word.hsk3_level {
            let selected = HskLevel::Hsk3(level);
            assert_eq!(selected.system(), HskSystem::Hsk3);
            assert!(HskSystem::Hsk3.levels().contains(&selected));
        }
    }
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
    assert_eq!(accepted.rejected_entries, 4);
    assert_eq!(accepted.words.len(), 3);
    assert_eq!(accepted.words[0].simplified, "unknown");
    assert_eq!(
        accepted.words[0].part_of_speech,
        vec![
            PartOfSpeech::Classifier,
            PartOfSpeech::Conjunction,
            PartOfSpeech::Noun,
        ]
    );
    assert_eq!(accepted.words[1].simplified, "unknown");
    assert_eq!(accepted.words[2].simplified, "valid");
    assert_eq!(accepted.words[2].hsk3_level, Some(Hsk3Level::SevenToNine));
    assert!(
        !accepted.diagnostics.iter().any(|event| {
            event.simplified == "unknown" && event.kind == Kind::UnknownPartOfSpeech
        })
    );
    Ok(())
}

#[test]
fn genuinely_unknown_part_of_speech_codes_still_produce_diagnostics()
-> Result<(), Box<dyn std::error::Error>> {
    let mut entries: serde_json::Value = serde_json::from_slice(REJECTED)?;
    entries[0]["pos"] = serde_json::json!(["n", "CC", "QT", "unmapped-source-tag"]);
    let imported = parse_hsk(&serde_json::to_vec(&entries)?)?;
    assert_eq!(
        imported.words[0].part_of_speech,
        vec![
            PartOfSpeech::Classifier,
            PartOfSpeech::Conjunction,
            PartOfSpeech::Noun,
        ]
    );
    let unknown = imported
        .diagnostics
        .iter()
        .find(|event| event.kind == Kind::UnknownPartOfSpeech)
        .expect("unrecognized source tags must remain diagnosable");
    assert_eq!(unknown.simplified, "unknown");
    assert!(unknown.detail.contains("unmapped-source-tag"));
    assert!(unknown.detail.contains("retaining entry"));
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
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM word_hsk_membership"
        )?,
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM (SELECT id, hsk2_level AS level FROM words WHERE hsk2_level IS NOT NULL UNION ALL SELECT id, hsk3_level AS level FROM words WHERE hsk3_level IS NOT NULL)"
        )?
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM word_pos_membership WHERE pos = ''"
        )?,
        "0"
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
fn source_senses_are_inserted_with_provenance_and_can_be_joined_by_reading()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    insert_dictionary(connection, &imported.words, &imported.senses)?;
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM word_senses")?,
        imported.senses.len().to_string()
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT gloss FROM word_senses WHERE word_id = (SELECT id FROM words WHERE simplified = '学习' AND pinyin = 'xué xí') AND sense_order = 2"
        )?,
        "he/she | learner"
    );
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM categories")?,
        "12"
    );
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM sense_categories")?,
        "0"
    );
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM sense_pos")?,
        "0"
    );
    Ok(())
}

#[test]
fn normalized_abbreviation_preserves_source_and_evidence() -> Result<(), Box<dyn std::error::Error>>
{
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    let mut senses = imported.senses.clone();
    senses[0].gloss = "supermarket (abbr. for 超级市场)".to_owned();
    let source_word_id = senses[0].word_id.as_i32();
    let source_order = senses[0].sense_order;
    insert_dictionary(connection, &imported.words, &senses)?;
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT gloss FROM word_senses WHERE word_id = {source_word_id} AND sense_order = {source_order}"
            )
        )?,
        "supermarket (abbr. for 超级市场)"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT definition FROM sense_display WHERE word_id = {source_word_id} AND sense_order = {source_order}"
            )
        )?,
        "supermarket"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT derivation FROM sense_display WHERE word_id = {source_word_id} AND sense_order = {source_order}"
            )
        )?,
        "explicit_annotation_removed"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT target_simplified FROM sense_relations WHERE word_id = {source_word_id} AND sense_order = {source_order}"
            )
        )?,
        "超级市场"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT extraction_method FROM sense_relations WHERE word_id = {source_word_id} AND sense_order = {source_order}"
            )
        )?,
        "explicit_source_annotation"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT source_annotation FROM sense_relations WHERE word_id = {source_word_id} AND sense_order = {source_order}"
            )
        )?,
        "(abbr. for 超级市场)"
    );
    Ok(())
}

#[test]
fn explicit_lexical_relations_are_typed_and_relation_only_senses_have_no_definition()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    let mut senses = imported.senses.clone();
    assert!(senses.len() >= 4);
    for (sense, gloss) in senses.iter_mut().take(4).zip([
        "related concept (see also 學習|学习)",
        "variant of 語言|语言[yu3 yan2]",
        "short for 中华人民共和国",
        "another spelling (also written 臺灣|台湾)",
    ]) {
        sense.gloss = gloss.to_owned();
    }
    let keys: Vec<_> = senses
        .iter()
        .take(4)
        .map(|sense| (sense.word_id.as_i32(), sense.sense_order))
        .collect();
    insert_dictionary(connection, &imported.words, &senses)?;
    for ((word_id, sense_order), relation_type) in
        keys.iter()
            .zip(["see_also", "variant_of", "short_for", "also_written_as"])
    {
        assert_eq!(
            scalar(
                connection,
                &format!(
                    "SELECT relation_type FROM sense_relations WHERE word_id = {word_id} AND sense_order = {sense_order}"
                )
            )?,
            relation_type
        );
    }
    let (word_id, sense_order) = keys[1];
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT derivation FROM sense_display WHERE word_id = {word_id} AND sense_order = {sense_order}"
            )
        )?,
        "relation_only"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT count(*)::VARCHAR FROM sense_display WHERE word_id = {word_id} AND sense_order = {sense_order} AND definition IS NULL"
            )
        )?,
        "1"
    );
    assert_eq!(
        scalar(
            connection,
            &format!(
                "SELECT target_simplified FROM sense_relations WHERE word_id = {word_id} AND sense_order = {sense_order}"
            )
        )?,
        "语言"
    );
    Ok(())
}

#[test]
fn malformed_sense_reference_rolls_back_complete_dictionary_transaction()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    let mut invalid = imported.senses.clone();
    invalid[0].sense_order = 0;
    assert!(insert_dictionary(connection, &imported.words, &invalid).is_err());
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM words")?,
        "0"
    );
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM word_senses")?,
        "0"
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
        Err(GeneratorError::Import(ImportError::MissingSource {
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
    assert_eq!(imported.source_entries, 11494);
    assert!(imported.rejected_entries < 64);
    assert!(imported.words.len() > 12100);
    assert!(!imported.diagnostics.is_empty());
    assert!(
        imported
            .diagnostics
            .iter()
            .all(|event| (1..=imported.source_entries).contains(&event.entry))
    );
    assert!(
        !imported
            .diagnostics
            .iter()
            .any(|event| event.kind == Kind::UnknownPartOfSpeech)
    );
    assert!(imported.words.iter().any(|word| word.simplified == "和"));
    assert!(imported.words.iter().any(|word| word.simplified == "分钟"));
    assert!(imported.words.iter().all(|word| word.english.is_some()));
    assert_eq!(
        imported
            .diagnostics
            .iter()
            .filter(|event| event.kind == Kind::RejectedEntry)
            .count(),
        imported.rejected_entries
    );
    assert_eq!(
        scalar(
            prepared.database().connection(),
            "SELECT count(*)::VARCHAR FROM words"
        )?,
        imported.words.len().to_string()
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

#[test]
fn generated_dictionary_has_snapshot_characters_and_unresolved_identity_candidates()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let database = GeneratorDatabase::open()?;
    create_schema(database.connection())?;
    insert_dictionary(database.connection(), &imported.words, &imported.senses)?;
    let connection = database.connection();
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM dictionary_versions"
        )?,
        "1"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM vocabulary_identities WHERE identity_status = 'unresolved'"
        )?,
        imported.words.len().to_string()
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM word_characters WHERE word_id = (SELECT id FROM words WHERE simplified = '学习' LIMIT 1)"
        )?,
        "2"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT character FROM word_characters WHERE word_id = (SELECT id FROM words WHERE simplified = '学习' LIMIT 1) AND character_position = 2"
        )?,
        "习"
    );
    Ok(())
}

#[test]
fn explicitly_sourced_character_readings_are_inserted_independently_of_words()
-> Result<(), Box<dyn std::error::Error>> {
    use zipeek_db_gen::import::{CharacterReading, insert_character_readings};

    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    let rows = [
        CharacterReading {
            character: "行".into(),
            traditional: "行".into(),
            pinyin_numeric: "hang2".into(),
            source_line: 1,
        },
        CharacterReading {
            character: "行".into(),
            traditional: "行".into(),
            pinyin_numeric: "xing2".into(),
            source_line: 2,
        },
    ];
    insert_character_readings(connection, &rows)?;
    assert_eq!(
        scalar(
            connection,
            "SELECT pinyin_numeric FROM character_readings WHERE character = '行' ORDER BY pinyin_numeric LIMIT 1"
        )?,
        "hang2"
    );
    assert!(
        insert_character_readings(connection, &rows).is_err(),
        "duplicate readings must be rejected"
    );
    Ok(())
}

#[test]
fn character_readings_bulk_insert_preserves_all_rows_across_chunks()
-> Result<(), Box<dyn std::error::Error>> {
    use zipeek_db_gen::import::{CharacterReading, insert_character_readings};

    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    let rows: Vec<CharacterReading> = (0..513)
        .map(|index| CharacterReading {
            character: char::from_u32(0x4E00 + index).unwrap().to_string(),
            traditional: char::from_u32(0x4E00 + index).unwrap().to_string(),
            pinyin_numeric: "yi1".into(),
            source_line: index as i32 + 1,
        })
        .collect();
    insert_character_readings(connection, &rows)?;
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM character_readings"
        )?,
        "513"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT max(source_line)::VARCHAR FROM character_readings"
        )?,
        "513"
    );
    Ok(())
}

#[test]
fn full_dictionary_rolls_back_words_if_character_reading_insertion_fails()
-> Result<(), Box<dyn std::error::Error>> {
    use zipeek_db_gen::import::{CharacterReading, insert_full_dictionary};

    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    let imported = parse_hsk(GROUPED)?;
    let duplicate = CharacterReading {
        character: "行".into(),
        traditional: "行".into(),
        pinyin_numeric: "xing2".into(),
        source_line: 7,
    };
    assert!(
        insert_full_dictionary(
            connection,
            &imported.words,
            &imported.senses,
            &[duplicate.clone(), duplicate],
        )
        .is_err()
    );
    assert_eq!(
        scalar(connection, "SELECT count(*)::VARCHAR FROM words")?,
        "0"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM character_readings"
        )?,
        "0"
    );
    Ok(())
}

#[test]
fn generated_sense_kinds_preserve_usage_references_but_exclude_them_from_practice()
-> Result<(), Box<dyn std::error::Error>> {
    let imported = parse_hsk(GROUPED)?;
    let mut senses = imported.senses.clone();
    assert!(senses.len() >= 3);
    senses[0].gloss = "used in 上聲|上声[shang3 sheng1]".to_owned();
    senses[1].gloss = "used in 丁丁[zheng1 zheng1]".to_owned();
    senses[2].gloss = "(bound form) up; upper; above; previous".to_owned();
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    insert_dictionary(connection, &imported.words, &senses)?;
    for (sense, kind) in
        senses
            .iter()
            .take(3)
            .zip(["usage_reference", "usage_reference", "lexical"])
    {
        assert_eq!(
            scalar(
                connection,
                &format!(
                    "SELECT sense_kind FROM sense_display WHERE word_id = {} AND sense_order = {}",
                    sense.word_id.as_i32(),
                    sense.sense_order
                )
            )?,
            kind
        );
        assert_eq!(
            scalar(
                connection,
                &format!(
                    "SELECT gloss FROM word_senses WHERE word_id = {} AND sense_order = {}",
                    sense.word_id.as_i32(),
                    sense.sense_order
                )
            )?,
            sense.gloss
        );
    }
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM sense_display WHERE sense_kind = 'usage_reference'"
        )?,
        "2"
    );
    Ok(())
}

#[test]
fn two_readings_of_shang_keep_reference_data_but_only_lexical_reading_is_playable()
-> Result<(), Box<dyn std::error::Error>> {
    let input = r#"[{"simplified":"上","radical":"一","level":["new-1","old-1"],"frequency":29,"pos":["f","v","g"],"forms":[{"traditional":"上","transcriptions":{"pinyin":"shǎng","numeric":"shang3"},"meanings":["used in 上声"]},{"traditional":"上","transcriptions":{"pinyin":"shàng","numeric":"shang4"},"meanings":["(bound form) up; upper; above; previous","to climb; to get onto; to go up"]}]}]"#.as_bytes();
    let imported = parse_hsk(input)?;
    let database = GeneratorDatabase::open()?;
    let connection = database.connection();
    create_schema(connection)?;
    insert_dictionary(connection, &imported.words, &imported.senses)?;
    assert_eq!(
        scalar(
            connection,
            "SELECT sense_kind FROM word_senses AS s JOIN sense_display AS d USING (word_id, sense_order) JOIN words AS w ON w.id = s.word_id WHERE w.simplified = '上' AND w.pinyin_numeric = 'shang3'"
        )?,
        "usage_reference"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT sense_kind FROM word_senses AS s JOIN sense_display AS d USING (word_id, sense_order) JOIN words AS w ON w.id = s.word_id WHERE w.simplified = '上' AND w.pinyin_numeric = 'shang4' AND s.sense_order = 1"
        )?,
        "lexical"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*)::VARCHAR FROM words AS w WHERE w.simplified = '上' AND EXISTS (SELECT 1 FROM sense_display AS d WHERE d.word_id = w.id AND d.sense_kind = 'lexical' AND d.definition IS NOT NULL)"
        )?,
        "1"
    );
    assert_eq!(
        scalar(
            connection,
            "SELECT gloss FROM word_senses AS s JOIN words AS w ON w.id = s.word_id WHERE w.pinyin_numeric = 'shang3'"
        )?,
        "used in 上声"
    );
    Ok(())
}
