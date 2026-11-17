use std::collections::{BTreeMap, BTreeSet};

use super::ledger::EvidenceJournal;
use super::mastery::{SkillProjection, rebuild_skill};
use super::model::{Skill, TargetKind};
use super::policy::MasteryPolicy;

pub fn rebuild_all(
    journal: &EvidenceJournal,
    policy: MasteryPolicy,
) -> BTreeMap<(String, &'static str), SkillProjection> {
    let targets: BTreeSet<_> = journal
        .active_observations()
        .into_iter()
        .filter(|observation| observation.skill.belongs_to(observation.target_kind))
        .map(|observation| (observation.target_key.clone(), observation.skill))
        .collect();
    targets
        .into_iter()
        .map(|(key, skill)| {
            let name = match skill {
                Skill::Recognition => "recognition",
                Skill::Listening => "listening",
                Skill::Pronunciation => "pronunciation",
                Skill::Typing => "typing",
                Skill::GrammarComprehension => "grammar_comprehension",
                Skill::GrammarProduction => "grammar_production",
            };
            let projection = rebuild_skill(journal, &key, skill, policy);
            ((key, name), projection)
        })
        .collect()
}

pub fn skills_for_target(kind: TargetKind) -> &'static [Skill] {
    match kind {
        TargetKind::Vocabulary => &[
            Skill::Recognition,
            Skill::Listening,
            Skill::Pronunciation,
            Skill::Typing,
        ],
        TargetKind::Grammar => &[Skill::GrammarComprehension, Skill::GrammarProduction],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_kinds_expose_only_their_supported_skills() {
        assert_eq!(skills_for_target(TargetKind::Vocabulary).len(), 4);
        assert_eq!(skills_for_target(TargetKind::Grammar).len(), 2);
        for target in [TargetKind::Vocabulary, TargetKind::Grammar] {
            let skills = skills_for_target(target);
            assert!(skills.iter().all(|skill| skill.belongs_to(target)));
            assert!(
                skills
                    .iter()
                    .enumerate()
                    .all(|(i, skill)| !skills[i + 1..].contains(skill))
            );
        }
    }

    #[test]
    fn empty_journal_has_no_phantom_projections() {
        let journal = EvidenceJournal::default();
        assert!(rebuild_all(&journal, MasteryPolicy::default()).is_empty());
    }
    #[test]
    fn rebuilding_practice_only_events_never_produces_learned_skills() {
        use crate::learning::{
            AssessmentObservation, AttemptCompletion, CueFlags, EvidenceKind, ExerciseAttempt,
            InputMode, ObservationOutcome, SkillState,
        };
        let attempt = ExerciseAttempt {
            attempt_id: "attempt-1".into(),
            submission_key: "submission-1".into(),
            session_id: "session-1".into(),
            exercise_instance_id: "practice-1".into(),
            exercise_kind: "speaking".into(),
            started_at_ms: 1,
            submitted_at_ms: 2,
            timezone_iana: "Europe/London".into(),
            learner_local_date: "2026-11-08".into(),
            input_mode: InputMode::Spoken,
            cues: CueFlags::default(),
            hints_used: 0,
            answer_revealed_before_submit: false,
            completion: AttemptCompletion::Submitted,
        };
        let observation = AssessmentObservation {
            observation_id: "observation-1".into(),
            attempt_id: "attempt-1".into(),
            target_kind: TargetKind::Vocabulary,
            target_key: "dictionary:example".into(),
            skill: Skill::Pronunciation,
            assessment_mode: "self_comparison".into(),
            evidence_kind: EvidenceKind::PracticeOnly,
            outcome: ObservationOutcome::Pass,
            score: Some(1.0),
            evaluator_version: None,
            rubric_id: None,
            rubric_version: None,
            model_id: None,
            evaluator_confidence: None,
            content_version: "practice-v1".into(),
            context_verified: false,
            target_was_obligatorily_tested: false,
        };
        let mut journal = EvidenceJournal::default();
        journal.append_attempt(attempt, vec![observation]).unwrap();
        let result = rebuild_all(&journal, MasteryPolicy::default());
        assert_eq!(result.len(), 1);
        let pronunciation = &result[&("dictionary:example".into(), "pronunciation")];
        assert_eq!(pronunciation.state, SkillState::InsufficientEvidence);
        assert_eq!(pronunciation.qualifying_successes, 0);
    }
}
