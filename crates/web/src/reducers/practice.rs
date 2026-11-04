use crate::services::practice_loader::PreparedPractice;
use std::rc::Rc;
use yew::Reducible;
use zipeek_core::practice::{ExerciseVariant, ParsedPracticeQuery};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PracticeRequest {
    pub variant: ExerciseVariant,
    pub query: ParsedPracticeQuery,
    pub attempt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PracticeStatus {
    Loading,
    Ready(Rc<PreparedPractice>),
    Failed(String),
    InsufficientPool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PracticeState {
    pub request: PracticeRequest,
    pub status: PracticeStatus,
}

pub(crate) enum PracticeAction {
    Begin(PracticeRequest),
    Finished(PracticeRequest, PracticeStatus),
}

impl Reducible for PracticeState {
    type Action = PracticeAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            PracticeAction::Begin(request) => Self {
                request,
                status: PracticeStatus::Loading,
            }
            .into(),
            PracticeAction::Finished(request, status)
                if request == self.request && self.status == PracticeStatus::Loading =>
            {
                Self { request, status }.into()
            }
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(attempt: u64, offset: &str) -> PracticeRequest {
        PracticeRequest {
            variant: ExerciseVariant::EngPinyin,
            query: ParsedPracticeQuery::from_pairs([("offset", offset)]),
            attempt,
        }
    }

    #[test]
    fn route_changes_and_retry_reject_old_and_duplicate_failures() {
        let first = request(0, "0");
        let next = request(0, "4");
        let state = Rc::new(PracticeState {
            request: first.clone(),
            status: PracticeStatus::Loading,
        });
        let state = state.reduce(PracticeAction::Begin(next.clone()));
        let stale = state.clone().reduce(PracticeAction::Finished(
            first,
            PracticeStatus::Failed("old".into()),
        ));
        assert!(Rc::ptr_eq(&state, &stale));
        let failed = state.reduce(PracticeAction::Finished(
            next.clone(),
            PracticeStatus::Failed("offline".into()),
        ));
        let retried = failed.reduce(PracticeAction::Begin(request(1, "4")));
        assert_eq!(retried.status, PracticeStatus::Loading);
        let stale = retried.clone().reduce(PracticeAction::Finished(
            next,
            PracticeStatus::Failed("old retry".into()),
        ));
        assert!(Rc::ptr_eq(&retried, &stale));
        let settled = retried.reduce(PracticeAction::Finished(
            request(1, "4"),
            PracticeStatus::Failed("failed again".into()),
        ));
        let duplicate = settled.clone().reduce(PracticeAction::Finished(
            request(1, "4"),
            PracticeStatus::Failed("duplicate".into()),
        ));
        assert!(Rc::ptr_eq(&settled, &duplicate));
    }
}
