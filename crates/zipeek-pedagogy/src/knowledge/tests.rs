use super::*;

fn policy() -> ScoringPolicy {
    ScoringPolicy::default()
}

fn state(category: KnowledgeCategory, dimension: KnowledgeDimension) -> KnowledgeState {
    let entity = KnowledgeEntityId::new(category, "canonical-1").unwrap();
    KnowledgeState::new(entity, dimension, 1)
}

fn event(s: &KnowledgeState, day: i64, name: &str, outcome: AssessmentOutcome) -> AssessmentEvent {
    AssessmentEvent {
        attempt_id: format!("{day}-{name}"),
        policy_epoch: s.policy_epoch,
        entity: s.entity.clone(),
        dimension: s.dimension,
        policy_day: day,
        context_key: Some(format!("context-{name}")),
        reading_key: None,
        outcome,
    }
}

fn qualify(engine: &KnowledgeEngine, s: &mut KnowledgeState, day: i64) -> Transition {
    let quota = policy().daily_quota(s.debt);
    let mut result = Transition::Ignored;
    for n in 0..quota {
        let e = event(s, day, &format!("{n}"), AssessmentOutcome::Correct);
        result = engine.apply(s, e).unwrap();
    }
    result
}

#[test]
fn category_dimensions_are_exact_and_no_contextual_use() {
    assert_eq!(
        KnowledgeCategory::Pinyin.dimensions(),
        &[
            KnowledgeDimension::Reading,
            KnowledgeDimension::Listening,
            KnowledgeDimension::Speaking
        ]
    );
    assert_eq!(
        KnowledgeCategory::Hanzi.dimensions(),
        &[
            KnowledgeDimension::Recognition,
            KnowledgeDimension::Pronunciation,
            KnowledgeDimension::Handwriting
        ]
    );
    assert_eq!(
        KnowledgeCategory::Vocabulary.dimensions(),
        &[
            KnowledgeDimension::Recognition,
            KnowledgeDimension::Listening,
            KnowledgeDimension::Speaking,
            KnowledgeDimension::Writing
        ]
    );
    assert_eq!(
        KnowledgeCategory::Grammar.dimensions(),
        &[KnowledgeDimension::GrammarPattern]
    );
}

#[test]
fn full_entity_scores_exactly_one_for_all_categories() {
    for category in [
        KnowledgeCategory::Pinyin,
        KnowledgeCategory::Hanzi,
        KnowledgeCategory::Vocabulary,
        KnowledgeCategory::Grammar,
    ] {
        let states: Vec<_> = category
            .dimensions()
            .iter()
            .map(|&d| {
                let mut s = state(category, d);
                s.chunks = 7;
                s.phase = RelapsePhase::Mastered;
                s
            })
            .collect();
        assert_eq!(score_entity(&states, policy()).unwrap().as_f64(), 1.0);
        let mut incomplete = states.clone();
        incomplete[0].chunks -= 1;
        assert!(score_entity(&incomplete, policy()).unwrap().as_f64() < 1.0);
        assert!(score_entity(&states[..states.len() - 1], policy()).is_none());
    }
}

#[test]
fn catalogue_normalizes_categories_and_rejects_duplicate_ids() {
    let mut owned = Vec::new();
    for category in [
        KnowledgeCategory::Pinyin,
        KnowledgeCategory::Hanzi,
        KnowledgeCategory::Vocabulary,
        KnowledgeCategory::Grammar,
    ] {
        let states: Vec<_> = category
            .dimensions()
            .iter()
            .map(|&d| {
                let mut s = state(category, d);
                s.chunks = 7;
                s.phase = RelapsePhase::Mastered;
                s
            })
            .collect();
        owned.push((states[0].entity.clone(), states));
    }
    let references: Vec<_> = owned
        .iter()
        .map(|(id, states)| (id.clone(), states.as_slice()))
        .collect();
    assert_eq!(
        score_catalogue(references.clone(), policy())
            .unwrap()
            .as_f64(),
        1.0
    );
    assert!(score_catalogue([references[0].clone(), references[0].clone()], policy()).is_none());
}

#[test]
fn failure_before_qualification_disqualifies_without_early_debt() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(
        KnowledgeCategory::Vocabulary,
        KnowledgeDimension::Recognition,
    );
    let e = event(&s, 1, "fail", AssessmentOutcome::Incorrect);
    assert_eq!(
        engine.apply(&mut s, e).unwrap(),
        Transition::DayDisqualified
    );
    qualify(&engine, &mut s, 1);
    assert_eq!(s.chunks, 0);
    qualify(&engine, &mut s, 2);
    assert_eq!(s.chunks, 1);
}

#[test]
fn locked_quota_and_late_failure_allow_debt_without_revoking_day() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(
        KnowledgeCategory::Vocabulary,
        KnowledgeDimension::Recognition,
    );
    for day in 1..=3 {
        qualify(&engine, &mut s, day);
    }
    let fail = event(&s, 4, "fail", AssessmentOutcome::Incorrect);
    assert_eq!(engine.apply(&mut s, fail).unwrap(), Transition::DebtAdded);
    let early_quota = s.day.as_ref().unwrap().required_successes;
    assert_eq!(early_quota, 10);
    let another = event(&s, 4, "second", AssessmentOutcome::Incorrect);
    engine.apply(&mut s, another).unwrap();
    assert_eq!(s.debt, 1);
    assert_eq!(s.day.as_ref().unwrap().required_successes, 10);
    assert_eq!(qualify(&engine, &mut s, 5), Transition::DebtRepaid);
    assert_eq!(s.day.as_ref().unwrap().required_successes, 12);
    assert_eq!(s.debt, 0);
    let earned = s.chunks;
    let late = event(&s, 5, "late", AssessmentOutcome::Incorrect);
    assert_eq!(engine.apply(&mut s, late).unwrap(), Transition::DebtAdded);
    assert_eq!(s.chunks, earned);
    assert_eq!(s.day.as_ref().unwrap().required_successes, 12);
    assert_eq!(s.debt, 1);
}

#[test]
fn relapse_removes_only_one_chunk_and_requires_five_recovery_days() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(
        KnowledgeCategory::Grammar,
        KnowledgeDimension::GrammarPattern,
    );
    for day in 1..=7 {
        qualify(&engine, &mut s, day);
    }
    let fail = event(&s, 8, "fail", AssessmentOutcome::Incorrect);
    assert_eq!(
        engine.apply(&mut s, fail).unwrap(),
        Transition::RelapseStarted
    );
    assert_eq!(
        (s.chunks, s.debt, s.phase),
        (6, 5, RelapsePhase::Recovering)
    );
    let again = event(&s, 8, "again", AssessmentOutcome::Incorrect);
    engine.apply(&mut s, again).unwrap();
    assert_eq!(s.chunks, 6);
    for day in 9..13 {
        assert_eq!(qualify(&engine, &mut s, day), Transition::DebtRepaid);
        assert_eq!(s.chunks, 6);
    }
    assert_eq!(qualify(&engine, &mut s, 13), Transition::RelapseRecovered);
    assert_eq!((s.chunks, s.debt, s.phase), (7, 0, RelapsePhase::Mastered));
}

#[test]
fn replay_and_distinct_contexts_are_enforced_and_observations_retained() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(KnowledgeCategory::Pinyin, KnowledgeDimension::Listening);
    let first = event(&s, 1, "one", AssessmentOutcome::Correct);
    assert_eq!(
        engine.apply(&mut s, first.clone()).unwrap(),
        Transition::CountedSuccess
    );
    assert_eq!(
        engine.apply(&mut s, first).unwrap(),
        Transition::DuplicateAttempt
    );
    let mut second = event(&s, 1, "two", AssessmentOutcome::Correct);
    second.context_key = Some("context-one".to_owned());
    assert_eq!(
        engine.apply(&mut s, second).unwrap(),
        Transition::IneligibleContext
    );
    assert_eq!(s.day.as_ref().unwrap().successes, 1);
    assert_eq!(s.observations.len(), 2);
}

#[test]
fn uncertain_invalid_and_evaluator_failure_never_change_score() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(KnowledgeCategory::Vocabulary, KnowledgeDimension::Writing);
    for outcome in [
        AssessmentOutcome::Uncertain,
        AssessmentOutcome::NotAssessed,
        AssessmentOutcome::Invalid,
        AssessmentOutcome::EvaluationFailure,
    ] {
        let e = event(&s, 1, &format!("{outcome:?}"), outcome);
        assert_eq!(engine.apply(&mut s, e).unwrap(), Transition::Ignored);
    }
    assert_eq!((s.chunks, s.debt), (0, 0));
    assert!(!s.day.as_ref().unwrap().disqualified);
}

#[test]
fn invalid_dimension_and_reading_are_rejected_without_mutation() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(KnowledgeCategory::Grammar, KnowledgeDimension::Listening);
    let before = s.clone();
    let e = event(&s, 1, "x", AssessmentOutcome::Correct);
    assert_eq!(
        engine.apply(&mut s, e),
        Err(KnowledgeError::InvalidDimension)
    );
    assert_eq!(s, before);
    let mut s = state(KnowledgeCategory::Hanzi, KnowledgeDimension::Pronunciation)
        .with_required_readings(["xing2".to_owned(), "hang2".to_owned()]);
    let before = s.clone();
    let e = event(&s, 1, "x", AssessmentOutcome::Correct);
    assert_eq!(engine.apply(&mut s, e), Err(KnowledgeError::InvalidReading));
    assert_eq!(s, before);
}

#[test]
fn balanced_readings_require_all_readings_not_just_total() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(KnowledgeCategory::Hanzi, KnowledgeDimension::Pronunciation)
        .with_required_readings(["xing2".to_owned(), "hang2".to_owned(), "other".to_owned()]);
    for n in 0..10 {
        let mut e = event(&s, 1, &format!("a{n}"), AssessmentOutcome::Correct);
        e.reading_key = Some(
            if n < 8 {
                "xing2"
            } else if n == 8 {
                "hang2"
            } else {
                "other"
            }
            .into(),
        );
        engine.apply(&mut s, e).unwrap();
    }
    assert_eq!(s.chunks, 0);
    for n in 0..4 {
        let mut e = event(&s, 1, &format!("b{n}"), AssessmentOutcome::Correct);
        e.reading_key = Some(if n < 2 { "hang2" } else { "other" }.into());
        engine.apply(&mut s, e).unwrap();
    }
    assert_eq!(s.chunks, 1);
    assert!(s.day.as_ref().unwrap().qualified);
}

#[test]
fn epoch_and_out_of_order_are_rejected() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(KnowledgeCategory::Vocabulary, KnowledgeDimension::Listening);
    let e = event(&s, 5, "start", AssessmentOutcome::Correct);
    engine.apply(&mut s, e).unwrap();
    let e = event(&s, 4, "past", AssessmentOutcome::Correct);
    assert_eq!(
        engine.apply(&mut s, e),
        Err(KnowledgeError::EarlierPolicyDay)
    );
    let mut e = event(&s, 5, "epoch", AssessmentOutcome::Correct);
    e.policy_epoch = 2;
    assert_eq!(
        engine.apply(&mut s, e),
        Err(KnowledgeError::DifferentPolicyEpoch)
    );
}

#[test]
fn comeback_at_one_hundred_valid_streak_successes_restores_day() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(
        KnowledgeCategory::Vocabulary,
        KnowledgeDimension::Recognition,
    );
    let fail = event(&s, 1, "mistake", AssessmentOutcome::Incorrect);
    assert_eq!(
        engine.apply(&mut s, fail).unwrap(),
        Transition::DayDisqualified
    );
    for n in 0..99 {
        let success = event(&s, 1, &format!("comeback-{n}"), AssessmentOutcome::Correct);
        assert_eq!(
            engine.apply(&mut s, success).unwrap(),
            Transition::CountedSuccess
        );
    }
    assert!(s.day.as_ref().unwrap().disqualified);
    assert_eq!(s.chunks, 0);
    let success = event(&s, 1, "comeback-99", AssessmentOutcome::Correct);
    assert_eq!(engine.apply(&mut s, success).unwrap(), Transition::Comeback);
    assert!(s.day.as_ref().unwrap().qualified);
    assert!(s.day.as_ref().unwrap().comeback_triggered);
    assert_eq!(s.chunks, 1);
    let late = event(&s, 1, "late", AssessmentOutcome::Incorrect);
    engine.apply(&mut s, late).unwrap();
    assert_eq!(s.chunks, 1);
}

#[test]
fn comeback_streak_resets_after_incorrect_and_does_not_erase_debt() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(
        KnowledgeCategory::Vocabulary,
        KnowledgeDimension::Recognition,
    );
    for d in 1..=3 {
        qualify(&engine, &mut s, d);
    }
    let fail = event(&s, 4, "mistake", AssessmentOutcome::Incorrect);
    assert_eq!(engine.apply(&mut s, fail).unwrap(), Transition::DebtAdded);
    for n in 0..60 {
        let success = event(&s, 4, &format!("first-{n}"), AssessmentOutcome::Correct);
        engine.apply(&mut s, success).unwrap();
    }
    assert_eq!(s.day.as_ref().unwrap().recovery_streak, 60);
    let second = event(&s, 4, "second-mistake", AssessmentOutcome::Incorrect);
    engine.apply(&mut s, second).unwrap();
    assert_eq!(s.day.as_ref().unwrap().recovery_streak, 0);
    for n in 0..100 {
        let success = event(&s, 4, &format!("second-{n}"), AssessmentOutcome::Correct);
        let result = engine.apply(&mut s, success).unwrap();
        if n == 99 {
            assert_eq!(result, Transition::Comeback);
        }
    }
    assert_eq!(s.chunks, 4);
    assert_eq!(s.debt, 0);
    assert_eq!(s.day.as_ref().unwrap().required_successes, 10);
}

#[test]
fn comeback_cannot_override_severe_mastery_relapse() {
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let mut s = state(
        KnowledgeCategory::Grammar,
        KnowledgeDimension::GrammarPattern,
    );
    for d in 1..=7 {
        qualify(&engine, &mut s, d);
    }
    let fail = event(&s, 8, "relapse", AssessmentOutcome::Incorrect);
    assert_eq!(
        engine.apply(&mut s, fail).unwrap(),
        Transition::RelapseStarted
    );
    for n in 0..110 {
        let success = event(
            &s,
            8,
            &format!("after-relapse-{n}"),
            AssessmentOutcome::Correct,
        );
        assert_ne!(engine.apply(&mut s, success).unwrap(), Transition::Comeback);
    }
    assert_eq!((s.chunks, s.debt), (6, 5));
    assert!(s.day.as_ref().unwrap().disqualified);
}

#[test]
fn evidence_validator_rejects_unauthorized_assisted_and_duplicate_targets() {
    use std::collections::{BTreeMap, BTreeSet};
    let s = state(KnowledgeCategory::Hanzi, KnowledgeDimension::Handwriting);
    let target = EvidenceTarget {
        entity: s.entity.clone(),
        dimension: s.dimension,
    };
    let mut manifest = ExerciseManifest {
        attempt_id: "attempt-1".into(),
        policy_epoch: 1,
        policy_day: 1,
        context_family: "draw-cat-from-meaning".into(),
        targets: BTreeMap::from([(
            target.clone(),
            AuthorizedTarget {
                modality: EvidenceModality::HandDrawn,
                requires_independent_production: true,
                required_readings: BTreeSet::new(),
            },
        )]),
    };
    let observation = DiagnosticObservation {
        target: target.clone(),
        outcome: AssessmentOutcome::Correct,
        modality: EvidenceModality::HandDrawn,
        independently_demonstrated: true,
        response_was_cued: false,
        reading_key: None,
    };
    assert_eq!(
        validate_evidence(&manifest, &[observation.clone()])
            .unwrap()
            .len(),
        1
    );
    let mut assisted = observation.clone();
    assisted.response_was_cued = true;
    assert_eq!(
        validate_evidence(&manifest, &[assisted]),
        Err(EvidenceValidationError::AssistedResponse)
    );
    assert_eq!(
        validate_evidence(&manifest, &[observation.clone(), observation.clone()]),
        Err(EvidenceValidationError::DuplicateTarget)
    );
    let mut unauthorized = observation.clone();
    unauthorized.target.dimension = KnowledgeDimension::Recognition;
    assert_eq!(
        validate_evidence(&manifest, &[unauthorized]),
        Err(EvidenceValidationError::UnauthorizedTarget)
    );
    manifest.attempt_id.clear();
    assert_eq!(
        validate_evidence(&manifest, &[observation]),
        Err(EvidenceValidationError::InvalidManifest)
    );
}

#[test]
fn ledger_batch_is_atomic_and_replayed_attempt_is_noop() {
    use std::collections::BTreeMap;
    let engine = KnowledgeEngine::new(policy()).unwrap();
    let s = state(
        KnowledgeCategory::Vocabulary,
        KnowledgeDimension::Recognition,
    );
    let t = EvidenceTarget {
        entity: s.entity.clone(),
        dimension: s.dimension,
    };
    let mut ledger = KnowledgeLedger {
        states: BTreeMap::from([(t.clone(), s.clone())]),
        ..Default::default()
    };
    let first = event(&s, 1, "one", AssessmentOutcome::Correct);
    let mut invalid = event(&s, 1, "one", AssessmentOutcome::Correct);
    invalid.dimension = KnowledgeDimension::Writing;
    let before = ledger.clone();
    assert_eq!(
        ledger.apply_validated_batch(&engine, vec![first.clone(), invalid]),
        Err(LedgerError::MissingTarget)
    );
    assert_eq!(ledger, before);
    assert!(matches!(
        ledger.apply_validated_batch(&engine, vec![first.clone()]),
        Ok(LedgerOutcome::Applied(_))
    ));
    let after = ledger.clone();
    assert_eq!(
        ledger.apply_validated_batch(&engine, vec![first]),
        Ok(LedgerOutcome::DuplicateAttempt)
    );
    assert_eq!(ledger, after);
}

#[test]
fn invalid_modality_never_becomes_hanzi_handwriting() {
    use std::collections::{BTreeMap, BTreeSet};
    let s = state(KnowledgeCategory::Hanzi, KnowledgeDimension::Handwriting);
    let target = EvidenceTarget {
        entity: s.entity.clone(),
        dimension: s.dimension,
    };
    let manifest = ExerciseManifest {
        attempt_id: "ime-attempt".into(),
        policy_epoch: 1,
        policy_day: 1,
        context_family: "enter-character".into(),
        targets: BTreeMap::from([(
            target.clone(),
            AuthorizedTarget {
                modality: EvidenceModality::FreeTextChinese,
                requires_independent_production: true,
                required_readings: BTreeSet::new(),
            },
        )]),
    };
    let report = DiagnosticObservation {
        target,
        modality: EvidenceModality::FreeTextChinese,
        independently_demonstrated: true,
        response_was_cued: false,
        reading_key: None,
        outcome: AssessmentOutcome::Correct,
    };
    assert_eq!(
        validate_evidence(&manifest, &[report]),
        Err(EvidenceValidationError::WrongModality)
    );
}

#[test]
fn technical_or_uncertain_reports_produce_no_score_events() {
    use std::collections::{BTreeMap, BTreeSet};
    let s = state(
        KnowledgeCategory::Vocabulary,
        KnowledgeDimension::Recognition,
    );
    let target = EvidenceTarget {
        entity: s.entity.clone(),
        dimension: s.dimension,
    };
    let manifest = ExerciseManifest {
        attempt_id: "retryable".into(),
        policy_epoch: 1,
        policy_day: 1,
        context_family: "original".into(),
        targets: BTreeMap::from([(
            target.clone(),
            AuthorizedTarget {
                modality: EvidenceModality::WrittenChinese,
                requires_independent_production: false,
                required_readings: BTreeSet::new(),
            },
        )]),
    };
    for outcome in [
        AssessmentOutcome::EvaluationFailure,
        AssessmentOutcome::Uncertain,
        AssessmentOutcome::Invalid,
        AssessmentOutcome::NotAssessed,
    ] {
        let report = DiagnosticObservation {
            target: target.clone(),
            outcome,
            modality: EvidenceModality::WrittenChinese,
            independently_demonstrated: false,
            response_was_cued: false,
            reading_key: None,
        };
        assert!(validate_evidence(&manifest, &[report]).unwrap().is_empty());
    }
}
