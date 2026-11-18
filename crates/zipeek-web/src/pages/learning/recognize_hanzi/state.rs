use std::rc::Rc;
use yew::Reducible;

use super::RecognitionHit;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum InputKind {
    Empty,
    Drawing,
    Image,
    Mixed,
}

impl InputKind {
    pub(super) fn from_browser(value: zipeek_browser::recognition_workspace::RecognitionInputKind) -> Self {
        use zipeek_browser::recognition_workspace::RecognitionInputKind as BrowserKind;
        match value {
            BrowserKind::Empty => Self::Empty,
            BrowserKind::Drawing => Self::Drawing,
            BrowserKind::Image => Self::Image,
            BrowserKind::Mixed => Self::Mixed,
        }
    }

    pub(super) fn has_input(&self) -> bool {
        !matches!(self, Self::Empty)
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct RecognitionState {
    pub(super) input: InputKind,
    pub(super) can_undo: bool,
    pub(super) busy: bool,
    pub(super) error: Option<String>,
    pub(super) results: Vec<RecognitionHit>,
    pub(super) generation: u64,
}

impl Default for RecognitionState {
    fn default() -> Self {
        Self {
            input: InputKind::Empty,
            can_undo: false,
            busy: false,
            error: None,
            results: Vec::new(),
            generation: 0,
        }
    }
}

pub(super) enum RecognitionAction {
    InputChanged { input: InputKind, can_undo: bool },
    InputFailed(String),
    Begin,
    Completed { generation: u64, hits: Vec<RecognitionHit> },
    Failed { generation: u64, reason: String },
}

impl Reducible for RecognitionState {
    type Action = RecognitionAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            RecognitionAction::InputChanged { input, can_undo } => {
                next.generation = next.generation.wrapping_add(1);
                next.input = input;
                next.can_undo = can_undo;
                next.busy = false;
                next.error = None;
                next.results.clear();
            }
            RecognitionAction::InputFailed(reason) => {
                next.generation = next.generation.wrapping_add(1);
                next.busy = false;
                next.error = Some(reason);
                next.results.clear();
            },
            RecognitionAction::Begin => {
                if next.busy || !next.input.has_input() { return self; }
                next.generation = next.generation.wrapping_add(1);
                next.busy = true;
                next.error = None;
                next.results.clear();
            }
            RecognitionAction::Completed { generation, hits } => {
                if generation != next.generation || !next.busy { return self; }
                next.busy = false;
                next.results = hits;
            }
            RecognitionAction::Failed { generation, reason } => {
                if generation != next.generation || !next.busy { return self; }
                next.busy = false;
                next.error = Some(format!("Recognition failed: {reason}"));
            }
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_results_cannot_replace_newer_input() {
        let state = Rc::new(RecognitionState::default());
        let state = state.reduce(RecognitionAction::InputChanged { input: InputKind::Drawing, can_undo: true });
        let state = state.reduce(RecognitionAction::Begin);
        let stale = state.generation;
        let state = state.reduce(RecognitionAction::InputChanged { input: InputKind::Image, can_undo: false });
        let state = state.reduce(RecognitionAction::Completed { generation: stale, hits: vec![] });
        assert_eq!(state.input, InputKind::Image);
        assert!(state.results.is_empty());
        assert!(!state.busy);
    }

    #[test]
    fn workspace_error_invalidates_pending_recognition() {
        let state = Rc::new(RecognitionState::default());
        let state = state.reduce(RecognitionAction::InputChanged {
            input: InputKind::Drawing,
            can_undo: true,
        });
        let state = state.reduce(RecognitionAction::Begin);
        let previous_generation = state.generation;
        let state = state.reduce(RecognitionAction::InputFailed("Canvas rendering failed".to_owned()));
        assert!(!state.busy);
        assert!(state.error.as_deref().unwrap_or_default().contains("Canvas rendering"));
        assert_ne!(state.generation, previous_generation);
        let state = state.reduce(RecognitionAction::Completed {
            generation: previous_generation,
            hits: vec![],
        });
        assert!(state.results.is_empty());
        assert!(state.error.is_some());
    }

    #[test]
    fn recognition_has_explicit_transition_guards() {
        let state = Rc::new(RecognitionState::default());
        let state = state.reduce(RecognitionAction::Begin);
        assert!(!state.busy);
        let state = state.reduce(RecognitionAction::InputChanged { input: InputKind::Drawing, can_undo: true });
        let state = state.reduce(RecognitionAction::Begin);
        assert!(state.busy);
        let state = state.reduce(RecognitionAction::Failed { generation: state.generation, reason: "unavailable".to_owned() });
        assert!(!state.busy);
        assert!(state.error.as_deref().unwrap_or_default().contains("unavailable"));
    }
}
