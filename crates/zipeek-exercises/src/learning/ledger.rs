use std::collections::{HashMap, HashSet};

use super::model::{AssessmentObservation, ExerciseAttempt};
use super::qualification::{EvidenceValidity, validate_observation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrectionKind {
    Invalidate,
    Supersede { replacement_observation_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    pub correction_id: String,
    pub observation_id: String,
    pub reason: String,
    pub corrected_at_ms: i64,
    pub kind: CorrectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalError {
    EmptySubmissionKey,
    DuplicateSubmission,
    DuplicateAttempt,
    DuplicateObservation,
    InvalidObservation(EvidenceValidity),
    UnknownObservation,
    DuplicateCorrection,
    AlreadyCorrected,
    InvalidCorrection,
}

#[derive(Debug, Default)]
pub struct EvidenceJournal {
    attempts: Vec<ExerciseAttempt>,
    observations: Vec<AssessmentObservation>,
    corrections: Vec<Correction>,
    submission_keys: HashSet<String>,
}

impl EvidenceJournal {
    pub fn append_attempt(
        &mut self,
        attempt: ExerciseAttempt,
        observations: Vec<AssessmentObservation>,
    ) -> Result<(), JournalError> {
        if attempt.submission_key.trim().is_empty() {
            return Err(JournalError::EmptySubmissionKey);
        }
        if self.submission_keys.contains(&attempt.submission_key) {
            let identical_attempt = self.attempts.iter().any(|entry| entry == &attempt);
            let previous_observations: Vec<_> = self
                .observations
                .iter()
                .filter(|entry| entry.attempt_id == attempt.attempt_id)
                .cloned()
                .collect();
            return if identical_attempt && previous_observations == observations {
                Ok(())
            } else {
                Err(JournalError::DuplicateSubmission)
            };
        }
        if self
            .attempts
            .iter()
            .any(|entry| entry.attempt_id == attempt.attempt_id)
        {
            return Err(JournalError::DuplicateAttempt);
        }
        if attempt.attempt_id.trim().is_empty()
            || attempt.session_id.trim().is_empty()
            || attempt.exercise_instance_id.trim().is_empty()
            || attempt.submitted_at_ms < attempt.started_at_ms
            || super::calendar::local_day_number(&attempt.learner_local_date).is_err()
        {
            return Err(JournalError::InvalidObservation(
                EvidenceValidity::InvalidIdentity,
            ));
        }
        let known_ids: HashSet<&str> = self
            .observations
            .iter()
            .map(|entry| entry.observation_id.as_str())
            .collect();
        let mut new_ids = HashSet::new();
        for observation in &observations {
            if known_ids.contains(observation.observation_id.as_str())
                || !new_ids.insert(observation.observation_id.as_str())
            {
                return Err(JournalError::DuplicateObservation);
            }
            let validity = validate_observation(&attempt, observation);
            if matches!(
                validity,
                EvidenceValidity::InvalidIdentity
                    | EvidenceValidity::InvalidAssessment
                    | EvidenceValidity::InvalidScore
            ) {
                return Err(JournalError::InvalidObservation(validity));
            }
        }
        self.submission_keys.insert(attempt.submission_key.clone());
        self.attempts.push(attempt);
        self.observations.extend(observations);
        Ok(())
    }

    pub fn append_correction(&mut self, correction: Correction) -> Result<(), JournalError> {
        if correction.correction_id.trim().is_empty()
            || correction.reason.trim().is_empty()
            || correction.corrected_at_ms < 0
        {
            return Err(JournalError::InvalidCorrection);
        }
        if self
            .corrections
            .iter()
            .any(|entry| entry.correction_id == correction.correction_id)
        {
            return Err(JournalError::DuplicateCorrection);
        }
        if self
            .corrections
            .iter()
            .any(|entry| entry.observation_id == correction.observation_id)
        {
            return Err(JournalError::AlreadyCorrected);
        }
        if !self
            .observations
            .iter()
            .any(|entry| entry.observation_id == correction.observation_id)
        {
            return Err(JournalError::UnknownObservation);
        }
        if let CorrectionKind::Supersede {
            replacement_observation_id,
        } = &correction.kind
        {
            let original = self
                .observations
                .iter()
                .find(|entry| entry.observation_id == correction.observation_id);
            let replacement = self
                .observations
                .iter()
                .find(|entry| &entry.observation_id == replacement_observation_id);
            if replacement_observation_id == &correction.observation_id
                || !matches!(
                    (original, replacement),
                    (Some(a), Some(b))
                        if a.target_kind == b.target_kind
                            && a.target_key == b.target_key
                            && a.skill == b.skill
                )
            {
                return Err(JournalError::InvalidCorrection);
            }
        }
        self.corrections.push(correction);
        Ok(())
    }

    pub fn active_observations(&self) -> Vec<&AssessmentObservation> {
        let invalidated: HashMap<&str, &CorrectionKind> = self
            .corrections
            .iter()
            .map(|entry| (entry.observation_id.as_str(), &entry.kind))
            .collect();
        self.observations
            .iter()
            .filter(|entry| !invalidated.contains_key(entry.observation_id.as_str()))
            .collect()
    }

    pub fn corrections(&self) -> &[Correction] {
        &self.corrections
    }

    pub fn attempts(&self) -> &[ExerciseAttempt] {
        &self.attempts
    }

    pub fn observations(&self) -> &[AssessmentObservation] {
        &self.observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::{
        AttemptCompletion, CueFlags, EvidenceKind, InputMode, ObservationOutcome, Skill, TargetKind,
    };

    fn fixture() -> (ExerciseAttempt, AssessmentObservation) {
        let attempt = ExerciseAttempt {
            attempt_id: "attempt-1".into(),
            submission_key: "submit-1".into(),
            session_id: "session-1".into(),
            exercise_instance_id: "round-1".into(),
            exercise_kind: "multiple_choice".into(),
            started_at_ms: 1,
            submitted_at_ms: 2,
            timezone_iana: "Europe/London".into(),
            learner_local_date: "2026-11-07".into(),
            input_mode: InputMode::Choice,
            cues: CueFlags {
                target_hanzi_visible: Some(true),
                ..CueFlags::default()
            },
            hints_used: 0,
            answer_revealed_before_submit: false,
            completion: AttemptCompletion::Submitted,
        };
        let observation = AssessmentObservation {
            observation_id: "observation-1".into(),
            attempt_id: "attempt-1".into(),
            target_kind: TargetKind::Vocabulary,
            target_key: "source:lexeme:reading:sense".into(),
            skill: Skill::Recognition,
            assessment_mode: "choice".into(),
            evidence_kind: EvidenceKind::Objective,
            outcome: ObservationOutcome::Pass,
            score: Some(1.0),
            evaluator_version: Some("v1".into()),
            rubric_id: None,
            rubric_version: None,
            model_id: None,
            evaluator_confidence: None,
            content_version: "dictionary-v1".into(),
            context_verified: true,
            target_was_obligatorily_tested: true,
        };
        (attempt, observation)
    }

    #[test]
    fn retries_cannot_double_count_a_submission() {
        let (attempt, observation) = fixture();
        let mut journal = EvidenceJournal::default();
        journal
            .append_attempt(attempt.clone(), vec![observation.clone()])
            .unwrap();
        assert_eq!(
            journal.append_attempt(attempt.clone(), vec![observation]),
            Ok(())
        );
        assert_eq!(
            journal.append_attempt(attempt, vec![]),
            Err(JournalError::DuplicateSubmission)
        );
        assert_eq!(journal.attempts().len(), 1);
    }

    #[test]
    fn contradictory_objective_grades_are_rejected_atomically() {
        let (attempt, mut observation) = fixture();
        observation.score = Some(0.0);
        let mut journal = EvidenceJournal::default();
        assert_eq!(
            journal.append_attempt(attempt, vec![observation]),
            Err(JournalError::InvalidObservation(
                EvidenceValidity::InvalidScore
            ))
        );
        assert!(journal.attempts().is_empty());
        assert!(journal.observations().is_empty());
    }

    #[test]
    fn invalid_batch_is_atomic_in_memory() {
        let (attempt, mut observation) = fixture();
        observation.score = Some(2.0);
        let mut journal = EvidenceJournal::default();
        assert_eq!(
            journal.append_attempt(attempt, vec![observation]),
            Err(JournalError::InvalidObservation(
                EvidenceValidity::InvalidScore
            ))
        );
        assert!(journal.attempts().is_empty());
        assert!(journal.observations().is_empty());
    }

    #[test]
    fn correction_does_not_modify_original_observation() {
        let (attempt, observation) = fixture();
        let mut journal = EvidenceJournal::default();
        journal.append_attempt(attempt, vec![observation]).unwrap();
        journal
            .append_correction(Correction {
                correction_id: "correction-1".into(),
                observation_id: "observation-1".into(),
                reason: "misconfigured prompt".into(),
                corrected_at_ms: 3,
                kind: CorrectionKind::Invalidate,
            })
            .unwrap();
        assert_eq!(journal.observations().len(), 1);
        assert!(journal.active_observations().is_empty());
    }

    #[test]
    fn invalid_local_day_cannot_enter_the_evidence_journal() {
        let (mut attempt, observation) = fixture();
        attempt.learner_local_date = "2026-02-30".into();
        let mut journal = EvidenceJournal::default();
        assert_eq!(
            journal.append_attempt(attempt, vec![observation]),
            Err(JournalError::InvalidObservation(
                EvidenceValidity::InvalidIdentity
            ))
        );
        assert!(journal.attempts().is_empty());
        assert!(journal.active_observations().is_empty());
    }

    #[test]
    fn corrections_cannot_replace_different_targets_or_skills() {
        let (attempt, original) = fixture();
        let mut journal = EvidenceJournal::default();
        journal
            .append_attempt(attempt.clone(), vec![original])
            .unwrap();
        let mut another_attempt = attempt;
        another_attempt.attempt_id = "attempt-2".into();
        another_attempt.submission_key = "submit-2".into();
        let (_, mut replacement) = fixture();
        replacement.observation_id = "observation-2".into();
        replacement.attempt_id = another_attempt.attempt_id.clone();
        replacement.target_key = "different-target".into();
        journal
            .append_attempt(another_attempt, vec![replacement])
            .unwrap();
        assert_eq!(
            journal.append_correction(Correction {
                correction_id: "correction-1".into(),
                observation_id: "observation-1".into(),
                reason: "invalid key".into(),
                corrected_at_ms: 3,
                kind: CorrectionKind::Supersede {
                    replacement_observation_id: "observation-2".into(),
                },
            }),
            Err(JournalError::InvalidCorrection)
        );
        assert_eq!(journal.active_observations().len(), 2);
    }
}
