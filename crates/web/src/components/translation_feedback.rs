use yew::prelude::*;
use zipeek_ai::Evaluation;

#[derive(Properties, PartialEq)]
pub struct TranslationFeedbackProps {
    pub evaluation: Evaluation,
    pub reference: String,
    pub on_continue: Callback<MouseEvent>,
}

#[function_component(TranslationFeedback)]
pub fn translation_feedback(props: &TranslationFeedbackProps) -> Html {
    let correct = props.evaluation.accurate;
    let panel_class = if correct {
        "sticky bottom-0 z-10 flex flex-col gap-3 rounded-xl border-2 border-green-600 bg-green-50 p-5 text-green-950 shadow-lg dark:bg-green-950 dark:text-green-50"
    } else {
        "sticky bottom-0 z-10 flex flex-col gap-3 rounded-xl border-2 border-red-600 bg-red-50 p-5 text-red-950 shadow-lg dark:bg-red-950 dark:text-red-50"
    };
    html! {
        <section data-component="translation-feedback" data-translation-feedback="" data-feedback-result={if correct { "correct" } else { "incorrect" }} class={panel_class} role="status" aria-live="polite">
            <h2 class="text-lg font-bold">{if correct { "Correct!" } else { "Not quite right" }}</h2>
            <p>{&props.evaluation.feedback}</p>
            {if correct { Html::default() } else { html! {
                <div class="flex flex-col gap-1">
                    <p class="text-sm font-semibold">{"Reference translation"}</p>
                    <p data-feedback-reference="" class="text-lg font-semibold">{&props.reference}</p>
                </div>
            }}}
            {if !props.evaluation.suggested_translation.trim().is_empty() && props.evaluation.suggested_translation.trim() != props.reference.trim() {
                html! { <p class="text-sm">{format!("Alternative suggested by AI: {}", props.evaluation.suggested_translation)}</p> }
            } else { Html::default() }}
            <button type="button" data-feedback-continue="" class="min-h-12 self-end rounded-lg bg-blue-600 px-6 py-3 font-semibold text-white" onclick={props.on_continue.clone()}>{"Continue"}</button>
        </section>
    }
}
