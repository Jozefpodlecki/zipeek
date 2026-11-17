use yew::prelude::*;
use zipeek_exercises::practice_draft::TranslationDirection;

use crate::reducers::practice_setup_flow::PracticeSetupAction;

#[derive(Properties, PartialEq)]
pub(super) struct TranslationSettingsProps {
    pub direction: TranslationDirection,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(TranslationSettings)]
pub(super) fn translation_settings(props: &TranslationSettingsProps) -> Html {
    html! {
        <section role="tabpanel" data-translation-setup="" class="rounded-2xl border border-line bg-surface p-6">
            <h3 class="text-lg font-semibold">{"Sentence translation"}</h3>
            <p class="mt-2 text-sm text-muted">
                {"Choose a translation direction. Vocabulary and the number of sentences are selected in the following steps. Configure the AI connection in Settings."}
            </p>
            <fieldset data-translation-direction-setup="" class="mt-4 flex flex-wrap gap-3">
                <legend class="mb-2 text-sm font-semibold">{"Translation direction"}</legend>
                {for [
                    (TranslationDirection::Mixed, "Mixed"),
                    (TranslationDirection::ChineseToEnglish, "Chinese to English"),
                    (TranslationDirection::EnglishToChinese, "English to Chinese"),
                ].into_iter().map(|(direction, label)| {
                    let on_action = props.on_action.clone();
                    let selected = props.direction == direction;
                    let on_change = Callback::from(move |_: Event| {
                        on_action.emit(PracticeSetupAction::SetTranslationDirection(direction));
                    });
                    html! {
                        <label class="flex min-h-11 items-center gap-2 rounded-lg border border-line px-3 py-2 text-sm">
                            <input type="radio" name="practice-translation-direction"
                                value={match direction {
                                    TranslationDirection::Mixed => "mixed",
                                    TranslationDirection::ChineseToEnglish => "zh-to-en",
                                    TranslationDirection::EnglishToChinese => "en-to-zh",
                                }} checked={selected} onchange={on_change}/>
                            {label}
                        </label>
                    }
                })}
            </fieldset>
        </section>
    }
}
