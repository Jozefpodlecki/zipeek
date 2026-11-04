use futures::future::{AbortHandle, Abortable};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::use_navigator;
use zipeek_ai::STARTER_SETS;
use zipeek_core::{Hsk2Level, Hsk3Level, HskLevel, Word};

use crate::{
    hooks::use_dictionary,
    route::Route,
    services::database::{BrowseField, BrowseSearch},
};

#[derive(Clone, PartialEq)]
enum SearchState {
    Loading,
    Ready(Vec<Word>),
    Failed(String),
}

fn selected_hsk(value: &str) -> Option<HskLevel> {
    let (system, number) = value.split_once(':')?;
    let number = number.parse::<i32>().ok()?;
    match system {
        "2" => Hsk2Level::try_from(number).ok().map(HskLevel::Hsk2),
        "3" => Hsk3Level::try_from(number).ok().map(HskLevel::Hsk3),
        _ => None,
    }
}

fn selected_field(value: &str) -> BrowseField {
    match value {
        "hanzi" => BrowseField::Hanzi,
        "pinyin" => BrowseField::Pinyin,
        "english" => BrowseField::English,
        _ => BrowseField::All,
    }
}

#[function_component(Browse)]
pub fn browse() -> Html {
    let dictionary = use_dictionary();
    let navigator = use_navigator();
    let query_text = use_state(String::new);
    let field = use_state(|| "all".to_owned());
    let hsk = use_state(String::new);
    let pos = use_state(String::new);
    let category = use_state(String::new);
    let word_set = use_state(String::new);
    let offset = use_state(|| 0u32);
    let request = use_state(|| BrowseSearch { limit: 20, ..Default::default() });
    let status = use_state(|| SearchState::Loading);
    let retry = use_state(|| 0u32);

    {
        let status = status.clone();
        use_effect_with(((*request).clone(), *retry, dictionary), move |(request, _, dictionary)| {
            status.set(SearchState::Loading);
            let (abort, registration) = AbortHandle::new_pair();
            let request = request.clone();
            let dictionary = dictionary.clone();
            let status = status.clone();
            spawn_local(async move {
                let future = async {
                    let result = match dictionary {
                        Some(handle) => match handle.database() {
                            Ok(database) => database.browse_search(&request).await.map_err(|error| error.to_string()),
                            Err(error) => Err(error.to_string()),
                        },
                        None => Err("Dictionary context unavailable".to_owned()),
                    };
                    status.set(match result {
                        Ok(words) => SearchState::Ready(words),
                        Err(error) => SearchState::Failed(error),
                    });
                };
                let _ = Abortable::new(future, registration).await;
            });
            move || abort.abort()
        });
    }

    let search = {
        let query_text = query_text.clone();
        let field = field.clone();
        let hsk = hsk.clone();
        let pos = pos.clone();
        let category = category.clone();
        let word_set = word_set.clone();
        let offset = offset.clone();
        let request = request.clone();
        Callback::from(move |_| {
            offset.set(0);
            request.set(BrowseSearch {
                text: (*query_text).clone(),
                field: selected_field(&field),
                hsk: selected_hsk(&hsk),
                part_of_speech: if pos.is_empty() { vec![] } else { vec![(*pos).clone()] },
                categories: if category.is_empty() { vec![] } else { vec![(*category).clone()] },
                word_set: if word_set.is_empty() { None } else { Some((*word_set).clone()) },
                offset: 0,
                limit: 20,
            });
        })
    };
    let previous = {
        let offset = offset.clone();
        let request = request.clone();
        Callback::from(move |_| {
            let next = (*offset).saturating_sub(20);
            offset.set(next);
            let mut changed = (*request).clone();
            changed.offset = next;
            request.set(changed);
        })
    };
    let next = {
        let offset = offset.clone();
        let request = request.clone();
        Callback::from(move |_| {
            let next = (*offset).saturating_add(20);
            offset.set(next);
            let mut changed = (*request).clone();
            changed.offset = next;
            request.set(changed);
        })
    };
    let on_query = {
        let query_text = query_text.clone();
        Callback::from(move |event: InputEvent| query_text.set(event.target_unchecked_into::<HtmlInputElement>().value()))
    };
    let on_field = {
        let field = field.clone();
        Callback::from(move |event: Event| field.set(event.target_unchecked_into::<HtmlSelectElement>().value()))
    };
    let on_hsk = {
        let hsk = hsk.clone();
        Callback::from(move |event: Event| hsk.set(event.target_unchecked_into::<HtmlSelectElement>().value()))
    };
    let on_pos = {
        let pos = pos.clone();
        Callback::from(move |event: Event| pos.set(event.target_unchecked_into::<HtmlSelectElement>().value()))
    };
    let on_category = {
        let category = category.clone();
        Callback::from(move |event: Event| category.set(event.target_unchecked_into::<HtmlSelectElement>().value()))
    };
    let on_set = {
        let word_set = word_set.clone();
        Callback::from(move |event: Event| word_set.set(event.target_unchecked_into::<HtmlSelectElement>().value()))
    };
    let on_retry = {
        let retry = retry.clone();
        Callback::from(move |_| retry.set((*retry).wrapping_add(1)))
    };
    let translate = {
        let navigator = navigator.clone();
        let word_set = word_set.clone();
        Callback::from(move |_| {
            if let Some(navigator) = &navigator {
                let set = if word_set.is_empty() { "travel" } else { word_set.as_str() };
                if let Err(error) = navigator.push_with_query(
                    &Route::SentenceTranslation,
                    &[("set", set)],
                ) {
                    log::error!("Could not open translation: {error}");
                }
            }
        })
    };
    let options = |entries: &[(&str, &str)]| html! {
        {for entries.iter().map(|(value, label)| html! { <option value={value.to_string()}>{label.to_string()}</option> })}
    };
    let has_next = matches!(&*status, SearchState::Ready(words) if words.len() == 20);
    html! {
        <section data-browse="" class="mx-auto flex max-w-5xl flex-col gap-6">
            <header class="flex flex-col gap-2">
                <h1 class="text-3xl font-bold tracking-tight">{"Browse dictionary"}</h1>
                <p class="text-sm text-muted">{"Search Chinese, pinyin and meanings. Combine filters to narrow results."}</p>
            </header>
            <div class="flex flex-col gap-4 rounded-xl border border-line p-4">
                <label class="flex flex-col gap-2 text-sm font-semibold">{"Search"}
                    <input data-browse-query="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*query_text).clone()} oninput={on_query} placeholder="汉字, pinyin, or English" maxlength="128" />
                </label>
                <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    <label class="flex flex-col gap-1 text-sm">{"Search in"}
                        <select data-browse-field="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*field).clone()} onchange={on_field}>
                            {options(&[("all","All fields"),("hanzi","Hanzi"),("pinyin","Pinyin"),("english","English meanings")])}
                        </select>
                    </label>
                    <label class="flex flex-col gap-1 text-sm">{"HSK"}
                        <select data-browse-hsk="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*hsk).clone()} onchange={on_hsk}>
                            <option value="">{"Any HSK level"}</option>
                            {for (1..=6).map(|level| html! { <option value={format!("2:{level}")}>{format!("HSK 2.0 · {level}")}</option> })}
                            {for (1..=7).map(|level| html! { <option value={format!("3:{level}")}>{if level == 7 {"HSK 3.0 · 7–9".to_owned()} else {format!("HSK 3.0 · {level}")}}</option> })}
                        </select>
                    </label>
                    <label class="flex flex-col gap-1 text-sm">{"Part of speech"}
                        <select data-browse-pos="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*pos).clone()} onchange={on_pos}>
                            {options(&[("","Any part of speech"),("noun","Noun"),("verb","Verb"),("adjective","Adjective"),("adverb","Adverb"),("pronoun","Pronoun"),("classifier","Classifier"),("preposition","Preposition"),("conjunction","Conjunction")])}
                        </select>
                    </label>
                    <label class="flex flex-col gap-1 text-sm">{"Category"}
                        <select data-browse-category="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*category).clone()} onchange={on_category}>
                            {options(&[("","Any category"),("travel","Travel"),("transport","Transport"),("food","Food and drink"),("shopping","Shopping"),("family","Family"),("education","Education"),("work","Work"),("health","Health"),("nature","Nature"),("time","Time"),("home","Home"),("communication","Communication")])}
                        </select>
                    </label>
                    <label class="flex flex-col gap-1 text-sm">{"Predefined Word Set"}
                        <select data-browse-word-set="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*word_set).clone()} onchange={on_set}>
                            <option value="">{"All words"}</option>
                            {for STARTER_SETS.iter().map(|set| html! { <option value={set.id}>{set.name}</option> })}
                        </select>
                    </label>
                </div>
                <button data-browse-search="" type="button" class="min-h-11 rounded-lg bg-blue-600 px-5 font-semibold text-white" onclick={search}>{"Search"}</button>
            </div>
            {match &*status {
                SearchState::Loading => html! { <p role="status" data-browse-loading="">{"Searching dictionary…"}</p> },
                SearchState::Failed(message) => html! { <div role="alert" data-browse-error="" class="flex flex-col gap-3 rounded-lg border border-line p-4"><p>{format!("Search failed: {message}")}</p><button type="button" class="min-h-11 rounded border border-line px-4" onclick={on_retry}>{"Retry"}</button></div> },
                SearchState::Ready(words) if words.is_empty() => html! { <p data-browse-empty="" role="status" class="rounded-lg border border-line p-5">{"No matching words. Try removing a filter or changing the search."}</p> },
                SearchState::Ready(words) => html! { <div class="flex flex-col gap-4" data-browse-results="">
                    <p role="status" class="text-sm text-muted">{format!("Showing {}–{}", *offset + 1, *offset + words.len() as u32)}</p>
                    <ul class="grid gap-3 sm:grid-cols-2">{for words.iter().map(|word| html! {
                        <li class="rounded-xl border border-line p-4" data-browse-word-id={word.id.to_string()}>
                            <p class="text-2xl font-semibold" lang="zh-Hans">{&word.simplified}</p>
                            {if let Some(traditional) = &word.traditional {if traditional != &word.simplified {html!{<p class="text-sm text-muted" lang="zh-Hant">{traditional}</p>}} else {Html::default()}} else {Html::default()}}
                            <p class="text-sm text-muted">{&word.pinyin}</p>
                            <p class="mt-2 text-sm">{word.english.as_deref().unwrap_or("Meaning unavailable").replace('|', " · ")}</p>
                        </li>
                    })}</ul>
                    <div class="flex items-center justify-between gap-3">
                        <button type="button" data-browse-previous="" class="min-h-11 rounded-lg border border-line px-4 disabled:opacity-50" disabled={*offset == 0} onclick={previous}>{"Previous"}</button>
                        <button type="button" data-browse-next="" class="min-h-11 rounded-lg border border-line px-4 disabled:opacity-50" disabled={!has_next} onclick={next}>{"Next"}</button>
                    </div>
                </div> },
            }}
            <p class="text-sm text-muted">{"Predefined Word Sets are read-only for now. Practice with them using Sentence Translation."}</p>
            <button type="button" class="min-h-11 rounded-lg border border-line px-4" onclick={translate}>{"Practice selected Word Set"}</button>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsk_selector_rejects_invalid_levels() {
        assert_eq!(selected_hsk("2:1"), Some(HskLevel::Hsk2(Hsk2Level::One)));
        assert_eq!(selected_hsk("3:7"), Some(HskLevel::Hsk3(Hsk3Level::SevenToNine)));
        assert_eq!(selected_hsk("2:7"), None);
        assert_eq!(selected_hsk("invalid"), None);
    }
}
