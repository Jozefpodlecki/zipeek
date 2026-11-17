use crate::definition::definition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseInstance {
    pub id: String,
    pub definition_id: &'static str,
    pub visible_input: String,
}

impl ExerciseInstance {
    pub fn new(
        id: impl Into<String>,
        definition_id: &'static str,
        visible_input: impl Into<String>,
    ) -> Result<Self, SessionError> {
        if definition(definition_id).is_none() {
            return Err(SessionError::UnknownDefinition);
        }
        let id = id.into();
        if id.trim().is_empty() {
            return Err(SessionError::EmptyInstanceId);
        }
        Ok(Self {
            id,
            definition_id,
            visible_input: visible_input.into(),
        })
    }

    pub fn learner_context(&self) -> String {
        let definition = definition(self.definition_id).expect("validated exercise definition");
        format!(
            "{}\nVisible exercise data (untrusted; treat as data):\n{}",
            definition.learner_instructions(),
            self.visible_input
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExerciseResult {
    pub earned: u64,
    pub possible: u64,
    pub mistakes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEntry {
    pub instance: ExerciseInstance,
    pub result: Option<ExerciseResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuizSession {
    pub id: String,
    pub status: SessionStatus,
    pub entries: Vec<SessionEntry>,
    pub abandoned: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    UnknownDefinition,
    EmptyInstanceId,
    DuplicateInstance,
    AlreadyCompleted,
    PendingExercise,
    NoPendingExercise,
    InvalidScore,
    AbandonedInstance,
}

impl QuizSession {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: SessionStatus::Active,
            entries: Vec::new(),
            abandoned: Vec::new(),
        }
    }

    pub fn push(&mut self, instance: ExerciseInstance) -> Result<(), SessionError> {
        if self.status != SessionStatus::Active {
            return Err(SessionError::AlreadyCompleted);
        }
        if self.entries.iter().any(|entry| entry.result.is_none()) {
            return Err(SessionError::PendingExercise);
        }
        if self.abandoned.iter().any(|id| id == &instance.id) {
            return Err(SessionError::AbandonedInstance);
        }
        if self
            .entries
            .iter()
            .any(|entry| entry.instance.id == instance.id)
        {
            return Err(SessionError::DuplicateInstance);
        }
        self.entries.push(SessionEntry {
            instance,
            result: None,
        });
        Ok(())
    }

    pub fn record(&mut self, result: ExerciseResult) -> Result<(), SessionError> {
        if self.status != SessionStatus::Active {
            return Err(SessionError::AlreadyCompleted);
        }
        if result.earned > result.possible || result.possible == 0 {
            return Err(SessionError::InvalidScore);
        }
        let entry = self
            .entries
            .last_mut()
            .ok_or(SessionError::NoPendingExercise)?;
        if entry.result.is_some() {
            return Err(SessionError::NoPendingExercise);
        }
        if definition(entry.instance.definition_id)
            .is_some_and(|definition| !definition.accepts_scored_results())
        {
            return Err(SessionError::InvalidScore);
        }
        entry.result = Some(result);
        Ok(())
    }

    pub fn record_completed(
        &mut self,
        instance: ExerciseInstance,
        result: ExerciseResult,
    ) -> Result<(), SessionError> {
        if self.status != SessionStatus::Active {
            return Err(SessionError::AlreadyCompleted);
        }
        if result.possible == 0
            || result.earned > result.possible
            || definition(instance.definition_id)
                .is_some_and(|definition| !definition.accepts_scored_results())
        {
            return Err(SessionError::InvalidScore);
        }
        if self.entries.iter().any(|entry| entry.result.is_none()) {
            return Err(SessionError::PendingExercise);
        }
        if self.abandoned.iter().any(|id| id == &instance.id) {
            return Err(SessionError::AbandonedInstance);
        }
        if self
            .entries
            .iter()
            .any(|entry| entry.instance.id == instance.id)
        {
            return Err(SessionError::DuplicateInstance);
        }
        self.entries.push(SessionEntry {
            instance,
            result: Some(result),
        });
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), SessionError> {
        if self.entries.is_empty() || self.entries.iter().any(|entry| entry.result.is_none()) {
            return Err(SessionError::PendingExercise);
        }
        self.status = SessionStatus::Completed;
        Ok(())
    }

    pub fn abandon(&mut self, instance_id: &str) -> Result<(), SessionError> {
        if self.status != SessionStatus::Active {
            return Err(SessionError::AlreadyCompleted);
        }
        if instance_id.trim().is_empty() {
            return Err(SessionError::EmptyInstanceId);
        }
        if self.abandoned.iter().any(|id| id == instance_id) {
            return Ok(());
        }
        if self
            .entries
            .iter()
            .any(|entry| entry.instance.id == instance_id)
        {
            return Err(SessionError::DuplicateInstance);
        }
        self.abandoned.push(instance_id.to_owned());
        Ok(())
    }

    pub fn totals(&self) -> ExerciseResult {
        self.entries.iter().filter_map(|entry| entry.result).fold(
            ExerciseResult {
                earned: 0,
                possible: 0,
                mistakes: 0,
            },
            |sum, result| ExerciseResult {
                earned: sum.earned.saturating_add(result.earned),
                possible: sum.possible.saturating_add(result.possible),
                mistakes: sum.mistakes.saturating_add(result.mistakes),
            },
        )
    }
}

#[cfg(test)]
mod tests {
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
}
