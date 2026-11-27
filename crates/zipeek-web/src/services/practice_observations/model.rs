use serde::{Deserialize, Serialize};

use crate::services::practice_session_store::{ExerciseOutcome, PracticeStep};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PracticeObservation {
    pub session_id: String,
    pub step_id: String,
    pub exercise_id: String,
    pub finished_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activated_at_ms: Option<i64>,
    pub outcome: ExerciseOutcome,
}

impl PracticeObservation {
    pub fn from_step(session_id: &str, step: &PracticeStep) -> Result<Self, String> {
        let outcome = step
            .outcome
            .clone()
            .ok_or("Missing completed exercise outcome")?;
        let finished_at_ms = step
            .finished_at
            .as_ref()
            .ok_or("Missing completed exercise timestamp")?
            .timestamp_millis();
        let activated_at_ms = step.started_at.as_ref().map(|time| time.timestamp_millis());
        let result = Self {
            session_id: session_id.to_owned(),
            step_id: step.id.clone(),
            exercise_id: step.exercise_id.clone(),
            finished_at_ms,
            activated_at_ms,
            outcome,
        };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !super::super::practice_session_store::valid_session_id(&self.session_id)
            || !self.step_id.starts_with(&format!("{}:", self.session_id))
            || !self.step_id[self.session_id.len() + 1..]
                .chars()
                .all(|c| c.is_ascii_digit())
            || self.step_id.len() == self.session_id.len() + 1
            || !zipeek_exercises::definition::supports_resumable_session_host(&self.exercise_id)
            || self.finished_at_ms <= 0
            || self
                .activated_at_ms
                .is_some_and(|start| start <= 0 || start > self.finished_at_ms)
        {
            return Err("Invalid Practice observation identity or timestamps".to_owned());
        }
        match &self.outcome {
            ExerciseOutcome::Assessed(assessment) if !assessment.valid() => {
                Err("Invalid assessed Practice evidence".to_owned())
            }
            _ => Ok(()),
        }
    }

    pub fn encoded(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| format!("Cannot encode observation: {error}"))
    }

    pub fn decoded(raw: &str) -> Result<Self, String> {
        let observation: Self = serde_json::from_str(raw)
            .map_err(|error| format!("Cannot decode observation: {error}"))?;
        observation.validate()?;
        Ok(observation)
    }

    pub fn is_observational_only(&self) -> bool {
        match &self.outcome {
            ExerciseOutcome::Assessed(assessment) => !assessment.item_evidence.is_empty()
                && assessment.item_evidence.iter().all(|item| matches!(
                    item.capability,
                    crate::services::practice_session_store::AssessmentCapability::CharacterSequenceConstruction
                        | crate::services::practice_session_store::AssessmentCapability::PinyinReading
                        | crate::services::practice_session_store::AssessmentCapability::VocabularyMeaningMatching
                )),
            _ => false,
        }
    }

    pub fn is_construction_only(&self) -> bool {
        use crate::services::practice_session_store::AssessmentCapability;
        match &self.outcome {
            ExerciseOutcome::Assessed(assessment) => {
                !assessment.item_evidence.is_empty()
                    && assessment.item_evidence.iter().all(|item| {
                        matches!(
                            item.capability,
                            AssessmentCapability::CharacterSequenceConstruction
                        )
                    })
            }
            _ => false,
        }
    }
}
