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
    let normalized = query.normalize(ExerciseVariant::MatchingEnglishPinyin, |_| None);
    let input = normalized.canonical_seed_input(ExerciseVariant::MatchingEnglishPinyin);
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
fn parsed_practice_levels_match_system_metadata() {
    for system in crate::HskSystem::ALL {
        for level in system.levels() {
            let query = ParsedPracticeQuery::from_pairs([
                (
                    "hsk",
                    if system == crate::HskSystem::Hsk2 {
                        "2.0"
                    } else {
                        "3.0"
                    },
                ),
                (
                    "level",
                    if level.as_i32() == 7 {
                        "7-9"
                    } else {
                        level.band_label()
                    },
                ),
            ]);
            assert_eq!(query.filter.level.system(), system);
            assert_eq!(query.filter.level, *level);
        }
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
    let normalized = query.normalize(ExerciseVariant::MatchingEnglishPinyin, |_| {
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
        let normalized = query.normalize(ExerciseVariant::MatchingEnglishPinyin, |id| {
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
    let normalized = query.normalize(ExerciseVariant::MatchingEnglishPinyin, |_| Some(&existing));
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
    let normalized = query.normalize(ExerciseVariant::MatchingEnglishPinyin, |_| Some(&existing));
    assert_eq!(normalized.seed, SeedDisposition::RecoveryRequired(id(1)));
    assert!(
        normalized
            .canonical_seed_input(ExerciseVariant::MatchingEnglishPinyin)
            .ends_with("|0|1")
    );
    existing.hsk3_level = Some(Hsk3Level::One);
    existing.english = None;
    assert_eq!(
        query
            .normalize(ExerciseVariant::MatchingEnglishPinyin, |_| Some(&existing))
            .seed,
        SeedDisposition::RecoveryRequired(id(1))
    );
    assert_eq!(
        query
            .normalize(ExerciseVariant::MatchingPinyinHanzi, |_| Some(&existing))
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
    let normalized = query.normalize(ExerciseVariant::MatchingEnglishPinyin, |id| {
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
        advance_offset(4, 8, ExerciseVariant::MatchingEnglishPinyin),
        OffsetAdvance {
            offset: 8,
            looped: false
        }
    );
    assert_eq!(
        advance_offset(8, 8, ExerciseVariant::MatchingEnglishPinyin),
        OffsetAdvance {
            offset: 0,
            looped: true
        }
    );
    assert_eq!(
        advance_offset(4, 8, ExerciseVariant::MultipleChoiceMeaningHanzi).offset,
        5
    );
    assert!(advance_offset(u64::MAX, u64::MAX, ExerciseVariant::MatchingEnglishPinyin).looped);
}

#[test]
fn fixed_progress_is_independent_of_wrapping_and_generation_unions() {
    let mut progress = FixedProgress::new(FixedRounds::Five);
    for _ in 0..4 {
        progress.finish_round();
    }
    assert!(!progress.is_complete());
    assert!(advance_offset(8, 8, ExerciseVariant::MatchingEnglishPinyin).looped);
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

#[test]
fn word_set_snapshot_is_portable_canonical_and_does_not_change_v1_seed_format() {
    let variant = ExerciseVariant::MultipleChoiceHanziMeaning;
    let parsed = ParsedPracticeQuery::from_pairs([("set_ids", "9,3,5"), ("offset", "4")]);
    assert!(parsed.warnings.is_empty());
    let normalized = parsed.normalize(variant, |_| None);
    assert_eq!(
        normalized
            .query_pairs()
            .into_iter()
            .find(|(key, _)| key == "set_ids")
            .unwrap()
            .1,
        "3,5,9"
    );
    assert_eq!(
        normalized.canonical_seed_input(variant),
        ParsedPracticeQuery::from_pairs([("offset", "4")])
            .normalize(variant, |_| None)
            .canonical_seed_input(variant)
    );
    let parsed_again = ParsedPracticeQuery::from_pairs(
        normalized
            .query_pairs()
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect::<Vec<_>>(),
    );
    assert_eq!(parsed_again.word_set, parsed.word_set);
}

#[test]
fn invalid_word_set_snapshot_never_silently_becomes_unrestricted_practice() {
    for value in ["", "1,,2", "2,2", "-1", "abc", "0", "2147483648"] {
        let parsed = ParsedPracticeQuery::from_pairs([("set_ids", value)]);
        assert!(parsed.word_set.is_none());
        assert!(
            parsed
                .warnings
                .iter()
                .any(|warning| warning.field == "set_ids")
        );
    }
    let overflow = (1..=129)
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert!(
        ParsedPracticeQuery::from_pairs([("set_ids", overflow.as_str())])
            .word_set
            .is_none()
    );
    let parsed = ParsedPracticeQuery::from_pairs([("set_ids", "2,1"), ("seed", "1")]);
    assert!(parsed.seed.is_none());
    assert!(
        parsed
            .warnings
            .iter()
            .any(|warning| warning.field == "seed")
    );
}

#[test]
fn filtered_practice_sources_preserve_legacy_seed_and_round_trip() {
    let variant = ExerciseVariant::MatchingEnglishPinyin;
    let legacy = ParsedPracticeQuery::from_pairs([("offset", "4")]).normalize(variant, |_| None);
    let source = "filters:2.3,3.1|shopping|noun";
    let parsed = ParsedPracticeQuery::from_pairs([("offset", "4"), ("source", source)]);
    assert_eq!(parsed.dictionary_source.as_deref(), Some(source));
    let filtered = parsed.normalize(variant, |_| None);
    assert_eq!(
        filtered.canonical_seed_input(variant),
        format!("{}|{source}", legacy.canonical_seed_input(variant))
    );
    let pairs = filtered.query_pairs();
    let replay = ParsedPracticeQuery::from_pairs(
        pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    );
    assert_eq!(replay.dictionary_source.as_deref(), Some(source));
    assert_eq!(
        replay
            .normalize(variant, |_| None)
            .canonical_seed_input(variant),
        filtered.canonical_seed_input(variant)
    );
    assert_eq!(
        legacy.canonical_seed_input(variant),
        "zipeek-round-v1|eng-pinyin|hsk3|1||4|none"
    );
}

#[test]
fn filtered_practice_source_rejects_mixed_word_set_seed_and_invalid_text() {
    for fields in [
        vec![("source", "filters:3.1||"), ("seed", "1")],
        vec![("source", "filters:3.1||"), ("set_ids", "1,2")],
        vec![("source", "filters:3.1|bad;sql|")],
        vec![("source", "filters:3.1|a b|")],
    ] {
        let parsed = ParsedPracticeQuery::from_pairs(fields);
        assert!(parsed.dictionary_source.is_none());
        assert!(
            parsed
                .warnings
                .iter()
                .any(|warning| warning.field == "source")
        );
    }
}
