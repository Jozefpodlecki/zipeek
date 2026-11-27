use std::rc::Rc;
use yew::Reducible;
use zipeek_core::Word;

#[derive(Clone, PartialEq)]
pub enum RegionLoad {
    WaitingForDictionary,
    Loading,
    Ready(Rc<Vec<Word>>),
    Failed(String),
    DictionaryUnavailable(String),
}

#[derive(Clone, PartialEq)]
pub struct VocabularyState {
    pub selected: usize,
    pub request: u32,
    pub page: usize,
    pub load: RegionLoad,
}

pub enum VocabularyAction {
    Select(usize),
    Retry,
    DictionaryWaiting {
        selected: usize,
        request: u32,
    },
    DictionaryFailed {
        selected: usize,
        request: u32,
        message: String,
    },
    Loaded {
        selected: usize,
        request: u32,
        result: Result<Rc<Vec<Word>>, String>,
    },
    PreviousPage,
    NextPage,
}

impl Default for VocabularyState {
    fn default() -> Self {
        Self {
            selected: 0,
            request: 0,
            page: 0,
            load: RegionLoad::WaitingForDictionary,
        }
    }
}

impl VocabularyState {
    pub fn page_count(&self, page_size: usize) -> usize {
        match &self.load {
            RegionLoad::Ready(words) => words.len().div_ceil(page_size).max(1),
            _ => 1,
        }
    }
}

impl Reducible for VocabularyState {
    type Action = VocabularyAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            VocabularyAction::Select(index) => {
                if index >= 100 || index == next.selected {
                    return self;
                }
                next.selected = index;
                next.request = next.request.wrapping_add(1);
                next.page = 0;
                next.load = RegionLoad::Loading;
            }
            VocabularyAction::Retry => {
                next.request = next.request.wrapping_add(1);
                next.page = 0;
                next.load = RegionLoad::Loading;
            }
            VocabularyAction::DictionaryWaiting { selected, request } => {
                if selected != next.selected || request != next.request {
                    return self;
                }
                next.page = 0;
                next.load = RegionLoad::WaitingForDictionary;
            }
            VocabularyAction::DictionaryFailed {
                selected,
                request,
                message,
            } => {
                if selected != next.selected || request != next.request {
                    return self;
                }
                next.page = 0;
                next.load = RegionLoad::DictionaryUnavailable(message);
            }
            VocabularyAction::Loaded {
                selected,
                request,
                result,
            } => {
                if selected != next.selected || request != next.request {
                    return self;
                }
                next.page = 0;
                next.load = match result {
                    Ok(words) => RegionLoad::Ready(words),
                    Err(error) => RegionLoad::Failed(error),
                };
            }
            VocabularyAction::PreviousPage => {
                next.page = next.page.saturating_sub(1);
            }
            VocabularyAction::NextPage => {
                if next.page + 1 < next.page_count(super::word_explorer::PAGE_SIZE) {
                    next.page += 1;
                }
            }
        }
        next.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_region_discards_stale_load_and_resets_page() {
        let state = Rc::new(VocabularyState::default());
        let current = state.reduce(VocabularyAction::Select(7));
        assert_eq!(current.selected, 7);
        assert!(matches!(&current.load, RegionLoad::Loading));
        let stale = current.clone().reduce(VocabularyAction::Loaded {
            selected: 0,
            request: 0,
            result: Err("stale".into()),
        });
        assert!(*stale == *current);
        let ready = stale.reduce(VocabularyAction::Loaded {
            selected: 7,
            request: current.request,
            result: Err("offline".into()),
        });
        assert!(matches!(&ready.load, RegionLoad::Failed(error) if error == "offline"));
        let retry = ready.reduce(VocabularyAction::Retry);
        assert!(matches!(&retry.load, RegionLoad::Loading));
        assert_eq!(retry.page, 0);
        assert_ne!(retry.request, current.request);
    }

    #[test]
    fn pagination_never_advances_without_words() {
        let state = Rc::new(VocabularyState::default());
        assert_eq!(state.clone().reduce(VocabularyAction::NextPage).page, 0);
        assert_eq!(state.clone().reduce(VocabularyAction::PreviousPage).page, 0);
        assert_eq!(state.page_count(4), 1);
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::*;

    #[test]
    fn waits_for_dictionary_then_accepts_selected_region_only() {
        let state = Rc::new(VocabularyState::default());
        assert!(matches!(&state.load, RegionLoad::WaitingForDictionary));
        let selected = state.reduce(VocabularyAction::Select(9));
        let waiting = selected
            .clone()
            .reduce(VocabularyAction::DictionaryWaiting {
                selected: 9,
                request: selected.request,
            });
        assert!(matches!(&waiting.load, RegionLoad::WaitingForDictionary));
        let stale = waiting.clone().reduce(VocabularyAction::DictionaryFailed {
            selected: 0,
            request: 0,
            message: "stale".into(),
        });
        assert!(*stale == *waiting);
        let ready = stale.reduce(VocabularyAction::Loaded {
            selected: 9,
            request: selected.request,
            result: Ok(Rc::new(Vec::new())),
        });
        assert!(matches!(&ready.load, RegionLoad::Ready(_)));
    }

    #[test]
    fn initialization_failure_and_retry_are_distinct_from_query_failure() {
        let state = Rc::new(VocabularyState::default());
        let failed = state.clone().reduce(VocabularyAction::DictionaryFailed {
            selected: state.selected,
            request: state.request,
            message: "offline".into(),
        });
        assert!(
            matches!(&failed.load, RegionLoad::DictionaryUnavailable(message) if message == "offline")
        );
        let waiting = failed.reduce(VocabularyAction::DictionaryWaiting {
            selected: state.selected,
            request: state.request,
        });
        assert!(matches!(&waiting.load, RegionLoad::WaitingForDictionary));
    }
}
