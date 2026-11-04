use crate::{contexts::database::DatabaseStatus, services::database::DatabaseInfo};
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatabaseState {
    pub request: u64,
    pub status: DatabaseStatus,
}

pub(crate) enum DatabaseAction {
    Retry(u64),
    Opened(u64, DatabaseInfo),
    Failed(u64, String),
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self {
            request: 0,
            status: DatabaseStatus::Loading,
        }
    }
}

impl Reducible for DatabaseState {
    type Action = DatabaseAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            DatabaseAction::Retry(request) => Self {
                request,
                status: DatabaseStatus::Loading,
            }
            .into(),
            DatabaseAction::Opened(request, info)
                if request == self.request && self.status == DatabaseStatus::Loading =>
            {
                Self {
                    request,
                    status: DatabaseStatus::Ready(info),
                }
                .into()
            }
            DatabaseAction::Failed(request, error)
                if request == self.request && self.status == DatabaseStatus::Loading =>
            {
                Self {
                    request,
                    status: DatabaseStatus::Failed(error),
                }
                .into()
            }
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> DatabaseInfo {
        DatabaseInfo {
            engine: "v1.4.3".into(),
            row_count: 12100,
        }
    }

    #[test]
    fn retries_reject_old_successes_and_failures() {
        let state = Rc::new(DatabaseState::default()).reduce(DatabaseAction::Retry(1));
        let state = state.reduce(DatabaseAction::Retry(2));
        let stale_success = state.clone().reduce(DatabaseAction::Opened(1, metadata()));
        let stale_failure = state
            .clone()
            .reduce(DatabaseAction::Failed(0, "late".into()));
        assert!(Rc::ptr_eq(&state, &stale_success));
        assert!(Rc::ptr_eq(&state, &stale_failure));
        let ready = state.reduce(DatabaseAction::Opened(2, metadata()));
        assert_eq!(ready.status, DatabaseStatus::Ready(metadata()));
    }

    #[test]
    fn settled_attempt_ignores_duplicate_completion_and_retry_clears_failure() {
        let failed =
            Rc::new(DatabaseState::default()).reduce(DatabaseAction::Failed(0, "offline".into()));
        assert_eq!(failed.status, DatabaseStatus::Failed("offline".into()));
        let duplicate = failed.clone().reduce(DatabaseAction::Opened(0, metadata()));
        assert!(Rc::ptr_eq(&failed, &duplicate));
        let pending = failed.reduce(DatabaseAction::Retry(1));
        assert_eq!(pending.status, DatabaseStatus::Loading);
        let ready = pending.reduce(DatabaseAction::Opened(1, metadata()));
        let duplicate = ready
            .clone()
            .reduce(DatabaseAction::Failed(1, "late".into()));
        assert!(Rc::ptr_eq(&ready, &duplicate));
    }
}
