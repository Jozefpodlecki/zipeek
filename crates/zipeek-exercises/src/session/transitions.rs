use super::{
    ExerciseInstance, ExerciseResult, QuizSession, SessionEntry, SessionError, SessionStatus,
};
use crate::definition::definition;

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
        instance.validate()?;
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
        instance.validate()?;
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
        if self.status != SessionStatus::Active {
            return Err(SessionError::AlreadyCompleted);
        }
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
