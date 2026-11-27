use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AnswerPhase {
    Editing,
    Reviewed { correct: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AnswerState {
    pub phase: AnswerPhase,
    pub mistakes: u32,
    pub attempts: u32,
    pub retry_notice: bool,
}

impl Default for AnswerState {
    fn default() -> Self {
        Self {
            phase: AnswerPhase::Editing,
            mistakes: 0,
            attempts: 0,
            retry_notice: false,
        }
    }
}

pub(super) enum AnswerAction {
    Submit { correct: bool, retry: bool },
    Edit,
}

impl Reducible for AnswerState {
    type Action = AnswerAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AnswerAction::Edit => Rc::new(Self {
                retry_notice: false,
                ..(*self).clone()
            }),
            AnswerAction::Submit { correct, retry } => {
                if !matches!(self.phase, AnswerPhase::Editing) {
                    return self;
                }
                Rc::new(Self {
                    phase: if !correct && retry {
                        AnswerPhase::Editing
                    } else {
                        AnswerPhase::Reviewed { correct }
                    },
                    mistakes: self.mistakes.saturating_add(u32::from(!correct)),
                    attempts: self.attempts.saturating_add(1),
                    retry_notice: !correct && retry,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_preserves_mistakes_without_advancing() {
        let state = Rc::new(AnswerState::default()).reduce(AnswerAction::Submit {
            correct: false,
            retry: true,
        });
        assert_eq!(state.phase, AnswerPhase::Editing);
        assert_eq!(state.mistakes, 1);
        assert_eq!(state.attempts, 1);
        assert!(state.retry_notice);
        assert_eq!(state.attempts, 1);
        let state = state.reduce(AnswerAction::Submit {
            correct: true,
            retry: true,
        });
        assert_eq!(state.phase, AnswerPhase::Reviewed { correct: true });
        assert_eq!(state.attempts, 2);
        assert_eq!(state.mistakes, 1);
    }

    #[test]
    fn no_retry_reviews_incorrect_answer_once() {
        let state = Rc::new(AnswerState::default()).reduce(AnswerAction::Submit {
            correct: false,
            retry: false,
        });
        let state = state.reduce(AnswerAction::Submit {
            correct: false,
            retry: false,
        });
        assert_eq!(state.phase, AnswerPhase::Reviewed { correct: false });
        assert_eq!(state.attempts, 1);
        assert_eq!(state.mistakes, 1);
    }
}
