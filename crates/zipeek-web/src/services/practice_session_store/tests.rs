use super::*;
use crate::models::{AnswerPolicy, AudioPreference};

fn valid_translation_instance() -> String {
    serde_json::json!({
        "kind": "sentence_translation",
        "sentence": {
            "direction": "zh-to-en",
            "chinese": "我喝水",
            "english": "I drink water",
            "target_words": ["喝水"],
            "word_bank": ["water", "drink", "I"],
            "solution": ["I", "drink", "water"]
        }
    })
    .to_string()
}

fn session() -> PracticeSession {
    PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174000".to_owned(),
        &["matching".to_owned(), "sentence-translation".to_owned()]
            .into_iter()
            .collect(),
        5,
        42,
        BTreeMap::from([("set_ids".to_owned(), "1,2".to_owned())]),
    )
    .unwrap()
}

#[test]
fn generation_transitions_reject_unknown_and_completed_steps_without_mutation() {
    let mut saved = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174190".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        2,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let first = saved.current_step().unwrap().id.clone();
    let unknown = format!("{}:999", saved.id);
    let initial = saved.clone();
    assert_eq!(
        saved.start_generation(&unknown),
        Err(PracticeSessionError::WrongSession)
    );
    assert_eq!(
        saved.generation_succeeded(&unknown, 1, "{}".to_owned()),
        Err(PracticeSessionError::WrongSession)
    );
    assert_eq!(
        saved.generation_failed(&unknown, 1, "network".to_owned()),
        Err(PracticeSessionError::WrongSession)
    );
    assert_eq!(
        saved.retry_generation(&unknown),
        Err(PracticeSessionError::WrongSession)
    );
    assert_eq!(saved, initial);

    saved.skip_current().unwrap();
    let after_skip = saved.clone();
    assert_eq!(
        saved.start_generation(&first),
        Err(PracticeSessionError::AlreadyFinished)
    );
    assert_eq!(
        saved.generation_succeeded(&first, 1, "{}".to_owned()),
        Err(PracticeSessionError::AlreadyFinished)
    );
    assert_eq!(
        saved.generation_failed(&first, 1, "network".to_owned()),
        Err(PracticeSessionError::AlreadyFinished)
    );
    assert_eq!(
        saved.retry_generation(&first),
        Err(PracticeSessionError::AlreadyFinished)
    );
    assert_eq!(saved, after_skip);
}

#[test]
fn stored_session_validation_rejects_corrupt_completion_and_generation_metadata() {
    let original = session();
    let mut corrupted = original.clone();
    corrupted.completed_at = Some(corrupted.created_at);
    assert_eq!(
        corrupted.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );

    let mut corrupted = original.clone();
    corrupted.revision = u64::MAX;
    assert_eq!(
        corrupted.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );

    let mut corrupted = original.clone();
    corrupted.steps[0].generation_attempt = 4;
    assert_eq!(
        corrupted.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );

    let mut corrupted = original.clone();
    corrupted.steps[0].id = "different".to_owned();
    assert_eq!(
        corrupted.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn session_serialization_retains_shared_configuration_and_step_references() {
    let mut saved = session();
    let config = saved.exercise_configurations.get_mut("matching").unwrap();
    if let ExerciseConfiguration::Quiz {
        reverse_matching, ..
    } = config
    {
        *reverse_matching = true;
    }
    let raw = serde_json::to_string(&saved).unwrap();
    let loaded = decode_stored_session(&saved.id, &raw).unwrap();
    assert_eq!(
        loaded.exercise_configurations,
        saved.exercise_configurations
    );
    assert!(loaded.steps.iter().all(|step| {
        loaded
            .exercise_configurations
            .contains_key(&step.configuration_id)
    }));
    assert_eq!(
        raw.matches("\"reverse_matching\"").count(),
        1 + saved
            .steps
            .iter()
            .filter(|step| step.exercise_id == "matching")
            .count()
    );
}

#[test]
fn matching_configuration_count_is_validated_and_round_trips() {
    let mut saved = session();
    if let Some(ExerciseConfiguration::Quiz { pair_count, .. }) =
        saved.exercise_configurations.get_mut("matching")
    {
        *pair_count = 7;
    }
    saved.validate().unwrap();
    let raw = serde_json::to_string(&saved).unwrap();
    assert_eq!(
        decode_stored_session(&saved.id, &raw)
            .unwrap()
            .exercise_configurations,
        saved.exercise_configurations
    );
    if let Some(ExerciseConfiguration::Quiz { pair_count, .. }) =
        saved.exercise_configurations.get_mut("matching")
    {
        *pair_count = 8;
    }
    assert_eq!(
        saved.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn new_session_requires_empty_valid_inventory() {
    assert_eq!(
        ensure_new_session_allowed(&PracticeSessionInventory::default()),
        Ok(())
    );
    let mut inventory = PracticeSessionInventory::default();
    inventory.sessions.push(session());
    assert_eq!(
        ensure_new_session_allowed(&inventory),
        Err(PracticeSessionError::ExistingSession)
    );
    inventory.sessions.clear();
    inventory.invalid_records = 1;
    assert_eq!(
        ensure_new_session_allowed(&inventory),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn inventory_decoder_preserves_completed_sessions_for_history_retry() {
    let mut completed = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174155".to_owned(),
        &["type-pinyin".to_owned()].into_iter().collect(),
        1,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let step_id = completed.current_step().unwrap().id.clone();
    completed
        .complete(&step_id, ExerciseOutcome::Completed)
        .unwrap();
    completed
        .stamp_completed_at(
            (completed.created_at + chrono::Duration::seconds(10)).timestamp_millis() as u64,
        )
        .unwrap();
    let raw = serde_json::to_string(&completed).unwrap();
    let restored = decode_stored_session(&completed.id, &raw).unwrap();
    assert!(restored.is_completed());
    assert_eq!(restored.completed_at, completed.completed_at);
    assert_eq!(
        decode_stored_session(&completed.id, "{"),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    assert_eq!(
        decode_stored_session("123e4567-e89b-72d3-a456-426614174156", &raw),
        Err(PracticeSessionError::WrongSession)
    );
}

#[test]
fn translation_scores_and_direction_survive_serialization() {
    let mut saved = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174101".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        5,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    if let Some(ExerciseConfiguration::SentenceTranslation { direction }) = saved
        .exercise_configurations
        .get_mut("sentence-translation")
    {
        *direction = "en-to-zh".to_owned();
    }
    let id = saved.current_step().unwrap().id.clone();
    let attempt = saved.start_generation(&id).unwrap();
    saved
        .generation_succeeded(&id, attempt, valid_translation_instance())
        .unwrap();
    saved
        .complete(
            &id,
            ExerciseOutcome::Scored {
                earned: 1,
                possible: 1,
            },
        )
        .unwrap();
    let decoded: PracticeSession =
        serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded.current, 1);
    assert_eq!(decoded.translation_direction(), "en-to-zh");
    assert_eq!(
        decoded.steps[0].outcome,
        Some(ExerciseOutcome::Scored {
            earned: 1,
            possible: 1
        })
    );
    assert!(decoded.current_step().is_some());
    let mut corrupt = decoded.clone();
    if let Some(ExerciseConfiguration::SentenceTranslation { direction }) = corrupt
        .exercise_configurations
        .get_mut("sentence-translation")
    {
        *direction = "unexpected".to_owned();
    }
    assert_eq!(
        corrupt.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn endless_translation_appends_after_scoring_and_skipping_and_can_finish() {
    let mut saved = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174102".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        2,
        91,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    saved.endless_translation = true;
    saved.validate().unwrap();
    assert_eq!(
        saved.finish_endless_translation(),
        Err(PracticeSessionError::NotReady)
    );
    let first = saved.current_step().unwrap().id.clone();
    let attempt = saved.start_generation(&first).unwrap();
    saved
        .generation_succeeded(&first, attempt, valid_translation_instance())
        .unwrap();
    saved
        .complete(
            &first,
            ExerciseOutcome::Scored {
                earned: 0,
                possible: 1,
            },
        )
        .unwrap();
    assert_eq!(saved.current, 1);
    assert_eq!(saved.steps.len(), 3);
    saved.skip_current().unwrap();
    assert_eq!(saved.current, 2);
    assert_eq!(saved.steps.len(), 4);
    saved.validate().unwrap();
    let restored: PracticeSession =
        serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
    assert_eq!(
        restored.current_step().unwrap().id,
        saved.current_step().unwrap().id
    );
    saved.finish_endless_translation().unwrap();
    assert!(saved.is_completed());
    assert!(!saved.endless_translation);
    saved.validate().unwrap();
    assert_eq!(saved.steps.len(), 2);
}

#[test]
fn configured_formats_are_persistable_and_reject_cross_mechanic_variants() {
    let mut saved = session();
    saved
        .configure_quiz_formats(&[ExerciseVariant::MatchingPinyinHanzi], &[], true)
        .unwrap();
    for step in saved
        .steps
        .iter()
        .filter(|step| step.exercise_id == "matching")
    {
        assert_eq!(step.quiz_format.as_deref(), Some("pinyin-hanzi"));
        assert!(step.reverse_matching);
    }
    assert_eq!(saved.validate(), Ok(()));
    saved
        .steps
        .iter_mut()
        .find(|step| step.exercise_id == "matching")
        .unwrap()
        .quiz_format = Some("meaning-to-hanzi".into());
    assert_eq!(
        saved.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn invalid_configuration_does_not_modify_an_existing_session() {
    let mut saved = session();
    let original = saved.clone();
    assert!(
        saved
            .configure_quiz_formats(&[ExerciseVariant::MultipleChoiceHanziMeaning], &[], false)
            .is_err()
    );
    assert_eq!(saved, original);
}

#[test]
fn skipped_generation_is_not_pending_and_cannot_be_started() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174009".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        2,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let first = session.current_step().unwrap().id.clone();
    assert!(session.pending_generation().any(|step| step.id == first));
    session.skip_current().unwrap();
    assert!(session.pending_generation().all(|step| step.id != first));
    assert_eq!(
        session.start_generation(&first),
        Err(PracticeSessionError::AlreadyFinished)
    );
    session.validate().unwrap();
}

#[test]
fn stale_generation_results_cannot_mutate_skipped_steps() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174010".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        1,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let id = session.current_step().unwrap().id.clone();
    let attempt = session.start_generation(&id).unwrap();
    session.skip_current().unwrap();
    assert_eq!(
        session.generation_succeeded(&id, attempt, "{}".to_owned()),
        Err(PracticeSessionError::AlreadyFinished)
    );
    assert_eq!(
        session.generation_failed(&id, attempt, "connection error".to_owned()),
        Err(PracticeSessionError::AlreadyFinished)
    );
    session.validate().unwrap();
}

#[test]
fn authored_and_grammar_exercises_restore_as_unscored_local_steps() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174001".to_owned(),
        &[
            "fill-blank".to_owned(),
            "audio-dictation".to_owned(),
            "sentence-builder".to_owned(),
            "spot-mistake".to_owned(),
            "write-hanzi".to_owned(),
        ]
        .into_iter()
        .collect(),
        5,
        23,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    assert!(
        session
            .steps
            .iter()
            .all(|step| matches!(&step.readiness, ExerciseReadiness::Ready))
    );
    let current_id = session.current_step().unwrap().id.clone();
    session
        .complete(&current_id, ExerciseOutcome::Completed)
        .unwrap();
    let saved = serde_json::to_string(&session).unwrap();
    let restored: PracticeSession = serde_json::from_str(&saved).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.current, 1);
    assert!(matches!(
        &restored.steps[0].outcome,
        Some(ExerciseOutcome::Completed)
    ));
}

#[test]
fn character_hunt_is_now_a_playable_unscored_uuid_host() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174001".to_owned(),
        &["character-hunt".to_owned()].into_iter().collect(),
        1,
        23,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    session.validate().unwrap();
    let step = session.current_step().unwrap();
    assert!(matches!(step.readiness, ExerciseReadiness::Ready));
    let id = step.id.clone();
    session.complete(&id, ExerciseOutcome::Completed).unwrap();
    session.validate().unwrap();
    assert!(matches!(
        session.steps[0].outcome,
        Some(ExerciseOutcome::Completed)
    ));
}

#[test]
fn pronunciation_practice_can_advance_without_scoring() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174001".to_owned(),
        &["speaking".to_owned()].into_iter().collect(),
        1,
        23,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let step = session.current_step().unwrap();
    assert!(matches!(step.readiness, ExerciseReadiness::Ready));
    let id = step.id.clone();
    session.complete(&id, ExerciseOutcome::Completed).unwrap();
    assert!(session.is_completed());
    session.validate().unwrap();
}

#[test]
fn unverified_uuid_steps_never_accept_scores_even_from_saved_sessions() {
    for exercise_id in [
        "speaking",
        "spot-mistake",
        "write-hanzi",
        "listening-choice",
        "type-pinyin",
        "matching",
        "multiple-choice",
    ] {
        let mut session = PracticeSession::new(
            "123e4567-e89b-72d3-a456-426614174001".to_owned(),
            &[exercise_id.to_owned()].into_iter().collect(),
            1,
            23,
            BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
        )
        .unwrap();
        let step_id = session.current_step().unwrap().id.clone();
        assert_eq!(
            session.complete(
                &step_id,
                ExerciseOutcome::Scored {
                    earned: 1,
                    possible: 1
                }
            ),
            Err(PracticeSessionError::InvalidResult)
        );
        assert_eq!(session.current, 0);
        session.steps[0].outcome = Some(ExerciseOutcome::Scored {
            earned: 1,
            possible: 1,
        });
        session.current = 1;
        assert_eq!(
            session.validate(),
            Err(PracticeSessionError::InvalidStoredSession)
        );
    }
}

#[test]
fn translation_score_must_be_single_point_in_memory_and_on_restore() {
    for (earned, possible) in [(0, 0), (1, 2), (2, 2), (2, 1)] {
        let mut session = PracticeSession::new(
            "123e4567-e89b-72d3-a456-426614174001".to_owned(),
            &["sentence-translation".to_owned()].into_iter().collect(),
            1,
            23,
            BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
        )
        .unwrap();
        let step_id = session.current_step().unwrap().id.clone();
        assert_eq!(
            session.complete(&step_id, ExerciseOutcome::Scored { earned, possible }),
            Err(PracticeSessionError::InvalidResult)
        );
        assert_eq!(session.current, 0);
        session.steps[0].outcome = Some(ExerciseOutcome::Scored { earned, possible });
        session.current = 1;
        assert_eq!(
            session.validate(),
            Err(PracticeSessionError::InvalidStoredSession)
        );
    }
}

#[test]
fn restoring_an_interrupted_request_does_not_consume_an_unstarted_retry() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174020".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        1,
        5,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let id = session.current_step().unwrap().id.clone();
    let first = session.start_generation(&id).unwrap();
    assert_eq!(first, 1);
    assert!(session.recover_interrupted_generation());
    assert_eq!(session.current_step().unwrap().generation_attempt, 1);
    assert_eq!(session.start_generation(&id), Ok(2));
    assert!(session.recover_interrupted_generation());
    assert_eq!(session.start_generation(&id), Ok(3));
    assert!(session.recover_interrupted_generation());
    assert!(matches!(
        session.current_step().unwrap().readiness,
        ExerciseReadiness::Failed(_)
    ));
    session.validate().unwrap();
}

#[test]
fn corrupted_generation_statuses_cannot_be_restored() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174021".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        1,
        7,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    session.steps[0].readiness = ExerciseReadiness::Generating;
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    session.steps[0].generation_attempt = 1;
    session.validate().unwrap();
    session.steps[0].generation_attempt = 4;
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    session.steps[0].generation_attempt = 1;
    session.steps[0].readiness = ExerciseReadiness::Failed(String::new());
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn completed_removal_requires_exact_committed_revision_and_payload() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174090".to_owned(),
        &["matching".to_owned()].into_iter().collect(),
        1,
        4,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    assert_eq!(
        completed_removal_allowed(&session, &session),
        Err(PracticeSessionError::NotReady)
    );
    let step = session.current_step().unwrap().id.clone();
    session.complete(&step, ExerciseOutcome::Completed).unwrap();
    assert!(session.is_completed());
    assert_eq!(completed_removal_allowed(&session, &session), Ok(()));

    let mut previous_revision = session.clone();
    previous_revision.revision -= 1;
    assert_eq!(
        completed_removal_allowed(&previous_revision, &session),
        Err(PracticeSessionError::OutOfDate)
    );

    let mut altered_outcome = session.clone();
    altered_outcome.steps[0].outcome = Some(ExerciseOutcome::Skipped);
    assert_eq!(
        completed_removal_allowed(&altered_outcome, &session),
        Err(PracticeSessionError::OutOfDate)
    );
}

#[test]
fn persisted_session_round_trip_and_validation() {
    let session = session();
    let copy: PracticeSession =
        serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
    assert_eq!(session, copy);
    copy.validate().unwrap();
}

#[test]
fn storage_decoder_rejects_oversized_records_and_wrong_ids() {
    let session = session();
    let raw = serde_json::to_string(&session).unwrap();
    assert_eq!(
        decode_stored_session(&session.id, &" ".repeat(MAX_STORED_SESSION_BYTES + 1)),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    assert_eq!(
        decode_stored_session("123e4567-e89b-72d3-a456-426614174999", &raw),
        Err(PracticeSessionError::WrongSession)
    );
    assert_eq!(decode_stored_session(&session.id, &raw), Ok(session));
}

#[test]
fn completed_record_removal_rejects_changed_payload_and_incomplete_record() {
    let mut saved = session();
    assert_eq!(
        completed_removal_allowed(&saved, &saved),
        Err(PracticeSessionError::NotReady)
    );
    for step in &mut saved.steps {
        step.readiness = ExerciseReadiness::Ready;
        step.outcome = Some(ExerciseOutcome::Completed);
        step.started_at = Some(saved.created_at);
        step.finished_at = Some(saved.created_at);
    }
    saved.current = saved.steps.len();
    assert_eq!(completed_removal_allowed(&saved, &saved), Ok(()));
    let mut altered = saved.clone();
    altered.revision += 1;
    assert_eq!(
        completed_removal_allowed(&saved, &altered),
        Err(PracticeSessionError::OutOfDate)
    );
}

#[test]
fn answer_preferences_survive_session_serialization() {
    let mut saved = PracticeSession::new(
        uuid::Uuid::now_v7().to_string(),
        &["character-builder".to_owned(), "type-pinyin".to_owned()]
            .into_iter()
            .collect(),
        4,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    if let Some(ExerciseConfiguration::CharacterBuilder(options)) =
        saved.exercise_configurations.get_mut("character-builder")
    {
        options.answer_policy = AnswerPolicy::RetryUntilCorrect;
        options.audio = AudioPreference::Manual;
    }
    if let Some(ExerciseConfiguration::TypePinyin(options)) =
        saved.exercise_configurations.get_mut("type-pinyin")
    {
        options.answer_policy = AnswerPolicy::Advance;
    }
    let restored: PracticeSession =
        serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
    restored.validate().unwrap();
    let builder = restored.exercise_configurations["character-builder"]
        .vocabulary_options()
        .unwrap();
    let pinyin = restored.exercise_configurations["type-pinyin"]
        .vocabulary_options()
        .unwrap();
    assert!(builder.retry_incorrect());
    assert!(builder.show_audio());
    assert!(!pinyin.retry_incorrect());
}

#[test]
fn completion_requires_active_ready_step() {
    let mut session = session();
    let current = session.current_step().unwrap().clone();
    if matches!(&current.readiness, ExerciseReadiness::Ready) {
        session
            .complete(&current.id, ExerciseOutcome::Completed)
            .unwrap();
        assert_eq!(session.current, 1);
        assert_eq!(session.revision, 1);
        assert_eq!(
            session.complete(&current.id, ExerciseOutcome::Completed),
            Err(PracticeSessionError::WrongSession)
        );
    } else {
        assert_eq!(
            session.complete(&current.id, ExerciseOutcome::Completed),
            Err(PracticeSessionError::NotReady)
        );
    }
}

#[test]
fn stale_generation_results_cannot_override_new_attempts() {
    let mut session = session();
    let step_id = session
        .steps
        .iter()
        .find(|step| matches!(&step.readiness, ExerciseReadiness::Queued))
        .unwrap()
        .id
        .clone();
    let first = session.start_generation(&step_id).unwrap();
    session
        .generation_failed(&step_id, first, "offline".to_owned())
        .unwrap();
    session.retry_generation(&step_id).unwrap();
    let second = session.start_generation(&step_id).unwrap();
    assert_eq!(
        session.generation_succeeded(&step_id, first, "stale".into()),
        Err(PracticeSessionError::StaleGeneration)
    );
    session
        .generation_succeeded(&step_id, second, valid_translation_instance())
        .unwrap();
    assert!(
        session
            .steps
            .iter()
            .any(|step| step.generated_instance.is_some())
    );
}

#[test]
fn skipping_unavailable_generation_does_not_create_fake_scores() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174000".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        2,
        29,
        BTreeMap::from([("set_ids".to_owned(), "1,2".to_owned())]),
    )
    .unwrap();
    assert!(matches!(
        session.current_step().unwrap().readiness,
        ExerciseReadiness::Queued
    ));
    session.skip_current().unwrap();
    assert_eq!(session.current, 1);
    assert_eq!(session.steps[0].outcome, Some(ExerciseOutcome::Skipped));
    session.validate().unwrap();
}

#[test]
fn interrupted_generation_can_be_requeued_without_accepting_stale_results() {
    let mut session = session();
    let step_id = session.pending_generation().next().unwrap().id.clone();
    let previous = session.start_generation(&step_id).unwrap();
    assert!(session.recover_interrupted_generation());
    assert!(!session.recover_interrupted_generation());
    assert_eq!(
        session.generation_succeeded(&step_id, previous, "{}".into()),
        Err(PracticeSessionError::StaleGeneration)
    );
    assert!(session.pending_generation().any(|step| step.id == step_id));
}

#[test]
fn prefetch_is_bounded_and_duplicate_tasks_are_ignored() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174000".to_owned(),
        &["sentence-translation".to_owned()].into_iter().collect(),
        5,
        77,
        BTreeMap::from([("set_ids".to_owned(), "1,2".to_owned())]),
    )
    .unwrap();
    assert_eq!(session.pending_generation().count(), 2);
    let first = session.pending_generation().next().unwrap().id.clone();
    let attempt = session.start_generation(&first).unwrap();
    session.queue_generation(2);
    assert_eq!(session.pending_generation().count(), 1);
    assert!(
        session
            .generation_succeeded(&first, attempt, "not json".into())
            .is_err()
    );
    session
        .generation_succeeded(&first, attempt, valid_translation_instance())
        .unwrap();
    assert_eq!(session.pending_generation().count(), 1);
}

#[test]
fn malformed_session_vocabulary_and_generated_payload_fail_closed() {
    let mut session = session();
    session.vocabulary.clear();
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    session
        .vocabulary
        .insert("set_ids".to_owned(), "1,2".to_owned());
    session
        .vocabulary
        .insert("source".to_owned(), "ignored".to_owned());
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    session.vocabulary.remove("source");
    session.validate().unwrap();
}

#[test]
fn invalid_score_and_ids_fail_closed() {
    assert!(!valid_session_id("not-a-uuid"));
    assert!(session_storage_key("not-a-uuid").is_err());
    assert_eq!(
        session_storage_key("123e4567-e89b-72d3-a456-426614174000").unwrap(),
        "zipeek.practice.session.v1.123e4567-e89b-72d3-a456-426614174000"
    );
    let mut session = session();
    session.steps[0].id = "bad".into();
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}
#[test]
fn character_hunt_difficulty_is_validated_with_its_host() {
    let mut session = session();
    session
        .vocabulary
        .insert("difficulty".into(), "hardcore".into());
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    session.steps[0].exercise_id = "character-hunt".into();
    assert!(session.validate().is_ok());
    session
        .vocabulary
        .insert("difficulty".into(), "invalid".into());
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn locally_prepared_steps_cannot_be_restored_with_forged_generation_history() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174090".to_owned(),
        &["matching".to_owned()].into_iter().collect(),
        1,
        4,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    session.validate().unwrap();
    session.steps[0].generation_attempt = 1;
    assert_eq!(
        session.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    session.steps[0].generation_attempt = 0;
    session.validate().unwrap();
}

#[test]
fn restored_session_rejects_revision_overflow_before_any_mutation() {
    let mut saved = session();
    saved.revision = u64::MAX;
    assert_eq!(
        saved.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    saved.revision = u64::MAX - 1;
    assert_eq!(saved.validate(), Ok(()));
}

#[test]
fn session_ids_require_canonical_uuid_v7() {
    assert!(valid_session_id(&uuid::Uuid::now_v7().to_string()));
    assert!(!valid_session_id("123e4567-e89b-42d3-a456-426614174000"));
    assert!(!valid_session_id("../../practice"));
}

#[test]
fn shared_configurations_are_referenced_not_copied_into_steps() {
    let saved = PracticeSession::new(
        uuid::Uuid::now_v7().to_string(),
        &["type-pinyin".to_owned()].into_iter().collect(),
        20,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    assert_eq!(saved.exercise_configurations.len(), 1);
    assert_eq!(saved.steps.len(), 20);
    assert!(
        saved
            .steps
            .iter()
            .all(|step| step.configuration_id == "type-pinyin")
    );
    let json = serde_json::to_value(&saved).unwrap();
    assert!(json["exercise_configurations"]["type-pinyin"].is_object());
    assert!(
        json["steps"]
            .as_array()
            .unwrap()
            .iter()
            .all(|step| step.get("settings").is_none())
    );
    saved.validate().unwrap();
}

#[test]
fn step_start_and_finish_times_round_trip_as_iso_datetimes() {
    let mut saved = PracticeSession::new(
        uuid::Uuid::now_v7().to_string(),
        &["type-pinyin".to_owned()].into_iter().collect(),
        1,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    assert!(saved.start_current().unwrap());
    assert!(!saved.start_current().unwrap());
    let step_id = saved.steps[0].id.clone();
    saved
        .complete(&step_id, ExerciseOutcome::Completed)
        .unwrap();
    saved
        .stamp_completed_at(
            (saved.created_at + chrono::Duration::seconds(1)).timestamp_millis() as u64,
        )
        .unwrap();
    let json = serde_json::to_value(&saved).unwrap();
    assert!(json["created_at"].as_str().unwrap().contains('T'));
    assert!(json["completed_at"].as_str().unwrap().contains('T'));
    assert!(
        json["steps"][0]["started_at"]
            .as_str()
            .unwrap()
            .contains('T')
    );
    assert!(
        json["steps"][0]["finished_at"]
            .as_str()
            .unwrap()
            .contains('T')
    );
    let restored: PracticeSession = serde_json::from_value(json).unwrap();
    assert_eq!(restored, saved);
    restored.validate().unwrap();
}

#[test]
fn invalid_step_configuration_reference_fails_closed() {
    let mut saved = session();
    saved.steps[0].configuration_id = "unregistered-variant".to_owned();
    assert_eq!(
        saved.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[test]
fn write_hanzi_configuration_is_part_of_validated_session() {
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174001".to_owned(),
        &["write-hanzi".to_owned()].into_iter().collect(),
        1,
        23,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let crate::services::practice_session_store::ExerciseConfiguration::WriteHanzi(options) =
        session
            .exercise_configurations
            .get_mut("write-hanzi")
            .unwrap()
    else {
        panic!("Write Hanzi must use its own configuration");
    };
    options.guidance = crate::models::WritingGuidance::FullGuide;
    options.animation = false;
    options.show_pinyin = false;
    options.show_meaning = true;
    options.playable_audio = false;
    options.assistance = crate::models::StrokeAssistance::AfterFive;
    session.validate().unwrap();
    let restored: PracticeSession =
        serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
    assert_eq!(
        restored.exercise_configurations,
        session.exercise_configurations
    );
}

#[test]
fn listening_configuration_is_validated_and_round_trips() {
    use crate::models::ListeningHintVisibility;
    let mut session = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174002".to_owned(),
        &["listening-choice".to_owned()].into_iter().collect(),
        1,
        29,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let ExerciseConfiguration::ListeningChoice(options) = session
        .exercise_configurations
        .get_mut("listening-choice")
        .unwrap()
    else {
        panic!("Listening Choice must have a typed configuration");
    };
    options.pinyin = ListeningHintVisibility::Always;
    options.meaning = ListeningHintVisibility::Never;
    options.autoplay = true;
    session.validate().unwrap();
    let restored: PracticeSession =
        serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
    assert_eq!(
        restored.exercise_configurations,
        session.exercise_configurations
    );
}

#[test]
fn stored_session_rejects_inverted_completion_and_step_timestamps() {
    let mut completed = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174176".to_owned(),
        &["type-pinyin".to_owned()].into_iter().collect(),
        1,
        42,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    )
    .unwrap();
    let step_id = completed.current_step().unwrap().id.clone();
    completed
        .complete(&step_id, ExerciseOutcome::Completed)
        .unwrap();
    assert!(completed.validate().is_ok());
    let start = completed.started_at.unwrap();
    completed.completed_at = Some(start - chrono::Duration::seconds(1));
    assert_eq!(
        completed.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
    completed.completed_at = None;
    completed.steps[0].finished_at = Some(completed.created_at - chrono::Duration::seconds(1));
    assert_eq!(
        completed.validate(),
        Err(PracticeSessionError::InvalidStoredSession)
    );
}

#[cfg(test)]
mod completion_timestamp_tests {
    use super::*;

    #[test]
    fn timestamp_is_once_only_and_survives_roundtrip() {
        let mut session = PracticeSession::new(
            "123e4567-e89b-72d3-a456-426614174001".to_owned(),
            &["type-pinyin".to_owned()].into_iter().collect(),
            1,
            27,
            BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
        )
        .unwrap();
        assert_eq!(
            session.stamp_completed_at(1_700_000_000_000),
            Err(PracticeSessionError::NotReady)
        );
        let id = session.current_step().unwrap().id.clone();
        session.complete(&id, ExerciseOutcome::Completed).unwrap();
        assert_eq!(
            session.stamp_completed_at(0),
            Err(PracticeSessionError::NotReady)
        );
        session
            .stamp_completed_at(
                (session.created_at + chrono::Duration::seconds(10)).timestamp_millis() as u64,
            )
            .unwrap();
        session
            .stamp_completed_at(
                (session.created_at + chrono::Duration::seconds(10)).timestamp_millis() as u64,
            )
            .unwrap();
        assert_eq!(
            session.stamp_completed_at(
                (session.created_at + chrono::Duration::seconds(11)).timestamp_millis() as u64
            ),
            Err(PracticeSessionError::InvalidResult)
        );
        let restored: PracticeSession =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        restored.validate().unwrap();
        assert_eq!(restored.completed_at, session.completed_at);
    }
}

#[test]
fn inventory_snapshot_requires_the_same_keys() {
    let original = vec!["one".to_owned(), "two".to_owned()];
    assert_eq!(
        inventory::inventory_keys_match(&original, &original),
        Ok(())
    );
    assert_eq!(
        inventory::inventory_keys_match(&original, &["one".to_owned(), "three".to_owned()]),
        Err(PracticeSessionError::OutOfDate)
    );
    assert_eq!(
        inventory::inventory_keys_match(&original, &["one".to_owned()]),
        Err(PracticeSessionError::OutOfDate)
    );
}

#[test]
fn ai_generated_matching_is_queued_saved_and_restored() {
    let mut saved = PracticeSession::new(
        "123e4567-e89b-72d3-a456-426614174321".to_owned(),
        &["matching".to_owned()].into_iter().collect(),
        1,
        17,
        BTreeMap::from([("source".to_owned(), "hsk:3:1".to_owned())]),
    ).unwrap();
    if let Some(ExerciseConfiguration::Quiz { ai_generated, .. }) =
        saved.exercise_configurations.get_mut("matching") {
        *ai_generated = true;
    }
    saved.steps[0].readiness = ExerciseReadiness::NeedsGeneration;
    saved.queue_generation(2);
    assert!(saved.validate().is_ok());
    let id = saved.steps[0].id.clone();
    let attempt = saved.start_generation(&id).unwrap();
    let raw = serde_json::json!({
        "kind":"matching",
        "words":[
            {"id":1,"hanzi":"你好","pinyin":"nǐ hǎo","meaning":"hello"},
            {"id":2,"hanzi":"谢谢","pinyin":"xièxie","meaning":"thank you"}
        ]
    }).to_string();
    saved.generation_succeeded(&id, attempt, raw).unwrap();
    assert!(saved.validate().is_ok());
    let stored = serde_json::to_string(&saved).unwrap();
    assert!(decode_stored_session(&saved.id, &stored).is_ok());
}
