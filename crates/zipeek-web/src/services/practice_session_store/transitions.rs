use super::*;

impl PracticeSession {
    /// Prepare the next window before committing progress. A scheduling error
    /// must never leave a completed step in a still-current slot.
    pub fn skip_current(&mut self) -> Result<(), PracticeSessionError> {
        let step = self
            .steps
            .get(self.current)
            .ok_or(PracticeSessionError::AlreadyFinished)?;
        if step.outcome.is_some() {
            return Err(PracticeSessionError::AlreadyFinished);
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .filter(|revision| *revision < u64::MAX)
            .ok_or(PracticeSessionError::InvalidStoredSession)?;
        let next_current = self.current + 1;
        let upcoming = self.plan_endless_steps(next_current)?;
        let now = chrono::Utc::now();
        self.started_at.get_or_insert(now);
        let step = &mut self.steps[self.current];
        step.started_at.get_or_insert(now);
        step.finished_at = Some(now);
        step.outcome = Some(ExerciseOutcome::Skipped);
        self.commit_advance(next_current, next_revision, upcoming);
        Ok(())
    }

    pub fn complete(
        &mut self,
        step_id: &str,
        outcome: ExerciseOutcome,
    ) -> Result<(), PracticeSessionError> {
        let step = self
            .steps
            .get(self.current)
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
            || (matches!(
                &outcome,
                ExerciseOutcome::Scored { .. } | ExerciseOutcome::Assessed(_)
            ) && !resumable_scores_supported(&step.exercise_id))
            || matches!(&outcome, ExerciseOutcome::Assessed(assessment) if !assessment.valid())
            || matches!(&outcome, ExerciseOutcome::Skipped)
        {
            return Err(PracticeSessionError::InvalidResult);
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .filter(|revision| *revision < u64::MAX)
            .ok_or(PracticeSessionError::InvalidStoredSession)?;
        let next_current = self.current + 1;
        let upcoming = self.plan_endless_steps(next_current)?;
        let now = chrono::Utc::now();
        self.started_at.get_or_insert(now);
        let step = &mut self.steps[self.current];
        step.started_at.get_or_insert(now);
        step.finished_at = Some(now);
        step.outcome = Some(outcome);
        self.commit_advance(next_current, next_revision, upcoming);
        Ok(())
    }

    fn commit_advance(
        &mut self,
        next_current: usize,
        next_revision: u64,
        upcoming: Vec<PracticeStep>,
    ) {
        self.current = next_current;
        self.steps.extend(upcoming);
        if self.session_mode.is_endless() && self.current == 1_000 && self.steps.len() == 1_000 {
            self.session_mode = SessionMode::Fixed;
        }
        self.queue_generation(3);
        self.revision = next_revision;
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

    /// Build the future exercise window without modifying the active session.
    /// This also lets completion preflight its scheduler before committing the
    /// current answer, avoiding half-applied progress on generation errors.
    fn plan_endless_steps(
        &self,
        next_current: usize,
    ) -> Result<Vec<PracticeStep>, PracticeSessionError> {
        if !self.session_mode.is_endless() {
            return Ok(Vec::new());
        }
        let selected = self.exercise_configurations.keys().cloned().collect();
        let mut upcoming = Vec::new();
        while (self.steps.len() + upcoming.len()).saturating_sub(next_current) < 3
            && self.steps.len() + upcoming.len() < 1_000
        {
            let ordinal = self.steps.len() + upcoming.len();
            let scheduled = zipeek_exercises::practice_queue::endless_step(
                &selected,
                ordinal,
                self.scheduling_seed,
            )
            .map_err(|error| PracticeSessionError::InvalidSchedule(format!("{error:?}")))?;
            let exercise_id = scheduled.exercise_id;
            let configuration = self
                .exercise_configurations
                .get(exercise_id)
                .ok_or(PracticeSessionError::InvalidStoredSession)?;
            let needs_generation = scheduled.preparation == Preparation::AiGenerated
                || matches!(
                    configuration,
                    ExerciseConfiguration::Quiz {
                        ai_generated: true,
                        ..
                    }
                ) && exercise_id == "matching";
            let (quiz_format, reverse_matching) = match configuration {
                ExerciseConfiguration::Quiz {
                    formats,
                    reverse_matching,
                    ..
                } => {
                    let count = formats.len();
                    if count == 0 {
                        return Err(PracticeSessionError::InvalidStoredSession);
                    }
                    let chosen = formats
                        .get((scheduled.instance_seed as usize) % count)
                        .ok_or(PracticeSessionError::InvalidStoredSession)?;
                    (
                        Some(chosen.clone()),
                        *reverse_matching && exercise_id == "matching",
                    )
                }
                _ => (None, false),
            };
            let readiness = if needs_generation {
                ExerciseReadiness::NeedsGeneration
            } else if zipeek_exercises::definition::supports_resumable_session_host(exercise_id) {
                ExerciseReadiness::Ready
            } else {
                ExerciseReadiness::Unavailable(
                    "A resumable session host is not implemented for this exercise".to_owned(),
                )
            };
            upcoming.push(PracticeStep {
                id: format!("{}:{ordinal}", self.id),
                exercise_id: exercise_id.to_owned(),
                configuration_id: exercise_id.to_owned(),
                instance_seed: scheduled.instance_seed,
                readiness,
                outcome: None,
                started_at: None,
                finished_at: None,
                generated_instance: None,
                generation_attempt: 0,
                quiz_format,
                reverse_matching,
            });
        }
        Ok(upcoming)
    }

    pub fn replenish_endless_steps(&mut self) -> Result<(), PracticeSessionError> {
        let upcoming = self.plan_endless_steps(self.current)?;
        self.steps.extend(upcoming);
        if self.session_mode.is_endless() && self.current == 1_000 && self.steps.len() == 1_000 {
            self.session_mode = SessionMode::Fixed;
        }
        Ok(())
    }

    pub fn finish_endless(&mut self) -> Result<(), PracticeSessionError> {
        if !self.session_mode.is_endless() || self.current == 0 {
            return Err(PracticeSessionError::NotReady);
        }
        let mut finished = self.clone();
        finished.steps.truncate(finished.current);
        // A mixed Endless session can finish before every selected exercise
        // appears. A finite completed snapshot should retain configuration
        // only for exercise families actually represented in its steps.
        let completed_exercises: std::collections::BTreeSet<_> = finished
            .steps
            .iter()
            .map(|step| step.exercise_id.clone())
            .collect();
        finished
            .exercise_configurations
            .retain(|id, _| completed_exercises.contains(id));
        if !completed_exercises.contains("character-hunt") {
            finished.vocabulary.remove("difficulty");
        }
        finished.session_mode = SessionMode::Fixed;
        finished.revision = self
            .revision
            .checked_add(1)
            .filter(|revision| *revision < u64::MAX)
            .ok_or(PracticeSessionError::InvalidStoredSession)?;
        finished.validate()?;
        *self = finished;
        Ok(())
    }
}
