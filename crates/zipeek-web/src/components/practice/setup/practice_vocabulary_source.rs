use crate::components::HskPreferencesSelector;
use web_sys::{HtmlInputElement, HtmlSelectElement, KeyboardEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::HskSystem;
use zipeek_exercises::practice_draft::{DictionaryFilters, HskBand, VocabularySource};

mod tabs;
use tabs::{VocabularyTab, VocabularyTabs};

use crate::{
    hooks::use_dictionary,
    services::word_sets::{self, UserWordSet},
};
use wasm_bindgen_futures::spawn_local;

#[derive(Properties, PartialEq)]
pub struct PracticeVocabularySourceProps {
    pub source: VocabularySource,
    pub hsk_standard: u8,
    pub personal_sets: Vec<UserWordSet>,
    pub on_change: Callback<VocabularySource>,
    pub on_toggle_hsk_band: Callback<HskBand>,
    pub on_select_hsk_standard: Callback<u8>,
}

#[function_component(PracticeVocabularySource)]
pub fn practice_vocabulary_source(props: &PracticeVocabularySourceProps) -> Html {
    let is_personal = matches!(&props.source, VocabularySource::PersonalSet { .. });
    let vocabulary_tab = use_state(|| VocabularyTab::Source);
    let on_select_vocabulary_tab = {
        let vocabulary_tab = vocabulary_tab.clone();
        Callback::from(move |tab| vocabulary_tab.set(tab))
    };
    let on_dictionary = {
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            on_change.emit(VocabularySource::Dictionary(DictionaryFilters::default()))
        })
    };
    let on_personal = {
        let on_change = props.on_change.clone();
        let first_set = props
            .personal_sets
            .iter()
            .find(|set| set.word_ids.len() >= word_sets::MIN_PRACTICE_WORDS)
            .map(|set| set.id.to_string());
        Callback::from(move |_| {
            if let Some(reference) = &first_set {
                on_change.emit(VocabularySource::PersonalSet {
                    reference: reference.clone(),
                });
            }
        })
    };
    let category_text = use_state(String::new);
    let suggestions = use_state(Vec::<String>::new);
    let loading_categories = use_state(|| false);
    let dictionary = use_dictionary();
    let suggestion_epoch = use_mut_ref(|| 0u64);
    {
        let suggestions = suggestions.clone();
        let loading_categories = loading_categories.clone();
        let suggestion_epoch = suggestion_epoch.clone();
        use_effect_with(
            ((*category_text).clone(), dictionary.clone()),
            move |(prefix, dictionary)| {
                let epoch = {
                    let mut value = suggestion_epoch.borrow_mut();
                    *value += 1;
                    *value
                };
                let prefix = prefix.trim().to_owned();
                if let Some(database) = dictionary.database().ok() {
                    loading_categories.set(true);
                    let suggestions = suggestions.clone();
                    let loading_categories = loading_categories.clone();
                    let generation = suggestion_epoch.clone();
                    spawn_local(async move {
                        let result = database.practice_category_suggestions(&prefix).await;
                        if *generation.borrow() == epoch {
                            loading_categories.set(false);
                            match result {
                                Ok(values) => suggestions.set(values),
                                Err(error) => {
                                    log::warn!("Category suggestions unavailable: {error}");
                                    suggestions.set(Vec::new());
                                }
                            }
                        }
                    });
                } else {
                    loading_categories.set(false);
                    suggestions.set(Vec::new());
                }
                || ()
            },
        );
    }
    let on_category_text = {
        let category_text = category_text.clone();
        Callback::from(move |event: InputEvent| {
            category_text.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };
    let on_choose_category = {
        let on_change = props.on_change.clone();
        let source = props.source.clone();
        let category_text = category_text.clone();
        Callback::from(move |category: String| {
            if let VocabularySource::Dictionary(mut filters) = source.clone() {
                if filters.add_category(&category).is_ok() {
                    on_change.emit(VocabularySource::Dictionary(filters));
                    category_text.set(String::new());
                }
            }
        })
    };
    let on_add_category = {
        let suggestions = suggestions.clone();
        let category_text = category_text.clone();
        let on_choose_category = on_choose_category.clone();
        Callback::from(move |_: MouseEvent| {
            let category = category_text.trim().to_ascii_lowercase();
            if suggestions.contains(&category) {
                on_choose_category.emit(category);
            }
        })
    };
    let on_category_keydown = {
        let on_choose_category = on_choose_category.clone();
        let category_text = category_text.clone();
        let suggestions = suggestions.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() != "Enter" || event.is_composing() {
                return;
            }
            event.prevent_default();
            let category = category_text.trim().to_ascii_lowercase();
            if suggestions.contains(&category) {
                on_choose_category.emit(category);
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
    let vocabulary_source_content = {
        if is_personal {
            html! {
                <label
                    class="flex flex-col gap-2 text-sm font-semibold"
                >
                    {"Selected Word Set"}
                    <select
                        data-vocabulary-set=""
                        value={match &props.source { VocabularySource::PersonalSet { reference } => reference.clone(), _ => props.personal_sets.first().map(|set| set.id.to_string()).unwrap_or_default() }}
                        onchange={on_saved}
                        class="min-h-11 rounded-lg border border-line bg-raised px-3"
                    >
                        {for props.personal_sets.iter().map(|set| html! {
                            <option
                                key={set.id.to_string()}
                                value={set.id.to_string()}
                                disabled={set.word_ids.len() < word_sets::MIN_PRACTICE_WORDS}
                            >
                                {format!("{} ({} words{})", set.name, set.word_ids.len(),
                                    if set.word_ids.len() < word_sets::MIN_PRACTICE_WORDS { " - needs 10" } else { "" })}
                            </option>
                        })}
                    </select>
                    <span
                        class="text-xs font-normal text-muted"
                    >
                        {format!("This set is the entire vocabulary source. HSK filters do not apply. Practice requires at least {} distinct saved words.", word_sets::MIN_PRACTICE_WORDS)}
                    </span>
                </label>
            }
        } else {
            let dictionary_source_panel = {
                if *vocabulary_tab == VocabularyTab::Source {
                    html! {
                    <div
                        role="tabpanel"
                        aria-label="Source and HSK levels"
                        data-vocabulary-panel="source"
                    >
                    <HskPreferencesSelector
                        standard={props.hsk_standard}
                        bands={active_filters.as_ref().map(|filters| filters.hsk.clone()).unwrap_or_default()}
                        on_standard={props.on_select_hsk_standard.clone()}
                        on_toggle={props.on_toggle_hsk_band.clone()}
                    />
                    </div>
                    }
                } else {
                    Html::default()
                }
            };
            let category_filters_panel = if *vocabulary_tab == VocabularyTab::Filters {
                let category_suggestion_list = {
                    if !suggestions.is_empty() {
                        html! {
                            <div
                                data-category-suggestions=""
                                class="flex max-h-64 flex-wrap content-start gap-2 overflow-y-auto rounded-lg border border-line/60 bg-surface/60 p-3"
                                aria-label="Available categories"
                            >
                                {for suggestions.iter().filter(|category| !active_filters.as_ref().is_some_and(|filters| filters.categories.contains(*category))).take(64).map(|category| {
                                    let category = category.clone();
                                    let label = category.clone();
                                    let category_for_selection = category.clone();
                                    let on_choose_category = on_choose_category.clone();
                                    let on_choose_part_of_speech = Callback::from(move |_: MouseEvent| {
                                        on_choose_category.emit(category_for_selection.clone());
                                    });
                                    html! {
                                        <button
                                            type="button"
                                            key={category.clone()}
                                            class="min-h-9 rounded-lg border border-line bg-raised px-3 text-sm hover:border-accent"
                                            onclick={on_choose_part_of_speech}
                                        >
                                            {label}
                                        </button>}
                                })}
                            </div>
                        }
                    } else {
                        Html::default()
                    }
                };
                html! {
                <div
                    role="tabpanel"
                    aria-label="Categories and filters"
                    data-vocabulary-panel="filters"
                    class="flex flex-col gap-2"
                >
                    <label
                        for="practice-category-input"
                        class="text-sm font-semibold"
                    >
                        {"Categories"}
                    </label>
                    <div
                        class="flex flex-wrap gap-2"
                    >
                        <input
                            id="practice-category-input"
                            data-category-input=""
                            type="text"
                            value={(*category_text).clone()}
                            oninput={on_category_text}
                            onkeydown={on_category_keydown}
                            placeholder="Filter the category list…"
                            aria-label="Filter available categories"
                            class="min-h-11 min-w-0 flex-1 rounded-lg border border-line bg-raised px-3 focus-visible:outline-offset-2 focus-visible:outline-1 focus-visible:outline-brand-action"
                        />
                        <button
                            type="button"
                            data-category-add=""
                            disabled={!suggestions.contains(&category_text.trim().to_ascii_lowercase())}
                            onclick={on_add_category}
                            class="min-h-11 rounded-lg border border-line bg-raised px-4 text-sm font-semibold disabled:opacity-50"
                        >
                            {"Add exact match"}
                        </button>
                    </div>
                    <div
                        class="flex flex-wrap gap-2"
                        aria-label="Selected categories"
                    >
                        {for active_filters.as_ref().into_iter().flat_map(|filters| filters.categories.iter()).map(|category| {
                            let source = props.source.clone();
                            let on_change = props.on_change.clone();
                            let selected_category = category.clone();
                            let on_choose_category = Callback::from(move |_| {
                                    if let VocabularySource::Dictionary(mut filters) = source.clone() {
                                        filters.remove_category(&selected_category);
                                        on_change.emit(VocabularySource::Dictionary(filters));
                                    }
                                });
                            html! {
                                <button
                                    type="button"
                                    key={category.clone()}
                                    data-category-chip={category.clone()}
                                    aria-label={format!("Remove {category} category")}
                                    class="min-h-9 rounded-full border border-accent bg-accent-soft px-3 text-sm"
                                    onclick={on_choose_category}
                                >
                                    {format!("{category} ×")}
                                </button>}
                        })}
                    </div>
                    {(*loading_categories).then(|| html! {
                        <p role="status" class="text-xs text-muted">{"Loading category choices…"}</p>
                    }).unwrap_or_default()}
                    {category_suggestion_list}
                    <p
                        class="text-xs text-muted"
                    >
                        {"Select categories from the list, or filter them by name. Selected categories apply to every exercise in this practice session."}
                    </p>
                </div>
                }
            } else {
                Html::default()
            };
            html! {
                <div
                    class="flex flex-col gap-4"
                >
                    <VocabularyTabs
                        selected={*vocabulary_tab}
                        on_select={on_select_vocabulary_tab}
                    />
                    {dictionary_source_panel}
                    {category_filters_panel}
                </div>
            }
        }
    };
    let element_classes_3 = classes!(
        "inline-flex",
        "min-h-11",
        "items-center",
        "gap-2",
        "rounded-lg",
        "border",
        "px-4",
        "text-sm",
        if !is_personal {
            "border-brand-action bg-brand-action text-white"
        } else {
            "border-line bg-raised"
        },
    );
    let element_classes_4 = classes!(
        "inline-flex",
        "min-h-11",
        "items-center",
        "gap-2",
        "rounded-lg",
        "border",
        "px-4",
        "text-sm",
        "disabled:opacity-50",
        if is_personal {
            "border-brand-action bg-brand-action text-white"
        } else {
            "border-line bg-raised"
        },
    );
    html! {
        <section
            data-component="practice-vocabulary-source"
            class="flex flex-col gap-4 rounded-2xl border border-line bg-surface p-4 sm:p-5"
        >
            <div
                class="flex flex-wrap items-center justify-between gap-2"
            >
                <h2
                    class="text-lg font-semibold"
                >
                    {"Select vocabulary"}
                </h2>

            </div>
            <div
                role="tablist"
                aria-label="Vocabulary source"
                class="flex flex-wrap gap-2 rounded-xl border border-line bg-raised p-1"
            >
                <button
                    type="button"
                    data-vocabulary-dictionary=""
                    role="tab"
                    aria-selected={(!is_personal).to_string()}
                    onclick={on_dictionary}
                    class={element_classes_3}
                >
                    <Icon
                        data={IconData::LUCIDE_BOOK_OPEN}
                        width="17px"
                        height="17px"
                    />{"Dictionary / HSK"}
                </button>
                <button
                    type="button"
                    data-vocabulary-personal=""
                    role="tab"
                    aria-selected={is_personal.to_string()}
                    disabled={!props.personal_sets.iter().any(|set| set.word_ids.len() >= word_sets::MIN_PRACTICE_WORDS)}
                    onclick={on_personal}
                    class={element_classes_4}
                >
                    <Icon
                        data={IconData::LUCIDE_BOOKMARK}
                        width="17px"
                        height="17px"
                    />{"My Word Sets"}
                </button>
            </div>
            {vocabulary_source_content}
        </section>
    }
}
