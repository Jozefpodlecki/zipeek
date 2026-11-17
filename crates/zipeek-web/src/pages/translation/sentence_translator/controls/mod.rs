mod direction_tabs;
mod sentence_input;
mod submit_action;

use web_sys::KeyboardEvent;
use yew::prelude::*;

use super::state::TranslationDirection;
use direction_tabs::TranslationDirectionTabs;
use sentence_input::TranslationSentenceInput;
use submit_action::TranslationSubmitAction;

#[derive(Properties, PartialEq)]
pub(super) struct TranslatorControlsProps {
    pub direction: TranslationDirection,
    pub source: String,
    pub busy: bool,
    pub model_ready: bool,
    pub on_direction: Callback<TranslationDirection>,
    pub on_source: Callback<InputEvent>,
    pub on_keydown: Callback<KeyboardEvent>,
    pub on_clear: Callback<MouseEvent>,
    pub on_submit: Callback<MouseEvent>,
}

#[function_component(TranslatorControls)]
pub(super) fn translator_controls(props: &TranslatorControlsProps) -> Html {
    html! {
        <div
            class="space-y-4"
        >
            <TranslationDirectionTabs
                direction={props.direction.clone()}
                on_direction={props.on_direction.clone()}
            />
            <section
                class="rounded-2xl border border-line bg-surface p-4 sm:p-5"
            >
                <TranslationSentenceInput
                    direction={props.direction.clone()}
                    source={props.source.clone()}
                    on_source={props.on_source.clone()}
                    on_keydown={props.on_keydown.clone()}
                    on_clear={props.on_clear.clone()}
                />
                <TranslationSubmitAction
                    busy={props.busy}
                    model_ready={props.model_ready}
                    has_source={!props.source.trim().is_empty()}
                    on_submit={props.on_submit.clone()}
                />
            </section>
        </div>
    }
}
