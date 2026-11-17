use std::collections::BTreeSet;

use crate::hooks::use_dictionary;
use crate::hooks::use_word_sets;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::{Word, WordId};

use crate::{
    components::browse::WordSetPanel,
    components::{ListenButton, WordSetsCommand},
};

#[function_component(WordSetsPage)]
pub fn word_sets_page() -> Html {
    let sets = use_word_sets();
    let dictionary = use_dictionary();
    let selected = sets
        .state
        .target
        .as_ref()
        .and_then(|id| {
            sets.state
                .sets
                .as_ref()
                .ok()
                .and_then(|items| crate::services::word_sets::find(items, id))
        })
        .cloned();
    let ids = selected
        .as_ref()
        .map(|set| set.word_ids.clone())
        .unwrap_or_default();
    let result = use_state(|| None::<Result<Vec<Word>, String>>);
    let search = use_state(String::new);
    let on_search = {
        let search = search.clone();
        Callback::from(move |event: InputEvent| {
            search.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };
    {
        let search = search.clone();
        use_effect_with(sets.state.target.clone(), move |_| {
            search.set(String::new());
        });
    }
    {
        let dictionary = dictionary.clone();
        let result = result.clone();
        use_effect_with(ids.clone(), move |ids| {
            let active = std::rc::Rc::new(std::cell::Cell::new(true));
            let pending = active.clone();
            let ids = ids.clone();
            result.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let loaded = async {
                    let database = dictionary.database().map_err(|error| error.to_string())?;
                    let ids = ids
                        .into_iter()
                        .map(|value| {
                            WordId::try_from(i64::from(value)).map_err(|error| error.to_string())
                        })
                        .collect::<Result<BTreeSet<_>, _>>()?;
                    database
                        .words_by_ids(&ids)
                        .await
                        .map_err(|error| error.to_string())
                }
                .await;
                if pending.get() {
                    result.set(Some(loaded));
                }
            });
            move || active.set(false)
        });
    }
    let remove = {
        let execute = sets.execute.clone();
        Callback::from(move |id: i32| execute.emit(WordSetsCommand::Toggle(id)))
    };
    let word_set_content = {
        if selected.is_none() {
            html! {
            <p
                class="text-sm text-muted"
            >
                {"Choose a Word Set to see saved vocabulary and manage its words."}
            </p>}
        } else if ids.is_empty() {
            html! {
            <div class="space-y-3 py-4">
                <p class="text-sm text-muted">{"This set is empty. Add words from Dictionary."}</p>
                <a href="#/browse" class="inline-flex min-h-10 items-center rounded-lg border border-line px-3 text-sm font-semibold text-accent hover:bg-raised" data-word-set-add-words="">{"Browse Dictionary"}</a>
            </div>}
        } else {
            match &*result {
                None => html! {
                <p
                    role="status"
                    class="text-sm text-muted"
                >
                    {"Loading saved vocabulary…"}
                </p>},
                Some(Err(error)) => html! {
                <p
                    role="alert"
                    class="text-sm text-red-500"
                >
                    {error}
                </p>},
                Some(Ok(words)) => {
                    let query = search.trim().to_lowercase();
                    let filtered: Vec<_> = words.iter().filter(|word| {
                        query.is_empty()
                            || word.simplified.contains(query.as_str())
                            || word.pinyin.to_lowercase().contains(&query)
                            || word.english.as_deref().unwrap_or_default().to_lowercase().contains(&query)
                    }).collect();
                    if filtered.is_empty() {
                        html! { <p role="status" class="py-6 text-sm text-muted">{"No words match your search."}</p> }
                    } else {
                    html! {
                    <ul
                        class="divide-y divide-line"
                    >
                        {for filtered.into_iter().map(|word| {
                            let id = word.id.as_i32();
                            let remove = remove.clone();
                            let on_remove_word = Callback::from(move |_| remove.emit(id));
                            html! {
                                <li
                                    key={id}
                                    class="flex flex-wrap items-center gap-x-3 gap-y-1 py-3 sm:flex-nowrap" data-word-set-word-id={id.to_string()}
                                >
                                    <strong
                                        lang="zh-Hans"
                                        class="shrink-0 text-lg"
                                    >
                                        {&word.simplified}
                                    </strong>
                                    <span
                                        class="shrink-0 text-sm text-accent"
                                    >
                                        {&word.pinyin}
                                    </span>
                                    <p
                                        class="min-w-[10rem] flex-1 text-sm leading-relaxed text-muted"
                                    >
                                        {word.english.as_deref().unwrap_or("Meaning unavailable").replace('|', " · ")}
                                    </p>
                                    <ListenButton
                                        hanzi={word.simplified.clone()}
                                        compact={true}
                                    />
                                    <button
                                        type="button"
                                        class="grid size-9 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-red-500/10 hover:text-red-500 focus-visible:bg-raised"
                                        data-remove-word-id={id.to_string()}
                                        aria-label={format!("Remove {} from this Word Set", word.simplified)}
                                        title="Remove word"
                                        onclick={on_remove_word}
                                    >
                                        <Icon data={IconData::LUCIDE_TRASH_2} width="17px" height="17px" />
                                    </button>
                                </li>
                            }
                        })}
                    </ul>
                    }
                    }
                },
            }
        }
    };
    html! {
        <section
            data-component="word-sets-page"
            class="mx-auto flex w-full max-w-7xl flex-col gap-5 py-4"
        >
            <header class="flex items-center justify-between gap-3">
                <div>
                    <h1 class="text-2xl font-bold">{"Word Sets"}</h1>
                    <p class="mt-1 text-sm text-muted">{"Manage your saved vocabulary."}</p>
                </div>
            </header>
            <div
                class="grid items-start gap-4 lg:grid-cols-[minmax(15rem,18rem)_minmax(0,1fr)]"
            >
                <WordSetPanel />
                <section
                    class="flex min-h-64 min-w-0 flex-col rounded-2xl border border-line bg-surface p-4 shadow-sm sm:p-5"
                    data-word-set-vocabulary=""
                    aria-label="Selected Word Set vocabulary"
                >
                    <h2
                        class="mb-1 text-xl font-semibold"
                    >
                        {selected.as_ref().map(|set| format!("{} ({} words)", set.name, set.word_ids.len())).unwrap_or_else(|| "Select a Word Set".to_owned())}
                    </h2>
                    <p class="mb-3 text-xs text-muted">{"Browse and manage words in this set."}</p>
                    {if !ids.is_empty() {
                        html! {
                            <label class="mb-3 flex items-center gap-2 rounded-lg border border-line bg-surface px-3">
                                <Icon data={IconData::LUCIDE_SEARCH} width="17px" height="17px" />
                                <span class="sr-only">{"Search words in selected set"}</span>
                                <input data-word-set-search="" type="search" class="min-h-11 min-w-0 flex-1 bg-transparent text-sm outline-none" placeholder="Search Hanzi, pinyin or meaning" value={(*search).clone()} oninput={on_search}/>
                            </label>
                        }
                    } else { Html::default() }}
                    <div data-word-set-scroll="">
                        {word_set_content}
                    </div>
                </section>
            </div>
        </section>
    }
}
