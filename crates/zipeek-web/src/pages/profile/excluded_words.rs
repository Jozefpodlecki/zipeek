use std::{cell::Cell, collections::BTreeSet, rc::Rc};

use crate::{
    hooks::{use_app_environment, use_dictionary},
    services::exclusions,
};
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::{Word, WordId};

#[function_component(ExcludedWords)]
pub(super) fn excluded_words() -> Html {
    let environment = use_app_environment();
    let dictionary = use_dictionary();
    let storage = environment.local_storage.clone();
    let ids = use_state(move || exclusions::read_saved(&storage));
    let words = use_state(|| None::<Result<Vec<Word>, String>>);
    let term = use_state(String::new);
    let suggestions = use_state(|| None::<Result<Vec<Word>, String>>);
    let active_index = use_state(|| 0_usize);
    let error = use_state(|| None::<String>);

    {
        let dictionary = dictionary.clone();
        let words = words.clone();
        use_effect_with((*ids).clone(), move |ids| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            let ids = ids.clone();
            words.set(None);
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
                    words.set(Some(result));
                }
            });
            move || live.set(false)
        });
    }
    {
        let dictionary = dictionary.clone();
        let suggestions = suggestions.clone();
        use_effect_with((*term).clone(), move |query| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            let query = query.trim().to_owned();
            suggestions.set(None);
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
                        suggestions.set(Some(result));
                    }
                });
            }
            move || live.set(false)
        });
    }

    let add = {
        let ids = ids.clone();
        let storage = environment.local_storage.clone();
        let error = error.clone();
        let term = term.clone();
        let active_index = active_index.clone();
        Callback::from(move |id: WordId| {
            let mut updated = (*ids).clone();
            updated.insert(id);
            match exclusions::write_saved(&storage, &updated) {
                Ok(()) => {
                    ids.set(updated);
                    term.set(String::new());
                    active_index.set(0);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    let on_input = {
        let term = term.clone();
        let active_index = active_index.clone();
        let error = error.clone();
        Callback::from(move |event: InputEvent| {
            term.set(event.target_unchecked_into::<HtmlInputElement>().value());
            active_index.set(0);
            error.set(None);
        })
    };
    let on_keydown = {
        let active_index = active_index.clone();
        let suggestions = suggestions.clone();
        let term = term.clone();
        let add = add.clone();
        Callback::from(move |event: KeyboardEvent| {
            let available = match &*suggestions {
                Some(Ok(items)) if !term.trim().is_empty() => items,
                _ => return,
            };
            match event.key().as_str() {
                "ArrowDown" if !available.is_empty() => {
                    event.prevent_default();
                    active_index.set((*active_index + 1) % available.len());
                }
                "ArrowUp" if !available.is_empty() => {
                    event.prevent_default();
                    active_index.set((*active_index + available.len() - 1) % available.len());
                }
                "Enter" if !available.is_empty() => {
                    event.prevent_default();
                    if let Some(word) = available.get(*active_index) {
                        add.emit(word.id);
                    }
                }
                "Escape" => {
                    event.prevent_default();
                    term.set(String::new());
                }
                _ => {}
            }
        })
    };
    let query = term.trim();
    html! {
        <section data-component="profile-excluded-words" class="rounded-2xl border border-line bg-surface p-4 sm:p-5">
            <div class="relative">
                <label for="profile-exclude-search" class="sr-only">{"Find a word to exclude"}</label>
                <div class="flex items-center gap-2 rounded-xl border border-line bg-raised/40 px-3 focus-within:border-accent">
                    <Icon data={IconData::LUCIDE_SEARCH} width="18px" height="18px" />
                    <input id="profile-exclude-search" type="search" role="combobox" aria-autocomplete="list" aria-expanded={(!query.is_empty()).to_string()} aria-controls="profile-exclude-options" autocomplete="off" placeholder="Search Chinese, pinyin or English…" value={(*term).clone()} oninput={on_input} onkeydown={on_keydown} class="min-h-11 min-w-0 flex-1 bg-transparent text-sm outline-none" />
                </div>
                {if query.is_empty() { Html::default() } else {
                    html! { <ul id="profile-exclude-options" role="listbox" class="absolute z-20 mt-1 max-h-80 w-full overflow-y-auto rounded-xl border border-line bg-surface p-1 shadow-xl">
                        {match &*suggestions {
                            None => html! { <li role="status" class="px-3 py-2 text-sm text-muted">{"Searching…"}</li> },
                            Some(Err(message)) => html! { <li role="alert" class="px-3 py-2 text-sm text-red-400">{message}</li> },
                            Some(Ok(items)) if items.is_empty() => html! { <li class="px-3 py-2 text-sm text-muted">{"No matches"}</li> },
                            Some(Ok(items)) => html! { <> {for items.iter().enumerate().map(|(index, word)| {
                                let id = word.id;
                                let excluded = ids.contains(&id);
                                let add = add.clone();
                                let onclick = Callback::from(move |_| add.emit(id));
                                html! { <li key={word.id.as_i32()} role="option" aria-selected={(index == *active_index).to_string()}>
                                    <button type="button" {onclick} disabled={excluded} class={classes!("flex", "w-full", "items-center", "gap-3", "rounded-lg", "px-3", "py-2", "text-left", "hover:bg-raised", "disabled:opacity-50", if index == *active_index { "bg-raised" } else { "" })}>
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
            {if let Some(message) = &*error { html! { <p role="alert" class="mt-2 text-sm text-red-400">{message}</p> } } else { Html::default() }}
            {match &*words {
                None => html! { <p role="status" class="mt-4 text-sm text-muted">{"Loading…"}</p> },
                Some(Err(message)) => html! { <p role="alert" class="mt-4 text-sm text-red-400">{message}</p> },
                Some(Ok(items)) if items.is_empty() => html! { <div class="flex flex-col items-center gap-2 py-10 text-muted"><Icon data={IconData::LUCIDE_BAN} width="24px" height="24px" /><span class="text-sm">{"No excluded words"}</span></div> },
                Some(Ok(items)) => html! { <ul class="mt-4 divide-y divide-line">
                    {for items.iter().map(|word| {
                        let id = word.id;
                        let storage = environment.local_storage.clone();
                        let ids = ids.clone();
                        let error = error.clone();
                        let on_remove = Callback::from(move |_| {
                            let mut updated = (*ids).clone();
                            updated.remove(&id);
                            match exclusions::write_saved(&storage, &updated) {
                                Ok(()) => { ids.set(updated); error.set(None); }
                                Err(message) => error.set(Some(message)),
                            }
                        });
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
