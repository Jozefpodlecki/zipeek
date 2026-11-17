use std::{collections::BTreeSet, rc::Rc};
use yew::Reducible;
use zipeek_exercises::practice_draft::HskBand;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct CrosswordWorkflow {
    pub bands: BTreeSet<HskBand>,
    pub loading: bool,
    pub error: String,
}

impl Default for CrosswordWorkflow {
    fn default() -> Self {
        Self {
            bands: [HskBand::new(3, 1).expect("valid HSK band")]
                .into_iter()
                .collect(),
            loading: false,
            error: String::new(),
        }
    }
}

pub(super) enum WorkflowAction {
    ToggleBand(HskBand),
    Begin,
    Regenerate,
    Complete,
    Fail(String),
}

impl Reducible for CrosswordWorkflow {
    type Action = WorkflowAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            WorkflowAction::ToggleBand(band) if !next.loading => {
                if !next.bands.insert(band.clone()) {
                    next.bands.remove(&band);
                }
                next.error.clear();
            }
            WorkflowAction::Begin if !next.loading => {
                if next.bands.is_empty() {
                    next.error = "Select at least one HSK band.".to_owned();
                } else {
                    next.loading = true;
                    next.error.clear();
                }
            }
            WorkflowAction::Regenerate => {
                next.loading = true;
                next.error.clear();
            }
            WorkflowAction::Complete => {
                next.loading = false;
                next.error.clear();
            }
            WorkflowAction::Fail(message) => {
                next.loading = false;
                next.error = message;
            }
            _ => {}
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_transitions_are_consistent() {
        let initial = Rc::new(CrosswordWorkflow::default());
        assert_eq!(initial.bands.len(), 1);
        let loading = initial.reduce(WorkflowAction::Begin);
        assert!(loading.loading);
        let unchanged = loading.clone().reduce(WorkflowAction::ToggleBand(
            HskBand::new(2, 2).expect("valid band"),
        ));
        assert_eq!(unchanged.bands, loading.bands);
        let failed = unchanged.reduce(WorkflowAction::Fail("database unavailable".into()));
        assert!(!failed.loading);
        assert_eq!(failed.error, "database unavailable");
        let recovered = failed.reduce(WorkflowAction::Begin);
        assert!(recovered.loading);
        assert!(recovered.error.is_empty());
        assert!(!recovered.reduce(WorkflowAction::Complete).loading);
    }

    #[test]
    fn empty_selection_does_not_start_loading() {
        let band = HskBand::new(3, 1).expect("valid band");
        let empty = Rc::new(CrosswordWorkflow::default()).reduce(WorkflowAction::ToggleBand(band));
        let attempted = empty.reduce(WorkflowAction::Begin);
        assert!(!attempted.loading);
        assert!(!attempted.error.is_empty());
    }
}
