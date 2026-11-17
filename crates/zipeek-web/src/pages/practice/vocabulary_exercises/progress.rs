use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SessionProgress {
    pub round: usize,
    pub mistakes: u32,
    pub completed: bool,
}

pub(super) enum ProgressAction {
    Reset,
    NextRound { mistakes: u32 },
    Finish { mistakes: u32 },
}

impl Reducible for SessionProgress {
    type Action = ProgressAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ProgressAction::Reset => Rc::new(Self::default()),
            ProgressAction::NextRound { mistakes } if !self.completed => Rc::new(Self {
                round: self.round.saturating_add(1),
                mistakes: self.mistakes.saturating_add(mistakes),
                completed: false,
            }),
            ProgressAction::Finish { mistakes } if !self.completed => Rc::new(Self {
                round: self.round,
                mistakes: self.mistakes.saturating_add(mistakes),
                completed: true,
            }),
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_totals_saturate_on_corrupt_or_extreme_counts() {
        let initial = Rc::new(SessionProgress {
            round: usize::MAX,
            mistakes: u32::MAX - 1,
            completed: false,
        });
        let next = initial.reduce(ProgressAction::NextRound { mistakes: 10 });
        assert_eq!(next.round, usize::MAX);
        assert_eq!(next.mistakes, u32::MAX);
        assert_eq!(
            next.reduce(ProgressAction::Finish { mistakes: 2 }).mistakes,
            u32::MAX
        );
    }

    #[test]
    fn progress_is_monotonic_until_reset() {
        let state =
            Rc::new(SessionProgress::default()).reduce(ProgressAction::NextRound { mistakes: 2 });
        assert_eq!((state.round, state.mistakes), (1, 2));
        let state = state.reduce(ProgressAction::Finish { mistakes: 1 });
        assert!(state.completed);
        assert_eq!(state.mistakes, 3);
        let state = state.reduce(ProgressAction::NextRound { mistakes: 4 });
        assert_eq!((state.round, state.mistakes), (1, 3));
        assert_eq!(
            *state.reduce(ProgressAction::Reset),
            SessionProgress::default()
        );
    }
}
