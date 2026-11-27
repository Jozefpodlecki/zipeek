mod state;

use std::{cell::Cell, rc::Rc};

use crate::{
    hooks::{use_app_environment, use_dictionary},
    services::exclusions,
};
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::WordId;

use self::state::{Action, ExcludedWordsState, ListState, QueryState};

#[function_component(ExcludedWords)]
pub(super) fn excluded_words() -> Html {
    let environment = use_app_environment();
    let dictionary = use_dictionary();
    let storage = environment.local_storage.clone();
    let state = use_reducer_eq(move || ExcludedWordsState::new(exclusions::read_saved(&storage)));

    {
        let dictionary = dictionary.clone();
        let state = state.clone();
        use_effect_with(state.ids.clone(), move |ids| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            let ids = ids.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = async {
                    let database = dictionary.database().map_err(|error| error.to_string())?;
                    database
                        .words_by_ids(&ids)
                        .await
                        .map_err(|error| error.to_string())
                }
                .await;
                if pending.get() {
                    state.dispatch(Action::WordsLoaded { ids, result });
                }
            });
            move || live.set(false)
        });
    }

    {
        let dictionary = dictionary.clone();
        let state = state.clone();
        use_effect_with(state.query.clone(), move |query| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            let query = query.trim().to_owned();
            if !query.is_empty() {
                wasm_bindgen_futures::spawn_local(async move {
                    let result = async {
                        let database = dictionary.database().map_err(|error| error.to_string())?;
                        database
                            .suggest_words(&query, 12)
                            .await
                            .map_err(|error| error.to_string())
                    }
                    .await;
                    if pending.get() {
                        state.dispatch(Action::SuggestionsLoaded { query, result });
                    }
                });
            }
            move || live.set(false)
        });
    }

    let save = {
        let state = state.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |(id, include): (WordId, bool)| {
            let mut ids = state.ids.clone();
            if include {
                ids.insert(id);
            } else {
                ids.remove(&id);
            }
            if ids == state.ids {
                return;
            }
            match exclusions::write_saved(&storage, &ids) {
                Ok(()) => state.dispatch(Action::ExclusionsSaved(ids)),
                Err(message) => state.dispatch(Action::SaveFailed(message)),
            }
        })
    };

    let on_input = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            state.dispatch(Action::QueryChanged(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            ));
        })
    };

    let on_keydown = {
        let state = state.clone();
        let save = save.clone();
        Callback::from(move |event: KeyboardEvent| match event.key().as_str() {
            "ArrowDown" if !state.available().is_empty() => {
                event.prevent_default();
                state.dispatch(Action::SelectionMoved(1));
            }
            "ArrowUp" if !state.available().is_empty() => {
                event.prevent_default();
                state.dispatch(Action::SelectionMoved(-1));
            }
            "Enter" if !state.available().is_empty() => {
                event.prevent_default();
                if let Some(id) = state.selected_word() {
                    save.emit((id, true));
                }
            }
            "Escape" if !state.query.is_empty() => {
                event.prevent_default();
                state.dispatch(Action::ClearQuery);
            }
            _ => {}
        })
    };

    let query_open = !state.query.trim().is_empty();
    html! {
        <section data-component="profile-excluded-words" class="rounded-2xl border border-line bg-surface p-4 sm:p-5">
            <div class="relative">
                <label for="profile-exclude-search" class="sr-only">{"Find a word to exclude"}</label>
                <div class="flex items-center gap-2 rounded-xl border border-line bg-raised/40 px-3 focus-within:border-accent">
                    <Icon data={IconData::LUCIDE_SEARCH} width="18px" height="18px" />
                    <input id="profile-exclude-search" type="search" role="combobox" aria-autocomplete="list" aria-expanded={query_open.to_string()} aria-controls="profile-exclude-options" autocomplete="off" placeholder="Search Chinese, pinyin or English…" value={state.query.clone()} oninput={on_input} onkeydown={on_keydown} class="min-h-11 min-w-0 flex-1 bg-transparent text-sm outline-none" />
                </div>
                {if !query_open { Html::default() } else {
                    html! { <ul id="profile-exclude-options" role="listbox" class="absolute z-20 mt-1 max-h-56 w-full overflow-y-auto rounded-xl border border-line bg-surface p-1 shadow-xl">
                        {match &state.suggestions {
                            QueryState::Idle | QueryState::Loading => html! { <li role="status" class="px-3 py-2 text-sm text-muted">{"Searching…"}</li> },
                            QueryState::Failed(message) => html! { <li role="alert" class="px-3 py-2 text-sm text-red-400">{message}</li> },
                            QueryState::Ready(items) if items.is_empty() => html! { <li class="px-3 py-2 text-sm text-muted">{"No matches"}</li> },
                            QueryState::Ready(items) => html! { <> {for items.iter().enumerate().map(|(index, word)| {
                                let id = word.id;
                                let excluded = state.ids.contains(&id);
                                let save = save.clone();
                                let onclick = Callback::from(move |_| save.emit((id, true)));
                                html! { <li key={word.id.as_i32()} role="option" aria-selected={(index == state.selected).to_string()}>
                                    <button type="button" {onclick} disabled={excluded} class={classes!("flex", "w-full", "items-center", "gap-3", "rounded-lg", "px-3", "py-2", "text-left", "hover:bg-raised", "disabled:opacity-50", if index == state.selected { "bg-raised" } else { "" })}>
                                        <strong lang="zh-Hans" class="shrink-0 text-lg">{&word.simplified}</strong>
                                        <span class="shrink-0 text-sm text-accent">{&word.pinyin}</span>
                                        <span class="min-w-0 flex-1 truncate text-sm text-muted">{word.english.as_deref().unwrap_or("")}</span>
                                        {if excluded { html! { <Icon data={IconData::LUCIDE_CHECK} width="16px" height="16px" /> } } else { Html::default() }}
                                    </button>
                                </li> }
                            })} </> },
                        }}
                    </ul> }
                }}
            </div>
            {if let Some(message) = &state.error { html! { <p role="alert" class="mt-2 text-sm text-red-400">{message}</p> } } else { Html::default() }}
            {match &state.words {
                ListState::Loading => html! { <p role="status" class="mt-4 text-sm text-muted">{"Loading…"}</p> },
                ListState::Failed(message) => html! { <p role="alert" class="mt-4 text-sm text-red-400">{message}</p> },
                ListState::Ready(items) if items.is_empty() => html! { <div class="flex flex-col items-center gap-2 py-10 text-muted"><Icon data={IconData::LUCIDE_SEARCH} width="24px" height="24px" /><span class="text-sm">{"No excluded words"}</span></div> },
                ListState::Ready(items) => html! { <ul class="mt-4 divide-y divide-line">
                    {for items.iter().map(|word| {
                        let id = word.id;
                        let save = save.clone();
                        let on_remove = Callback::from(move |_| save.emit((id, false)));
                        html! { <li key={word.id.as_i32()} class="flex min-h-14 items-center gap-3 py-2">
                            <strong lang="zh-Hans" class="text-lg">{&word.simplified}</strong>
                            <span class="text-sm text-accent">{&word.pinyin}</span>
                            <span class="min-w-0 flex-1 truncate text-sm text-muted">{word.english.as_deref().unwrap_or("")}</span>
                            <button type="button" title="Remove exclusion" aria-label={format!("Remove {} from exclusions", word.simplified)} class="grid size-10 shrink-0 place-items-center rounded-lg text-muted hover:bg-raised hover:text-ink" onclick={on_remove}><Icon data={IconData::LUCIDE_X} width="18px" height="18px" /></button>
                        </li> }
                    })}
                </ul> },
            }}
        </section>
    }
}
