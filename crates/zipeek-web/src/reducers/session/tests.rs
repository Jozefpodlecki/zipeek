use super::*;

use crate::reducers::matching::Side;
use zipeek_core::{WordId, practice::ParsedPracticeQuery, round::Card};

fn card(id: i64) -> Card {
    Card {
        word_id: WordId::try_from(id).unwrap(),
        label: id.to_string(),
    }
}
fn key(variant: ExerciseVariant, offset: u64) -> SessionKey {
    let query = ParsedPracticeQuery::from_pairs([
        ("mode", "fixed"),
        ("rounds", "5"),
        ("offset", &offset.to_string()),
    ])
    .normalize(variant, |_| None);
    SessionKey::new(
        variant,
        query,
        vec![
            ExerciseVariant::MultipleChoiceHanziMeaning,
            ExerciseVariant::MatchingHanziMeaning,
        ],
    )
}
fn content(variant: ExerciseVariant) -> RoundContent {
    if variant.is_mcq() {
        RoundContent::Mcq {
            prompt: card(1),
            options: (1..=3).map(card).collect(),
            correct_id: card(1).word_id,
        }
    } else {
        RoundContent::Matching {
            left: (1..=4).map(card).collect(),
            right: (1..=4).rev().map(card).collect(),
        }
    }
}
fn pick(state: Rc<SessionState>, side: Side, index: usize) -> Rc<SessionState> {
    let key = state.key.clone().unwrap();
    let epoch = state.generation;
    state.reduce(SessionAction::MatchSelect(
        key,
        epoch,
        CardChoice { side, index },
    ))
}
#[test]
fn learner_visible_round_data_never_exposes_matching_word_ids() {
    let round = RoundContent::Matching {
        left: vec![Card {
            word_id: WordId::try_from(999).unwrap(),
            label: "苹果".to_string(),
        }],
        right: vec![Card {
            word_id: WordId::try_from(999).unwrap(),
            label: "apple".to_string(),
        }],
    };
    let visible = SessionState::visible_round(&round);
    assert!(visible.contains("L0: 苹果"));
    assert!(visible.contains("R0: apple"));
    assert!(!visible.contains("999"));
}

#[test]
fn mixed_fixed_rounds_keep_shared_stats_and_weighted_required_items() {
    let mut variant = ExerciseVariant::MatchingHanziMeaning;
    let mut offset = 0;
    let mut state = Rc::new(SessionState::default()).reduce(SessionAction::Ready(
        key(variant, offset),
        content(variant),
        1000,
    ));
    for round in 0..5 {
        let current = state.key.clone().unwrap();
        if variant.is_mcq() {
            state = state.reduce(SessionAction::McqSelect(
                current.clone(),
                usize::from(round == 1),
            ));
        } else {
            if round == 0 {
                state = pick(pick(state, Side::Left, 0), Side::Right, 2);
                let ActiveRound::Matching(s) = &state.active else {
                    panic!("Expected matching")
                };
                let serial = s.wrong.as_ref().unwrap().serial;
                let epoch = s.epoch;
                state = state.reduce(SessionAction::ClearWrong(current.clone(), epoch, serial));
            }
            for index in 0..4 {
                state = pick(pick(state, Side::Left, index), Side::Right, 3 - index);
            }
        }
        if round == 4 {
            state = state.reduce(SessionAction::Finish(current.clone(), 5000));
            let stats = state.active.stats();
            assert_eq!(
                (
                    stats.correct,
                    stats.first_attempt,
                    stats.mistakes,
                    stats.highest_streak,
                    state.required
                ),
                (13, 11, 2, 9, 14)
            );
            assert_eq!(stats.finished_at.unwrap() - stats.started_at.unwrap(), 4000);
            assert!(stats.progress.unwrap().is_complete());
            let domain = state.domain_session.as_ref().unwrap();
            assert_eq!(domain.entries.len(), 5);
            assert_eq!(
                domain.totals(),
                ExerciseResult {
                    earned: 13,
                    possible: 14,
                    mistakes: 2,
                }
            );
            assert_eq!(
                domain.status,
                zipeek_exercises::session::SessionStatus::Completed
            );
            let repeated = state
                .clone()
                .reduce(SessionAction::Finish(current.clone(), 9000));
            assert!(Rc::ptr_eq(&state, &repeated));
        } else {
            offset += if variant.is_mcq() { 1 } else { 4 };
            variant = if variant.is_mcq() {
                ExerciseVariant::MatchingHanziMeaning
            } else {
                ExerciseVariant::MultipleChoiceHanziMeaning
            };
            let target = key(variant, offset);
            state = state.reduce(SessionAction::Advance(
                current.clone(),
                target.clone(),
                false,
            ));
            state = state.reduce(SessionAction::Ready(target, content(variant), 2000));
            assert_eq!(
                state.active.stats().progress.unwrap().completed(),
                round + 1
            );
            let late = state
                .clone()
                .reduce(SessionAction::ClearWrong(current, 1, 1));
            assert!(Rc::ptr_eq(&state, &late));
        }
    }
    let current = state.key.clone().unwrap();
    let target = key(ExerciseVariant::MultipleChoiceHanziMeaning, 30);
    state = state.reduce(SessionAction::Restart(current, target.clone()));
    state = state.reduce(SessionAction::Ready(
        target.clone(),
        content(target.variant),
        10000,
    ));
    assert_eq!(
        (
            state.required,
            state.active.stats().correct,
            state.active.stats().first_attempt
        ),
        (1, 0, 0)
    );
    assert_eq!(state.active.stats().started_at, Some(10000));
    assert_eq!(state.active.stats().progress.unwrap().completed(), 0);
}
#[test]
fn duplicate_ready_and_answers_do_not_count_twice_and_external_query_resets() {
    let current = key(ExerciseVariant::MultipleChoiceHanziMeaning, 0);
    let state = Rc::new(SessionState::default()).reduce(SessionAction::Ready(
        current.clone(),
        content(current.variant),
        1000,
    ));
    let state = state.reduce(SessionAction::McqSelect(current.clone(), 0));
    let duplicate = state
        .clone()
        .reduce(SessionAction::McqSelect(current.clone(), 1));
    assert!(Rc::ptr_eq(&state, &duplicate));
    let duplicate = state.clone().reduce(SessionAction::Ready(
        current.clone(),
        content(current.variant),
        2000,
    ));
    assert!(Rc::ptr_eq(&state, &duplicate));
    let changed = key(ExerciseVariant::MatchingHanziMeaning, 8);
    let reset = state.reduce(SessionAction::Ready(
        changed.clone(),
        content(changed.variant),
        3000,
    ));
    assert_eq!((reset.required, reset.active.stats().correct), (4, 0));
    assert_eq!(reset.active.stats().started_at, Some(3000));
    let stale = reset.clone().reduce(SessionAction::McqSelect(current, 0));
    assert!(Rc::ptr_eq(&reset, &stale));
}
