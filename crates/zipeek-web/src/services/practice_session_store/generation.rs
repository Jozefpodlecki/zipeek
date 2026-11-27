use super::*;

impl PracticeSession {
    pub fn recover_interrupted_generation(&mut self) -> bool {
        let mut changed = false;
        for step in self.steps.iter_mut().skip(self.current) {
            if matches!(&step.readiness, ExerciseReadiness::Generating) {
                step.readiness = if step.generation_attempt >= 3 {
                    ExerciseReadiness::Failed("Preparation interrupted too many times".to_owned())
                } else {
                    ExerciseReadiness::Queued
                };
                changed = true;
            }
        }
        if changed {
            self.revision += 1;
        }
        changed
    }

    pub fn queue_generation(&mut self, lookahead: usize) {
        let mut prepared_or_queued = 0;
        for step in self.steps.iter_mut().skip(self.current) {
            if step.outcome.is_some() {
                continue;
            }
            match &step.readiness {
                ExerciseReadiness::Ready
                | ExerciseReadiness::Queued
                | ExerciseReadiness::Generating => prepared_or_queued += 1,
                ExerciseReadiness::NeedsGeneration if prepared_or_queued < lookahead => {
                    step.readiness = ExerciseReadiness::Queued;
                    prepared_or_queued += 1;
                }
                _ => {}
            }
        }
    }

    pub fn pending_generation(&self) -> impl Iterator<Item = &PracticeStep> {
        self.steps.iter().skip(self.current).filter(|step| {
            step.outcome.is_none() && matches!(&step.readiness, ExerciseReadiness::Queued)
        })
    }

    fn pending_step_index(&self, step_id: &str) -> Result<usize, PracticeSessionError> {
        let index = self
            .steps
            .iter()
            .position(|step| step.id == step_id)
            .ok_or(PracticeSessionError::WrongSession)?;
        if index < self.current {
            return Err(PracticeSessionError::AlreadyFinished);
        }
        Ok(index)
    }

    pub fn start_generation(&mut self, step_id: &str) -> Result<u32, PracticeSessionError> {
        let index = self.pending_step_index(step_id)?;
        let step = &mut self.steps[index];
        if step.generation_attempt >= 3 {
            return Err(PracticeSessionError::InvalidResult);
        }
        if step.outcome.is_some() || !matches!(&step.readiness, ExerciseReadiness::Queued) {
            return Err(PracticeSessionError::NotReady);
        }
        let attempt = step
            .generation_attempt
            .checked_add(1)
            .ok_or(PracticeSessionError::InvalidStoredSession)?;
        step.generation_attempt = attempt;
        step.readiness = ExerciseReadiness::Generating;
        self.revision += 1;
        Ok(attempt)
    }

    pub fn generation_succeeded(
        &mut self,
        step_id: &str,
        attempt: u32,
        instance: String,
    ) -> Result<(), PracticeSessionError> {
        let index = self.pending_step_index(step_id)?;
        let step = &mut self.steps[index];
        if step.generation_attempt != attempt {
            return Err(PracticeSessionError::StaleGeneration);
        }
        if !matches!(&step.readiness, ExerciseReadiness::Generating) {
            return Err(PracticeSessionError::NotReady);
        }
        if crate::services::practice_generated::GeneratedInstance::decode(
            &instance,
            &step.exercise_id,
        )
        .is_err()
        {
            return Err(PracticeSessionError::InvalidResult);
        }
        step.generated_instance = Some(instance);
        step.readiness = ExerciseReadiness::Ready;
        self.revision += 1;
        Ok(())
    }

    pub fn generation_failed(
        &mut self,
        step_id: &str,
        attempt: u32,
        error: String,
    ) -> Result<(), PracticeSessionError> {
        let index = self.pending_step_index(step_id)?;
        let step = &mut self.steps[index];
        if step.generation_attempt != attempt {
            return Err(PracticeSessionError::StaleGeneration);
        }
        if !matches!(&step.readiness, ExerciseReadiness::Generating) {
            return Err(PracticeSessionError::NotReady);
        }
        let message = error.trim().chars().take(512).collect::<String>();
        step.readiness = ExerciseReadiness::Failed(if message.is_empty() {
            "Preparation failed without a diagnostic".to_owned()
        } else {
            message
        });
        self.revision += 1;
        Ok(())
    }

    pub fn retry_generation(&mut self, step_id: &str) -> Result<(), PracticeSessionError> {
        let index = self.pending_step_index(step_id)?;
        let step = &mut self.steps[index];
        if !matches!(&step.readiness, ExerciseReadiness::Failed(_)) {
            return Err(PracticeSessionError::NotReady);
        }
        if step.generation_attempt >= 3 {
            return Err(PracticeSessionError::InvalidResult);
        }
        step.readiness = ExerciseReadiness::Queued;
        step.generated_instance = None;
        self.revision += 1;
        Ok(())
    }
}
