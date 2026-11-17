use yew::prelude::*;

use crate::{
    components::practice::shared::checkbox::PracticeCheckbox, models::VocabularyExerciseOptions,
};

#[derive(Properties, PartialEq)]
pub(crate) struct PracticeAnswerSettingsProps {
    pub options: VocabularyExerciseOptions,
    pub on_retry_incorrect: Callback<bool>,
    pub on_show_audio: Callback<bool>,
    pub on_auto_check: Callback<bool>,
    pub on_pinyin_hint: Callback<bool>,
}

#[function_component(PracticeAnswerSettings)]
pub(crate) fn practice_answer_settings(props: &PracticeAnswerSettingsProps) -> Html {
    let is_builder = matches!(
        props.options,
        VocabularyExerciseOptions::CharacterBuilder(_)
    );
    html! {
        <fieldset class="space-y-2 rounded-xl border border-line bg-surface p-5" data-practice-answer-options="">
            <legend class="px-2 font-semibold">{"Answer options"}</legend>
            <PracticeCheckbox
                checked={props.options.retry_incorrect()}
                on_change={props.on_retry_incorrect.clone()}
                label="Allow retry after an incorrect answer"
                test_id="retry"
            />
            {if is_builder {
                html! {<>
                    <PracticeCheckbox
                        checked={props.options.show_audio()}
                        on_change={props.on_show_audio.clone()}
                        label="Show pronunciation button"
                        test_id="audio"
                    />
                    <PracticeCheckbox
                        checked={props.options.automatic_check()}
                        on_change={props.on_auto_check.clone()}
                        label="Check automatically when all slots are filled"
                        description="Incorrect answers briefly highlight and return to the bank when retry is enabled."
                        test_id="auto-check"
                    />
                </>}
            } else {
                html! {
                    <PracticeCheckbox
                        checked={props.options.show_hint()}
                        on_change={props.on_pinyin_hint.clone()}
                        label="Show the Pinyin reading as an input hint"
                        description="Guided copying, not independent recall."
                        test_id="pinyin-hint"
                    />
                }
            }}
        </fieldset>
    }
}
