use super::*;

impl PracticeSession {
    pub fn skip_current(&mut self) -> Result<(), PracticeSessionError> {
        let step = self
            .steps
            .get_mut(self.current)
            .ok_or(PracticeSessionError::AlreadyFinished)?;
        if step.outcome.is_some() {
            return Err(PracticeSessionError::AlreadyFinished);
        }
        let now = chrono::Utc::now();
        self.started_at.get_or_insert(now);
        step.started_at.get_or_insert(now);
        step.finished_at = Some(now);
        step.outcome = Some(ExerciseOutcome::Skipped);
        self.current += 1;
        self.append_endless_step()?;
        self.queue_generation(2);
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    pub fn complete(
        &mut self,
        step_id: &str,
        outcome: ExerciseOutcome,
    ) -> Result<(), PracticeSessionError> {
        let step = self
            .steps
            .get_mut(self.current)
            .ok_or(PracticeSessionError::AlreadyFinished)?;
        if step.id != step_id {
            return Err(PracticeSessionError::WrongSession);
        }
        if step.outcome.is_some() {
            return Err(PracticeSessionError::AlreadyFinished);
        }
        if !matches!(&step.readiness, ExerciseReadiness::Ready) {
            return Err(PracticeSessionError::NotReady);
        }
        if matches!(&outcome, ExerciseOutcome::Scored { earned, possible } if *possible != 1 || *earned > 1)
            || (matches!(&outcome, ExerciseOutcome::Scored { .. })
                && !resumable_scores_supported(&step.exercise_id))
        {
            return Err(PracticeSessionError::InvalidResult);
        }
        let now = chrono::Utc::now();
        self.started_at.get_or_insert(now);
        step.started_at.get_or_insert(now);
        step.finished_at = Some(now);
        step.outcome = Some(outcome);
        self.current += 1;
        self.append_endless_step()?;
        self.queue_generation(2);
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    pub fn start_current(&mut self) -> Result<bool, PracticeSessionError> {
        let step = self
            .steps
            .get_mut(self.current)
            .ok_or(PracticeSessionError::AlreadyFinished)?;
        if !matches!(step.readiness, ExerciseReadiness::Ready) {
            return Ok(false);
        }
        if step.started_at.is_some() {
            return Ok(false);
        }
        let now = chrono::Utc::now();
        self.started_at.get_or_insert(now);
        step.started_at = Some(now);
        Ok(true)
    }

    pub fn stamp_completed_at(&mut self, unix_ms: u64) -> Result<(), PracticeSessionError> {
        let timestamp = chrono::DateTime::from_timestamp_millis(
            i64::try_from(unix_ms).map_err(|_| PracticeSessionError::InvalidResult)?,
        )
        .ok_or(PracticeSessionError::InvalidResult)?;
        if !self.is_completed() || unix_ms == 0 {
            return Err(PracticeSessionError::NotReady);
        }
        match self.completed_at {
            Some(existing) if existing != timestamp => Err(PracticeSessionError::InvalidResult),
            Some(_) => Ok(()),
            None => {
                self.completed_at = Some(timestamp);
                Ok(())
            }
        }
    }

    fn append_endless_step(&mut self) -> Result<(), PracticeSessionError> {
        if self.endless_translation && self.steps.len() == 1_000 {
            self.endless_translation = false;
        }
        if self.endless_translation {
            let ordinal = self.steps.len();
            let scheduled = schedule(
                &["sentence-translation".to_owned()].into_iter().collect(),
                ordinal + 1,
                self.scheduling_seed,
            )
            .map_err(|error| PracticeSessionError::InvalidSchedule(format!("{error:?}")))?;
            let next = scheduled
                .last()
                .ok_or(PracticeSessionError::InvalidResult)?;
            self.steps.push(PracticeStep {
                id: format!("{}:{ordinal}", self.id),
                exercise_id: "sentence-translation".to_owned(),
                configuration_id: "sentence-translation".to_owned(),
                instance_seed: next.instance_seed,
                readiness: ExerciseReadiness::NeedsGeneration,
                outcome: None,
                started_at: None,
                finished_at: None,
                generated_instance: None,
                generation_attempt: 0,
                quiz_format: None,
                reverse_matching: false,
            });
        }
        Ok(())
    }

    pub fn finish_endless_translation(&mut self) -> Result<(), PracticeSessionError> {
        if !self.endless_translation || self.current == 0 {
            return Err(PracticeSessionError::NotReady);
        }
        self.steps.truncate(self.current);
        self.endless_translation = false;
        self.revision = self.revision.saturating_add(1);
        self.validate()
    }
}
