use super::{
    ExerciseConfiguration, ExerciseOutcome, ExerciseReadiness, PracticeSession,
    PracticeSessionError, resumable_scores_supported, valid_session_id,
    valid_translation_direction,
};
use zipeek_core::practice::ExerciseVariant;

impl PracticeSession {
    pub fn validate(&self) -> Result<(), PracticeSessionError> {
        if !valid_session_id(&self.id)
            || self.revision == u64::MAX
            || self.steps.is_empty()
            || self.steps.len() > 1_000
            || self.current > self.steps.len()
            || (self.completed_at.is_some() && !self.is_completed())
            || self.created_at.timestamp() <= 0
            || self.started_at.is_some_and(|at| at < self.created_at)
            || self.completed_at.is_some_and(|at| at < self.created_at)
            || !valid_translation_direction(self.translation_direction())
            || self.exercise_configurations.len()
                != self
                    .steps
                    .iter()
                    .map(|step| step.exercise_id.as_str())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
            || self
                .exercise_configurations
                .iter()
                .any(|(id, configuration)| {
                    !self.steps.iter().any(|step| step.exercise_id == *id)
                        || match (id.as_str(), configuration) {
                            ("listening-choice", ExerciseConfiguration::ListeningChoice(_))
                            | ("write-hanzi", ExerciseConfiguration::WriteHanzi(_))
                            | ("type-pinyin", ExerciseConfiguration::TypePinyin(_))
                            | ("character-builder", ExerciseConfiguration::CharacterBuilder(_)) => {
                                false
                            }
                            (
                                "matching",
                                ExerciseConfiguration::Quiz {
                                    formats,
                                    reverse_matching: _,
                                    pair_count,
                                },
                            ) => {
                                !(2..=7).contains(pair_count)
                                    || formats.is_empty()
                                    || formats.iter().any(|format| {
                                        zipeek_core::practice::ExerciseVariant::from_seed_token(
                                            format,
                                        )
                                        .is_none_or(|variant| variant.is_mcq())
                                    })
                            }
                            (
                                "multiple-choice",
                                ExerciseConfiguration::Quiz {
                                    formats,
                                    reverse_matching,
                                    pair_count,
                                },
                            ) => {
                                *reverse_matching
                                    || *pair_count != 4
                                    || formats.is_empty()
                                    || formats.iter().any(|format| {
                                        zipeek_core::practice::ExerciseVariant::from_seed_token(
                                            format,
                                        )
                                        .is_none_or(|variant| !variant.is_mcq())
                                    })
                            }
                            (
                                "sentence-translation",
                                ExerciseConfiguration::SentenceTranslation { direction },
                            ) => !valid_translation_direction(direction),
                            (
                                "sentence-transformation",
                                ExerciseConfiguration::SentenceTransformation(options),
                            ) => !options.valid(),
                            (_, ExerciseConfiguration::Standard) => matches!(
                                id.as_str(),
                                "matching"
                                    | "multiple-choice"
                                    | "write-hanzi"
                                    | "type-pinyin"
                                    | "character-builder"
                                    | "sentence-translation"
                                    | "sentence-transformation"
                            ),
                            _ => true,
                        }
                })
            || (self.endless_translation
                && (self
                    .steps
                    .iter()
                    .any(|step| step.exercise_id != "sentence-translation")
                    || self.is_completed()))
            || self.vocabulary.len() > 2
            || (self.vocabulary.contains_key("source") == self.vocabulary.contains_key("set_ids"))
            || !self
                .vocabulary
                .iter()
                .all(|(key, value)| match key.as_str() {
                    "source" | "set_ids" => !value.is_empty() && value.len() <= 16_384,
                    "difficulty" => {
                        zipeek_exercises::character_hunt::Difficulty::parse(value).is_some()
                            && self
                                .steps
                                .iter()
                                .any(|step| step.exercise_id == "character-hunt")
                    }
                    _ => false,
                })
        {
            return Err(PracticeSessionError::InvalidStoredSession);
        }
        for (index, step) in self.steps.iter().enumerate() {
            if step.id != format!("{}:{index}", self.id)
                || step.configuration_id != step.exercise_id
                || !self
                    .exercise_configurations
                    .contains_key(&step.configuration_id)
                || zipeek_exercises::definition::definition(&step.exercise_id).is_none()
                || (index < self.current && step.outcome.is_none())
                || (step.started_at.is_some() != step.finished_at.is_some()
                    && step.outcome.is_some())
                || (step.finished_at.is_some() && step.outcome.is_none())
                || step.started_at.is_some_and(|at| at < self.created_at)
                || step
                    .finished_at
                    .is_some_and(|at| step.started_at.is_none_or(|start| at < start))
                || (index >= self.current && step.outcome.is_some())
                || (match step.exercise_id.as_str() {
                    "matching" | "multiple-choice" => {
                        step.quiz_format
                            .as_deref()
                            .and_then(ExerciseVariant::from_seed_token)
                            .is_none_or(|variant| {
                                variant.is_mcq() != (step.exercise_id == "multiple-choice")
                            })
                            || (step.exercise_id == "multiple-choice" && step.reverse_matching)
                    }
                    _ => step.quiz_format.is_some() || step.reverse_matching,
                })
                || step.generated_instance.as_ref().is_some_and(|instance| {
                    crate::services::practice_generated::GeneratedInstance::decode(
                        instance,
                        &step.exercise_id,
                    )
                    .is_err()
                })
                || (step.generated_instance.is_some() && step.generation_attempt == 0)
                || (step.generation_attempt > 3)
                || (matches!(
                    &step.readiness,
                    ExerciseReadiness::Generating | ExerciseReadiness::Failed(_)
                ) && step.generation_attempt == 0)
                || (matches!(
                    &step.readiness,
                    ExerciseReadiness::Queued | ExerciseReadiness::NeedsGeneration
                ) && step.generated_instance.is_some())
                || (matches!(&step.readiness, ExerciseReadiness::Failed(reason) if reason.trim().is_empty() || reason.len() > 512))
                || (!zipeek_exercises::practice_queue::requires_generation(&step.exercise_id)
                    && step.generation_attempt != 0)
                || (step.generated_instance.is_some()
                    && !matches!(&step.readiness, ExerciseReadiness::Ready))
                || (!zipeek_exercises::practice_queue::requires_generation(&step.exercise_id)
                    && !zipeek_exercises::definition::supports_resumable_session_host(
                        &step.exercise_id,
                    )
                    && (!matches!(&step.readiness, ExerciseReadiness::Unavailable(_))
                        || step.generated_instance.is_some()))
                || (zipeek_exercises::practice_queue::requires_generation(&step.exercise_id)
                    && matches!(&step.readiness, ExerciseReadiness::Ready)
                    && step.generated_instance.is_none())
                || (!zipeek_exercises::practice_queue::requires_generation(&step.exercise_id)
                    && zipeek_exercises::definition::supports_resumable_session_host(
                        &step.exercise_id,
                    )
                    && (!matches!(&step.readiness, ExerciseReadiness::Ready)
                        || step.generated_instance.is_some()))
            {
                return Err(PracticeSessionError::InvalidStoredSession);
            }
            if let ExerciseReadiness::Unavailable(reason) = &step.readiness {
                if reason.trim().is_empty() || reason.len() > 512 {
                    return Err(PracticeSessionError::InvalidStoredSession);
                }
            }
            if let Some(ExerciseOutcome::Scored { earned, possible }) = &step.outcome {
                if *possible != 1 || *earned > 1 || !resumable_scores_supported(&step.exercise_id) {
                    return Err(PracticeSessionError::InvalidStoredSession);
                }
            }
        }
        Ok(())
    }
}
