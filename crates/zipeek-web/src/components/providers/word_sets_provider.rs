use yew::prelude::*;

use crate::{
    hooks::use_app_environment,
    reducers::{
        browse::{BrowseAction, BrowseInput, BrowseState},
        word_sets::{WordSetsAction, WordSetsFeedback, WordSetsState},
    },
    services::word_sets,
};

#[derive(Clone)]
pub(crate) enum WordSetsCommand {
    Create(String),
    CreateFromCourse {
        name: String,
        word_ids: Vec<i32>,
        source: word_sets::CourseWordSetSource,
    },
    Rename(String),
    Delete,
    Toggle(i32),
    Import(String),
}

#[derive(Clone, PartialEq)]
pub(crate) struct WordSetsContext {
    pub state: UseReducerHandle<WordSetsState>,
    pub execute: Callback<WordSetsCommand>,
    pub select: Callback<Option<word_sets::WordSetId>>,
    pub reload: Callback<()>,
}

#[derive(Properties, PartialEq)]
pub(crate) struct WordSetsProviderProps {
    pub children: Children,
}

#[function_component(WordSetsProvider)]
pub(crate) fn word_sets_provider(props: &WordSetsProviderProps) -> Html {
    let environment = use_app_environment();
    let initial_storage = environment.local_storage.clone();
    let state = use_reducer_eq(move || WordSetsState::load(&initial_storage));
    let browse = use_context::<UseReducerHandle<BrowseState>>();

    let execute = {
        let state = state.clone();
        let browse = browse.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |command: WordSetsCommand| {
            let Ok(existing) = &state.sets else {
                state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error(
                    "Word Sets unavailable; retry loading".into(),
                )));
                return;
            };
            let course_source = match &command {
                WordSetsCommand::CreateFromCourse { source, .. } => Some(source.clone()),
                _ => None,
            };
            let mut next = existing.clone();
            let mutation = match command {
                WordSetsCommand::Create(name) => word_sets::create(&mut next, &name)
                    .map(|_| ("Word Set created".to_owned(), false, None)),
                WordSetsCommand::CreateFromCourse {
                    name,
                    word_ids,
                    source,
                } => word_sets::create_from_course(&mut next, &name, &word_ids, source)
                    .map(|_| ("Course Word Set created".to_owned(), false, None)),
                WordSetsCommand::Rename(name) => state
                    .target
                    .as_ref()
                    .ok_or_else(|| "Choose a personal Word Set".to_owned())
                    .and_then(|id| word_sets::rename(&mut next, id, &name))
                    .map(|_| ("Word Set renamed".to_owned(), false, None)),
                WordSetsCommand::Delete => state
                    .target
                    .as_ref()
                    .ok_or_else(|| "Choose a personal Word Set".to_owned())
                    .and_then(|id| word_sets::delete(&mut next, id))
                    .map(|_| ("Word Set deleted".to_owned(), true, None)),
                WordSetsCommand::Toggle(word_id) => state
                    .target
                    .as_ref()
                    .ok_or_else(|| "Choose a personal Word Set".to_owned())
                    .and_then(|set_id| {
                        word_sets::toggle_word(&mut next, set_id, word_id)?;
                        Ok(("Word Set updated".to_owned(), false, Some(set_id.clone())))
                    }),
                WordSetsCommand::Import(raw) => {
                    word_sets::merge_import(&mut next, &raw).map(|added| {
                        (
                            format!(
                                "Word Sets imported: {added} new set(s); matching names merged"
                            ),
                            false,
                            state.target.clone(),
                        )
                    })
                }
            };
            match mutation.and_then(|(message, invalidate, changed_id)| {
                word_sets::write(&storage, &next)?;
                Ok((message, invalidate, changed_id))
            }) {
                Ok((message, invalidate, changed_id)) => {
                    if invalidate {
                        let _ = word_sets::set_active_dictionary_set(&storage, None);
                        if let Some(browse) = &browse {
                            browse
                                .dispatch(BrowseAction::Input(BrowseInput::WordSet(String::new())));
                        }
                        state.dispatch(WordSetsAction::Removed(next, message));
                    } else {
                        if let Some(id) = changed_id {
                            if let Some(browse) = &browse
                                && browse.word_set == format!("saved:{id}")
                                && browse.request.saved_word_ids.is_some()
                                && let Some(set) = word_sets::find(&next, &id)
                            {
                                browse.dispatch(BrowseAction::SetChanged(set.word_ids.clone()));
                            }
                        }
                        state.dispatch(WordSetsAction::Committed(next, message.clone()));
                        if let Some(source) = course_source {
                            state.dispatch(WordSetsAction::CourseFeedback(
                                source,
                                WordSetsFeedback::Success(message),
                            ));
                        }
                    }
                }
                Err(error) => {
                    if let Some(source) = course_source {
                        state.dispatch(WordSetsAction::CourseFeedback(
                            source,
                            WordSetsFeedback::Error(error),
                        ));
                    } else {
                        state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error(error)));
                    }
                }
            }
        })
    };

    let select = {
        let state = state.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |id: Option<word_sets::WordSetId>| {
            if let Some(selected) = id.as_ref() {
                if !state
                    .sets
                    .as_ref()
                    .is_ok_and(|sets| word_sets::find(sets, selected).is_some())
                {
                    return;
                }
            }
            match word_sets::set_active_dictionary_set(&storage, id.as_ref()) {
                Ok(()) => state.dispatch(WordSetsAction::Select(id)),
                Err(error) => {
                    state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error(error)))
                }
            }
        })
    };

    let reload = {
        let state = state.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |()| state.dispatch(WordSetsAction::Loaded(word_sets::read(&storage))))
    };

    let context = WordSetsContext {
        state,
        execute,
        select,
        reload,
    };
    html! {
        <ContextProvider<WordSetsContext> context={context}>
            {props.children.clone()}
        </ContextProvider<WordSetsContext>>
    }
}
