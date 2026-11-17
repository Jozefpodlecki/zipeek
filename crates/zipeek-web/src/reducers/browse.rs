use std::rc::Rc;
use yew::Reducible;
use zipeek_core::Word;

use crate::services::database::BrowseSearch;

pub(crate) const PAGE_SIZE: u32 = 5;

#[derive(Clone, PartialEq)]
pub(crate) enum SearchState {
    Loading,
    Ready(Vec<Word>),
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub(crate) struct BrowseState {
    pub request: BrowseSearch,
    pub offset: u32,
    pub status: SearchState,
    pub blocked: Option<String>,
    pub generation: u64,
    pub query_text: String,
    pub field: String,
    pub hsk: String,
    pub pos: String,
    pub category: String,
    pub word_set: String,
}

pub(crate) enum BrowseInput {
    QueryText(String),
    Field(String),
    Hsk(String),
    PartOfSpeech(String),
    Category(String),
    WordSet(String),
}

pub(crate) enum BrowseAction {
    Input(BrowseInput),
    Search(BrowseSearch),
    ClearFilters,
    Previous,
    Next,
    SetChanged(Vec<i32>),
    Invalidate(String),
    Completed(u64, Result<Vec<Word>, String>),
}

pub(crate) fn initial_dictionary_query(hash: &str) -> String {
    let Some((_, encoded)) = hash.split_once("?query=") else {
        return String::new();
    };
    let encoded = encoded.split('&').next().unwrap_or("");
    js_sys::decode_uri_component(encoded)
        .ok()
        .map(String::from)
        .unwrap_or_default()
        .chars()
        .take(128)
        .collect()
}

impl BrowseState {
    pub(crate) fn from_search_query(query: &str) -> Self {
        Self {
            query_text: query.chars().take(128).collect(),
            ..Self::default()
        }
    }

    pub(crate) fn from_hash(hash: &str) -> Self {
        Self {
            query_text: initial_dictionary_query(hash),
            ..Self::default()
        }
    }
}

impl Default for BrowseState {
    fn default() -> Self {
        Self {
            request: BrowseSearch {
                limit: PAGE_SIZE,
                ..Default::default()
            },
            offset: 0,
            status: SearchState::Loading,
            blocked: None,
            generation: 0,
            query_text: String::new(),
            field: "all".to_owned(),
            hsk: String::new(),
            pos: String::new(),
            category: String::new(),
            word_set: String::new(),
        }
    }
}

impl Reducible for BrowseState {
    type Action = BrowseAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            BrowseAction::Input(input) => match input {
                BrowseInput::QueryText(value) => next.query_text = value,
                BrowseInput::Field(value) => next.field = value,
                BrowseInput::Hsk(value) => next.hsk = value,
                BrowseInput::PartOfSpeech(value) => next.pos = value,
                BrowseInput::Category(value) => next.category = value,
                BrowseInput::WordSet(value) => next.word_set = value,
            },
            BrowseAction::Search(request) => {
                next.offset = request.offset;
                next.request = request;
                next.blocked = None;
                next.status = SearchState::Loading;
                next.generation = next.generation.wrapping_add(1);
            }
            BrowseAction::ClearFilters => {
                next.query_text.clear();
                next.field = "all".to_owned();
                next.hsk.clear();
                next.pos.clear();
                next.category.clear();
                next.word_set.clear();
            }
            BrowseAction::Previous => {
                if self.blocked.is_some() || self.offset == 0 {
                    return self;
                }
                next.offset = self.offset.saturating_sub(PAGE_SIZE);
                next.request.offset = next.offset;
                next.status = SearchState::Loading;
                next.generation = next.generation.wrapping_add(1);
            }
            BrowseAction::Next => {
                if self.blocked.is_some()
                    || !matches!(&self.status, SearchState::Ready(words) if words.len() > PAGE_SIZE as usize)
                {
                    return self;
                }
                next.offset = self.offset.saturating_add(PAGE_SIZE);
                next.request.offset = next.offset;
                next.status = SearchState::Loading;
                next.generation = next.generation.wrapping_add(1);
            }
            BrowseAction::SetChanged(ids) => {
                if self.request.saved_word_ids.is_none() {
                    return self;
                }
                next.request.saved_word_ids = Some(ids);
                next.offset = 0;
                next.request.offset = 0;
                next.status = SearchState::Loading;
                next.blocked = None;
                next.generation = next.generation.wrapping_add(1);
            }
            BrowseAction::Invalidate(message) => {
                next.offset = 0;
                next.request.offset = 0;
                next.status = SearchState::Failed(message.clone());
                next.blocked = Some(message);
                next.generation = next.generation.wrapping_add(1);
            }
            BrowseAction::Completed(generation, result) => {
                if generation != self.generation || self.blocked.is_some() {
                    return self;
                }
                next.status = match result {
                    Ok(words) => SearchState::Ready(words),
                    Err(error) => SearchState::Failed(error),
                };
            }
        }
        next.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_filters_preserves_completed_results_until_requery() {
        let state = Rc::new(BrowseState::default())
            .reduce(BrowseAction::Input(BrowseInput::QueryText("hello".into())))
            .reduce(BrowseAction::Input(BrowseInput::Hsk("3:6".into())));
        let generation = state.generation;
        let cleared = state.reduce(BrowseAction::ClearFilters);
        assert!(cleared.query_text.is_empty());
        assert!(cleared.hsk.is_empty());
        assert_eq!(cleared.field, "all");
        assert_eq!(cleared.generation, generation);
    }

    #[test]
    fn invalid_selection_blocks_stale_responses_and_pagination() {
        let state =
            Rc::new(BrowseState::default()).reduce(BrowseAction::Invalidate("Set deleted".into()));
        assert_eq!(state.offset, state.request.offset);
        assert!(state.blocked.is_some());
        let stale = state.clone().reduce(BrowseAction::Completed(0, Ok(vec![])));
        assert!(Rc::ptr_eq(&state, &stale));
        let unchanged = state.clone().reduce(BrowseAction::Next);
        assert!(Rc::ptr_eq(&state, &unchanged));
    }

    #[test]
    fn fresh_search_clears_block_and_rejects_old_responses() {
        let previous = Rc::new(BrowseState::default());
        let failed = previous
            .clone()
            .reduce(BrowseAction::Invalidate("Changed".into()));
        let search = failed.reduce(BrowseAction::Search(BrowseSearch {
            offset: 40,
            limit: PAGE_SIZE,
            ..Default::default()
        }));
        assert!(search.blocked.is_none());
        assert_eq!(search.offset, 40);
        let stale = search
            .clone()
            .reduce(BrowseAction::Completed(previous.generation, Ok(vec![])));
        assert!(Rc::ptr_eq(&search, &stale));
        let completed = search
            .clone()
            .reduce(BrowseAction::Completed(search.generation, Ok(vec![])));
        assert!(matches!(&completed.status, SearchState::Ready(words) if words.is_empty()));
    }

    #[test]
    fn next_requires_a_full_page_plus_lookahead() {
        let state = Rc::new(BrowseState::default()).reduce(BrowseAction::Completed(0, Ok(vec![])));
        let unchanged = state.clone().reduce(BrowseAction::Next);
        assert!(Rc::ptr_eq(&state, &unchanged));
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn previous_preserves_request_and_rejects_late_page_results() {
        let mut state = BrowseState::default();
        state.offset = PAGE_SIZE;
        state.request.offset = PAGE_SIZE;
        state.generation = 8;
        state.status = SearchState::Ready(vec![]);
        let previous = Rc::new(state).reduce(BrowseAction::Previous);
        assert_eq!(previous.offset, 0);
        assert_eq!(previous.request.offset, 0);
        assert_eq!(previous.generation, 9);
        assert!(matches!(previous.status, SearchState::Loading));
        let stale = previous
            .clone()
            .reduce(BrowseAction::Completed(8, Ok(vec![])));
        assert!(Rc::ptr_eq(&previous, &stale));
    }

    #[test]
    fn set_mutation_keeps_scope_and_resets_page() {
        let state = Rc::new(BrowseState::default()).reduce(BrowseAction::Search(BrowseSearch {
            saved_word_ids: Some(vec![1, 2]),
            offset: 40,
            limit: PAGE_SIZE,
            ..Default::default()
        }));
        let next = state.reduce(BrowseAction::SetChanged(vec![2]));
        assert_eq!(next.request.saved_word_ids, Some(vec![2]));
        assert_eq!(next.request.offset, 0);
        assert_eq!(next.offset, 0);
    }
}
