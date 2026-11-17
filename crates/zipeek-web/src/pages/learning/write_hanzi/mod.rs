mod ranking;
mod results;
mod route_query;
mod search;
mod state;
mod workspace;
use gloo::timers::callback::Timeout;
use std::{cell::Cell, rc::Rc};

use crate::hooks::use_dictionary;
use crate::hooks::use_word_sets;
use crate::services::word_sets::{UserWordSet, WordSetId};
use yew::prelude::*;
use yew_router::prelude::use_location;
use zipeek_core::Word;

use results::{visible_words, VocabularyResults};
use state::{WriteHanziAction, WriteHanziState};
use workspace::WritingWorkspace;

#[function_component(WriteHanziPage)]
pub fn write_hanzi_page() -> Html {
    let location = use_location();
    let dictionary = use_dictionary();
    let sets = use_word_sets();
    let state = use_reducer(WriteHanziState::default);
    let direct_word_id = route_query::selected_word_id(location.as_ref());
    {
        let dictionary = dictionary.clone();
        let state = state.clone();
        use_effect_with(direct_word_id, move |requested| {
            let active = Rc::new(Cell::new(true));
            if let Some(id) = *requested {
                let pending = active.clone();
                let dictionary = dictionary.clone();
                let state = state.clone();
                state.dispatch(WriteHanziAction::DirectLoading);
                wasm_bindgen_futures::spawn_local(async move {
                    let response = async {
                        let database = dictionary.database().map_err(|error| error.to_string())?;
                        database
                            .word_by_id(id)
                            .await
                            .map_err(|error| error.to_string())
                    }
                    .await;
                    if pending.get() {
                        state.dispatch(WriteHanziAction::DirectLoaded(response));
                    }
                });
            }
            move || active.set(false)
        });
    }

    {
        let state = state.clone();
        let current = state.query.clone();
        let mode = state.mode;
        use_effect_with((mode, current), move |(mode, query)| {
            let timeout = if *mode == state::VocabularyMode::Search {
                let query = query.trim().to_owned();
                Some(Timeout::new(300, move || {
                    state.dispatch(WriteHanziAction::SubmitSearch(query))
                }))
            } else {
                None
            };
            move || drop(timeout)
        });
    }

    let available_sets = sets.state.sets.as_ref().ok();
    let ids = available_sets
        .and_then(|items| {
            state
                .selected_set
                .as_ref()
                .and_then(|id| crate::services::word_sets::find(items, id))
                .or_else(|| {
                    if state.selected_set.is_none() {
                        items.first()
                    } else {
                        None
                    }
                })
        })
        .map(|set| set.word_ids.clone())
        .unwrap_or_default();

    {
        let dictionary = dictionary.clone();
        let state = state.clone();
        use_effect_with(
            (state.mode, state.selected_set.clone(), ids, state.submitted.clone()),
            move |(mode, selected_set, ids, submitted)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                let source_mode = *mode;
                let mode = mode.as_str().to_owned();
                let selected_set = selected_set.clone();
                let ids = ids.clone();
                let submitted = submitted.clone();
                state.dispatch(WriteHanziAction::SearchLoading);
                let state = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let response = search::load(&dictionary, &mode, &ids, &submitted).await;
                    if pending.get() {
                        state.dispatch(WriteHanziAction::SearchLoaded {
                            mode: source_mode,
                            selected_set,
                            submitted,
                            result: response,
                        });
                    }
                });
                move || alive.set(false)
            },
        );
    }

    let choose_word = {
        let state = state.clone();
        Callback::from(move |word: Word| state.dispatch(WriteHanziAction::ChooseWord(word)))
    };
    let switch_mode = {
        let state = state.clone();
        Callback::from(move |next: state::VocabularyMode| {
            state.dispatch(WriteHanziAction::SwitchMode(next))
        })
    };
    let on_set_change = {
        let state = state.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlSelectElement = event.target_unchecked_into();
            if let Ok(index) = input
                .value()
                .parse::<crate::services::word_sets::WordSetId>()
            {
                state.dispatch(WriteHanziAction::SelectSet(index));
            }
        })
    };
    let on_query_input = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            state.dispatch(WriteHanziAction::QueryChanged(input.value()));
        })
    };

    let on_search_keydown = {
        let state = state.clone();
        Callback::from(move |event: KeyboardEvent| {
            match event.key().as_str() {
                "ArrowDown" => {
                    event.prevent_default();
                    state.dispatch(WriteHanziAction::MoveHighlight(1));
                }
                "ArrowUp" => {
                    event.prevent_default();
                    state.dispatch(WriteHanziAction::MoveHighlight(-1));
                }
                "Escape" => state.dispatch(WriteHanziAction::CloseSuggestions),
                "Enter" => {
                    if !state.suggestions_open { return; }
                    let props = results::VocabularyResultsProps {
                        results: state.results.clone(),
                        mode: state.mode.as_str().to_owned(),
                        query: state.query.clone(),
                        submitted: state.submitted.clone(),
                        selected: state.chosen.clone(),
                        expanded: state.suggestions_open,
                        highlighted: state.highlighted,
                        visible_count: state.visible_count,
                        on_more: Callback::noop(),
                        on_select: Callback::noop(),
                        on_open: Callback::noop(),
                    };
                    let matches = visible_words(&props);
                    if let Some(word) = matches.get(state.highlighted) {
                        event.prevent_default();
                        state.dispatch(WriteHanziAction::ChooseWord(word.clone()));
                    }
                }
                _ => {}
            }
        })
    };
    let on_search_focus = {
        let state = state.clone();
        Callback::from(move |_: FocusEvent| state.dispatch(WriteHanziAction::OpenSuggestions))
    };
    let on_more = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(WriteHanziAction::ShowMore))
    };
    let on_open_suggestions = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(WriteHanziAction::OpenSuggestions))
    };
    let on_character = {
        let state = state.clone();
        Callback::from(move |index| state.dispatch(WriteHanziAction::SelectCharacter(index)))
    };

    let direct_word_status = {
        if !state.direct_status.is_empty() {
            html! {
            <p
                role="status"
                data-write-hanzi-deeplink-status=""
                class="text-xs text-muted"
            >
                {state.direct_status.clone()}
            </p>}
        } else {
            Html::default()
        }
    };
    html! {
        <main
            data-component="write-hanzi-page"
            class="mx-auto flex w-full max-w-7xl flex-col gap-3 p-2 sm:p-3 writing-desktop:h-full writing-desktop:min-h-0 writing-desktop:overflow-hidden writing-desktop:!p-0"
        >
            <div
                class="grid min-h-0 items-start gap-3 lg:grid-cols-[minmax(270px,320px)_minmax(0,1fr)] writing-desktop:h-full writing-desktop:flex-1 writing-desktop:items-stretch"
            >
                <section
                    class="min-h-0 min-w-0 rounded-2xl border border-line bg-surface p-3 space-y-3 writing-desktop:flex writing-desktop:flex-col writing-desktop:overflow-hidden"
                    aria-label="Choose vocabulary for writing"
                >
                    <h2
                        class="text-base font-semibold"
                    >
                        {"Vocabulary finder"}
                    </h2>
                    {direct_word_status}
                    <VocabularySourceControls
                        mode={state.mode}
                        selected_set={state.selected_set.clone()}
                        query={state.query.clone()}
                        submitted={state.submitted.clone()}
                        loading={state.results.is_none()}
                        available_sets={available_sets.cloned()}
                        on_switch_mode={switch_mode}
                        on_set_change={on_set_change}
                        on_query_input={on_query_input}
                        on_keydown={on_search_keydown}
                        on_focus={on_search_focus}
                        expanded={state.suggestions_open}
                        highlighted={state.highlighted}
                    />
                    <VocabularyResults
                        results={state.results.clone()}
                        mode={state.mode.as_str().to_owned()}
                        submitted={state.submitted.clone()}
                        query={state.query.clone()}
                        selected={state.chosen.clone()}
                        expanded={state.suggestions_open}
                        highlighted={state.highlighted}
                        visible_count={state.visible_count}
                        on_more={on_more}
                        on_select={choose_word}
                        on_open={on_open_suggestions}
                    />
                </section>
                <WritingWorkspace
                    word={state.chosen.clone()}
                    character_index={state.character_index}
                    on_character={on_character}
                />
            </div>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct VocabularySourceControlsProps {
    mode: state::VocabularyMode,
    selected_set: Option<WordSetId>,
    query: String,
    submitted: String,
    loading: bool,
    available_sets: Option<Vec<UserWordSet>>,
    on_switch_mode: Callback<state::VocabularyMode>,
    on_set_change: Callback<Event>,
    on_query_input: Callback<InputEvent>,
    on_keydown: Callback<KeyboardEvent>,
    on_focus: Callback<FocusEvent>,
    expanded: bool,
    highlighted: usize,
}

#[function_component(VocabularySourceControls)]
fn vocabulary_source_controls(props: &VocabularySourceControlsProps) -> Html {
    let available_sets = props.available_sets.as_deref();
    let on_set_change = props.on_set_change.clone();
    let on_query_input = props.on_query_input.clone();
    let vocabulary_mode_content = {
        if props.mode == state::VocabularyMode::Sets {
            match available_sets {
                None => html! {
                <p
                    role="alert"
                    class="text-sm text-red-500"
                >
                    {"Saved Word Sets could not be read."}
                </p>},
                Some(items) if items.is_empty() => html! {
                <p
                    class="text-sm text-muted"
                >
                    {"No saved Word Sets yet. Add words in Dictionary or use Search vocabulary."}
                </p>},
                Some(items) => html! {
                    <div
                        class="space-y-2"
                    >
                        <label
                            for="write-hanzi-set"
                            class="text-sm font-semibold"
                        >
                            {"Word Set"}
                        </label>
                        <select
                            id="write-hanzi-set"
                            class="w-full rounded-lg border border-line bg-raised p-3"
                            value={props.selected_set.as_ref().map(ToString::to_string).or_else(|| items.first().map(|set| set.id.to_string())).unwrap_or_default()}
                            onchange={on_set_change}
                        >
                            {for items.iter().map(|set| html! {
                                <option
                                    value={set.id.to_string()}
                                >
                                    {format!("{} ({} words)", set.name, set.word_ids.len())}
                                </option>})}
                        </select>
                    </div>
                },
            }
        } else {
            Html::default()
        }
    };
    let search_progress_indicator = if props.mode == state::VocabularyMode::Search
        && (props.query.trim() != props.submitted.trim()
            || props.loading && !props.query.trim().is_empty()) {
        html! {<span role="status" aria-label="Searching" class="h-4 w-4 shrink-0 animate-spin rounded-full border-2 border-blue-400 border-t-transparent" />}
    } else { Html::default() };
    html! {
        <>
                    <div
                        class="grid grid-cols-2 gap-2"
                        role="group"
                        aria-label="Vocabulary source"
                    >
                        {for [(state::VocabularyMode::Sets, "My Word Sets"), (state::VocabularyMode::Search, "Search vocabulary")].into_iter().map(|(value, label)| {
                            let switch_mode = props.on_switch_mode.clone();
                            let on_switch_mode = Callback::from(move |_| switch_mode.emit(value));
                            let element_classes_1 = classes!(
                                            "rounded-lg",
                                            "border",
                                            "px-3",
                                            "py-2",
                                            "text-sm",
                                            "font-semibold",
                                            if props.mode == value { "border-blue-500 bg-brand-action text-white" }
                                        else { "border-line bg-raised text-muted" },
                                        );
                            html! {
                                <button
                                    key={value.as_str()}
                                    type="button"
                                    aria-pressed={(props.mode == value).to_string()}
                                    class={element_classes_1}
                                    onclick={on_switch_mode}
                                >
                                    {label}
                                </button>
                            }
                        })}
                    </div>
            {vocabulary_mode_content}
            <div class="space-y-2">
                <label for="write-hanzi-search" class="text-sm font-semibold">{"Find a word"}</label>
                <div class="flex items-center gap-2 rounded-lg border border-line bg-raised px-2 focus-within:border-accent/60">
                    <input
                        id="write-hanzi-search"
                        type="text"
                        maxlength="128"
                        role="combobox"
                        aria-autocomplete="list"
                        aria-controls="write-hanzi-suggestions"
                        aria-expanded={props.expanded.to_string()}
                        aria-activedescendant={if props.expanded && !props.loading {Some(format!("write-hanzi-suggestion-{}", props.highlighted))} else {None}}
                        autocomplete="off"
                        placeholder={if props.mode == state::VocabularyMode::Sets {"Filter this set"} else {"English, Hanzi or Pinyin"}}
                        class="min-w-0 flex-1 border-0 bg-transparent px-2 py-2 text-sm outline-none shadow-none focus:outline-none focus:ring-0 focus:shadow-none write-hanzi-combobox"
                        value={props.query.clone()}
                        oninput={props.on_query_input.clone()}
                        onkeydown={props.on_keydown.clone()}
                        onfocus={props.on_focus.clone()}
                    />
                    {search_progress_indicator}
                </div>
                <p class="text-xs text-muted">{"Type to search, use ↑ ↓ to navigate and Enter to select."}</p>
            </div>
        </>
    }
}
