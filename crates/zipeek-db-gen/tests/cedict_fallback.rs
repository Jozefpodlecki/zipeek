use std::path::Path;

use duckdb_neo::Parameters;
use zipeek_db_gen::{
    import::{
        CedictIndex, FallbackSummary, ImportDiagnosticKind as Kind, ImportError,
        apply_cedict_fallback, parse_hsk,
    },
    import_primary, prepare,
};

const PRIMARY: &[u8] = include_bytes!("fixtures/fallback/hsk.json");
const CEDICT: &[u8] = include_bytes!("fixtures/fallback/cedict.u8");

#[test]
fn fallback_only_fills_empty_primary_glosses_and_preserves_every_other_field()
-> Result<(), Box<dyn std::error::Error>> {
    let mut imported = parse_hsk(PRIMARY)?;
    let original = imported.words.clone();
    let summary = apply_cedict_fallback(&mut imported, CEDICT)?;
    assert_eq!(
        summary,
        FallbackSummary {
            filled: 5,
            unresolved: 2,
            skipped_variants: 2
        }
    );
    for (before, after) in original.iter().zip(&imported.words) {
        let mut expected = before.clone();
        expected.english = after.english.clone();
        assert_eq!(expected, *after);
        if before.english.is_some() {
            assert_eq!(before.english, after.english);
        }
    }
    for (word, numeric, english) in [
        ("女", "nü3", Some("female ｜ woman|daughter")),
        ("行", "hang2", Some("line｜row")),
        ("行", "xing2", Some("Primary Stay")),
        ("吗", "ma", Some("question particle")),
        ("儿", "r5", Some("suffix")),
        ("呣", "ng5", Some("hmm")),
        ("无", "wu2", None),
        ("乙", "yi3", None),
        ("甲", "jia3", Some("Original ｜ gloss|Second")),
    ] {
        let found = imported
            .words
            .iter()
            .find(|row| row.simplified == word && row.pinyin_numeric.as_deref() == Some(numeric))
            .unwrap();
        assert_eq!(found.english.as_deref(), english, "{word} {numeric}");
    }
    assert_eq!(imported.words.len(), original.len());
    assert!(
        imported
            .diagnostics
            .iter()
            .any(|event| event.kind == Kind::GlossPipeReplacement
                && event.detail.contains("female | woman"))
    );
    assert_eq!(
        imported
            .diagnostics
            .iter()
            .filter(|event| event.kind == Kind::FallbackUnavailable)
            .count(),
        2
    );
    Ok(())
}

#[test]
fn fallback_is_deterministic_and_never_replaces_primary_glosses()
-> Result<(), Box<dyn std::error::Error>> {
    let mut first = parse_hsk(PRIMARY)?;
    let mut second = first.clone();
    assert_eq!(
        apply_cedict_fallback(&mut first, CEDICT)?,
        apply_cedict_fallback(&mut second, CEDICT)?
    );
    assert_eq!(first, second);
    for word in &mut first.words {
        word.english = Some("Keep me".into());
    }
    let before = first.clone();
    assert_eq!(
        apply_cedict_fallback(&mut first, b"\xff")?,
        FallbackSummary::default()
    );
    assert_eq!(first, before);
    Ok(())
}

#[test]
fn display_tone_marks_can_match_without_rewriting_the_source_pinyin()
-> Result<(), Box<dyn std::error::Error>> {
    let mut imported = parse_hsk(PRIMARY)?;
    let word = imported
        .words
        .iter_mut()
        .find(|word| word.simplified == "女")
        .unwrap();
    word.pinyin_numeric = None;
    let display = word.pinyin.clone();
    apply_cedict_fallback(&mut imported, CEDICT)?;
    let word = imported
        .words
        .iter()
        .find(|word| word.simplified == "女")
        .unwrap();
    assert_eq!(word.pinyin, display);
    assert_eq!(word.pinyin_numeric, None);
    assert_eq!(word.english.as_deref(), Some("female ｜ woman|daughter"));
    Ok(())
}

#[test]
fn malformed_fallback_data_reports_the_source_line_and_reason() {
    assert!(matches!(
        CedictIndex::parse(b"\xff"),
        Err(ImportError::CedictUtf8(_))
    ));
    for line in [
        "only-one [x1] /gloss/",
        "a b x1 /gloss/",
        "a b [x1 /gloss/",
        "a b [] /gloss/",
        "a b [x1] gloss",
    ] {
        let input = format!("# header\n{line}\n");
        let error = CedictIndex::parse(input.as_bytes()).unwrap_err();
        assert!(matches!(error, ImportError::CedictLine { line: 2, .. }));
        assert!(error.to_string().contains(line));
    }
}

#[test]
fn empty_fallback_retains_null_and_does_not_drop_readings() -> Result<(), Box<dyn std::error::Error>>
{
    let mut imported = parse_hsk(PRIMARY)?;
    let before = imported.words.clone();
    let summary = apply_cedict_fallback(&mut imported, b"# empty\r\n\r\n")?;
    assert_eq!(summary.unresolved, 7);
    assert_eq!(summary.filled, 0);
    assert_eq!(imported.words, before);
    Ok(())
}

#[test]
fn verified_pipeline_applies_fallback_before_sql_insertion()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fallback/MANIFEST.toml");
    let prepared = prepare(&manifest)?;
    let imported = import_primary(&prepared)?;
    assert_eq!(
        imported
            .words
            .iter()
            .filter(|word| word.english.is_some())
            .count(),
        7
    );
    let mut result = prepared.database().connection().query(
        "SELECT count(*)::VARCHAR FROM words WHERE english IS NULL",
        Parameters::None,
    )?;
    let chunk = result.next_chunk()?.ok_or("missing query chunk")?;
    let reader = chunk.get_vector_at::<String>(0)?.reader()?;
    assert_eq!(reader.iter().next().flatten(), Some("2"));
    Ok(())
}

#[test]
fn preserved_cedict_snapshot_parses_all_pinned_rows_and_keeps_primary_output()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/MANIFEST.toml");
    let prepared = prepare(&manifest)?;
    let cedict = prepared
        .inputs()
        .sources()
        .iter()
        .find(|source| source.name() == "cc-cedict-fallback")
        .unwrap();
    assert_eq!(
        CedictIndex::parse(cedict.file().bytes())?.entry_count(),
        125320
    );
    let primary = prepared
        .inputs()
        .sources()
        .iter()
        .find(|source| source.name() == "hsk-primary")
        .unwrap();
    let expected = parse_hsk(primary.file().bytes())?;
    assert_eq!(import_primary(&prepared)?, expected);
    Ok(())
}

#[test]
fn non_hsk_cedict_entries_become_searchable_words_with_source_senses()
-> Result<(), Box<dyn std::error::Error>> {
    use zipeek_db_gen::import::expand_with_cedict;

    let mut imported = parse_hsk(PRIMARY)?;
    let hsk_count = imported.words.len();
    let source = "搖滾樂 摇滚乐 [yao2 gun3 yue4] /rock music/rock 'n roll/\n行 行 [xing2] /to walk/\n行 行 [hang2] /row/\n";
    let expansion = expand_with_cedict(&mut imported, source.as_bytes())?;
    assert!(expansion.added_words >= 1);
    let rock = imported
        .words
        .iter()
        .find(|word| word.simplified == "摇滚乐")
        .unwrap();
    assert_eq!(rock.traditional.as_deref(), Some("搖滾樂"));
    assert_eq!(rock.hsk2_level, None);
    assert_eq!(rock.hsk3_level, None);
    assert_eq!(rock.english.as_deref(), Some("rock music|rock 'n roll"));
    assert!(rock.id.as_i32() as usize > hsk_count);
    let senses: Vec<_> = imported
        .senses
        .iter()
        .filter(|sense| sense.word_id == rock.id)
        .collect();
    assert_eq!(senses.len(), 2);
    assert!(senses.iter().all(|sense| sense.source == "cc-cedict"));
    assert!(
        expansion
            .character_readings
            .iter()
            .any(|reading| reading.character == "行" && reading.pinyin_numeric == "xing2")
    );
    assert!(
        expansion
            .character_readings
            .iter()
            .any(|reading| reading.character == "行" && reading.pinyin_numeric == "hang2")
    );
    Ok(())
}

#[test]
fn cedict_expansion_is_deterministic_and_does_not_duplicate_hsk_readings()
-> Result<(), Box<dyn std::error::Error>> {
    use zipeek_db_gen::import::expand_with_cedict;

    let source = b"\xe8\xa1\x8c \xe8\xa1\x8c [xing2] /to walk/\n\xe6\x90\x96\xe6\xbb\x9a\xe6\xa8\x82 \xe6\x91\x87\xe6\xbb\x9a\xe4\xb9\x90 [yao2 gun3 yue4] /rock music/\n";
    let mut first = parse_hsk(PRIMARY)?;
    let mut second = first.clone();
    let first_summary = expand_with_cedict(&mut first, source)?;
    let second_summary = expand_with_cedict(&mut second, source)?;
    assert_eq!(first_summary, second_summary);
    assert_eq!(first, second);
    let keys: std::collections::BTreeSet<_> = first
        .words
        .iter()
        .map(|word| {
            (
                word.simplified.as_str(),
                zipeek_core::normalize_for_comparison(
                    word.pinyin_numeric.as_deref().unwrap_or(&word.pinyin),
                ),
            )
        })
        .collect();
    assert_eq!(keys.len(), first.words.len());
    Ok(())
}

#[test]
fn fallback_preparation_provides_individual_meaning_metadata_tables()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fallback/MANIFEST.toml");
    let prepared = prepare(&manifest)?;
    let connection = prepared.database().connection();
    for query in [
        "SELECT word_id, sense_order, definition FROM sense_display LIMIT 1",
        "SELECT word_id, sense_order, qualifier_kind, qualifier_code FROM sense_qualifiers LIMIT 1",
    ] {
        let _result = connection.query(query, Parameters::None)?;
    }
    Ok(())
}
