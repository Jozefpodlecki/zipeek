use std::rc::Rc;

use yew::prelude::*;

use crate::services::practice_session_store::{PracticeSession, PracticeSessionError};

#[derive(Clone, PartialEq, Eq, Default)]
pub(super) struct SessionState {
    pub snapshot: Option<Result<PracticeSession, PracticeSessionError>>,
}

pub(super) enum SessionAction {
    Replace(Option<Result<PracticeSession, PracticeSessionError>>),
}

impl Reducible for SessionState {
    type Action = SessionAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            SessionAction::Replace(snapshot) => Rc::new(Self { snapshot }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_reducer_handles_restore_and_reset() {
        let initial = Rc::new(SessionState::default());
        let restored = initial.reduce(SessionAction::Replace(Some(Err(
            PracticeSessionError::NotFound,
        ))));
        assert_eq!(restored.snapshot, Some(Err(PracticeSessionError::NotFound)));
        let reset = restored.reduce(SessionAction::Replace(None));
        assert!(reset.snapshot.is_none());
    }
}
