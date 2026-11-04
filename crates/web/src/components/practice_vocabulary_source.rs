use web_sys::{HtmlInputElement, HtmlSelectElement, KeyboardEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::practice_draft::{DictionaryFilters, HskBand, VocabularySource};
use zipeek_core::{WordId, practice::ExerciseVariant, round::distinct_playable_answers};
use std::collections::BTreeSet;

#[derive(Clone, PartialEq, Eq)]
pub struct PoolPreview {
    pub matching_rows: u32,
    pub answer_counts: Vec<(ExerciseVariant, usize)>,
}

impl PoolPreview {
    pub fn supports(&self, variant: ExerciseVariant) -> bool {
        self.answer_counts.iter().any(|(candidate, count)| *candidate == variant && *count >= if variant.is_mcq() { 3 } else { 8 })
    }
}

const ALL_VARIANTS: [ExerciseVariant; 6] = [
    ExerciseVariant::EngPinyin, ExerciseVariant::PinyinHanzi, ExerciseVariant::HanziEng,
    ExerciseVariant::HanziToMeaning, ExerciseVariant::MeaningToHanzi, ExerciseVariant::MeaningToPinyin,
];

use crate::{contexts::database::Dictionary, services::word_sets::UserWordSet};
use wasm_bindgen_futures::spawn_local;

#[derive(Properties, PartialEq)]
pub struct PracticeVocabularySourceProps {
    pub source: VocabularySource,
    pub personal_sets: Vec<UserWordSet>,
    pub on_change: Callback<VocabularySource>,
    pub exclusions: Option<BTreeSet<WordId>>,
    pub on_pool_count: Callback<(VocabularySource, Option<BTreeSet<WordId>>, Option<Result<PoolPreview, String>>)> ,
}

#[function_component(PracticeVocabularySource)]
pub fn practice_vocabulary_source(props: &PracticeVocabularySourceProps) -> Html {
    let is_personal = matches!(&props.source, VocabularySource::PersonalSet { .. });
    let hsk_mode = use_state(|| match &props.source {
        VocabularySource::Dictionary(filters) => {
            let has_2 = filters.hsk.iter().any(|band| band.system == 2);
            let has_3 = filters.hsk.iter().any(|band| band.system == 3);
            match (has_2, has_3) {
                (true, true) => "mixed".to_owned(),
                (true, false) => "2".to_owned(),
                _ => "3".to_owned(),
            }
        }
        VocabularySource::PersonalSet { .. } => "3".to_owned(),
    });
    let on_dictionary = {
        let on_change = props.on_change.clone();
        Callback::from(move |_| on_change.emit(VocabularySource::Dictionary(DictionaryFilters::default())))
    };
    let on_personal = {
        let on_change = props.on_change.clone();
        let available = !props.personal_sets.is_empty();
        Callback::from(move |_| {
            if available {
                on_change.emit(VocabularySource::PersonalSet { reference: "0".to_owned() });
            }
        })
    };
    let category_text = use_state(String::new);
    let suggestions = use_state(Vec::<String>::new);
    let dictionary = use_context::<Dictionary>();
    let suggestion_epoch = use_mut_ref(|| 0u64);
    {
        let suggestions = suggestions.clone();
        let suggestion_epoch = suggestion_epoch.clone();
        use_effect_with(((*category_text).clone(), dictionary.clone()), move |(prefix, dictionary)| {
            let epoch = {
                let mut value = suggestion_epoch.borrow_mut();
                *value += 1;
                *value
            };
            let prefix = prefix.trim().to_owned();
            if prefix.is_empty() {
                suggestions.set(Vec::new());
            } else if let Some(database) = dictionary.as_ref().and_then(|dictionary| dictionary.database().ok()) {
                let suggestions = suggestions.clone();
                let generation = suggestion_epoch.clone();
                spawn_local(async move {
                    let result = database.practice_category_suggestions(&prefix).await;
                    if *generation.borrow() == epoch {
                        match result {
                            Ok(values) => suggestions.set(values),
                            Err(error) => {
                                log::warn!("Category suggestions unavailable: {error}");
                                suggestions.set(Vec::new());
                            }
                        }
                    }
                });
            }
            || ()
        });
    }
    let pool_count = use_state(|| None::<Result<PoolPreview, String>>);
    let pool_epoch = use_mut_ref(|| 0u64);
    {
        let pool_count = pool_count.clone();
        let pool_epoch = pool_epoch.clone();
        let dictionary = dictionary.clone();
        let source = props.source.clone();
        let exclusions = props.exclusions.clone();
        let on_pool_count = props.on_pool_count.clone();
        use_effect_with((source, dictionary, exclusions), move |(source, dictionary, exclusions)| {
            let epoch = {
                let mut value = pool_epoch.borrow_mut();
                *value += 1;
                *value
            };
            pool_count.set(None);
            on_pool_count.emit((source.clone(), exclusions.clone(), None));
            if let VocabularySource::Dictionary(filters) = source {
                if filters.hsk.is_empty() && filters.categories.is_empty() && filters.parts_of_speech.is_empty() {
                    let empty = PoolPreview { matching_rows: 0, answer_counts: vec![] };
                    pool_count.set(Some(Ok(empty.clone())));
                    on_pool_count.emit((source.clone(), exclusions.clone(), Some(Ok(empty))));
                } else if let (Some(database), Some(excluded)) = (dictionary.as_ref().and_then(|context| context.database().ok()), exclusions.as_ref()) {
                    let filters = filters.clone();
                    let pool_count = pool_count.clone();
                    let generation = pool_epoch.clone();
                    let on_pool_count = on_pool_count.clone();
                    let selected_source = source.clone();
                    let selected_exclusions = exclusions.clone();
                    let excluded = excluded.clone();
                    spawn_local(async move {
                        let count = database.practice_exact_pool_for_filters(&filters, &excluded).await
                            .map(|words| PoolPreview {
                                matching_rows: words.len() as u32,
                                answer_counts: ALL_VARIANTS.iter().map(|variant| (*variant, distinct_playable_answers(&words, *variant))).collect(),
                            }).map_err(|error| error.to_string());
                        if *generation.borrow() == epoch {
                            pool_count.set(Some(count.clone()));
                            on_pool_count.emit((selected_source, selected_exclusions, Some(count)));
                        }
                    });
                } else {
                    let error = "Dictionary is not ready".to_owned();
                    pool_count.set(Some(Err(error.clone())));
                    on_pool_count.emit((source.clone(), exclusions.clone(), Some(Err(error))));
                }
            }
            || ()
        });
    }
    let on_category_text = {
        let category_text = category_text.clone();
        Callback::from(move |event: InputEvent| {
            category_text.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };
    let on_add_category = {
        let suggestions = suggestions.clone();
        let on_change = props.on_change.clone();
        let source = props.source.clone();
        let category_text = category_text.clone();
        Callback::from(move |_| {
            if let VocabularySource::Dictionary(mut filters) = source.clone() {
                let category = category_text.trim().to_ascii_lowercase();
                if suggestions.contains(&category) {
                    if filters.add_category(&category).is_ok() {
                        on_change.emit(VocabularySource::Dictionary(filters));
                        category_text.set(String::new());
                    }
                }
            }
        })
    };
    let on_category_keydown = {
        let on_change = props.on_change.clone();
        let source = props.source.clone();
        let category_text = category_text.clone();
        let suggestions = suggestions.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() != "Enter" { return; }
            event.prevent_default();
            let category = category_text.trim().to_ascii_lowercase();
            if !suggestions.contains(&category) { return; }
            if let VocabularySource::Dictionary(mut filters) = source.clone() {
                if filters.add_category(&category).is_ok() {
                    on_change.emit(VocabularySource::Dictionary(filters));
                    category_text.set(String::new());
                }
            }
        })
    };
    let on_saved = {
        let on_change = props.on_change.clone();
        Callback::from(move |event: Event| {
            on_change.emit(VocabularySource::PersonalSet {
                reference: event.target_unchecked_into::<HtmlSelectElement>().value(),
            });
        })
    };
    let active_filters = match &props.source {
        VocabularySource::Dictionary(filters) => Some(filters.clone()),
        _ => None,
    };
    html! {
        <section data-component="practice-vocabulary-source" class="flex flex-col gap-4 rounded-2xl border border-line bg-surface p-4 sm:p-5">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h2 class="text-lg font-semibold">{"Vocabulary source"}</h2>
                <span class="text-xs text-muted">{"Shared by every selected exercise"}</span>
            </div>
            <div role="group" aria-label="Vocabulary source type" class="flex flex-wrap gap-2">
                <button type="button" data-vocabulary-dictionary="" aria-pressed={(!is_personal).to_string()} onclick={on_dictionary}
                    class={classes!("inline-flex", "min-h-11", "items-center", "gap-2", "rounded-lg", "border", "px-4", "text-sm", if !is_personal {"border-blue-400 bg-blue-600 text-white"} else {"border-line bg-raised"})}>
                    <Icon data={IconData::LUCIDE_BOOK_OPEN} width="17px" height="17px" />{"Dictionary / HSK"}
                </button>
                <button type="button" data-vocabulary-personal="" aria-pressed={is_personal.to_string()} disabled={props.personal_sets.is_empty()} onclick={on_personal}
                    class={classes!("inline-flex", "min-h-11", "items-center", "gap-2", "rounded-lg", "border", "px-4", "text-sm", "disabled:opacity-50", if is_personal {"border-blue-400 bg-blue-600 text-white"} else {"border-line bg-raised"})}>
                    <Icon data={IconData::LUCIDE_BOOKMARK} width="17px" height="17px" />{"My Word Sets"}
                </button>
            </div>
            {if is_personal {
                html! {
                    <label class="flex flex-col gap-2 text-sm font-semibold">{"Selected Word Set"}
                        <select data-vocabulary-set="" value={match &props.source { VocabularySource::PersonalSet { reference } => reference.clone(), _ => "0".to_owned() }} onchange={on_saved}
                            class="min-h-11 rounded-lg border border-line bg-raised px-3">
                            {for props.personal_sets.iter().enumerate().map(|(index, set)| html! {
                                <option key={index} value={index.to_string()}>{format!("{} ({} words)", set.name, set.word_ids.len())}</option>
                            })}
                        </select>
                        <span class="text-xs font-normal text-muted">{"This set is the entire vocabulary source. HSK filters do not apply."}</span>
                    </label>
                }
            } else {
                html! {
                    <div class="flex flex-col gap-4">
                        <fieldset class="flex flex-col gap-2">
                            <legend class="text-sm font-semibold">{"HSK standard"}</legend>
                            <p class="text-xs text-muted">{"Choose HSK 2.0, HSK 3.0, or Mixed. Selecting a single standard replaces levels from the other standard."}</p>
                            <div role="group" aria-label="HSK standard" data-vocabulary-hsk-standard="" class="inline-flex flex-wrap gap-2 rounded-xl border border-line bg-raised p-1">
                                {for [("2", "HSK 2.0"), ("3", "HSK 3.0"), ("mixed", "Mixed")].into_iter().map(|(mode, label)| {
                                    let selected = *hsk_mode == mode;
                                    let hsk_mode = hsk_mode.clone();
                                    let source = props.source.clone();
                                    let on_change = props.on_change.clone();
                                    html! {<button type="button" key={mode} data-hsk-standard={mode} aria-pressed={selected.to_string()}
                                        class={classes!("min-h-10", "rounded-lg", "px-4", "text-sm", "font-semibold", if selected {"bg-blue-600 text-white"} else {"text-muted hover:bg-surface hover:text-ink"})}
                                        onclick={Callback::from(move |_| {
                                            hsk_mode.set(mode.to_owned());
                                            if let VocabularySource::Dictionary(mut filters) = source.clone() {
                                                if mode != "mixed" {
                                                    let system = if mode == "2" { 2 } else { 3 };
                                                    filters.hsk.retain(|band| band.system == system);
                                                    if filters.hsk.is_empty() {
                                                        filters.hsk.insert(HskBand::new(system, 1).expect("known HSK band"));
                                                    }
                                                    on_change.emit(VocabularySource::Dictionary(filters));
                                                }
                                            }
                                        })}>{label}</button>}
                                })}
                            </div>
                            <div class="mt-2 text-sm font-semibold">{if *hsk_mode == "mixed" {"HSK 2.0 + 3.0 levels"} else if *hsk_mode == "2" {"HSK 2.0 levels"} else {"HSK 3.0 levels"}}</div>
                            <p class="text-xs text-muted">{"Select one or more levels (OR). Categories and parts of speech narrow the result (AND)."}</p>
                            <div class="flex flex-wrap gap-2" data-vocabulary-hsk-multi="">
                                {for [(2u8, 1u8), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6), (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6), (3, 7)].into_iter().filter(|(system, _)| *hsk_mode == "mixed" || (*hsk_mode == "2" && *system == 2) || (*hsk_mode == "3" && *system == 3)).map(|(system, level)| {
                                    let band = HskBand::new(system, level).expect("known HSK band");
                                    let selected = active_filters.as_ref().is_some_and(|filters| filters.hsk.contains(&band));
                                    let source = props.source.clone();
                                    let on_change = props.on_change.clone();
                                    let label = if system == 3 && level == 7 { "3.0 · 7–9".to_owned() } else {format!("{system}.0 · {level}")};
                                    html! {<button type="button" key={format!("{system}:{level}")} aria-pressed={selected.to_string()} data-hsk-band={format!("{system}:{level}")}
                                        class={classes!("min-h-10", "rounded-lg", "border", "px-3", "text-sm", if selected {"border-blue-400 bg-blue-600 text-white"} else {"border-line bg-raised"})}
                                        onclick={Callback::from(move |_| {
                                            if let VocabularySource::Dictionary(mut filters) = source.clone() {
                                                if !filters.hsk.remove(&band) { filters.hsk.insert(band.clone()); }
                                                on_change.emit(VocabularySource::Dictionary(filters));
                                            }
                                        })}>{label}</button>}
                                })}
                            </div>
                        </fieldset>
                        <div class="flex flex-col gap-2">
                            <label for="practice-category-input" class="text-sm font-semibold">{"Category"}</label>
                            <div class="flex flex-wrap gap-2">
                                <input id="practice-category-input" data-category-input="" type="text" value={(*category_text).clone()} oninput={on_category_text} onkeydown={on_category_keydown}
                                    placeholder="Find a category, e.g. travel" list="practice-categories" class="min-h-11 min-w-0 flex-1 rounded-lg border border-line bg-raised px-3" />
                                <datalist id="practice-categories">{for suggestions.iter().map(|category| html! {<option key={category.clone()} value={category.clone()} />})}</datalist>
                                <button type="button" data-category-add="" disabled={!suggestions.contains(&category_text.trim().to_ascii_lowercase())} onclick={on_add_category}
                                    class="min-h-11 rounded-lg border border-line bg-raised px-4 text-sm font-semibold disabled:opacity-50">{"Add category"}</button>
                            </div>
                            <div class="flex flex-wrap gap-2" aria-label="Selected categories">
                                {for active_filters.as_ref().into_iter().flat_map(|filters| filters.categories.iter()).map(|category| {
                                    let source = props.source.clone();
                                    let on_change = props.on_change.clone();
                                    let selected_category = category.clone();
                                    html! {<button type="button" key={category.clone()} data-category-chip={category.clone()} aria-label={format!("Remove {category} category")}
                                        class="min-h-9 rounded-full border border-blue-400 bg-blue-950/40 px-3 text-sm"
                                        onclick={Callback::from(move |_| {
                                            if let VocabularySource::Dictionary(mut filters) = source.clone() {
                                                filters.remove_category(&selected_category);
                                                on_change.emit(VocabularySource::Dictionary(filters));
                                            }
                                        })}>{format!("{category} ×")}</button>}
                                })}
                            </div>
                            {if !suggestions.is_empty() && !category_text.is_empty() { html! {
                                <div data-category-suggestions="" class="flex flex-wrap gap-2" aria-label="Suggested categories">
                                    {for suggestions.iter().filter(|category| !active_filters.as_ref().is_some_and(|filters| filters.categories.contains(*category))).take(8).map(|category| {
                                        let category = category.clone();
                                        let source = props.source.clone();
                                        let on_change = props.on_change.clone();
                                        let category_text = category_text.clone();
                                        let label = category.clone();
                                        html! {<button type="button" key={category.clone()} class="min-h-9 rounded-lg border border-line bg-raised px-3 text-sm hover:border-accent"
                                            onclick={Callback::from(move |_| {
                                                if let VocabularySource::Dictionary(mut filters) = source.clone() {
                                                    if filters.add_category(&category).is_ok() {
                                                        on_change.emit(VocabularySource::Dictionary(filters));
                                                        category_text.set(String::new());
                                                    }
                                                }
                                            })}>{label}</button>}
                                    })}
                                </div>
                            }} else {Html::default()}}
                            <p class="text-xs text-muted">{"Choose a suggested category or press Enter to add an exact match. All selected exercise filters are applied to the same dictionary source."}</p>
                        </div>
                        <div data-vocabulary-pool-count="" role="status" aria-live="polite" class="rounded-xl border border-line bg-raised px-4 py-3 text-sm">
                            {match &*pool_count {
                                None => html! {<span class="text-muted">{"Checking available vocabulary…"}</span>},
                                Some(Ok(preview)) if preview.matching_rows == 0 => html! {<span class="text-amber-400">{"No matching words after exclusions. Change your filters before starting."}</span>},
                                Some(Ok(preview)) => html! {<div class="space-y-1">
                                    <p>{format!("{} matching dictionary words after exclusions", preview.matching_rows)}</p>
                                    <p class="text-xs text-muted">{"Distinct playable answers by question format (minimum: 8 Matching, 3 MCQ):"}</p>
                                    <div class="flex flex-wrap gap-2">{for preview.answer_counts.iter().map(|(variant, count)| html! {
                                        <span key={variant.seed_token()} class={classes!("rounded-md", "px-2", "py-1", "text-xs", if preview.supports(*variant) {"bg-emerald-950/40 text-emerald-300"} else {"bg-amber-950/40 text-amber-300"})}>{format!("{}: {}", match variant {
                                            ExerciseVariant::EngPinyin => "English → Pinyin",
                                            ExerciseVariant::PinyinHanzi => "Pinyin → Hanzi",
                                            ExerciseVariant::HanziEng => "Hanzi → English",
                                            ExerciseVariant::HanziToMeaning => "Hanzi → Meaning",
                                            ExerciseVariant::MeaningToHanzi => "Meaning → Hanzi",
                                            ExerciseVariant::MeaningToPinyin => "Meaning → Pinyin",
                                        }, count)}</span>
                                    })}</div>
                                    <p class="text-xs text-muted">{"These counts are a necessary eligibility check; a deterministic round can still fail if its candidate selection cannot find distinct answers."}</p>
                                </div>},
                                Some(Err(error)) => html! {<span class="text-red-400">{format!("Could not check vocabulary: {error}")}</span>},
                            }}
                        </div>
                    </div>
                }
            }}
        </section>
    }
}
