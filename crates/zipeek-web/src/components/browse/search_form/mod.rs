mod filters;
use self::filters::{apply_choice, filter_choices, selected_field, selected_hsk};
use crate::hooks::use_browse_state;
use crate::hooks::use_word_sets;
use gloo::timers::callback::Timeout;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
#[cfg(test)]
use zipeek_core::{Hsk2Level, Hsk3Level, HskLevel};

use crate::{
    reducers::browse::{BrowseAction, BrowseInput, BrowseState, PAGE_SIZE},
    services::{database::BrowseSearch, word_sets},
};

pub(super) fn submit_search(
    state: &UseReducerHandle<BrowseState>,
    _sets: &Result<Vec<word_sets::UserWordSet>, String>,
    reset_page: bool,
) {
    let request = BrowseSearch {
        text: state.query_text.clone(),
        field: selected_field(&state.field),
        hsk: selected_hsk(&state.hsk),
        part_of_speech: if state.pos.is_empty() {
            vec![]
        } else {
            vec![state.pos.clone()]
        },
        categories: if state.category.is_empty() {
            vec![]
        } else {
            vec![state.category.clone()]
        },
        word_set: None,
        saved_word_ids: None,
        offset: if reset_page { 0 } else { state.offset },
        limit: PAGE_SIZE,
    };
    state.dispatch(BrowseAction::Search(request));
}

#[function_component(BrowseSearchForm)]
pub(crate) fn browse_search_form() -> Html {
    let state = use_browse_state();
    let sets = use_word_sets();
    let filter_text = use_state(String::new);
    let focused = use_state(|| false);
    let highlighted = use_state(|| 0usize);
    let filter_options: Vec<_> = filter_choices()
        .into_iter()
        .filter(|choice| {
            let search = filter_text.trim().to_lowercase();
            search.is_empty()
                || choice.label.to_lowercase().contains(&search)
                || choice.value.to_lowercase().contains(&search)
        })
        .take(8)
        .collect();
    let on_query = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            state.dispatch(BrowseAction::Input(BrowseInput::QueryText(
                event.target_unchecked_into::<HtmlInputElement>().value(),
            )))
        })
    };
    let on_filter_input = {
        let filter_text = filter_text.clone();
        let highlighted = highlighted.clone();
        Callback::from(move |event: InputEvent| {
            filter_text.set(event.target_unchecked_into::<HtmlInputElement>().value());
            highlighted.set(0);
        })
    };
    let on_filter_key = {
        let highlighted = highlighted.clone();
        let filter_options = filter_options.clone();
        let state = state.clone();
        let filter_text = filter_text.clone();
        let focused = focused.clone();
        Callback::from(move |event: KeyboardEvent| match event.key().as_str() {
            "ArrowDown" => {
                event.prevent_default();
                if !filter_options.is_empty() {
                    highlighted.set((*highlighted + 1) % filter_options.len());
                }
            }
            "ArrowUp" => {
                event.prevent_default();
                if !filter_options.is_empty() {
                    highlighted
                        .set((*highlighted + filter_options.len() - 1) % filter_options.len());
                }
            }
            "Enter" => {
                if let Some(choice) =
                    filter_options.get((*highlighted).min(filter_options.len().saturating_sub(1)))
                {
                    event.prevent_default();
                    apply_choice(&state, choice);
                    filter_text.set(String::new());
                    highlighted.set(0);
                    focused.set(false);
                }
            }
            "Escape" => {
                focused.set(false);
                filter_text.set(String::new());
            }
            _ => (),
        })
    };
    let on_filter_focus = {
        let focused = focused.clone();
        Callback::from(move |_| focused.set(true))
    };
    let last_query = use_mut_ref(|| state.query_text.clone());
    {
        let state = state.clone();
        let sets_snapshot = sets.state.sets.clone();
        let last_query = last_query.clone();
        let dependencies = (
            state.query_text.clone(),
            state.field.clone(),
            state.hsk.clone(),
            state.pos.clone(),
            state.category.clone(),
        );
        use_effect_with(dependencies, move |(query, _, _, _, _)| {
            let delay = if *last_query.borrow() != *query {
                320
            } else {
                0
            };
            *last_query.borrow_mut() = query.clone();
            let timeout = Timeout::new(delay, move || submit_search(&state, &sets_snapshot, true));
            move || drop(timeout)
        });
    }
    let has_filters = state.field != "all"
        || !state.hsk.is_empty()
        || !state.pos.is_empty()
        || !state.category.is_empty();
    let clear = {
        let state = state.clone();
        let filter_text = filter_text.clone();
        Callback::from(move |_| {
            state.dispatch(BrowseAction::ClearFilters);
            filter_text.set(String::new());
        })
    };
    let chips: Vec<(String, &'static str)> = [
        (
            if state.field != "all" {
                format!("Search: {}", state.field)
            } else {
                String::new()
            },
            "field",
        ),
        (
            selected_hsk(&state.hsk)
                .map(|_| format!("HSK {}", state.hsk.replace(':', " · ")))
                .unwrap_or_default(),
            "hsk",
        ),
        (
            if state.pos.is_empty() {
                String::new()
            } else {
                format!("PoS: {}", state.pos)
            },
            "pos",
        ),
        (
            if state.category.is_empty() {
                String::new()
            } else {
                format!("Topic: {}", state.category)
            },
            "category",
        ),
    ]
    .into_iter()
    .filter(|(label, _)| !label.is_empty())
    .collect();
    let search_suggestions_content = {
        if *focused && !filter_options.is_empty() {
            html! {
                <ul
                    id="browse-filter-suggestions"
                    role="listbox"
                    class="absolute inset-x-0 top-full z-30 mt-1 max-h-72 overflow-y-auto rounded-xl border border-line bg-surface p-1 shadow-xl"
                >
                    {for filter_options.iter().enumerate().map(|(i,choice)| {
                        let state = state.clone();
                        let choice = choice.clone();
                        let callback_choice = choice.clone();
                        let filter_text = filter_text.clone();
                        let focused = focused.clone();
                        let highlighted = highlighted.clone();
                        let on_choose_suggestion = {
                            let selected_highlight = highlighted.clone();
                            Callback::from(move |_| {
                                apply_choice(&state, &callback_choice);
                                filter_text.set(String::new());
                                selected_highlight.set(0);
                                focused.set(false);
                            })
                        };
                        let element_classes_1 = classes!(
                                            "w-full",
                                            "rounded-lg",
                                            "px-3",
                                            "py-2",
                                            "text-left",
                                            "text-sm",
                                            "hover:bg-accent-soft",
                                            if i == *highlighted {"bg-accent-soft"} else {""},
                                        );
                        html! {
                            <li
                                key={format!("{}-{}",choice.field,choice.value)}
                                role="option"
                                aria-selected={(i == *highlighted).to_string()}
                            >
                                <button
                                    type="button"
                                    class={element_classes_1}
                                    onclick={on_choose_suggestion}
                                >
                                {choice.label.clone()}
                                </button>
                                </li>}
                    })}
                </ul>
            }
        } else {
            Html::default()
        }
    };
    let clear_filters_action = {
        if has_filters {
            html! {
                <button
                    data-browse-clear=""
                    type="button"
                    aria-label="Clear all filters"
                    title="Clear all filters"
                    class="absolute right-1 top-6 inline-flex size-9 items-center justify-center rounded-lg text-muted hover:bg-accent-soft hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"
                    onclick={clear}
                >
                    <Icon
                        data={IconData::LUCIDE_X}
                        width="18px"
                        height="18px"
                    />
                </button>
            }
        } else {
            Html::default()
        }
    };

    html! {
        <section
            data-component="browse-search-form"
            class="relative flex flex-col gap-3 rounded-xl border border-line bg-surface p-4 shadow-sm"
        >
            <label
                class="flex flex-col gap-1 text-xs text-muted"
            >
                <span>
                    {"Dictionary search"}
                </span>
                <input
                    data-browse-query=""
                    type="search"
                    aria-label="Search dictionary"
                    class="min-h-12 w-full rounded-lg border border-line bg-surface px-4 text-base"
                    value={state.query_text.clone()}
                    oninput={on_query}
                    placeholder="Search Hanzi, pinyin or English meanings"
                    maxlength="128"
                />
            </label>
            <div
                class="relative"
            >
                <label
                    class="flex flex-col gap-1 text-xs text-muted"
                >
                    <span>
                        {"Add a filter · use ↑ ↓ and Enter to choose"}
                    </span>
                    <input
                        data-browse-filter-autocomplete=""
                        type="text"
                        role="combobox"
                        aria-label="Add dictionary filter"
                        aria-autocomplete="list"
                        aria-expanded={(*focused).to_string()}
                        aria-controls="browse-filter-suggestions"
                        class="min-h-11 w-full rounded-lg border border-line bg-surface px-4 pr-12 text-sm"
                        value={(*filter_text).clone()}
                        oninput={on_filter_input}
                        onkeydown={on_filter_key}
                        onfocus={on_filter_focus}
                        placeholder="Try HSK 3, noun, animals or English…"
                    />
                </label>
                {clear_filters_action}
                {search_suggestions_content}
            </div>
            <div
                class="flex min-h-9 flex-wrap items-center gap-2"
            >
                {for chips.into_iter().map(|(label,kind)| {
                    let state = state.clone();
                    let on_remove_filter = Callback::from(move |_| {
                        let action = match kind {
                            "field" => BrowseInput::Field("all".to_owned()),
                            "hsk" => BrowseInput::Hsk(String::new()),
                            "pos" => BrowseInput::PartOfSpeech(String::new()),
                            _ => BrowseInput::Category(String::new()),
                        };
                        state.dispatch(BrowseAction::Input(action));
                    });
                    html! {
                        <button
                            key={kind}
                            type="button"
                            class="rounded-full border border-accent bg-accent-soft px-3 py-1.5 text-xs font-semibold text-accent"
                            aria-label={format!("Remove {label}")}
                            onclick={on_remove_filter}
                        >
                            {format!("{label} ×")}
                        </button>}
                })}

            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hsk_selector_rejects_invalid_levels() {
        assert_eq!(selected_hsk("2:1"), Some(HskLevel::Hsk2(Hsk2Level::One)));
        assert_eq!(
            selected_hsk("3:7"),
            Some(HskLevel::Hsk3(Hsk3Level::SevenToNine))
        );
        assert_eq!(selected_hsk("2:7"), None);
    }
    #[test]
    fn browse_filter_catalog_matches_canonical_hsk_levels() {
        let choices = filter_choices();
        for system in zipeek_core::HskSystem::ALL {
            for level in system.levels() {
                let value = format!("{}:{}", system.code(), level.as_i32());
                let choice = choices
                    .iter()
                    .find(|item| item.field == "hsk" && item.value == value)
                    .expect("every supported HSK level must be selectable");
                assert_eq!(choice.label, level.badge_label());
                assert_eq!(selected_hsk(&choice.value), Some(*level));
            }
        }
        assert_eq!(
            choices.iter().filter(|item| item.field == "hsk").count(),
            13
        );
    }

    #[test]
    fn autocomplete_has_distinct_taxonomy_and_lexical_choices() {
        let choices = filter_choices();
        assert!(
            choices
                .iter()
                .any(|choice| choice.field == "category" && choice.value == "nature")
        );
        assert!(
            choices
                .iter()
                .any(|choice| choice.field == "hsk" && choice.value == "3:7")
        );
        assert!(
            choices
                .iter()
                .any(|choice| choice.field == "field" && choice.value == "english")
        );
    }
}
