use super::model::{AssessmentObservation, ExerciseAttempt, InputMode, Skill};
use super::qualification::{EvidenceValidity, validate_observation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRequirement {
    IsolatedAllowed,
    PhraseRequired,
    SentenceRequired,
    NotMeaningfullyTestable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentedContext {
    Isolated,
    Phrase,
    Sentence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssessmentPolicy {
    pub context: ContextRequirement,
    pub policy_version: &'static str,
}

impl ContextRequirement {
    pub fn permits(self, presented: PresentedContext) -> bool {
        match self {
            Self::IsolatedAllowed => true,
            Self::PhraseRequired => presented != PresentedContext::Isolated,
            Self::SentenceRequired => presented == PresentedContext::Sentence,
            Self::NotMeaningfullyTestable => false,
        }
    }
}

impl AssessmentPolicy {
    pub const DEFAULT_VERSION: &'static str = "assessment-v1";

    pub fn evaluate(
        self,
        presented: PresentedContext,
        attempt: &ExerciseAttempt,
        observation: &AssessmentObservation,
    ) -> EvidenceValidity {
        if self.policy_version.trim().is_empty() {
            return EvidenceValidity::InsufficientProvenance;
        }
        if self.context == ContextRequirement::NotMeaningfullyTestable {
            return EvidenceValidity::NotAssessed;
        }
        if !self.context.permits(presented) {
            return EvidenceValidity::MissingContext;
        }
        if observation.skill == Skill::Typing && attempt.input_mode == InputMode::PinyinText {
            return EvidenceValidity::IncompatibleInput;
        }
        validate_observation(attempt, observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sentence_and_phrase_requirements_are_distinct() {
        assert!(ContextRequirement::IsolatedAllowed.permits(PresentedContext::Isolated));
        assert!(!ContextRequirement::PhraseRequired.permits(PresentedContext::Isolated));
        assert!(ContextRequirement::PhraseRequired.permits(PresentedContext::Phrase));
        assert!(ContextRequirement::PhraseRequired.permits(PresentedContext::Sentence));
        assert!(!ContextRequirement::SentenceRequired.permits(PresentedContext::Phrase));
        assert!(ContextRequirement::SentenceRequired.permits(PresentedContext::Sentence));
        assert!(!ContextRequirement::NotMeaningfullyTestable.permits(PresentedContext::Sentence));
    }

    #[test]
    fn contextual_requirement_does_not_assume_isolated_prompt_is_valid() {
        assert_eq!(
            ContextRequirement::SentenceRequired,
            ContextRequirement::SentenceRequired
        );
        assert_ne!(PresentedContext::Isolated, PresentedContext::Sentence);
        assert_eq!(AssessmentPolicy::DEFAULT_VERSION, "assessment-v1");
    }

    #[test]
    fn empty_policy_versions_cannot_claim_authoritative_assessment() {
        let invalid = AssessmentPolicy {
            context: ContextRequirement::IsolatedAllowed,
            policy_version: "  ",
        };
        assert_eq!(invalid.policy_version.trim(), "");
        assert!(ContextRequirement::NotMeaningfullyTestable != ContextRequirement::IsolatedAllowed);
    }

    #[test]
    fn context_requirements_never_promote_unassessed_prompts() {
        for context in [
            PresentedContext::Isolated,
            PresentedContext::Phrase,
            PresentedContext::Sentence,
        ] {
            assert!(!ContextRequirement::NotMeaningfullyTestable.permits(context));
        }
        assert!(!ContextRequirement::SentenceRequired.permits(PresentedContext::Isolated));
        assert!(!ContextRequirement::PhraseRequired.permits(PresentedContext::Isolated));
    }
}
