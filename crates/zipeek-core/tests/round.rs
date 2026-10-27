mod support;

use std::collections::BTreeSet;
use zipeek_core::{
    Word, WordId,
    practice::{ExerciseVariant, ParsedPracticeQuery, fnv1a64},
    round::{
        BuildError, CandidatePool, CandidateView, Round, RoundCandidates, RoundContent, build_round,
    },
};

fn build(
    pool: &CandidatePool,
    variant: ExerciseVariant,
    offset: &str,
) -> Result<Round, BuildError> {
    let query = ParsedPracticeQuery::from_pairs([("offset", offset)]).normalize(variant, |_| None);
    build_round(
        RoundCandidates {
            initial: pool,
            original_seed: None,
            recovery: &[],
            recovered_views: &[],
        },
        variant,
        &query,
    )
}

#[test]
fn native_golden_vectors_lock_the_complete_stream_and_layout() {
    let report = support::report();
    assert_eq!(
        report,
        include_str!("fixtures/round-golden.txt").replace("\r\n", "\n")
    );
    assert_eq!(fnv1a64(&report), 0xa94efad5496532a0);
}

#[test]
fn all_variants_return_only_playable_distinct_visible_cards() {
    for (_, round) in support::vectors() {
        match round.content {
            RoundContent::Matching { left, right } => {
                assert_eq!((left.len(), right.len()), (4, 4));
                assert_eq!(
                    left.iter()
                        .map(|card| card.word_id)
                        .collect::<BTreeSet<_>>(),
                    right.iter().map(|card| card.word_id).collect()
                );
                assert_eq!(
                    right
                        .iter()
                        .map(|card| &card.label)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    4
                );
                assert_eq!(round.trace.views.last().unwrap().reserves.len(), 4);
            }
            RoundContent::Mcq {
                prompt,
                options,
                correct_id,
            } => {
                assert_eq!(options.len(), 3);
                assert_eq!(prompt.word_id, correct_id);
                assert_eq!(
                    options
                        .iter()
                        .filter(|card| card.word_id == correct_id)
                        .count(),
                    1
                );
                assert_eq!(
                    options
                        .iter()
                        .map(|card| &card.label)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    3
                );
            }
        }
    }
}

#[test]
fn recovery_preserves_original_hash_context_and_stops_early_or_falls_back() {
    let vectors = support::vectors();
    let first = &vectors
        .iter()
        .find(|(name, _)| name == "recovery-first")
        .unwrap()
        .1;
    let fallback = &vectors
        .iter()
        .find(|(name, _)| name == "recovery-fallback")
        .unwrap()
        .1;
    assert_eq!(first.trace.recovery_draws.len(), 1);
    assert!(!first.trace.recovery_fallback);
    assert_eq!(fallback.trace.recovery_draws.len(), 10);
    assert!(fallback.trace.recovery_fallback);
    assert_eq!(fallback.seed, Some(WordId::try_from(1).unwrap()));
    assert_eq!(
        first.canonical_input,
        "zipeek-round-v1|eng-pinyin|hsk3|3|verb|0|999"
    );
    assert_eq!(first.canonical_input, fallback.canonical_input);
    assert_eq!(first.context.level.as_i32(), 1);
    assert!(first.context.part_of_speech.is_empty());
    let tenth = &vectors
        .iter()
        .find(|(name, _)| name == "recovery-tenth")
        .unwrap()
        .1;
    assert_eq!(tenth.trace.recovery_draws.len(), 10);
    assert!(!tenth.trace.recovery_fallback);
    assert_eq!(tenth.seed, Some(WordId::try_from(48).unwrap()));
}

#[test]
fn first_clean_window_is_selected_and_wrap_never_splits_it() {
    let mut words = support::words();
    for word in &mut words[..4] {
        word.english = Some("same".to_owned());
    }
    let pool = support::pool(&words);
    let round = build(&pool, ExerciseVariant::HanziEng, "0").unwrap();
    assert_eq!(round.effective_offset, 3);
    assert_eq!(
        round.trace.views[0]
            .correct
            .iter()
            .map(|id| id.as_i32())
            .collect::<Vec<_>>(),
        [4, 5, 6, 7]
    );
    let pool = support::pool(&support::words());
    assert_eq!(
        build(&pool, ExerciseVariant::HanziEng, "19")
            .unwrap()
            .effective_offset,
        0
    );
}

#[test]
fn malformed_and_excluded_pools_fail_without_silently_reordering() {
    let words = support::words();
    let mut pool = support::pool(&words);
    pool.strict.swap(0, 1);
    assert!(matches!(
        build(&pool, ExerciseVariant::HanziEng, "0"),
        Err(BuildError::InvalidPool { .. })
    ));
    let pool = support::pool(&words);
    let query = ParsedPracticeQuery::from_pairs([("exclude", "1")])
        .normalize(ExerciseVariant::HanziEng, |id| {
            words.iter().find(|word| word.id == id)
        });
    assert!(matches!(
        build_round(
            RoundCandidates {
                initial: &pool,
                original_seed: None,
                recovery: &[],
                recovered_views: &[]
            },
            ExerciseVariant::HanziEng,
            &query
        ),
        Err(BuildError::InvalidPool { .. })
    ));
}

#[test]
fn empty_and_duplicate_only_views_exhaust_safely() {
    for pool in [CandidatePool::default(), {
        let mut words = support::words();
        for word in &mut words {
            word.english = Some("same".to_owned());
        }
        support::pool(&words)
    }] {
        let Err(BuildError::InsufficientPool { attempts, .. }) =
            build(&pool, ExerciseVariant::HanziEng, "0")
        else {
            panic!("expected insufficient pool");
        };
        assert_eq!(attempts.len(), 3);
        assert!(
            attempts
                .iter()
                .all(|attempt| attempt.rejections <= attempt.candidate_count.min(50))
        );
    }
}

#[test]
fn widening_starts_at_zero_and_reports_the_chosen_view() {
    let mut pool = support::pool(&support::words());
    pool.strict.clear();
    let round = build(&pool, ExerciseVariant::PinyinHanzi, "10").unwrap();
    assert_eq!(round.selected_view, CandidateView::HskWidened);
    assert_eq!(round.effective_offset, 0);
    assert!(round.canonical_input.ends_with("|10|none"));
}

#[test]
fn normalization_uses_only_the_visible_answer_label() {
    let original = support::words();
    for (variant, alter) in [
        (
            ExerciseVariant::EngPinyin,
            (|word: &mut Word| word.pinyin = "YI1".to_owned()) as fn(&mut Word),
        ),
        (ExerciseVariant::PinyinHanzi, |word: &mut Word| {
            word.simplified = "一".to_owned()
        }),
        (ExerciseVariant::HanziEng, |word: &mut Word| {
            word.english = Some("  ONE  ".to_owned())
        }),
    ] {
        let mut words = original.clone();
        alter(&mut words[1]);
        let round = build(&support::pool(&words), variant, "0").unwrap();
        assert_eq!(round.effective_offset, 1);
    }
    let mut words = original;
    words[0].english = Some("to eat".to_owned());
    words[1].english = Some("eat".to_owned());
    assert_eq!(
        build(&support::pool(&words), ExerciseVariant::HanziEng, "0")
            .unwrap()
            .effective_offset,
        0
    );
}

#[test]
fn recovery_fails_safely_without_an_eligible_candidate_or_context_views() {
    let words = support::words();
    let mut bad = words[0].clone();
    bad.hsk3_level = None;
    let pool = support::pool(&words);
    let query = ParsedPracticeQuery::from_pairs([("seed", "1")])
        .normalize(ExerciseVariant::EngPinyin, |_| Some(&bad));
    for recovery in [vec![], vec![bad.clone()]] {
        let error = build_round(
            RoundCandidates {
                initial: &pool,
                original_seed: Some(&bad),
                recovery: &recovery,
                recovered_views: &[],
            },
            ExerciseVariant::EngPinyin,
            &query,
        )
        .unwrap_err();
        let BuildError::RecoveryUnavailable { attempts } = error else {
            panic!("expected recovery failure");
        };
        assert_eq!(attempts.len(), if recovery.is_empty() { 0 } else { 10 });
    }
    assert!(matches!(
        build_round(
            RoundCandidates {
                initial: &pool,
                original_seed: Some(&bad),
                recovery: &words,
                recovered_views: &[]
            },
            ExerciseVariant::EngPinyin,
            &query
        ),
        Err(BuildError::MissingRecoveryViews { .. })
    ));
}

#[test]
fn supplied_views_cannot_cross_hsk_systems_or_change_rows() {
    let mut pool = support::pool(&support::words());
    pool.strict[0].english = Some("changed".to_owned());
    assert!(matches!(
        build(&pool, ExerciseVariant::HanziEng, "0"),
        Err(BuildError::InvalidPool { .. })
    ));
    let mut words = support::words();
    words[0].hsk3_level = None;
    words[0].hsk2_level = Some(zipeek_core::Hsk2Level::One);
    assert!(matches!(
        build(&support::pool(&words), ExerciseVariant::HanziEng, "0"),
        Err(BuildError::InvalidPool { .. })
    ));
}

#[test]
fn failed_distractor_view_is_discarded_and_stream_continues() {
    let vectors = support::vectors();
    let round = &vectors
        .iter()
        .find(|(name, _)| name == "distractor-fallback")
        .unwrap()
        .1;
    assert_eq!(round.trace.views.len(), 2);
    assert_eq!(round.trace.views[0].reserves.len(), 3);
    assert_eq!(round.trace.views[0].rejections, 12);
    assert_eq!(
        round.trace.views[1]
            .reserves
            .iter()
            .map(|id| id.as_i32())
            .collect::<Vec<_>>(),
        [5, 6, 10, 16]
    );
    assert_eq!(round.selected_view, CandidateView::HskWidened);
}

#[test]
fn exclusions_are_applied_before_offsets_and_equivalent_seed_urls_replay() {
    let words = support::words();
    let filtered = support::pool(&words[1..]);
    let query = ParsedPracticeQuery::from_pairs([("exclude", "1"), ("offset", "4")])
        .normalize(ExerciseVariant::PinyinHanzi, |id| {
            words.iter().find(|word| word.id == id)
        });
    let round = build_round(
        RoundCandidates {
            initial: &filtered,
            original_seed: None,
            recovery: &[],
            recovered_views: &[],
        },
        ExerciseVariant::PinyinHanzi,
        &query,
    )
    .unwrap();
    assert_eq!(round.trace.views[0].correct[0].as_i32(), 6);
    let pool = support::pool(&words);
    let mut rounds = Vec::new();
    for offset in ["0", "999"] {
        let query = ParsedPracticeQuery::from_pairs([("seed", "1"), ("offset", offset)])
            .normalize(ExerciseVariant::EngPinyin, |_| Some(&words[0]));
        rounds.push(
            build_round(
                RoundCandidates {
                    initial: &pool,
                    original_seed: Some(&words[0]),
                    recovery: &[],
                    recovered_views: &[],
                },
                ExerciseVariant::EngPinyin,
                &query,
            )
            .unwrap(),
        );
    }
    assert_eq!(rounds[0], rounds[1]);
}

#[test]
fn dropping_pos_is_the_last_fallback() {
    let mut words = support::words();
    for (index, word) in words.iter_mut().enumerate() {
        word.part_of_speech = vec![if index < 7 {
            zipeek_core::PartOfSpeech::Noun
        } else {
            zipeek_core::PartOfSpeech::Verb
        }];
    }
    let mut pool = support::pool(&words);
    pool.strict.truncate(7);
    pool.hsk_widened.truncate(7);
    let query = ParsedPracticeQuery::from_pairs([("pos", "noun")])
        .normalize(ExerciseVariant::PinyinHanzi, |_| None);
    let round = build_round(
        RoundCandidates {
            initial: &pool,
            original_seed: None,
            recovery: &[],
            recovered_views: &[],
        },
        ExerciseVariant::PinyinHanzi,
        &query,
    )
    .unwrap();
    assert_eq!(round.selected_view, CandidateView::AnyPos);
    assert_eq!(round.trace.views.len(), 3);
}

#[test]
fn left_label_duplicates_are_allowed_and_missing_english_is_variant_specific() {
    let mut words = support::words();
    for word in &mut words[..4] {
        word.english = Some("same".to_owned());
    }
    assert_eq!(
        build(&support::pool(&words), ExerciseVariant::EngPinyin, "0")
            .unwrap()
            .effective_offset,
        0
    );
    words[0].english = None;
    assert_eq!(
        build(&support::pool(&words), ExerciseVariant::PinyinHanzi, "0")
            .unwrap()
            .effective_offset,
        0
    );
    assert_eq!(
        build(&support::pool(&words), ExerciseVariant::EngPinyin, "0")
            .unwrap()
            .effective_offset,
        1
    );
}

#[test]
fn seeded_partner_requests_remove_even_rejected_rows() {
    let mut words = support::words();
    for word in &mut words[1..6] {
        word.pinyin = "yi1".to_owned();
    }
    let pool = support::pool(&words);
    let query = ParsedPracticeQuery::from_pairs([("seed", "1")])
        .normalize(ExerciseVariant::EngPinyin, |_| Some(&words[0]));
    let round = build_round(
        RoundCandidates {
            initial: &pool,
            original_seed: Some(&words[0]),
            recovery: &[],
            recovered_views: &[],
        },
        ExerciseVariant::EngPinyin,
        &query,
    )
    .unwrap();
    for trace in &round.trace.views {
        assert_eq!(
            trace.partner_draws.iter().collect::<BTreeSet<_>>().len(),
            trace.partner_draws.len()
        );
        assert!(trace.correct.iter().skip(1).all(|id| id.as_i32() > 6));
    }
}

#[test]
fn english_cards_and_uniqueness_use_only_the_first_stored_gloss() {
    let mut words = support::words();
    words[0].english = Some("one|first".to_owned());
    words[1].english = Some(" ONE |different".to_owned());
    let pool = support::pool(&words);
    assert_eq!(
        build(&pool, ExerciseVariant::HanziEng, "0")
            .unwrap()
            .effective_offset,
        1
    );
    let round = build(&pool, ExerciseVariant::EngPinyin, "0").unwrap();
    let RoundContent::Matching { left, .. } = round.content else {
        panic!("expected matching");
    };
    assert_eq!(
        left.iter()
            .find(|card| card.word_id.as_i32() == 1)
            .unwrap()
            .label,
        "one"
    );
    assert!(left.iter().all(|card| !card.label.contains('|')));
}
