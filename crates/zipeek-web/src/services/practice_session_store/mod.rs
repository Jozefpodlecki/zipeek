use std::collections::{BTreeMap, BTreeSet};
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::practice_queue::{Preparation, schedule};

mod generation;
mod model;
mod transitions;
mod validation;

pub(crate) const SESSION_KEY_PREFIX: &str = "zipeek.practice.session.v1.";
pub(crate) const SESSION_CHANGE_EVENT: &str = "zipeek:practice-session-changed";

fn notify_session_change() {
    if let Ok(event) = web_sys::Event::new(SESSION_CHANGE_EVENT) {
        let _ = gloo::utils::window().dispatch_event(&event);
    }
}
const MAX_STORED_SESSION_BYTES: usize = 4 * 1024 * 1024;

pub(crate) fn session_storage_key(id: &str) -> Result<String, PracticeSessionError> {
    if !valid_session_id(id) {
        return Err(PracticeSessionError::InvalidId);
    }
    Ok(format!("{SESSION_KEY_PREFIX}{id}"))
}

pub(crate) use model::{
    ExerciseConfiguration, ExerciseOutcome, ExerciseReadiness, PracticeSession,
    PracticeSessionError, PracticeStep,
};
use model::{resumable_scores_supported, valid_translation_direction};

impl PracticeSession {
    pub fn new(
        id: String,
        exercises: &BTreeSet<String>,
        count: usize,
        seed: u64,
        vocabulary: BTreeMap<String, String>,
    ) -> Result<Self, PracticeSessionError> {
        if !valid_session_id(&id) {
            return Err(PracticeSessionError::InvalidId);
        }
        let scheduled = schedule(exercises, count, seed)
            .map_err(|error| PracticeSessionError::InvalidSchedule(format!("{error:?}")))?;
        let steps = scheduled
            .into_iter()
            .map(|exercise| PracticeStep {
                id: format!("{}:{}", id, exercise.ordinal),
                exercise_id: exercise.exercise_id.to_owned(),
                configuration_id: exercise.exercise_id.to_owned(),
                instance_seed: exercise.instance_seed,
                readiness: if exercise.preparation == Preparation::AiGenerated {
                    ExerciseReadiness::NeedsGeneration
                } else if !zipeek_exercises::definition::supports_resumable_session_host(
                    exercise.exercise_id,
                ) {
                    ExerciseReadiness::Unavailable(
                        "A resumable session host is not implemented for this exercise".to_owned(),
                    )
                } else {
                    ExerciseReadiness::Ready
                },
                outcome: None,
                started_at: None,
                finished_at: None,
                generated_instance: None,
                generation_attempt: 0,
                quiz_format: match exercise.exercise_id {
                    "matching" => Some(
                        ExerciseVariant::MatchingHanziMeaning
                            .seed_token()
                            .to_owned(),
                    ),
                    "multiple-choice" => Some(
                        ExerciseVariant::MultipleChoiceHanziMeaning
                            .seed_token()
                            .to_owned(),
                    ),
                    _ => None,
                },
                reverse_matching: false,
            })
            .collect();
        let mut session = Self {
            id,
            revision: 0,
            current: 0,
            vocabulary,
            exercise_configurations: exercises
                .iter()
                .map(|id| (id.clone(), ExerciseConfiguration::for_exercise(id)))
                .collect(),
            created_at: chrono::Utc::now(),
            started_at: None,
            endless_translation: false,
            scheduling_seed: seed,
            completed_at: None,
            steps,
        };
        session.queue_generation(2);
        Ok(session)
    }

    pub fn configure_quiz_formats(
        &mut self,
        matching: &[ExerciseVariant],
        multiple_choice: &[ExerciseVariant],
        reverse_matching: bool,
    ) -> Result<(), PracticeSessionError> {
        for (exercise, variants) in [("matching", matching), ("multiple-choice", multiple_choice)] {
            if self.steps.iter().any(|step| step.exercise_id == exercise)
                && (variants.is_empty()
                    || variants
                        .iter()
                        .any(|variant| variant.is_mcq() != (exercise == "multiple-choice")))
            {
                return Err(PracticeSessionError::InvalidSchedule(format!(
                    "Invalid selected formats for {exercise}"
                )));
            }
        }
        for step in &mut self.steps {
            let formats = match step.exercise_id.as_str() {
                "matching" => matching,
                "multiple-choice" => multiple_choice,
                _ => continue,
            };
            let chosen = formats[(step.instance_seed as usize) % formats.len()];
            step.quiz_format = Some(chosen.seed_token().to_owned());
            step.reverse_matching = step.exercise_id == "matching" && reverse_matching;
        }
        for (id, variants) in [("matching", matching), ("multiple-choice", multiple_choice)] {
            if self.exercise_configurations.contains_key(id) {
                self.exercise_configurations.insert(
                    id.to_owned(),
                    ExerciseConfiguration::Quiz {
                        formats: variants
                            .iter()
                            .map(|variant| variant.seed_token().to_owned())
                            .collect(),
                        reverse_matching: id == "matching" && reverse_matching,
                        pair_count: 4,
                        ai_generated: false,
                        show_pinyin: true,
                    },
                );
            }
        }
        self.validate()
    }

    pub fn translation_direction(&self) -> &str {
        match self.exercise_configurations.get("sentence-translation") {
            Some(ExerciseConfiguration::SentenceTranslation { direction }) => direction,
            _ => "mixed",
        }
    }

    pub fn current_step(&self) -> Option<&PracticeStep> {
        self.steps.get(self.current)
    }

    pub fn is_completed(&self) -> bool {
        self.current == self.steps.len()
    }
}

pub(crate) fn valid_session_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id)
        .is_ok_and(|uuid| uuid.get_version_num() == 7 && uuid.to_string() == id)
}

mod codec;
mod inventory;
mod persistence;

pub(crate) use persistence::PracticeSessionStore;

#[cfg(test)]
use codec::decode_stored_session;
#[cfg(test)]
use inventory::{PracticeSessionInventory, completed_removal_allowed, ensure_new_session_allowed};

#[cfg(test)]
mod tests;
