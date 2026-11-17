use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::calendar::local_day_number;
use super::ledger::EvidenceJournal;
use super::model::{ObservationOutcome, Skill};
use super::qualification::{EvidenceValidity, validate_observation};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawSkillEvidence {
    pub observed_occurrences: usize,
    pub qualifying_successes: usize,
    pub qualifying_failures: usize,
    pub distinct_success_days: usize,
    pub last_success_ms: Option<i64>,
    pub last_failure_ms: Option<i64>,
}

pub fn raw_skill_evidence(
    journal: &EvidenceJournal,
    target_key: &str,
    skill: Skill,
) -> RawSkillEvidence {
    let attempts: HashMap<&str, _> = journal
        .attempts()
        .iter()
        .map(|attempt| (attempt.attempt_id.as_str(), attempt))
        .collect();
    let mut metrics = RawSkillEvidence::default();
    let mut days = BTreeSet::new();
    let mut ordered: BTreeMap<(i64, &str), _> = BTreeMap::new();
    for observation in journal.active_observations() {
        if observation.target_key == target_key && observation.skill == skill {
            if let Some(attempt) = attempts.get(observation.attempt_id.as_str()) {
                ordered.insert(
                    (attempt.submitted_at_ms, observation.observation_id.as_str()),
                    (attempt, observation),
                );
            }
        }
    }
    for (_, (attempt, observation)) in ordered {
        metrics.observed_occurrences += 1;
        if validate_observation(attempt, observation) != EvidenceValidity::Qualifying {
            continue;
        }
        let Ok(day) = local_day_number(&attempt.learner_local_date) else {
            continue;
        };
        match observation.outcome {
            ObservationOutcome::Pass => {
                metrics.qualifying_successes += 1;
                days.insert(day);
                metrics.last_success_ms = Some(attempt.submitted_at_ms);
            }
            ObservationOutcome::Fail => {
                metrics.qualifying_failures += 1;
                metrics.last_failure_ms = Some(attempt.submitted_at_ms);
            }
            ObservationOutcome::Partial
            | ObservationOutcome::Unscorable
            | ObservationOutcome::NotTested => {}
        }
    }
    metrics.distinct_success_days = days.len();
    metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::{
        AssessmentObservation, AttemptCompletion, CueFlags, EvidenceKind, ExerciseAttempt,
        InputMode, TargetKind,
    };

    fn record(journal: &mut EvidenceJournal, id: &str, day: &str, passed: bool) {
        let attempt = ExerciseAttempt {
            attempt_id: id.into(),
            submission_key: format!("submit-{id}"),
            session_id: "session".into(),
            exercise_instance_id: format!("round-{id}"),
            exercise_kind: "multiple_choice".into(),
            started_at_ms: 1,
            submitted_at_ms: 2 + journal.attempts().len() as i64,
            timezone_iana: "Europe/London".into(),
            learner_local_date: day.into(),
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
            observation_id: format!("observation-{id}"),
            attempt_id: id.into(),
            target_kind: TargetKind::Vocabulary,
            target_key: "lexeme-a".into(),
            skill: Skill::Recognition,
            assessment_mode: "choice".into(),
            evidence_kind: EvidenceKind::Objective,
            outcome: if passed {
                ObservationOutcome::Pass
            } else {
                ObservationOutcome::Fail
            },
            score: Some(if passed { 1.0 } else { 0.0 }),
            evaluator_version: Some("v1".into()),
            rubric_id: None,
            rubric_version: None,
            model_id: None,
            evaluator_confidence: None,
            content_version: "dictionary-v1".into(),
            context_verified: true,
            target_was_obligatorily_tested: true,
        };
        journal.append_attempt(attempt, vec![observation]).unwrap();
    }

    #[test]
    fn impossible_local_dates_cannot_inflate_raw_success_counts() {
        let mut journal = EvidenceJournal::default();
        record(&mut journal, "valid", "2026-11-01", true);
        assert!(local_day_number("2026-02-30").is_err());
        let metrics = raw_skill_evidence(&journal, "lexeme-a", Skill::Recognition);
        assert_eq!(metrics.observed_occurrences, 1);
        assert_eq!(metrics.qualifying_successes, 1);
        assert_eq!(metrics.distinct_success_days, 1);
    }

    #[test]
    fn success_days_are_distinct_and_failure_does_not_erase_history() {
        let mut journal = EvidenceJournal::default();
        record(&mut journal, "1", "2026-11-01", true);
        record(&mut journal, "2", "2026-11-01", true);
        record(&mut journal, "3", "2026-11-02", false);
        let result = raw_skill_evidence(&journal, "lexeme-a", Skill::Recognition);
        assert_eq!(result.qualifying_successes, 2);
        assert_eq!(result.distinct_success_days, 1);
        assert_eq!(result.qualifying_failures, 1);
        assert_eq!(
            raw_skill_evidence(&journal, "lexeme-a", Skill::Typing).observed_occurrences,
            0
        );
    }

    #[test]
    fn review_metrics_are_target_and_skill_scoped() {
        let mut journal = EvidenceJournal::default();
        record(&mut journal, "success", "2026-11-01", true);
        record(&mut journal, "failure", "2026-11-02", false);
        let recognition = raw_skill_evidence(&journal, "lexeme-a", Skill::Recognition);
        assert_eq!(recognition.qualifying_successes, 1);
        assert_eq!(recognition.qualifying_failures, 1);
        assert_eq!(
            raw_skill_evidence(&journal, "lexeme-b", Skill::Recognition),
            RawSkillEvidence::default()
        );
        assert_eq!(
            raw_skill_evidence(&journal, "lexeme-a", Skill::Listening),
            RawSkillEvidence::default()
        );
    }

    #[test]
    fn duplicate_submission_replay_does_not_inflate_raw_evidence() {
        let mut journal = EvidenceJournal::default();
        record(&mut journal, "one", "2026-11-01", true);
        let attempt = journal.attempts()[0].clone();
        let observation = journal.observations()[0].clone();
        journal.append_attempt(attempt, vec![observation]).unwrap();
        let snapshot = raw_skill_evidence(&journal, "lexeme-a", Skill::Recognition);
        let restored = raw_skill_evidence(&journal, "lexeme-a", Skill::Recognition);
        assert_eq!(snapshot, restored);
        assert_eq!(snapshot.observed_occurrences, 1);
        assert_eq!(snapshot.qualifying_successes, 1);
    }
}
