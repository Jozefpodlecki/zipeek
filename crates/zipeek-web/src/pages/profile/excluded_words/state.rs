use std::{collections::BTreeSet, rc::Rc};

use yew::Reducible;
use zipeek_core::{Word, WordId};

#[derive(Clone, PartialEq)]
pub(super) enum QueryState {
    Idle,
    Loading,
    Ready(Vec<Word>),
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub(super) enum ListState {
    Loading,
    Ready(Vec<Word>),
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub(super) struct ExcludedWordsState {
    pub ids: BTreeSet<WordId>,
    pub query: String,
    pub selected: usize,
    pub suggestions: QueryState,
    pub words: ListState,
    pub error: Option<String>,
}

pub(super) enum Action {
    QueryChanged(String),
    SuggestionsLoaded {
        query: String,
        result: Result<Vec<Word>, String>,
    },
    SelectionMoved(isize),
    ClearQuery,
    ExclusionsSaved(BTreeSet<WordId>),
    WordsLoaded {
        ids: BTreeSet<WordId>,
        result: Result<Vec<Word>, String>,
    },
    SaveFailed(String),
}

impl ExcludedWordsState {
    pub fn new(ids: BTreeSet<WordId>) -> Self {
        Self {
            ids,
            query: String::new(),
            selected: 0,
            suggestions: QueryState::Idle,
            words: ListState::Loading,
            error: None,
        }
    }

    pub fn available(&self) -> &[Word] {
        match &self.suggestions {
            QueryState::Ready(words) => words,
            _ => &[],
        }
    }

    pub fn selected_word(&self) -> Option<WordId> {
        self.available()
            .get(self.selected)
            .filter(|word| !self.ids.contains(&word.id))
            .map(|word| word.id)
    }
}

impl Reducible for ExcludedWordsState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            Action::QueryChanged(value) => {
                next.query = value;
                next.selected = 0;
                next.error = None;
                next.suggestions = if next.query.trim().is_empty() {
                    QueryState::Idle
                } else {
                    QueryState::Loading
                };
            }
            Action::SuggestionsLoaded { query, result } => {
                if query != next.query.trim() {
                    return self;
                }
                next.selected = 0;
                next.suggestions = match result {
                    Ok(words) => QueryState::Ready(words),
                    Err(message) => QueryState::Failed(message),
                };
            }
            Action::SelectionMoved(direction) => {
                let count = next.available().len();
                if count != 0 {
                    next.selected =
                        (next.selected as isize + direction).rem_euclid(count as isize) as usize;
                }
            }
            Action::ClearQuery => {
                next.query.clear();
                next.selected = 0;
                next.suggestions = QueryState::Idle;
            }
            Action::ExclusionsSaved(ids) => {
                next.ids = ids;
                next.words = ListState::Loading;
                next.error = None;
                next.query.clear();
                next.selected = 0;
                next.suggestions = QueryState::Idle;
            }
            Action::WordsLoaded { ids, result } => {
                if ids != next.ids {
                    return self;
                }
                next.words = match result {
                    Ok(words) => ListState::Ready(words),
                    Err(message) => ListState::Failed(message),
                };
            }
            Action::SaveFailed(message) => next.error = Some(message),
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(state: Rc<ExcludedWordsState>, action: Action) -> Rc<ExcludedWordsState> {
        state.reduce(action)
    }

    #[test]
    fn changed_query_drops_stale_results() {
        let state = Rc::new(ExcludedWordsState::new(BTreeSet::new()));
        let state = apply(state, Action::QueryChanged("shop".into()));
        let state = apply(state, Action::QueryChanged("store".into()));
        let state = apply(
            state,
            Action::SuggestionsLoaded {
                query: "shop".into(),
                result: Err("old error".into()),
            },
        );
        assert_eq!(state.query, "store");
        assert!(matches!(state.suggestions, QueryState::Loading));
    }

    #[test]
    fn saved_exclusions_clear_search_and_keep_existing_ids() {
        let ids = BTreeSet::from([WordId::try_from(1_i64).unwrap()]);
        let state = apply(
            Rc::new(ExcludedWordsState::new(BTreeSet::new())),
            Action::QueryChanged("shop".into()),
        );
        let state = apply(state, Action::ExclusionsSaved(ids.clone()));
        assert_eq!(state.ids, ids);
        assert!(state.query.is_empty());
        assert!(matches!(state.suggestions, QueryState::Idle));
        assert!(matches!(state.words, ListState::Loading));
    }

    #[test]
    fn stale_word_list_does_not_replace_current_ids() {
        let ids = BTreeSet::from([WordId::try_from(1_i64).unwrap()]);
        let state = Rc::new(ExcludedWordsState::new(ids));
        let state = apply(
            state,
            Action::WordsLoaded {
                ids: BTreeSet::new(),
                result: Err("old load".into()),
            },
        );
        assert!(matches!(state.words, ListState::Loading));
    }
}
