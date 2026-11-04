use super::*;
use crate::{Hsk3Level, Word, WordId};

fn id(value: i64) -> WordId {
    WordId::try_from(value).unwrap()
}

fn word(value: i64) -> Word {
    Word {
        id: id(value),
        simplified: "好".to_owned(),
        traditional: None,
        pinyin: "hǎo".to_owned(),
        pinyin_numeric: None,
        english: Some("good".to_owned()),
        part_of_speech: vec![],
        hsk2_level: None,
        hsk3_level: Some(Hsk3Level::One),
        frequency: None,
    }
}

#[test]
fn canonical_v1_matches_published_golden() {
    let query = ParsedPracticeQuery::from_pairs([
        ("level", "3"),
        ("pos", "verb,noun,verb"),
        ("offset", "10"),
    ]);
    let normalized = query.normalize(ExerciseVariant::EngPinyin, |_| None);
    let input = normalized.canonical_seed_input(ExerciseVariant::EngPinyin);
    assert_eq!(input, "zipeek-round-v1|eng-pinyin|hsk3|3|noun,verb|10|none");
    assert_eq!(fnv1a64(&input), 0xa0e531d76df62844);
}

#[test]
fn levels_default_in_selected_system_without_broadening() {
    for (system, entered, expected) in [
        ("2.0", "7", 1),
        ("2.0", "6", 6),
        ("3.0", "7-9", 7),
        ("3.0", "99", 1),
        ("bad", "3", 3),
    ] {
        let query = ParsedPracticeQuery::from_pairs([("hsk", system), ("level", entered)]);
        assert_eq!(query.filter.level.as_i32(), expected);
        assert_eq!(
            query.filter.level.system() == crate::HskSystem::Hsk2,
            system == "2.0"
        );
    }
}

#[test]
fn malformed_numbers_and_unknown_tokens_are_diagnosed() {
    for value in ["", "-1", "+1", " 1", "1.0", "18446744073709551616"] {
        let query = ParsedPracticeQuery::from_pairs([("offset", value), ("seed", value)]);
        assert_eq!((query.offset, query.seed), (0, None));
        assert!(!query.warnings.is_empty());
    }
    let query = ParsedPracticeQuery::from_pairs([
        ("pos", "verb,unknown,noun,Noun,verb"),
        ("exclude", "2,1,2,-1,abc,2147483648"),
    ]);
    assert_eq!(query.filter.pos_tokens(), "noun,verb");
    assert_eq!(query.exclusions, [id(1), id(2)].into());
    assert_eq!(query.warnings.len(), 5);
}

#[test]
fn fixed_ignores_seed_without_lookup_or_offset_reset() {
    let query = ParsedPracticeQuery::from_pairs([
        ("mode", "fixed"),
        ("rounds", "5"),
        ("seed", "1"),
        ("offset", "40"),
    ]);
    let normalized = query.normalize(ExerciseVariant::EngPinyin, |_| {
        panic!("fixed seed must not be looked up")
    });
    assert_eq!(normalized.mode, PracticeMode::Fixed(FixedRounds::Five));
    assert_eq!(
        (normalized.offset, normalized.seed),
        (40, SeedDisposition::Unseeded)
    );
    assert!(
        normalized
            .warnings
            .iter()
            .any(|warning| warning.field == "seed")
    );
}

#[test]
fn absent_missing_and_excluded_seed_keep_entered_offset() {
    let existing = word(1);
    for pairs in [
        vec![],
        vec![("seed", "2")],
        vec![("seed", "1"), ("exclude", "1,999")],
    ] {
        let query = ParsedPracticeQuery::from_pairs(pairs.into_iter().chain([("offset", "12")]));
        let normalized = query.normalize(ExerciseVariant::EngPinyin, |id| {
            (id == existing.id).then_some(&existing)
        });
        assert_eq!(
            (normalized.offset, normalized.seed),
            (12, SeedDisposition::Unseeded)
        );
        assert!(!normalized.exclusions.contains(&id(999)));
    }
}

#[test]
fn retained_seed_zeros_offset_and_keeps_original_filter() {
    let existing = word(1);
    let query = ParsedPracticeQuery::from_pairs([("seed", "1"), ("level", "3"), ("offset", "12")]);
    let normalized = query.normalize(ExerciseVariant::EngPinyin, |_| Some(&existing));
    assert_eq!(
        (normalized.offset, normalized.seed),
        (0, SeedDisposition::Ready(id(1)))
    );
    assert_eq!(normalized.filter.level.as_i32(), 3);
}

#[test]
fn recovery_keeps_bad_seed_for_single_stream_builder() {
    let query = ParsedPracticeQuery::from_pairs([("seed", "1"), ("offset", "12")]);
    let mut existing = word(1);
    existing.hsk3_level = None;
    let normalized = query.normalize(ExerciseVariant::EngPinyin, |_| Some(&existing));
    assert_eq!(normalized.seed, SeedDisposition::RecoveryRequired(id(1)));
    assert!(
        normalized
            .canonical_seed_input(ExerciseVariant::EngPinyin)
            .ends_with("|0|1")
    );
    existing.hsk3_level = Some(Hsk3Level::One);
    existing.english = None;
    assert_eq!(
        query
            .normalize(ExerciseVariant::EngPinyin, |_| Some(&existing))
            .seed,
        SeedDisposition::RecoveryRequired(id(1))
    );
    assert_eq!(
        query
            .normalize(ExerciseVariant::PinyinHanzi, |_| Some(&existing))
            .seed,
        SeedDisposition::Ready(id(1))
    );
}

#[test]
fn canonical_pairs_round_trip_without_extra_encoding_or_saved_state() {
    let words = [word(1), word(2)];
    let query = ParsedPracticeQuery::from_pairs([
        ("pos", "verb,noun"),
        ("exclude", "02,1,2"),
        ("rounds", "20"),
    ]);
    let normalized = query.normalize(ExerciseVariant::EngPinyin, |id| {
        words.iter().find(|word| word.id == id)
    });
    let pairs = normalized.query_pairs();
    assert!(pairs.contains(&("exclude".to_owned(), "1,2".to_owned())));
    assert!(pairs.contains(&("pos".to_owned(), "noun,verb".to_owned())));
    assert!(!pairs.iter().any(|(key, _)| key == "rounds" || key == "v"));
    let parsed = ParsedPracticeQuery::from_pairs(
        pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    );
    assert_eq!(parsed.filter, normalized.filter);
    assert_eq!(parsed.exclusions, normalized.exclusions);
    assert!(ParsedPracticeQuery::default().exclusions.is_empty());
}

#[test]
fn offsets_wrap_safely_with_variant_specific_steps() {
    assert_eq!(
        advance_offset(4, 8, ExerciseVariant::EngPinyin),
        OffsetAdvance {
            offset: 8,
            looped: false
        }
    );
    assert_eq!(
        advance_offset(8, 8, ExerciseVariant::EngPinyin),
        OffsetAdvance {
            offset: 0,
            looped: true
        }
    );
    assert_eq!(
        advance_offset(4, 8, ExerciseVariant::MeaningToHanzi).offset,
        5
    );
    assert!(advance_offset(u64::MAX, u64::MAX, ExerciseVariant::EngPinyin).looped);
}

#[test]
fn fixed_progress_is_independent_of_wrapping_and_generation_unions() {
    let mut progress = FixedProgress::new(FixedRounds::Five);
    for _ in 0..4 {
        progress.finish_round();
    }
    assert!(!progress.is_complete());
    assert!(advance_offset(8, 8, ExerciseVariant::EngPinyin).looped);
    assert_eq!(progress.completed(), 4);
    for _ in 0..3 {
        progress.finish_round();
    }
    assert!(progress.is_complete());
    assert_eq!(progress.completed(), 5);
    assert_eq!(FixedProgress::new(FixedRounds::Five).completed(), 0);
    assert_eq!(
        union_exclusions([&[id(1), id(2)].into(), &[id(2), id(3)].into()]),
        [id(1), id(2), id(3)].into()
    );
}
