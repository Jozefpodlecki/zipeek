use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::{
    models::{ListeningChoiceOptions, ListeningHintVisibility},
    reducers::practice_setup_flow::PracticeSetupAction,
};

#[derive(Properties, PartialEq)]
pub(super) struct ListeningSettingsProps {
    pub options: ListeningChoiceOptions,
    pub on_action: Callback<PracticeSetupAction>,
}

fn select_visibility(value: &str) -> Option<ListeningHintVisibility> {
    match value {
        "never" => Some(ListeningHintVisibility::Never),
        "after" => Some(ListeningHintVisibility::AfterAnswer),
        "always" => Some(ListeningHintVisibility::Always),
        _ => None,
    }
}

#[function_component(ListeningSettings)]
pub(super) fn listening_settings(props: &ListeningSettingsProps) -> Html {
    let on_pinyin = {
        let dispatch = props.on_action.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target().and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                if let Some(value) = select_visibility(&input.value()) {
                    dispatch.emit(PracticeSetupAction::SetListeningPinyin(value));
                }
            }
        })
    };
    let on_meaning = {
        let dispatch = props.on_action.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target().and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                if let Some(value) = select_visibility(&input.value()) {
                    dispatch.emit(PracticeSetupAction::SetListeningMeaning(value));
                }
            }
        })
    };
    let on_autoplay = {
        let dispatch = props.on_action.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target().and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                dispatch.emit(PracticeSetupAction::SetListeningAutoplay(input.checked()));
            }
        })
    };
    html! {
        <section role="tabpanel" data-listening-settings="" class="space-y-5 rounded-2xl border border-line bg-surface p-5">
            <h3 class="text-lg font-semibold">{"Listening choice"}</h3>
            <p class="text-sm text-muted">{"Choose when to reveal learning aids. Hints are applied to every answer choice."}</p>
            {for [
                ("pinyin", "Show pinyin", props.options.pinyin, on_pinyin),
                ("meaning", "Show meanings", props.options.meaning, on_meaning),
            ].into_iter().map(|(id, title, selected, on_change)| html! {
                <fieldset key={id} class="space-y-2">
                    <legend class="text-sm font-semibold">{title}</legend>
                    {for [
                        ("never", ListeningHintVisibility::Never, "Never"),
                        ("after", ListeningHintVisibility::AfterAnswer, "After answering (recommended)"),
                        ("always", ListeningHintVisibility::Always, "Always"),
                    ].into_iter().map(|(key, value, label)| html! {
                        <label key={key} class="flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border border-line bg-raised p-3">
                            <input type="radio" name={format!("listening-{id}")} value={key} checked={selected == value}
                                onchange={on_change.clone()} data-listening-visibility={format!("{id}-{key}")} />
                            <span>{label}</span>
                        </label>
                    })}
                </fieldset>
            })}
            <label class="flex min-h-11 items-center gap-3 rounded-lg border border-line bg-raised p-3">
                <input type="checkbox" checked={props.options.autoplay} onchange={on_autoplay} data-listening-autoplay="" />
                <span class="flex flex-col gap-1">
                    <span class="font-semibold">{"Play audio automatically"}</span>
                    <span class="text-xs text-muted">{"Attempt playback when the question loads. The speaker remains available for replay."}</span>
                </span>
            </label>
        </section>
    }
}
