use super::state::{CloudSetupState, SetupAction};
use crate::services::word_sets::UserWordSet;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct VocabularySourceProps {
    pub setup: UseReducerHandle<CloudSetupState>,
    pub sets: Vec<UserWordSet>,
}

#[function_component(VocabularySource)]
pub(super) fn vocabulary_source(props: &VocabularySourceProps) -> Html {
    let setup = props.setup.clone();
    let on_hsk = {
        let setup = setup.clone();
        Callback::from(move |_| setup.dispatch(SetupAction::Source("hsk".to_owned())))
    };
    let on_sets = {
        let setup = setup.clone();
        Callback::from(move |_| setup.dispatch(SetupAction::Source("sets".to_owned())))
    };
    let on_word_set = {
        let setup = setup.clone();
        Callback::from(move |event: Event| {
            let select: web_sys::HtmlSelectElement = event.target_unchecked_into();
            setup.dispatch(SetupAction::WordSet(select.value()));
        })
    };
    let hsk_classes = classes!(
        "rounded-lg",
        "px-4",
        "py-2",
        "font-semibold",
        if setup.source == "hsk" {
            "bg-brand-action"
        } else {
            "border"
        },
        if setup.source == "hsk" {
            "text-white"
        } else {
            "border-line"
        }
    );
    let sets_classes = classes!(
        "rounded-lg",
        "px-4",
        "py-2",
        "font-semibold",
        if setup.source == "sets" {
            "bg-brand-action"
        } else {
            "border"
        },
        if setup.source == "sets" {
            "text-white"
        } else {
            "border-line"
        }
    );
    html! {
        <div class="space-y-3">
            <div class="flex flex-wrap gap-2">
                <button type="button" class={hsk_classes} aria-pressed={(setup.source == "hsk").to_string()} onclick={on_hsk}>{"HSK vocabulary"}</button>
                <button type="button" class={sets_classes} aria-pressed={(setup.source == "sets").to_string()} onclick={on_sets}>{"My Word Sets"}</button>
            </div>
            {if setup.source == "hsk" {
                html! { <HskSelector setup={setup.clone()} /> }
            } else {
                html! {
                    <div class="space-y-2">
                        <label for="cloud-match-word-set" class="font-semibold">{"Word Set"}</label>
                        <select id="cloud-match-word-set" class="w-full rounded-lg border border-line bg-raised px-4 py-3" onchange={on_word_set}>
                            {for props.sets.iter().map(|set| html! {
                                <option value={set.id.to_string()} selected={setup.selected_set == set.id.to_string()}>{format!("{} ({} words)", set.name, set.word_ids.len())}</option>
                            })}
                        </select>
                    </div>
                }
            }}
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct HskSelectorProps {
    setup: UseReducerHandle<CloudSetupState>,
}

#[function_component(HskSelector)]
fn hsk_selector(props: &HskSelectorProps) -> Html {
    let setup = props.setup.clone();
    html! {
        <div class="space-y-3">
            <p class="font-semibold">{"HSK standard"}</p>
            <div class="flex gap-2">
                {for [2u8, 3u8].into_iter().map(|standard| {
                    let selected = setup.standard == standard;
                    let on_select = {
                        let setup = setup.clone();
                        Callback::from(move |_| setup.dispatch(SetupAction::Standard(standard)))
                    };
                    let standard_classes = classes!("rounded-lg", "px-4", "py-2", if selected {"bg-brand-action"} else {"border"}, if selected {"text-white"} else {"border-line"});
                    html! {<button type="button" aria-pressed={selected.to_string()} class={standard_classes} onclick={on_select}>{format!("HSK {standard}.0")}</button>}
                })}
            </div>
            <p class="font-semibold">{"Level"}</p>
            <div class="flex flex-wrap gap-2">
                {for (1..=if setup.standard == 2 {6} else {7}).map(|level| {
                    let selected = setup.level == level;
                    let on_select = {
                        let setup = setup.clone();
                        Callback::from(move |_| setup.dispatch(SetupAction::Level(level)))
                    };
                    let level_classes = classes!("min-w-10", "rounded-lg", "border", "px-3", "py-2", if selected {"border-blue-500"} else {"border-line"}, if selected {"bg-brand-action"} else {""}, if selected {"text-white"} else {""});
                    html! {<button type="button" aria-pressed={selected.to_string()} class={level_classes} onclick={on_select}>{if level == 7 {"7–9".to_owned()} else {level.to_string()}}</button>}
                })}
            </div>
        </div>
    }
}
