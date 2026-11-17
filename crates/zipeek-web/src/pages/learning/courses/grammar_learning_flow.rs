pub use zipeek_exercises::grammar_learning::{LearningPhase, PracticeAction, can_advance, choice_indices, step_at, step_count};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PracticeState(pub zipeek_exercises::grammar_learning::PracticeState);

impl std::ops::Deref for PracticeState {
    type Target = zipeek_exercises::grammar_learning::PracticeState;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl PracticeState {
    pub fn new(point_count: usize) -> Self {
        Self(zipeek_exercises::grammar_learning::PracticeState::new(point_count))
    }
}

impl yew::Reducible for PracticeState {
    type Action = PracticeAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut next = (*self).clone();
        if next.0.apply(action) { std::rc::Rc::new(next) } else { self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yew::Reducible;

    #[test]
    fn reducer_reuses_pure_transition_invariants() {
        let state = std::rc::Rc::new(PracticeState::new(5));
        let state = state.reduce(PracticeAction::Reveal(0));
        let state = state.reduce(PracticeAction::Choose { point: 0, choice: 0 });
        assert!(state.revealed[0]);
        assert_eq!(state.answers[0], Some(0));
    }
}
