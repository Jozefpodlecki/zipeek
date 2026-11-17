use std::rc::Rc;

use crate::services::word_sets::WordSetId;
use yew::Reducible;
use zipeek_core::Word;
use zipeek_hanzi_writing::hanzi_characters;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum VocabularyMode {
    Sets,
    Search,
}

impl VocabularyMode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Sets => "sets",
            Self::Search => "search",
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct WriteHanziState {
    pub mode: VocabularyMode,
    pub selected_set: Option<WordSetId>,
    pub query: String,
    pub submitted: String,
    pub results: Option<Result<Vec<Word>, String>>,
    pub chosen: Option<Word>,
    pub suggestions_open: bool,
    pub highlighted: usize,
    pub visible_count: usize,
    pub character_index: usize,
    pub direct_status: String,
}

impl Default for WriteHanziState {
    fn default() -> Self {
        Self {
            mode: VocabularyMode::Sets,
            selected_set: None,
            query: String::new(),
            submitted: String::new(),
            results: None,
            chosen: None,
            suggestions_open: true,
            highlighted: 0,
            visible_count: 7,
            character_index: 0,
            direct_status: String::new(),
        }
    }
}

pub(super) enum WriteHanziAction {
    SwitchMode(VocabularyMode),
    SelectSet(WordSetId),
    QueryChanged(String),
    SubmitSearch(String),
    SearchLoading,
    SearchLoaded {
        mode: VocabularyMode,
        selected_set: Option<WordSetId>,
        submitted: String,
        result: Result<Vec<Word>, String>,
    },
    OpenSuggestions,
    CloseSuggestions,
    MoveHighlight(i32),
    ShowMore,
    ChooseWord(Word),
    SelectCharacter(usize),
    DirectLoading,
    DirectLoaded(Result<Option<Word>, String>),
}

impl Reducible for WriteHanziState {
    type Action = WriteHanziAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            WriteHanziAction::SwitchMode(mode) => {
                if next.mode != mode {
                    next.mode = mode;
                    next.highlighted = 0;
                    next.visible_count = 7;
                    next.results = None;
                    next.chosen = None;
                    next.suggestions_open = true;
                }
            }
            WriteHanziAction::SelectSet(index) => {
                if next.selected_set.as_ref() != Some(&index) {
                    next.selected_set = Some(index);
                    next.highlighted = 0;
                    next.visible_count = 7;
                    next.results = None;
                    next.chosen = None;
                    next.suggestions_open = true;
                }
            }
            WriteHanziAction::QueryChanged(query) => {
                if next.query != query {
                    next.query = query;
                    next.highlighted = 0;
                    next.visible_count = 7;
                    next.suggestions_open = true;
                    if next.mode == VocabularyMode::Search {
                        next.results = None;
                    }
                }
            }
            WriteHanziAction::SubmitSearch(query) => {
                if next.submitted != query {
                    next.submitted = query;
                    next.results = None;
                }
            }
            WriteHanziAction::SearchLoading => next.results = None,
            WriteHanziAction::SearchLoaded { mode, selected_set, submitted, result } => {
                if next.mode != mode
                    || next.selected_set != selected_set
                    || next.submitted != submitted
                    || (mode == VocabularyMode::Search && next.query.trim() != submitted)
                {
                    return Rc::new(next);
                }
                next.highlighted = 0;
                next.results = Some(result);
            }
            WriteHanziAction::OpenSuggestions => next.suggestions_open = true,
            WriteHanziAction::CloseSuggestions => next.suggestions_open = false,
            WriteHanziAction::MoveHighlight(delta) => {
                let length = next.results.as_ref().and_then(|result| result.as_ref().ok())
                    .map(|words| {
                        if next.mode == VocabularyMode::Sets && !next.query.trim().is_empty() {
                            words.iter().filter(|word| super::ranking::score(word, &next.query) > 0).count()
                        } else {
                            words.len()
                        }
                    })
                    .unwrap_or(0)
                    .min(next.visible_count);
                if length > 0 {
                    next.highlighted = (next.highlighted as i32 + delta)
                        .rem_euclid(length as i32) as usize;
                    next.suggestions_open = true;
                }
            }
            WriteHanziAction::ShowMore => {
                next.visible_count = next.visible_count.saturating_add(7).min(60);
            }
            WriteHanziAction::ChooseWord(word) => {
                next.chosen = Some(word);
                next.character_index = 0;
                next.suggestions_open = false;
            }
            WriteHanziAction::SelectCharacter(index) => {
                if let Some(word) = &next.chosen {
                    if index < hanzi_characters(&word.simplified).len() {
                        next.character_index = index;
                    }
                }
            }
            WriteHanziAction::DirectLoading => next.direct_status = "Loading selected word…".to_owned(),
            WriteHanziAction::DirectLoaded(result) => match result {
                Ok(Some(word)) if !hanzi_characters(&word.simplified).is_empty() => {
                    next.chosen = Some(word);
                    next.character_index = 0;
                    next.suggestions_open = false;
                    next.direct_status.clear();
                }
                Ok(Some(_)) => next.direct_status = "The requested entry has no supported Hanzi characters.".to_owned(),
                Ok(None) => next.direct_status = "The requested dictionary word was not found.".to_owned(),
                Err(error) => next.direct_status = format!("Could not load selected word: {error}"),
            },
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_opens_suggestions_and_keeps_pending_query_distinct() {
        let original = Rc::new(WriteHanziState { suggestions_open: false, ..Default::default() });
        let updated = original.reduce(WriteHanziAction::QueryChanged("water".into()));
        assert_eq!(updated.query, "water");
        assert_eq!(updated.submitted, "");
        assert!(updated.suggestions_open);
        assert_eq!(updated.visible_count, 7);
    }

    #[test]
    fn changing_query_invalidates_visible_results_immediately() {
        let original = Rc::new(WriteHanziState {
            query: "old".into(), submitted: "old".into(), results: Some(Ok(Vec::new())),
            ..Default::default()
        });
        let changed = original.reduce(WriteHanziAction::QueryChanged("new".into()));
        assert!(changed.results.is_none());
        let submitted = changed.reduce(WriteHanziAction::SubmitSearch("new".into()));
        assert_eq!(submitted.submitted, "new");
    }

    #[test]
    fn switching_source_invalidates_results() {
        let original = Rc::new(WriteHanziState { results: Some(Ok(Vec::new())), ..Default::default() });
        let switched = original.reduce(WriteHanziAction::SwitchMode(VocabularyMode::Search));
        assert!(switched.suggestions_open);
        assert!(switched.results.is_none());
    }

    #[test]
    fn stale_search_completion_is_ignored() {
        let state = Rc::new(WriteHanziState {
            mode: VocabularyMode::Search, query: "new".into(), submitted: "old".into(),
            ..Default::default()
        });
        let updated = state.reduce(WriteHanziAction::SearchLoaded {
            mode: VocabularyMode::Search,
            selected_set: None,
            submitted: "old".into(),
            result: Ok(Vec::new()),
        });
        assert!(updated.results.is_none());
    }

    #[test]
    fn stale_result_from_an_earlier_query_is_ignored() {
        let state = Rc::new(WriteHanziState {
            mode: VocabularyMode::Search,
            query: "new".into(),
            submitted: "new".into(),
            ..Default::default()
        });
        let updated = state.reduce(WriteHanziAction::SearchLoaded {
            mode: VocabularyMode::Search,
            selected_set: None,
            submitted: "old".into(),
            result: Ok(Vec::new()),
        });
        assert!(updated.results.is_none());
    }

    #[test]
    fn escape_closes_suggestions_until_reopened() {
        let state = Rc::new(WriteHanziState::default());
        let closed = state.reduce(WriteHanziAction::CloseSuggestions);
        assert!(!closed.suggestions_open);
        let opened = closed.reduce(WriteHanziAction::OpenSuggestions);
        assert!(opened.suggestions_open);
    }
}
