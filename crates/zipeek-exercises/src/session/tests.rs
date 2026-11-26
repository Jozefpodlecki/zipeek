use super::*;

fn instance(id: &str, definition_id: &'static str) -> ExerciseInstance {
    ExerciseInstance::new(id, definition_id, "visible prompt").unwrap()
}

#[test]
fn mixed_session_tracks_instances_without_a_kind_enum() {
    let mut session = QuizSession::new("practice-1");
    session.push(instance("1", "matching")).unwrap();
    assert_eq!(
        session.push(instance("2", "multiple-choice")),
        Err(SessionError::PendingExercise)
    );
    session
        .record(ExerciseResult {
            earned: 3,
            possible: 4,
            mistakes: 1,
        })
        .unwrap();
    session.push(instance("2", "multiple-choice")).unwrap();
    session
        .record(ExerciseResult {
            earned: 1,
            possible: 1,
            mistakes: 0,
        })
        .unwrap();
    session.complete().unwrap();
    assert_eq!(
        session.totals(),
        ExerciseResult {
            earned: 4,
            possible: 5,
            mistakes: 1
        }
    );
    assert_eq!(session.status, SessionStatus::Completed);
    assert_eq!(
        session.push(instance("3", "sentence-translation")),
        Err(SessionError::AlreadyCompleted)
    );
}

#[test]
fn invalid_state_changes_are_rejected() {
    assert_eq!(
        ExerciseInstance::new("", "matching", ""),
        Err(SessionError::EmptyInstanceId)
    );
    assert_eq!(
        ExerciseInstance::new("x", "missing", ""),
        Err(SessionError::UnknownDefinition)
    );
    let mut session = QuizSession::new("x");
    assert_eq!(session.complete(), Err(SessionError::PendingExercise));
    session.push(instance("1", "sentence-translation")).unwrap();
    assert_eq!(
        session.record(ExerciseResult {
            earned: 2,
            possible: 1,
            mistakes: 0
        }),
        Err(SessionError::InvalidScore)
    );
    session
        .record(ExerciseResult {
            earned: 1,
            possible: 1,
            mistakes: 0,
        })
        .unwrap();
    assert_eq!(
        session.record(ExerciseResult {
            earned: 1,
            possible: 1,
            mistakes: 0
        }),
        Err(SessionError::NoPendingExercise)
    );
    assert_eq!(
        session.push(instance("1", "matching")),
        Err(SessionError::DuplicateInstance)
    );
}

#[test]
fn translation_session_records_evaluation_once_and_preserves_prior_outcomes() {
    let mut session = QuizSession::new("translation-local");
    session
        .push(instance("translation-0", "sentence-translation"))
        .unwrap();
    assert_eq!(
        session.push(instance("translation-1", "sentence-translation")),
        Err(SessionError::PendingExercise)
    );
    session
        .record(ExerciseResult {
            earned: 0,
            possible: 1,
            mistakes: 1,
        })
        .unwrap();
    assert_eq!(
        session.record(ExerciseResult {
            earned: 1,
            possible: 1,
            mistakes: 0
        }),
        Err(SessionError::NoPendingExercise)
    );
    session
        .push(instance("translation-1", "sentence-translation"))
        .unwrap();
    session
        .record(ExerciseResult {
            earned: 1,
            possible: 1,
            mistakes: 0,
        })
        .unwrap();
    assert_eq!(
        session.totals(),
        ExerciseResult {
            earned: 1,
            possible: 2,
            mistakes: 1
        }
    );
    assert_eq!(session.entries.len(), 2);
}

#[test]
fn completed_result_submission_is_atomic_on_validation_failure() {
    let mut session = QuizSession::new("shared");
    let bad_score = ExerciseResult {
        earned: 2,
        possible: 1,
        mistakes: 0,
    };
    assert_eq!(
        session.record_completed(instance("one", "matching"), bad_score),
        Err(SessionError::InvalidScore)
    );
    assert!(session.entries.is_empty());

    let success = ExerciseResult {
        earned: 1,
        possible: 1,
        mistakes: 0,
    };
    session
        .record_completed(instance("one", "matching"), success)
        .unwrap();
    let snapshot = session.clone();
    assert_eq!(
        session.record_completed(instance("one", "matching"), success),
        Err(SessionError::DuplicateInstance)
    );
    assert_eq!(session, snapshot);

    session.complete().unwrap();
    assert_eq!(
        session.record_completed(instance("two", "multiple-choice"), success),
        Err(SessionError::AlreadyCompleted)
    );
    assert_eq!(session, snapshot_with_completed_status(snapshot));
}

#[test]
fn abandoning_is_idempotent_terminal_and_unscored() {
    let mut session = QuizSession::new("one");
    session.abandon("unfinished").unwrap();
    session.abandon("unfinished").unwrap();
    assert_eq!(session.abandoned, vec!["unfinished"]);
    assert_eq!(
        session.record_completed(
            instance("unfinished", "matching"),
            ExerciseResult {
                earned: 1,
                possible: 1,
                mistakes: 0
            }
        ),
        Err(SessionError::AbandonedInstance)
    );
    assert_eq!(
        session.totals(),
        ExerciseResult {
            earned: 0,
            possible: 0,
            mistakes: 0
        }
    );
    session
        .record_completed(
            instance("replacement", "multiple-choice"),
            ExerciseResult {
                earned: 1,
                possible: 1,
                mistakes: 0,
            },
        )
        .unwrap();
    assert_eq!(
        session.abandon("replacement"),
        Err(SessionError::DuplicateInstance)
    );
    assert_eq!(session.entries.len(), 1);
}

fn snapshot_with_completed_status(mut session: QuizSession) -> QuizSession {
    session.status = SessionStatus::Completed;
    session
}

#[test]
fn completed_submission_refuses_pending_exercises_without_mutation() {
    let mut session = QuizSession::new("shared");
    session.push(instance("pending", "matching")).unwrap();
    let snapshot = session.clone();
    assert_eq!(
        session.record_completed(
            instance("second", "multiple-choice"),
            ExerciseResult {
                earned: 1,
                possible: 1,
                mistakes: 0
            }
        ),
        Err(SessionError::PendingExercise)
    );
    assert_eq!(session, snapshot);
}

#[test]
fn learner_context_exposes_definition_not_hidden_solution() {
    let instance = instance("1", "sentence-translation");
    let text = instance.learner_context();
    assert!(text.contains("ordered array"));
    assert!(text.contains("visible prompt"));
    assert!(!text.contains("correct_id"));
}
#[test]
fn practice_only_definition_never_accepts_scored_result() {
    let mut session = QuizSession::new("unscored");
    let result = ExerciseResult {
        earned: 1,
        possible: 1,
        mistakes: 0,
    };
    assert_eq!(
        session.record_completed(instance("practice-1", "speaking"), result),
        Err(SessionError::InvalidScore)
    );
    assert!(session.entries.is_empty());
    session.push(instance("practice-1", "speaking")).unwrap();
    assert_eq!(session.record(result), Err(SessionError::InvalidScore));
    assert!(session.entries[0].result.is_none());
}

#[test]
fn repeated_completion_is_rejected_without_mutating_result() {
    let mut session = QuizSession::new("once");
    session
        .record_completed(
            instance("one", "matching"),
            ExerciseResult {
                earned: 1,
                possible: 1,
                mistakes: 0,
            },
        )
        .unwrap();
    session.complete().unwrap();
    let snapshot = session.clone();
    assert_eq!(session.complete(), Err(SessionError::AlreadyCompleted));
    assert_eq!(session, snapshot);
}

#[test]
fn public_instance_fields_cannot_bypass_session_validation() {
    let scored = ExerciseResult {
        earned: 1,
        possible: 1,
        mistakes: 0,
    };
    let invalid = [
        (
            ExerciseInstance {
                id: " ".to_owned(),
                definition_id: "matching",
                visible_input: String::new(),
            },
            SessionError::EmptyInstanceId,
        ),
        (
            ExerciseInstance {
                id: "valid".to_owned(),
                definition_id: "unknown-exercise",
                visible_input: String::new(),
            },
            SessionError::UnknownDefinition,
        ),
    ];
    for (instance, expected) in invalid {
        assert_eq!(instance.validate(), Err(expected));
        let mut session = QuizSession::new("validation");
        let snapshot = session.clone();
        assert_eq!(session.push(instance.clone()), Err(expected));
        assert_eq!(session, snapshot);
        assert_eq!(session.record_completed(instance, scored), Err(expected));
        assert_eq!(session, snapshot);
    }
}
