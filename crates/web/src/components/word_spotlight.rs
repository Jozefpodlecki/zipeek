use std::collections::BTreeSet;

use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::Word;

use crate::{components::ListenButton, contexts::database::Dictionary};


fn relation_label(relation_type: &str) -> &str {
    match relation_type {
        "abbreviation_of" => "Abbreviation of",
        "short_for" => "Short for",
        "variant_of" => "Variant of",
        "also_written_as" => "Also written as",
        "see_also" => "See also",
        _ => "Reference",
    }
}

#[derive(Properties, PartialEq)]
pub struct WordSpotlightProps {
    #[prop_or_default]
    pub daily: bool,
}

#[function_component(WordSpotlight)]
pub fn word_spotlight(props: &WordSpotlightProps) -> Html {
    let dictionary = use_context::<Dictionary>().expect("Dictionary provider is required");
    let day = (js_sys::Date::now() / 86_400_000.0).floor() as u32;
    let refresh = use_state(|| 0_u32);
    let result = use_state(|| None::<Result<(Word, Vec<String>, Option<String>, Vec<crate::services::database::BrowseRelation>), String>>);
    {
        let dictionary = dictionary.clone();
        let result = result.clone();
        let day = if props.daily { Some(day) } else { None };
        use_effect_with((day, *refresh), move |(day, _)| {
            let day = *day;
            let active = std::rc::Rc::new(std::cell::Cell::new(true));
            let pending = active.clone();
            result.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let loaded = async {
                    let database = dictionary.database().map_err(|error| error.to_string())?;
                    let exclusions = BTreeSet::new();
                    let count = database.count_words(&exclusions).await.map_err(|error| error.to_string())?;
                    if count == 0 {
                        return Err("The dictionary contains no HSK words".to_owned());
                    }
                    let offset = match day {
                        Some(day) => day % count,
                        None => {
                            let mut bytes = [0_u8; 4];
                            getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
                            u32::from_le_bytes(bytes) % count
                        }
                    };
                    let word = database.word_at_offset(offset, &exclusions).await
                        .map_err(|error| error.to_string())?
                        .ok_or_else(|| "Selected word is unavailable".to_owned())?;
                    let categories = database.word_categories(word.id).await.map_err(|error| error.to_string())?;
                    let meaning = database.word_display_definition(word.id).await.map_err(|error| error.to_string())?;
                    let relations = database.word_relations(word.id).await.map_err(|error| error.to_string())?;
                    Ok((word, categories, meaning, relations))
                }.await;
                if pending.get() {
                    result.set(Some(loaded));
                }
            });
            move || active.set(false)
        });
    }
    let show_example = use_state(|| false);
    let on_refresh = {
        let refresh = refresh.clone();
        let show_example = show_example.clone();
        Callback::from(move |_| { show_example.set(false); refresh.set((*refresh).wrapping_add(1)); })
    };
    let on_example = {
        let show_example = show_example.clone();
        Callback::from(move |_| show_example.set(!*show_example))
    };
    html! {
        <section data-component="word-spotlight" data-daily-word={if props.daily {"available"} else {"random"}}
            class="relative mx-auto w-full max-w-4xl rounded-[2rem] border border-accent/25 bg-surface/90 px-5 py-9 shadow-xl shadow-black/10 backdrop-blur-xl sm:px-10 sm:py-12">
            {match &*result {
                None => html! {<p role="status" class="py-16 text-center text-sm text-muted">{"Loading vocabulary…"}</p>},
                Some(Err(error)) => html! {<p role="alert" class="py-16 text-center text-sm text-red-500">{error}</p>},
                Some(Ok((word, categories, meaning, relations))) => {
                    let link = format!("#/browse?query={}", js_sys::encode_uri_component(&word.simplified));
                    let display_meaning = meaning.as_deref().filter(|text| !text.is_empty()).map(|text| text.replace('|', " · "));
                    html! {
                        <div class="mx-auto flex max-w-3xl flex-col items-center gap-4 text-center">
                            <div class="flex items-center justify-center gap-4">
                                <strong class="text-7xl font-semibold tracking-wide sm:text-8xl" lang="zh-Hans">{&word.simplified}</strong>
                                <ListenButton hanzi={word.simplified.clone()} />
                            </div>
                            {if let Some(traditional) = word.traditional.as_ref().filter(|value| *value != &word.simplified) {
                                html! {<span class="text-lg text-muted" lang="zh-Hant">{traditional}</span>}
                            } else {Html::default()}}
                            <span class="text-2xl font-semibold text-accent">{&word.pinyin}</span>
                            {if let Some(display_meaning) = display_meaning {
                                html! {<p class="max-w-2xl text-lg leading-relaxed sm:text-xl">{display_meaning}</p>}
                            } else {Html::default()}}
                            {if !relations.is_empty() { html! {
                                <div data-word-cross-reference="" class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 text-sm text-muted">
                                    {for relations.iter().map(|reference| {
                                        let reference_link = format!("#/browse?query={}", js_sys::encode_uri_component(&reference.target_simplified));
                                        html! {
                                            <span class="inline-flex items-center gap-1" title={reference.source_annotation.clone()}>
                                                {relation_label(&reference.relation_type)}{" "}
                                                <a href={reference_link} lang="zh-Hans" class="font-medium text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent" title="Look up this expression in Dictionary">{&reference.target_simplified}</a>
                                            </span>
                                        }
                                    })}
                                </div>
                            }} else {Html::default()}}
                            <div class="mt-2 flex flex-wrap items-center justify-center gap-2" data-word-metadata="">
                                {if let Some(level) = word.hsk3_level {
                                    html! {<span class="rounded-full border border-line bg-raised px-3 py-1.5 text-xs font-medium">{format!("HSK 3.0 · {}", if level.as_i32() == 7 {"7–9".to_owned()} else {level.as_i32().to_string()})}</span>}
                                } else {Html::default()}}
                                {if let Some(level) = word.hsk2_level {
                                    html! {<span class="rounded-full border border-line bg-raised px-3 py-1.5 text-xs font-medium">{format!("HSK 2.0 · {}", level.as_i32())}</span>}
                                } else {Html::default()}}
                                {for word.part_of_speech.iter().map(|pos| html! {<span class="rounded-full border border-line bg-raised px-3 py-1.5 text-xs font-medium">{pos.as_str().replace('_', " ")}</span>})}
                                {for categories.iter().map(|category| html! {<span class="rounded-full border border-accent/40 bg-accent-soft px-3 py-1.5 text-xs font-medium text-accent">{category}</span>})}
                            </div>
                            <div class={classes!("mt-5", "grid", "w-full", "grid-cols-1", "gap-3", if props.daily {"sm:grid-cols-2"} else {"sm:grid-cols-3"})} data-spotlight-actions="">
                                {if !props.daily {html! {
                                    <button type="button" data-next-random-word="" onclick={on_refresh}
                                        class="inline-flex min-h-12 items-center justify-center gap-2 rounded-xl bg-blue-600 px-4 text-sm font-semibold text-white transition-colors hover:bg-blue-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">
                                        <Icon data={IconData::LUCIDE_SHUFFLE} width="18px" height="18px" />{"Another word"}
                                    </button>
                                }} else {Html::default()}}
                                <a class="inline-flex min-h-12 items-center justify-center gap-2 rounded-xl border border-line bg-raised px-4 text-sm font-semibold text-ink transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent" href={link}>
                                    <Icon data={IconData::LUCIDE_BOOK_OPEN} width="18px" height="18px" />{"Open in Dictionary"}
                                </a>
                                <button type="button" data-example-sentence="" onclick={on_example} aria-expanded={show_example.to_string()}
                                    class="inline-flex min-h-12 items-center justify-center gap-2 rounded-xl border border-line bg-raised px-4 text-sm font-semibold transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-accent">
                                    <Icon data={IconData::LUCIDE_FILE_TEXT} width="18px" height="18px" />{"Example sentence"}
                                </button>
                            </div>
                            {if *show_example {html! {
                                <div role="status" class="w-full rounded-xl border border-line bg-raised p-4 text-left text-sm leading-relaxed text-muted">
                                    {"Example sentences aren't available in the current dictionary data yet. This will be enabled when verified examples are added."}
                                </div>
                            }} else {Html::default()}}
                        </div>
                    }
                },
            }}
        </section>
    }
}
