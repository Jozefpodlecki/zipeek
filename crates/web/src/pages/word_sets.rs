use std::collections::BTreeSet;

use yew::prelude::*;
use zipeek_core::{Word, WordId};

use crate::{
    components::{WordSetsCommand, WordSetsContext, ListenButton},
    components::browse::WordSetPanel,
    contexts::database::Dictionary,
};

#[function_component(WordSetsPage)]
pub fn word_sets_page() -> Html {
    let sets = use_context::<WordSetsContext>().expect("Word Sets provider is required");
    let dictionary = use_context::<Dictionary>().expect("Dictionary provider is required");
    let selected = sets.state.target.parse::<usize>().ok()
        .and_then(|index| sets.state.sets.as_ref().ok().and_then(|items| items.get(index)))
        .cloned();
    let ids = selected.as_ref().map(|set| set.word_ids.clone()).unwrap_or_default();
    let result = use_state(|| None::<Result<Vec<Word>, String>>);
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
                    let ids = ids.into_iter()
                        .map(|value| WordId::try_from(i64::from(value)).map_err(|error| error.to_string()))
                        .collect::<Result<BTreeSet<_>, _>>()?;
                    database.words_by_ids(&ids).await.map_err(|error| error.to_string())
                }.await;
                if pending.get() { result.set(Some(loaded)); }
            });
            move || active.set(false)
        });
    }
    let remove = {
        let execute = sets.execute.clone();
        Callback::from(move |id: i32| execute.emit(WordSetsCommand::Toggle(id)))
    };
    html! {
        <section data-component="word-sets-page" class="mx-auto flex w-full max-w-6xl flex-col gap-6 py-4">
            <header class="flex flex-col gap-2">
                <h1 class="text-3xl font-bold">{"My Word Sets"}</h1>
                <p class="text-sm text-muted">{"Organize vocabulary, review your saved words, and choose a set for Practice."}</p>
                <a href="#/browse" class="w-fit text-sm font-semibold text-accent hover:underline">{"Find more words in Dictionary"}</a>
            </header>
            <div class="grid items-start gap-6 lg:grid-cols-[minmax(17rem,21rem)_minmax(0,1fr)]">
                <WordSetPanel />
                <section class="min-w-0 min-h-64 rounded-2xl border border-line bg-surface p-6 shadow-sm" aria-label="Selected Word Set vocabulary">
                    <h2 class="mb-4 text-xl font-semibold">{selected.as_ref().map(|set| format!("{} ({} words)", set.name, set.word_ids.len())).unwrap_or_else(|| "Select a Word Set".to_owned())}</h2>
                    {if selected.is_none() {
                        html! {<p class="text-sm text-muted">{"Choose a Word Set to see saved vocabulary and manage its words."}</p>}
                    } else if ids.is_empty() {
                        html! {<p class="text-sm text-muted">{"This set is empty. Use Dictionary to add vocabulary."}</p>}
                    } else {
                        match &*result {
                            None => html! {<p role="status" class="text-sm text-muted">{"Loading saved vocabulary…"}</p>},
                            Some(Err(error)) => html! {<p role="alert" class="text-sm text-red-500">{error}</p>},
                            Some(Ok(words)) => html! {
                                <ul class="divide-y divide-line">
                                    {for words.iter().map(|word| {
                                        let id = word.id.as_i32();
                                        let remove = remove.clone();
                                        html! {
                                            <li key={id} class="flex flex-wrap items-center gap-3 py-3">
                                                <strong lang="zh-Hans" class="text-lg">{&word.simplified}</strong>
                                                <span class="text-sm text-muted">{&word.pinyin}</span>
                                                <p class="min-w-40 flex-1 text-sm">{word.english.as_deref().unwrap_or("Meaning unavailable").replace('|', " · ")}</p>
                                                <ListenButton hanzi={word.simplified.clone()} compact={true} />
                                                <button type="button" class="min-h-9 rounded-lg border border-line px-3 text-xs hover:border-accent" onclick={Callback::from(move |_| remove.emit(id))}>{"Remove"}</button>
                                            </li>
                                        }
                                    })}
                                </ul>
                            }
                        }
                    }}
                </section>
            </div>
        </section>
    }
}
