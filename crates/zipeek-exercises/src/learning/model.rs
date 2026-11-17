#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Skill {
    Recognition,
    Listening,
    Pronunciation,
    Typing,
    GrammarComprehension,
    GrammarProduction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Vocabulary,
    Grammar,
}

impl Skill {
    pub fn belongs_to(self, target: TargetKind) -> bool {
        match target {
            TargetKind::Vocabulary => matches!(
                self,
                Self::Recognition | Self::Listening | Self::Pronunciation | Self::Typing
            ),
            TargetKind::Grammar => {
                matches!(self, Self::GrammarComprehension | Self::GrammarProduction)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Choice,
    Spoken,
    HanziIme,
    HanziDirect,
    Handwriting,
    PinyinText,
    FreeText,
    Copying,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    Objective,
    HumanRated,
    ModelRated,
    SelfReport,
    PracticeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationOutcome {
    Pass,
    Partial,
    Fail,
    Unscorable,
    NotTested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptCompletion {
    Submitted,
    Skipped,
    Abandoned,
}

impl AttemptCompletion {
    pub fn contributes_assessed_evidence(self) -> bool {
        matches!(self, Self::Submitted)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CueFlags {
    pub target_hanzi_visible: Option<bool>,
    pub target_pinyin_visible: Option<bool>,
    pub target_translation_visible: Option<bool>,
    pub audio_prompt_available: Option<bool>,
    pub ime_candidates_visible: Option<bool>,
    pub autocorrect_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseAttempt {
    pub attempt_id: String,
    pub submission_key: String,
    pub session_id: String,
    pub exercise_instance_id: String,
    pub exercise_kind: String,
    pub started_at_ms: i64,
    pub submitted_at_ms: i64,
    pub timezone_iana: String,
    pub learner_local_date: String,
    pub input_mode: InputMode,
    pub cues: CueFlags,
    pub hints_used: u32,
    pub answer_revealed_before_submit: bool,
    pub completion: AttemptCompletion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssessmentObservation {
    pub observation_id: String,
    pub attempt_id: String,
    pub target_kind: TargetKind,
    pub target_key: String,
    pub skill: Skill,
    pub assessment_mode: String,
    pub evidence_kind: EvidenceKind,
    pub outcome: ObservationOutcome,
    pub score: Option<f64>,
    pub evaluator_version: Option<String>,
    pub rubric_id: Option<String>,
    pub rubric_version: Option<String>,
    pub model_id: Option<String>,
    pub evaluator_confidence: Option<f64>,
    pub content_version: String,
    pub context_verified: bool,
    pub target_was_obligatorily_tested: bool,
}

impl AssessmentObservation {
    pub fn is_targeted_to(&self, target: TargetKind, skill: Skill) -> bool {
        self.target_kind == target && self.skill == skill && skill.belongs_to(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_observations_cannot_masquerade_as_vocabulary_skills() {
        let grammar = AssessmentObservation {
            observation_id: "o".into(),
            attempt_id: "a".into(),
            target_kind: TargetKind::Grammar,
            target_key: "grammar:ba".into(),
            skill: Skill::GrammarProduction,
            assessment_mode: "sentence".into(),
            evidence_kind: EvidenceKind::PracticeOnly,
            outcome: ObservationOutcome::NotTested,
            score: None,
            evaluator_version: None,
            rubric_id: None,
            rubric_version: None,
            model_id: None,
            evaluator_confidence: None,
            content_version: "v1".into(),
            context_verified: false,
            target_was_obligatorily_tested: false,
        };
        assert!(grammar.is_targeted_to(TargetKind::Grammar, Skill::GrammarProduction));
        assert!(!grammar.is_targeted_to(TargetKind::Vocabulary, Skill::GrammarProduction));
        assert!(!grammar.is_targeted_to(TargetKind::Grammar, Skill::Recognition));
    }
    #[test]
    fn only_submitted_attempts_are_assessment_candidates() {
        assert!(AttemptCompletion::Submitted.contributes_assessed_evidence());
        assert!(!AttemptCompletion::Skipped.contributes_assessed_evidence());
        assert!(!AttemptCompletion::Abandoned.contributes_assessed_evidence());
    }
}
