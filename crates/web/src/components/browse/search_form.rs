use gloo::timers::callback::Timeout;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_ai::STARTER_SETS;
use zipeek_core::{Hsk2Level, Hsk3Level, HskLevel};

use crate::{
    components::WordSetsContext,
    reducers::browse::{BrowseAction, BrowseInput, BrowseState, PAGE_SIZE},
    services::{database::{BrowseField, BrowseSearch}, word_sets},
};

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

pub(super) fn submit_search(state: &UseReducerHandle<BrowseState>, sets: &Result<Vec<word_sets::UserWordSet>, String>, reset_page: bool) {
    let snapshot = match sets {
        Ok(sets) => word_sets::browse_snapshot(sets, &state.word_set),
        Err(error) if state.word_set.starts_with("saved:") => Err(error.clone()),
        Err(_) => Ok(None),
    };
    let saved_word_ids = match snapshot {
        Ok(ids) => ids,
        Err(error) => {
            state.dispatch(BrowseAction::Invalidate(error));
            return;
        }
    };
    let request = BrowseSearch {
        text: state.query_text.clone(),
        field: selected_field(&state.field),
        hsk: selected_hsk(&state.hsk),
        part_of_speech: if state.pos.is_empty() { vec![] } else { vec![state.pos.clone()] },
        categories: if state.category.is_empty() { vec![] } else { vec![state.category.clone()] },
        word_set: if state.word_set.is_empty() || state.word_set.starts_with("saved:") { None } else { Some(state.word_set.clone()) },
        saved_word_ids,
        offset: if reset_page { 0 } else { state.offset },
        limit: PAGE_SIZE,
    };
    state.dispatch(BrowseAction::Search(request));
}

#[function_component(BrowseSearchForm)]
pub(crate) fn browse_search_form() -> Html {
    let state = use_context::<UseReducerHandle<BrowseState>>().unwrap();
    let sets = use_context::<WordSetsContext>().unwrap();
    let on_query = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| state.dispatch(BrowseAction::Input(BrowseInput::QueryText(event.target_unchecked_into::<HtmlInputElement>().value()))))
    };
    let on_field = {
        let state = state.clone();
        Callback::from(move |event: Event| state.dispatch(BrowseAction::Input(BrowseInput::Field(event.target_unchecked_into::<HtmlSelectElement>().value()))))
    };
    let on_hsk = {
        let state = state.clone();
        Callback::from(move |event: Event| state.dispatch(BrowseAction::Input(BrowseInput::Hsk(event.target_unchecked_into::<HtmlSelectElement>().value()))))
    };
    let on_pos = {
        let state = state.clone();
        Callback::from(move |event: Event| state.dispatch(BrowseAction::Input(BrowseInput::PartOfSpeech(event.target_unchecked_into::<HtmlSelectElement>().value()))))
    };
    let on_category = {
        let state = state.clone();
        Callback::from(move |event: Event| state.dispatch(BrowseAction::Input(BrowseInput::Category(event.target_unchecked_into::<HtmlSelectElement>().value()))))
    };
    let on_set = {
        let state = state.clone();
        Callback::from(move |event: Event| state.dispatch(BrowseAction::Input(BrowseInput::WordSet(event.target_unchecked_into::<HtmlSelectElement>().value()))))
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
            state.word_set.clone(),
            sets_snapshot.clone(),
        );
        use_effect_with(dependencies, move |(query, _, _, _, _, _, _)| {
            let delay = if *last_query.borrow() != *query { 320 } else { 0 };
            *last_query.borrow_mut() = query.clone();
            let timeout = Timeout::new(delay, move || {
                submit_search(&state, &sets_snapshot, true);
            });
            move || {
                drop(timeout);
            }
        });
    }
    let refresh = {
        let state = state.clone();
        let sets = sets.clone();
        Callback::from(move |_| submit_search(&state, &sets.state.sets, true))
    };
    let clear = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(BrowseAction::ClearFilters))
    };
    let options = |entries: &[(&str, &str)]| html! {
        {for entries.iter().map(|(value, label)| html! { <option value={value.to_string()}>{label.to_string()}</option> })}
    };
    html! {
        <div data-component="browse-search-form" class="flex flex-col gap-3 rounded-xl border border-line bg-surface p-3 shadow-sm sm:p-4">
            <div class="flex items-center gap-2">
                <label class="min-w-0 flex-1">
                    <span class="sr-only">{"Search dictionary"}</span>
                    <input data-browse-query="" type="search" aria-label="Search dictionary" class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm" value={state.query_text.clone()} oninput={on_query} placeholder="Search hanzi, pinyin, or English" maxlength="128" />
                </label>
                <button data-browse-refresh="" type="button" aria-label="Refresh results" title="Refresh results" class="flex min-h-10 min-w-10 items-center justify-center rounded-lg border border-line hover:border-accent" onclick={refresh}>
                    <Icon data={IconData::LUCIDE_REFRESH_CW} width="18px" height="18px" />
                </button>
                <button data-browse-clear="" type="button" class="min-h-10 rounded-lg border border-line px-3 text-sm hover:border-accent" onclick={clear}> {"Clear"} </button>
            </div>
            <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-5">
                <label class="flex min-w-0 flex-col gap-1 text-xs text-muted">{"Search in"}
                    <select data-browse-field="" class="min-h-10 w-full rounded-lg border border-line bg-surface px-2 text-sm" value={state.field.clone()} onchange={on_field}>
                        {options(&[("all","All fields"),("hanzi","Hanzi"),("pinyin","Pinyin"),("english","English meanings")])}
                    </select>
                </label>
                <label class="flex min-w-0 flex-col gap-1 text-xs text-muted">{"HSK"}
                    <select data-browse-hsk="" class="min-h-10 w-full rounded-lg border border-line bg-surface px-2 text-sm" value={state.hsk.clone()} onchange={on_hsk}>
                        <option value="">{"Any level"}</option>
                        {for (1..=6).map(|level| html! { <option value={format!("2:{level}")}>{format!("HSK 2.0 · {level}")}</option> })}
                        {for (1..=7).map(|level| html! { <option value={format!("3:{level}")}>{if level == 7 {"HSK 3.0 · 7–9".to_owned()} else {format!("HSK 3.0 · {level}")}}</option> })}
                    </select>
                </label>
                <label class="flex min-w-0 flex-col gap-1 text-xs text-muted">{"Part of speech"}
                    <select data-browse-pos="" class="min-h-10 w-full rounded-lg border border-line bg-surface px-2 text-sm" value={state.pos.clone()} onchange={on_pos}>
                        {options(&[("","Any part of speech"),("noun","Noun"),("verb","Verb"),("adjective","Adjective"),("adverb","Adverb"),("pronoun","Pronoun"),("classifier","Classifier"),("preposition","Preposition"),("conjunction","Conjunction")])}
                    </select>
                </label>
                <label class="flex min-w-0 flex-col gap-1 text-xs text-muted">{"Category"}
                    <select data-browse-category="" class="min-h-10 w-full rounded-lg border border-line bg-surface px-2 text-sm" value={state.category.clone()} onchange={on_category}>
                        {options(&[("","Any category"),("travel","Travel"),("transport","Transport"),("food","Food and drink"),("shopping","Shopping"),("family","Family"),("education","Education"),("work","Work"),("health","Health"),("nature","Nature"),("time","Time"),("home","Home"),("communication","Communication")])}
                    </select>
                </label>
                <label class="flex min-w-0 flex-col gap-1 text-xs text-muted">{"Word Set"}
                    <select data-browse-word-set="" class="min-h-10 w-full rounded-lg border border-line bg-surface px-2 text-sm" value={state.word_set.clone()} onchange={on_set}>
                        <option value="">{"All words"}</option>
                        {for STARTER_SETS.iter().map(|set| html! { <option value={set.id}>{set.name}</option> })}
                        {if let Ok(saved) = &sets.state.sets { html! {<optgroup label="My Word Sets">{for saved.iter().enumerate().map(|(index, set)| html! {<option value={format!("saved:{index}")}>{format!("{} ({} words)", set.name, set.word_ids.len())}</option>})}</optgroup>} } else {Html::default()}}
                    </select>
                </label>
            </div>
        </div>
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
