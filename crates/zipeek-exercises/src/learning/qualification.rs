use super::calendar::local_day_number;
use super::model::{
    AssessmentObservation, EvidenceKind, ExerciseAttempt, InputMode, ObservationOutcome, Skill,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceValidity {
    Qualifying,
    InvalidIdentity,
    InvalidAssessment,
    InvalidScore,
    InsufficientProvenance,
    NotAssessed,
    PracticeOnly,
    AnswerExposed,
    Hinted,
    IncompatibleInput,
    MissingContext,
}

pub fn validate_observation(
    attempt: &ExerciseAttempt,
    observation: &AssessmentObservation,
) -> EvidenceValidity {
    if attempt.attempt_id.trim().is_empty()
        || attempt.submission_key.trim().is_empty()
        || attempt.exercise_instance_id.trim().is_empty()
        || attempt.session_id.trim().is_empty()
        || attempt.timezone_iana.trim().is_empty()
        || local_day_number(&attempt.learner_local_date).is_err()
        || attempt.submitted_at_ms < attempt.started_at_ms
        || observation.observation_id.trim().is_empty()
        || observation.attempt_id != attempt.attempt_id
        || observation.target_key.trim().is_empty()
    {
        return EvidenceValidity::InvalidIdentity;
    }
    if !observation.skill.belongs_to(observation.target_kind)
        || observation.assessment_mode.trim().is_empty()
    {
        return EvidenceValidity::InvalidAssessment;
    }
    if (matches!(observation.evidence_kind, EvidenceKind::Objective)
        && matches!(
            observation.outcome,
            ObservationOutcome::Pass | ObservationOutcome::Fail
        )
        && observation.score.is_none())
        || observation
            .score
            .is_some_and(|score| !score.is_finite() || !(0.0..=1.0).contains(&score))
        || observation
            .evaluator_confidence
            .is_some_and(|confidence| !confidence.is_finite() || !(0.0..=1.0).contains(&confidence))
    {
        return EvidenceValidity::InvalidScore;
    }
    if matches!(observation.evidence_kind, EvidenceKind::Objective)
        && matches!(
            observation.outcome,
            ObservationOutcome::Pass | ObservationOutcome::Fail
        )
        && observation.score.is_some_and(|score| {
            (observation.outcome == ObservationOutcome::Pass && score < 1.0)
                || (observation.outcome == ObservationOutcome::Fail && score > 0.0)
        })
    {
        return EvidenceValidity::InvalidScore;
    }
    if !attempt.completion.contributes_assessed_evidence()
        || matches!(
            observation.outcome,
            ObservationOutcome::NotTested | ObservationOutcome::Unscorable
        )
    {
        return EvidenceValidity::NotAssessed;
    }
    if matches!(
        observation.evidence_kind,
        EvidenceKind::SelfReport | EvidenceKind::PracticeOnly
    ) || attempt.input_mode == InputMode::Copying
    {
        return EvidenceValidity::PracticeOnly;
    }
    if attempt.answer_revealed_before_submit {
        return EvidenceValidity::AnswerExposed;
    }
    if attempt.hints_used > 0 {
        return EvidenceValidity::Hinted;
    }
    if observation.content_version.trim().is_empty()
        || observation
            .evaluator_version
            .as_deref()
            .is_none_or(str::is_empty)
    {
        return EvidenceValidity::InsufficientProvenance;
    }
    match observation.skill {
        Skill::Recognition if attempt.cues.target_hanzi_visible != Some(true) => {
            return EvidenceValidity::IncompatibleInput;
        }
        Skill::Listening
            if attempt.cues.target_hanzi_visible != Some(false)
                || attempt.cues.target_pinyin_visible == Some(true)
                || attempt.cues.target_translation_visible == Some(true)
                || attempt.cues.audio_prompt_available != Some(true) =>
        {
            return EvidenceValidity::IncompatibleInput;
        }
        Skill::Typing
            if !matches!(
                attempt.input_mode,
                InputMode::HanziIme | InputMode::HanziDirect | InputMode::Handwriting
            ) || attempt.cues.target_hanzi_visible != Some(false)
                || attempt.cues.target_pinyin_visible == Some(true) =>
        {
            return EvidenceValidity::IncompatibleInput;
        }
        Skill::Typing
            if attempt.input_mode == InputMode::Handwriting
                && (!matches!(
                    observation.evidence_kind,
                    EvidenceKind::HumanRated | EvidenceKind::ModelRated
                ) || observation.rubric_id.as_deref().is_none_or(str::is_empty)
                    || observation
                        .rubric_version
                        .as_deref()
                        .is_none_or(str::is_empty)) =>
        {
            return EvidenceValidity::InsufficientProvenance;
        }
        Skill::Pronunciation
            if attempt.input_mode != InputMode::Spoken
                || !matches!(
                    observation.evidence_kind,
                    EvidenceKind::HumanRated | EvidenceKind::ModelRated
                )
                || observation.rubric_id.as_deref().is_none_or(str::is_empty)
                || observation
                    .rubric_version
                    .as_deref()
                    .is_none_or(str::is_empty)
                || !observation.context_verified =>
        {
            return EvidenceValidity::MissingContext;
        }
        Skill::GrammarProduction | Skill::GrammarComprehension
            if !observation.target_was_obligatorily_tested
                || !observation.context_verified
                || observation.rubric_id.as_deref().is_none_or(str::is_empty)
                || observation
                    .rubric_version
                    .as_deref()
                    .is_none_or(str::is_empty) =>
        {
            return EvidenceValidity::MissingContext;
        }
        _ => {}
    }
    EvidenceValidity::Qualifying
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::{AttemptCompletion, CueFlags, TargetKind};

    fn attempt(mode: InputMode) -> ExerciseAttempt {
        ExerciseAttempt {
            attempt_id: "attempt-1".into(),
            submission_key: "submit-1".into(),
            session_id: "session-1".into(),
            exercise_instance_id: "round-1".into(),
            exercise_kind: "typed_word".into(),
            started_at_ms: 1,
            submitted_at_ms: 2,
            timezone_iana: "Europe/London".into(),
            learner_local_date: "2026-11-07".into(),
            input_mode: mode,
            cues: CueFlags {
                target_hanzi_visible: Some(false),
                ..CueFlags::default()
            },
            hints_used: 0,
            answer_revealed_before_submit: false,
            completion: AttemptCompletion::Submitted,
        }
    }

    fn observation(skill: Skill) -> AssessmentObservation {
        AssessmentObservation {
            observation_id: "observation-1".into(),
            attempt_id: "attempt-1".into(),
            target_kind: TargetKind::Vocabulary,
            target_key: "cedict:study:xue2xi2:sense-1".into(),
            skill,
            assessment_mode: "written_word".into(),
            evidence_kind: EvidenceKind::Objective,
            outcome: ObservationOutcome::Pass,
            score: Some(1.0),
            evaluator_version: Some("test-evaluator-v1".into()),
            rubric_id: None,
            rubric_version: None,
            model_id: None,
            evaluator_confidence: None,
            content_version: "dictionary-v1".into(),
            context_verified: false,
            target_was_obligatorily_tested: false,
        }
    }

    #[test]
    fn contradictory_objective_verdicts_do_not_earn_mastery() {
        let submitted = attempt(InputMode::HanziIme);
        let mut result = observation(Skill::Typing);
        result.score = Some(0.0);
        assert_eq!(
            validate_observation(&submitted, &result),
            EvidenceValidity::InvalidScore
        );
        result.outcome = ObservationOutcome::Fail;
        result.score = Some(1.0);
        assert_eq!(
            validate_observation(&submitted, &result),
            EvidenceValidity::InvalidScore
        );
        result.score = Some(0.0);
        assert_eq!(
            validate_observation(&submitted, &result),
            EvidenceValidity::Qualifying
        );
    }

    #[test]
    fn hanzi_ime_can_qualify_for_writing_but_pinyin_cannot() {
        let target = observation(Skill::Typing);
        assert_eq!(
            validate_observation(&attempt(InputMode::HanziIme), &target),
            EvidenceValidity::Qualifying
        );
        assert_eq!(
            validate_observation(&attempt(InputMode::PinyinText), &target),
            EvidenceValidity::IncompatibleInput
        );
        assert_eq!(
            validate_observation(&attempt(InputMode::Copying), &target),
            EvidenceValidity::PracticeOnly
        );
    }

    #[test]
    fn revealed_answers_and_hints_are_not_recall_evidence() {
        let target = observation(Skill::Typing);
        let mut submission = attempt(InputMode::HanziDirect);
        submission.answer_revealed_before_submit = true;
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::AnswerExposed
        );
        submission.answer_revealed_before_submit = false;
        submission.hints_used = 1;
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::Hinted
        );
    }

    #[test]
    fn handwriting_practice_only_is_not_qualifying_evidence() {
        let mut target = observation(Skill::Typing);
        target.evidence_kind = EvidenceKind::PracticeOnly;
        let submission = attempt(InputMode::Handwriting);
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::PracticeOnly
        );
        target.evidence_kind = EvidenceKind::SelfReport;
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::PracticeOnly
        );
    }

    #[test]
    fn handwriting_requires_verified_evaluation_rubric() {
        let mut target = observation(Skill::Typing);
        let submission = attempt(InputMode::Handwriting);
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::InsufficientProvenance
        );
        target.evidence_kind = EvidenceKind::ModelRated;
        target.rubric_id = Some("stroke-rubric".into());
        target.rubric_version = Some("v1".into());
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::Qualifying
        );
    }

    #[test]
    fn self_report_cannot_prove_pronunciation() {
        let mut submission = attempt(InputMode::Spoken);
        submission.cues.target_hanzi_visible = Some(true);
        let mut target = observation(Skill::Pronunciation);
        target.evidence_kind = EvidenceKind::SelfReport;
        target.context_verified = true;
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::PracticeOnly
        );
        target.evidence_kind = EvidenceKind::HumanRated;
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::MissingContext
        );
        target.rubric_id = Some("pronunciation-rubric".into());
        target.rubric_version = Some("v1".into());
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::Qualifying
        );
    }

    #[test]
    fn grammar_requires_an_obligatory_target_and_rubric() {
        let mut submission = attempt(InputMode::FreeText);
        submission.cues.target_hanzi_visible = Some(true);
        let mut target = observation(Skill::GrammarProduction);
        target.target_kind = TargetKind::Grammar;
        target.context_verified = true;
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::MissingContext
        );
        target.target_was_obligatorily_tested = true;
        target.rubric_id = Some("ba-pattern".into());
        target.rubric_version = Some("1".into());
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::Qualifying
        );
    }

    #[test]
    fn missing_audio_or_exposed_hanzi_does_not_prove_listening() {
        let mut submission = attempt(InputMode::Choice);
        let target = observation(Skill::Listening);
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::IncompatibleInput
        );
        submission.cues.audio_prompt_available = Some(true);
        assert_eq!(
            validate_observation(&submission, &target),
            EvidenceValidity::Qualifying
        );
    }

    #[test]
    fn fabricated_local_dates_cannot_qualify_for_mastery() {
        let target = observation(Skill::Typing);
        for date in ["2026-02-30", "2026-11-00", "2026-11-8", "", "2026-13-01"] {
            let mut submitted = attempt(InputMode::HanziIme);
            submitted.learner_local_date = date.into();
            assert_eq!(
                validate_observation(&submitted, &target),
                EvidenceValidity::InvalidIdentity,
                "{date}"
            );
        }
    }

    #[test]
    fn objective_verdicts_require_a_numeric_score() {
        let submission = attempt(InputMode::HanziIme);
        let mut result = observation(Skill::Typing);
        result.score = None;
        assert_eq!(
            validate_observation(&submission, &result),
            EvidenceValidity::InvalidScore
        );
        result.score = Some(1.0);
        assert_eq!(
            validate_observation(&submission, &result),
            EvidenceValidity::Qualifying
        );
    }
}
