use serde::{Deserialize, Serialize};

/// An evaluator's actual verdict. Never derive a partial verdict from a
/// completion-only callback, an estimated score, or assistance alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExerciseVerdict {
    Correct,
    PartiallyCorrect,
    Incorrect,
}

/// Evidence supplied by the exercise evaluator at final submission.
/// Counts describe the entire attempt history, not only the last retry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssessmentCapability {
    CharacterSequenceConstruction,
    PinyinReading,
    VocabularyMeaningMatching,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssessmentEvidence {
    pub capability: AssessmentCapability,
    pub target: String,
    pub verdict: ExerciseVerdict,
    pub attempts: u16,
    pub mistakes: u16,
    pub hints: u16,
    pub assisted: bool,
}

impl AssessmentEvidence {
    pub fn valid(&self) -> bool {
        !self.target.trim().is_empty()
            && self.target.len() <= 128
            && !self.target.chars().any(char::is_control)
            && self.attempts > 0
            && self.mistakes <= self.attempts
            && (!matches!(self.verdict, ExerciseVerdict::Correct)
                || (self.mistakes == 0 && self.hints == 0 && !self.assisted))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExerciseAssessment {
    pub verdict: ExerciseVerdict,
    pub attempts: u16,
    pub mistakes: u16,
    pub hints: u16,
    pub assisted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_evidence: Vec<AssessmentEvidence>,
}

impl ExerciseAssessment {
    pub fn binary(correct: bool) -> Self {
        Self {
            verdict: if correct {
                ExerciseVerdict::Correct
            } else {
                ExerciseVerdict::Incorrect
            },
            attempts: 1,
            mistakes: u16::from(!correct),
            hints: 0,
            assisted: false,
            item_evidence: Vec::new(),
        }
    }

    pub fn valid(&self) -> bool {
        self.attempts > 0
            && self.mistakes <= self.attempts
            && self.item_evidence.len() <= 160
            && self.item_evidence.iter().all(AssessmentEvidence::valid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExerciseOutcome {
    /// New typed result; includes independently assessed partial correctness.
    Assessed(ExerciseAssessment),
    /// Legacy binary results remain readable in persisted sessions.
    Scored {
        earned: u16,
        possible: u16,
    },
    /// A completion signal does not imply a correctness verdict.
    Completed,
    Skipped,
}

impl ExerciseOutcome {
    pub fn verdict(&self) -> Option<ExerciseVerdict> {
        match self {
            Self::Assessed(assessment) => Some(assessment.verdict),
            Self::Scored {
                earned: 1,
                possible: 1,
            } => Some(ExerciseVerdict::Correct),
            Self::Scored {
                earned: 0,
                possible: 1,
            } => Some(ExerciseVerdict::Incorrect),
            _ => None,
        }
    }

    pub fn is_completed(&self) -> bool {
        !matches!(self, Self::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_success_cannot_contain_mistakes_or_assistance() {
        let mut evidence = AssessmentEvidence {
            capability: AssessmentCapability::CharacterSequenceConstruction,
            target: "朋友".into(),
            verdict: ExerciseVerdict::Correct,
            attempts: 1,
            mistakes: 0,
            hints: 0,
            assisted: false,
        };
        assert!(evidence.valid());
        evidence.mistakes = 1;
        assert!(!evidence.valid());
        evidence.verdict = ExerciseVerdict::PartiallyCorrect;
        assert!(evidence.valid());
        evidence.target = " ".into();
        assert!(!evidence.valid());
    }

    #[test]
    fn new_observational_capabilities_do_not_claim_knowledge_dimensions() {
        for capability in [
            AssessmentCapability::PinyinReading,
            AssessmentCapability::VocabularyMeaningMatching,
        ] {
            let evidence = AssessmentEvidence {
                capability,
                target: "朋友".into(),
                verdict: ExerciseVerdict::PartiallyCorrect,
                attempts: 2,
                mistakes: 1,
                hints: 0,
                assisted: false,
            };
            assert!(evidence.valid());
            let json = serde_json::to_string(&evidence).unwrap();
            assert!(!json.contains("knowledge_dimension"));
            assert_eq!(
                serde_json::from_str::<AssessmentEvidence>(&json).unwrap(),
                evidence
            );
        }
        let mut assessment = ExerciseAssessment::binary(true);
        assessment.mistakes = 2;
        assert!(!assessment.valid());
    }

    #[test]
    fn extended_vocabulary_evidence_preserves_twenty_eight_item_rounds() {
        let mut assessment = ExerciseAssessment::binary(true);
        for _ in 0..160 {
            assessment.item_evidence.push(AssessmentEvidence {
                capability: AssessmentCapability::VocabularyMeaningMatching,
                target: "朋友".to_owned(),
                verdict: ExerciseVerdict::Correct,
                attempts: 1,
                mistakes: 0,
                hints: 0,
                assisted: false,
            });
        }
        assert!(assessment.valid());
        assessment
            .item_evidence
            .push(assessment.item_evidence[0].clone());
        assert!(!assessment.valid());
    }

    #[test]
    fn item_evidence_round_trips_with_an_existing_assessment() {
        let mut assessment = ExerciseAssessment::binary(true);
        assessment.item_evidence.push(AssessmentEvidence {
            capability: AssessmentCapability::CharacterSequenceConstruction,
            target: "好".into(),
            verdict: ExerciseVerdict::Correct,
            attempts: 1,
            mistakes: 0,
            hints: 0,
            assisted: false,
        });
        let stored = serde_json::to_string(&assessment).unwrap();
        assert_eq!(
            serde_json::from_str::<ExerciseAssessment>(&stored).unwrap(),
            assessment
        );
        assert!(assessment.valid());
        assert!(!stored.contains("knowledge_dimension"));
    }
}
