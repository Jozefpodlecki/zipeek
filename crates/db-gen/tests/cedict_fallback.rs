use std::path::Path;

use db_gen::{
    import::{
        CedictIndex, FallbackSummary, ImportDiagnosticKind as Kind, ImportError,
        apply_cedict_fallback, parse_hsk,
    },
    import_primary, prepare,
};
use duckdb_neo::Parameters;

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
