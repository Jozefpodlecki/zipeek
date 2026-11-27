use std::rc::Rc;
use yew::Reducible;

use crate::services::practice_session_store::AssessmentEvidence;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SessionProgress {
    pub round: usize,
    pub mistakes: u32,
    pub attempts: u32,
    pub correct_rounds: usize,
    pub completed: bool,
    pub item_evidence: Vec<AssessmentEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct VocabularyRoundResult {
    pub correct: bool,
    pub attempts: u32,
    pub mistakes: u32,
    pub item_evidence: Vec<AssessmentEvidence>,
}

impl VocabularyRoundResult {
    pub fn assessed(self) -> crate::services::practice_session_store::ExerciseAssessment {
        use crate::services::practice_session_store::{ExerciseAssessment, ExerciseVerdict};
        let hints = self
            .item_evidence
            .iter()
            .map(|item| u32::from(item.hints))
            .sum::<u32>()
            .min(u16::MAX as u32) as u16;
        let assisted = self.item_evidence.iter().any(|item| item.assisted);
        let partial_item = self
            .item_evidence
            .iter()
            .any(|item| !matches!(item.verdict, ExerciseVerdict::Correct));
        ExerciseAssessment {
            verdict: if !self.correct {
                ExerciseVerdict::Incorrect
            } else if self.mistakes > 0 || hints > 0 || assisted || partial_item {
                ExerciseVerdict::PartiallyCorrect
            } else {
                ExerciseVerdict::Correct
            },
            attempts: self.attempts.clamp(1, u16::MAX as u32) as u16,
            mistakes: self.mistakes.min(u16::MAX as u32) as u16,
            hints,
            assisted,
            item_evidence: self.item_evidence,
        }
    }
}

pub(super) enum ProgressAction {
    Reset,
    NextRound { result: VocabularyRoundResult },
    Finish { result: VocabularyRoundResult },
}

impl Reducible for SessionProgress {
    type Action = ProgressAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ProgressAction::Reset => Rc::new(Self::default()),
            ProgressAction::NextRound { result } if !self.completed => Rc::new(Self {
                round: self.round.saturating_add(1),
                mistakes: self.mistakes.saturating_add(result.mistakes),
                attempts: self.attempts.saturating_add(result.attempts),
                correct_rounds: self
                    .correct_rounds
                    .saturating_add(usize::from(result.correct)),
                completed: false,
                item_evidence: self
                    .item_evidence
                    .iter()
                    .cloned()
                    .chain(result.item_evidence)
                    .collect(),
            }),
            ProgressAction::Finish { result } if !self.completed => Rc::new(Self {
                round: self.round,
                mistakes: self.mistakes.saturating_add(result.mistakes),
                attempts: self.attempts.saturating_add(result.attempts),
                correct_rounds: self
                    .correct_rounds
                    .saturating_add(usize::from(result.correct)),
                completed: true,
                item_evidence: self
                    .item_evidence
                    .iter()
                    .cloned()
                    .chain(result.item_evidence)
                    .collect(),
            }),
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_assessment_preserves_final_verdict_and_observed_counts() {
        use crate::services::practice_session_store::ExerciseVerdict;
        let clean = VocabularyRoundResult {
            correct: true,
            attempts: 1,
            mistakes: 0,
            item_evidence: Vec::new(),
        }
        .assessed();
        assert_eq!(clean.verdict, ExerciseVerdict::Correct);
        assert_eq!((clean.attempts, clean.mistakes), (1, 0));
        let incorrect = VocabularyRoundResult {
            correct: false,
            attempts: 3,
            mistakes: 3,
            item_evidence: Vec::new(),
        }
        .assessed();
        assert_eq!(incorrect.verdict, ExerciseVerdict::Incorrect);
        assert_eq!((incorrect.attempts, incorrect.mistakes), (3, 3));
    }

    #[test]
    fn corrected_round_has_partial_verdict() {
        use crate::services::practice_session_store::ExerciseVerdict;
        let corrected = VocabularyRoundResult {
            correct: true,
            attempts: 2,
            mistakes: 1,
            item_evidence: Vec::new(),
        }
        .assessed();
        assert_eq!(corrected.verdict, ExerciseVerdict::PartiallyCorrect);
    }

    #[test]
    fn progress_totals_saturate_on_corrupt_or_extreme_counts() {
        let initial = Rc::new(SessionProgress {
            round: usize::MAX,
            mistakes: u32::MAX - 1,
            attempts: u32::MAX - 1,
            correct_rounds: 0,
            completed: false,
            item_evidence: Vec::new(),
        });
        let next = initial.reduce(ProgressAction::NextRound {
            result: VocabularyRoundResult {
                correct: true,
                attempts: 10,
                mistakes: 10,
                item_evidence: Vec::new(),
            },
        });
        assert_eq!(next.round, usize::MAX);
        assert_eq!(next.mistakes, u32::MAX);
        assert_eq!(
            next.reduce(ProgressAction::Finish {
                result: VocabularyRoundResult {
                    correct: false,
                    attempts: 1,
                    mistakes: 2,
                    item_evidence: Vec::new(),
                },
            })
            .mistakes,
            u32::MAX
        );
    }

    #[test]
    fn progress_is_monotonic_until_reset() {
        let state = Rc::new(SessionProgress::default()).reduce(ProgressAction::NextRound {
            result: VocabularyRoundResult {
                correct: true,
                attempts: 3,
                mistakes: 2,
                item_evidence: Vec::new(),
            },
        });
        assert_eq!((state.round, state.mistakes), (1, 2));
        let state = state.reduce(ProgressAction::Finish {
            result: VocabularyRoundResult {
                correct: false,
                attempts: 1,
                mistakes: 1,
                item_evidence: Vec::new(),
            },
        });
        assert!(state.completed);
        assert_eq!(state.mistakes, 3);
        let state = state.reduce(ProgressAction::NextRound {
            result: VocabularyRoundResult {
                correct: true,
                attempts: 5,
                mistakes: 4,
                item_evidence: Vec::new(),
            },
        });
        assert_eq!((state.round, state.mistakes), (1, 3));
        assert_eq!(
            *state.reduce(ProgressAction::Reset),
            SessionProgress::default()
        );
    }
}
