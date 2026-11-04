use yew::prelude::*;

use crate::{
    reducers::{
        browse::{BrowseAction, BrowseInput, BrowseState},
        word_sets::{WordSetsAction, WordSetsFeedback, WordSetsState},
    },
    services::word_sets,
};

#[derive(Clone)]
pub(crate) enum WordSetsCommand {
    Create(String),
    Rename(String),
    Delete,
    Toggle(i32),
    Import(String),
}

#[derive(Clone, PartialEq)]
pub(crate) struct WordSetsContext {
    pub state: UseReducerHandle<WordSetsState>,
    pub execute: Callback<WordSetsCommand>,
    pub reload: Callback<()>,
}

#[derive(Properties, PartialEq)]
pub(crate) struct WordSetsProviderProps {
    pub children: Children,
}

#[function_component(WordSetsProvider)]
pub(crate) fn word_sets_provider(props: &WordSetsProviderProps) -> Html {
    let state = use_reducer_eq(WordSetsState::load);
    let browse = use_context::<UseReducerHandle<BrowseState>>();

    let execute = {
        let state = state.clone();
        let browse = browse.clone();
        Callback::from(move |command: WordSetsCommand| {
            let Ok(existing) = &state.sets else {
                state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error("Word Sets unavailable; retry loading".into())));
                return;
            };
            let mut next = existing.clone();
            let mutation = match command {
                WordSetsCommand::Create(name) => word_sets::create(&mut next, &name).map(|_| ("Word Set created".to_owned(), false, None)),
                WordSetsCommand::Rename(name) => state.target.parse::<usize>()
                    .map_err(|_| "Choose a personal Word Set".to_owned())
                    .and_then(|index| word_sets::rename(&mut next, index, &name))
                    .map(|_| ("Word Set renamed".to_owned(), false, None)),
                WordSetsCommand::Delete => state.target.parse::<usize>()
                    .map_err(|_| "Choose a personal Word Set".to_owned())
                    .and_then(|index| word_sets::delete(&mut next, index))
                    .map(|_| ("Word Set deleted".to_owned(), true, None)),
                WordSetsCommand::Toggle(id) => state.target.parse::<usize>()
                    .map_err(|_| "Choose a personal Word Set".to_owned())
                    .and_then(|index| {
                        word_sets::toggle_word(&mut next, index, id)?;
                        Ok(("Word Set updated".to_owned(), false, Some(index)))
                    }),
                WordSetsCommand::Import(raw) => word_sets::merge_import(&mut next, &raw)
                    .map(|added| (format!("Word Sets imported: {added} new set(s); matching names merged"), true, None)),
            };
            match mutation.and_then(|(message, invalidate, changed_index)| {
                word_sets::write(&next)?;
                Ok((message, invalidate, changed_index))
            }) {
                Ok((message, invalidate, changed_index)) => {
                    if invalidate {
                        if let Some(browse) = &browse {
                            browse.dispatch(BrowseAction::Input(BrowseInput::WordSet(String::new())));
                        }
                        state.dispatch(WordSetsAction::Removed(next, message));
                    } else {
                        if let Some(index) = changed_index {
                            if let Some(browse) = &browse && browse.word_set == format!("saved:{index}") && browse.request.saved_word_ids.is_some() {
                                browse.dispatch(BrowseAction::SetChanged(next[index].word_ids.clone()));
                            }
                        }
                        state.dispatch(WordSetsAction::Committed(next, message));
                    }
                }
                Err(error) => state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error(error))),
            }
        })
    };

    let reload = {
        let state = state.clone();
        Callback::from(move |()| state.dispatch(WordSetsAction::Loaded(word_sets::read())))
    };

    let context = WordSetsContext { state, execute, reload };
    html! {
        <ContextProvider<WordSetsContext> context={context}>
            {props.children.clone()}
        </ContextProvider<WordSetsContext>>
    }
}
