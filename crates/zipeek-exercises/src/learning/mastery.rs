use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::calendar::local_day_number;
use super::ledger::EvidenceJournal;
use super::model::{ObservationOutcome, Skill};
use super::policy::{MasteryPolicy, SkillState};
use super::qualification::{EvidenceValidity, validate_observation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillProjection {
    pub state: SkillState,
    pub successful_days: usize,
    pub qualifying_successes: usize,
    pub qualifying_failures: usize,
    pub delayed_successes: usize,
    pub longest_success_gap_days: i64,
    pub best_calendar_streak_days: usize,
    pub latest_failure_ms: Option<i64>,
    pub latest_success_ms: Option<i64>,
    pub policy_version: &'static str,
}

pub fn rebuild_skill(
    journal: &EvidenceJournal,
    target_key: &str,
    skill: Skill,
    policy: MasteryPolicy,
) -> SkillProjection {
    assert!(policy.validate(), "invalid mastery policy");
    let attempts: HashMap<_, _> = journal
        .attempts()
        .iter()
        .map(|attempt| (attempt.attempt_id.as_str(), attempt))
        .collect();
    let mut events = BTreeMap::new();
    let mut unqualified = false;
    for observation in journal.active_observations() {
        if observation.target_key != target_key || observation.skill != skill {
            continue;
        }
        let Some(attempt) = attempts.get(observation.attempt_id.as_str()) else {
            continue;
        };
        let validity = validate_observation(attempt, observation);
        if validity != EvidenceValidity::Qualifying {
            unqualified = true;
            continue;
        }
        let Ok(date) = local_day_number(&attempt.learner_local_date) else {
            unqualified = true;
            continue;
        };
        events.insert(
            (attempt.submitted_at_ms, observation.observation_id.as_str()),
            (date, observation.outcome),
        );
    }
    let mut successful_days = BTreeSet::new();
    let mut failed_days = BTreeSet::new();
    let mut successes = 0;
    let mut failures = 0;
    let mut latest_success = None;
    let mut latest_failure = None;
    for ((time, _), (day, outcome)) in events {
        match outcome {
            ObservationOutcome::Pass => {
                successes += 1;
                successful_days.insert(day);
                latest_success = Some(time);
            }
            ObservationOutcome::Fail => {
                failures += 1;
                failed_days.insert(day);
                latest_failure = Some(time);
            }
            ObservationOutcome::Partial
            | ObservationOutcome::NotTested
            | ObservationOutcome::Unscorable => {}
        }
    }
    let credited_days: Vec<_> = successful_days.difference(&failed_days).copied().collect();
    let mut delayed = 0;
    let mut longest_gap = 0;
    let mut streak = 0;
    let mut best_streak = 0;
    let mut previous = None;
    for &day in &credited_days {
        if let Some(last) = previous {
            let gap = day - last;
            longest_gap = longest_gap.max(gap);
            if gap >= policy.minimum_delayed_success_gap_days {
                delayed += 1;
            }
            streak = if gap == 1 { streak + 1 } else { 1 };
        } else {
            streak = 1;
        }
        best_streak = best_streak.max(streak);
        previous = Some(day);
    }
    let state = if failures > 0 && latest_failure > latest_success {
        SkillState::NeedsReview
    } else if successes == 0 && failures == 0 {
        if unqualified {
            SkillState::InsufficientEvidence
        } else {
            SkillState::Unassessed
        }
    } else if credited_days.len() >= policy.minimum_distinct_success_days && delayed > 0 {
        SkillState::Learned
    } else {
        SkillState::Learning
    };
    SkillProjection {
        state,
        successful_days: credited_days.len(),
        qualifying_successes: successes,
        qualifying_failures: failures,
        delayed_successes: delayed,
        longest_success_gap_days: longest_gap,
        best_calendar_streak_days: best_streak,
        latest_failure_ms: latest_failure,
        latest_success_ms: latest_success,
        policy_version: MasteryPolicy::VERSION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::{
        AssessmentObservation, AttemptCompletion, CueFlags, EvidenceKind, ExerciseAttempt,
        InputMode, TargetKind,
    };

    fn submit(journal: &mut EvidenceJournal, id: usize, day: &str, outcome: ObservationOutcome) {
        let attempt = ExerciseAttempt {
            attempt_id: format!("attempt-{id}"),
            submission_key: format!("submission-{id}"),
            session_id: "session".into(),
            exercise_instance_id: format!("exercise-{id}"),
            exercise_kind: "multiple_choice".into(),
            started_at_ms: id as i64 * 10,
            submitted_at_ms: id as i64 * 10 + 1,
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
            attempt_id: attempt.attempt_id.clone(),
            target_kind: TargetKind::Vocabulary,
            target_key: "reading:sense".into(),
            skill: Skill::Recognition,
            assessment_mode: "choice".into(),
            evidence_kind: EvidenceKind::Objective,
            outcome,
            score: Some(if outcome == ObservationOutcome::Pass {
                1.0
            } else {
                0.0
            }),
            evaluator_version: Some("objective-v1".into()),
            rubric_id: None,
            rubric_version: None,
            model_id: None,
            evaluator_confidence: None,
            content_version: "catalog-1".into(),
            context_verified: true,
            target_was_obligatorily_tested: true,
        };
        journal.append_attempt(attempt, vec![observation]).unwrap();
    }

    #[test]
    fn seven_consecutive_success_days_do_not_prove_delayed_recall() {
        let mut journal = EvidenceJournal::default();
        for day in 1..=7 {
            submit(
                &mut journal,
                day,
                &format!("2026-11-{day:02}"),
                ObservationOutcome::Pass,
            );
        }
        let result = rebuild_skill(
            &journal,
            "reading:sense",
            Skill::Recognition,
            MasteryPolicy::default(),
        );
        assert_eq!(result.successful_days, 7);
        assert_eq!(result.delayed_successes, 0);
        assert_eq!(result.state, SkillState::Learning);
    }

    #[test]
    fn delayed_review_enables_learned_and_failure_regresses_only_target_skill() {
        let mut journal = EvidenceJournal::default();
        for day in 1..=7 {
            submit(
                &mut journal,
                day,
                &format!("2026-10-{day:02}"),
                ObservationOutcome::Pass,
            );
        }
        submit(&mut journal, 8, "2026-10-15", ObservationOutcome::Pass);
        let policy = MasteryPolicy::default();
        assert_eq!(
            rebuild_skill(&journal, "reading:sense", Skill::Recognition, policy).state,
            SkillState::Learned
        );
        submit(&mut journal, 9, "2026-10-16", ObservationOutcome::Fail);
        let result = rebuild_skill(&journal, "reading:sense", Skill::Recognition, policy);
        assert_eq!(result.state, SkillState::NeedsReview);
        assert_eq!(result.qualifying_successes, 8);
        assert_eq!(
            rebuild_skill(&journal, "reading:sense", Skill::Typing, policy).state,
            SkillState::Unassessed
        );
    }

    #[test]
    fn new_correct_attempt_after_failure_clears_review_state_without_instant_mastery() {
        let mut journal = EvidenceJournal::default();
        submit(&mut journal, 1, "2026-11-01", ObservationOutcome::Pass);
        submit(&mut journal, 2, "2026-11-02", ObservationOutcome::Fail);
        let policy = MasteryPolicy::default();
        assert_eq!(
            rebuild_skill(&journal, "reading:sense", Skill::Recognition, policy).state,
            SkillState::NeedsReview
        );
        submit(&mut journal, 3, "2026-11-03", ObservationOutcome::Pass);
        let projection = rebuild_skill(&journal, "reading:sense", Skill::Recognition, policy);
        assert_eq!(projection.state, SkillState::Learning);
        assert_eq!(projection.qualifying_failures, 1);
        assert_eq!(projection.successful_days, 2);
    }

    #[test]
    fn failure_on_same_day_removes_day_credit_but_not_historical_successes() {
        let mut journal = EvidenceJournal::default();
        submit(&mut journal, 1, "2026-11-01", ObservationOutcome::Pass);
        submit(&mut journal, 2, "2026-11-02", ObservationOutcome::Pass);
        submit(&mut journal, 3, "2026-11-02", ObservationOutcome::Fail);
        let result = rebuild_skill(
            &journal,
            "reading:sense",
            Skill::Recognition,
            MasteryPolicy::default(),
        );
        assert_eq!(result.successful_days, 1);
        assert_eq!(result.qualifying_successes, 2);
        assert_eq!(result.qualifying_failures, 1);
    }
    #[test]
    fn unqualified_attempts_never_create_a_learned_projection() {
        let mut reference = EvidenceJournal::default();
        submit(&mut reference, 1, "2026-11-01", ObservationOutcome::Pass);
        let attempt = reference.attempts()[0].clone();
        let mut observation = reference.observations()[0].clone();
        observation.evidence_kind = EvidenceKind::PracticeOnly;
        let mut journal = EvidenceJournal::default();
        journal.append_attempt(attempt, vec![observation]).unwrap();
        let result = rebuild_skill(
            &journal,
            "reading:sense",
            Skill::Recognition,
            MasteryPolicy::default(),
        );
        assert_eq!(result.qualifying_successes, 0);
        assert_eq!(result.delayed_successes, 0);
        assert_eq!(result.state, SkillState::InsufficientEvidence);
    }
}
