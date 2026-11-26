use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::{
    models::{StrokeAssistance, WriteHanziOptions, WritingGuidance},
    reducers::practice_setup_flow::PracticeSetupAction,
};

#[derive(Properties, PartialEq)]
pub(super) struct WritingSettingsProps {
    pub options: WriteHanziOptions,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(WritingSettings)]
pub(super) fn writing_settings(props: &WritingSettingsProps) -> Html {
    let on_guidance = {
        let on_action = props.on_action.clone();
        Callback::from(move |event: Event| {
            let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            let guidance = match input.value().as_str() {
                "none" => WritingGuidance::None,
                "start" => WritingGuidance::StartCue,
                "full" => WritingGuidance::FullGuide,
                _ => return,
            };
            on_action.emit(PracticeSetupAction::SetWritingGuidance(guidance));
        })
    };
    let on_assistance = {
        let on_action = props.on_action.clone();
        Callback::from(move |event: Event| {
            let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            let assistance = match input.value().as_str() {
                "three" => StrokeAssistance::AfterThree,
                "five" => StrokeAssistance::AfterFive,
                "off" => StrokeAssistance::Off,
                _ => return,
            };
            on_action.emit(PracticeSetupAction::SetWritingAssistance(assistance));
        })
    };
    let on_animation = {
        let on_action = props.on_action.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                on_action.emit(PracticeSetupAction::SetWritingAnimation(input.checked()));
            }
        })
    };
    let on_pinyin = toggle_setting(
        props.on_action.clone(),
        PracticeSetupAction::SetWritingPinyin,
    );
    let on_meaning = toggle_setting(
        props.on_action.clone(),
        PracticeSetupAction::SetWritingMeaning,
    );
    let on_audio = toggle_setting(
        props.on_action.clone(),
        PracticeSetupAction::SetWritingAudio,
    );
    let selected = props.options.guidance;
    let selected_key = match selected {
        WritingGuidance::None => "none",
        WritingGuidance::StartCue => "start",
        WritingGuidance::FullGuide => "full",
    };
    html! {
        <section role="tabpanel" data-write-hanzi-settings="" class="space-y-5 rounded-2xl border border-line bg-surface p-5">
            <h3 class="text-lg font-semibold">{"Write Hanzi"}</h3>
            <fieldset class="space-y-3" data-writing-guidance={selected_key}>
                <legend class="text-sm font-semibold">{"Guidance"}</legend>
                {for [
                    ("none", WritingGuidance::None, "None", "Write without stroke hints"),
                    ("start", WritingGuidance::StartCue, "Stroke hint", "Preview the next stroke direction, with more help after mistakes"),
                    ("full", WritingGuidance::FullGuide, "Full guide", "Show the next stroke outline and starting point"),
                ].into_iter().map(|(id, value, title, detail)| html! {
                    <label key={id} class="flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border border-line bg-raised p-3">
                        <input type="radio" name="write-hanzi-guidance" value={id} checked={selected == value}
                            onchange={on_guidance.clone()} data-writing-guidance-option={id} />
                        <span class="flex flex-col gap-1">
                            <span class="font-semibold">{title}</span>
                            <span class="text-xs text-muted">{detail}</span>
                        </span>
                    </label>
                })}
            </fieldset>
            <label class="flex min-h-11 items-center gap-3 rounded-lg border border-line bg-raised p-3">
                <input type="checkbox" checked={props.options.animation} onchange={on_animation}
                    data-writing-animation-enabled="" />
                <span class="flex flex-col gap-1">
                    <span class="font-semibold">{"Enable stroke-order animation"}</span>
                    <span class="text-xs text-muted">{"Show the Watch control while practising"}</span>
                </span>
            </label>
            <fieldset class="space-y-3" data-writing-assistance="">
                <legend class="text-sm font-semibold">{"Help after mistakes"}</legend>
                <p class="text-xs text-muted">{"After repeated unsuccessful attempts, demonstrate and complete only the current stroke. Assisted strokes are counted separately."}</p>
                {for [
                    ("three", StrokeAssistance::AfterThree, "After 3 mistakes"),
                    ("five", StrokeAssistance::AfterFive, "After 5 mistakes"),
                    ("off", StrokeAssistance::Off, "Never advance automatically"),
                ].into_iter().map(|(id, value, label)| html! {
                    <label key={id} class="flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border border-line bg-raised p-3">
                        <input type="radio" name="write-hanzi-assistance" value={id}
                            checked={props.options.assistance == value} onchange={on_assistance.clone()}
                            data-writing-assistance-option={id} />
                        <span class="font-medium">{label}</span>
                    </label>
                })}
            </fieldset>
            <fieldset class="space-y-3" data-writing-prompt-settings="">
                <legend class="text-sm font-semibold">{"Word prompt"}</legend>
                <p class="text-xs text-muted">{"Choose which learning aids appear beneath the Hanzi title."}</p>
                {for [
                    ("pinyin", "Show Pinyin", "Display the dictionary reading", props.options.show_pinyin, on_pinyin),
                    ("meaning", "Show meaning", "Display the dictionary definition", props.options.show_meaning, on_meaning),
                    ("audio", "Play pronunciation", "Show a Mandarin Listen button", props.options.playable_audio, on_audio),
                ].into_iter().map(|(id, title, description, checked, onchange)| html! {
                    <label key={id} class="flex min-h-11 cursor-pointer items-center gap-3 rounded-lg border border-line bg-raised p-3">
                        <input type="checkbox" {checked} {onchange} data-writing-prompt-option={id} />
                        <span class="flex flex-col gap-1">
                            <span class="font-semibold">{title}</span>
                            <span class="text-xs text-muted">{description}</span>
                        </span>
                    </label>
                })}
            </fieldset>
        </section>
    }
}

fn toggle_setting(
    on_action: Callback<PracticeSetupAction>,
    action: fn(bool) -> PracticeSetupAction,
) -> Callback<Event> {
    Callback::from(move |event: Event| {
        if let Some(input) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            on_action.emit(action(input.checked()));
        }
    })
}
