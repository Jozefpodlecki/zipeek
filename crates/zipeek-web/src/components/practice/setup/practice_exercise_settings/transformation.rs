use yew::prelude::*;
use zipeek_exercises::sentence_transformation::{
    AnswerMode, ITEMS, pattern_index_for_label, selected_pattern_for_query,
};

use crate::models::SentenceTransformationOptions;
use crate::reducers::practice_setup_flow::PracticeSetupAction;

#[derive(Properties, PartialEq)]
pub(super) struct TransformationSettingsProps {
    pub options: SentenceTransformationOptions,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(TransformationSettings)]
pub(super) fn transformation_settings(props: &TransformationSettingsProps) -> Html {
    let search = use_state(|| String::new());
    let selected_label = if props.options.pattern_index == 0 {
        String::new()
    } else {
        ITEMS
            .get(usize::from(props.options.pattern_index - 1))
            .map_or(String::new(), |item| item.target_structure.to_owned())
    };
    let on_search = {
        let search = search.clone();
        let on_action = props.on_action.clone();
        let current_pattern = props.options.pattern_index;
        Callback::from(move |event: InputEvent| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            let value = input.value();
            let selected = selected_pattern_for_query(current_pattern, &value);
            if selected != current_pattern {
                on_action.emit(PracticeSetupAction::SetTransformationPattern(selected));
            }
            search.set(value);
        })
    };
    let choose_all = {
        let search = search.clone();
        let on_action = props.on_action.clone();
        Callback::from(move |_: MouseEvent| {
            search.set(String::new());
            on_action.emit(PracticeSetupAction::SetTransformationPattern(0));
        })
    };
    html! {
        <section role="tabpanel" data-transformation-setup="" class="space-y-5 rounded-2xl border border-line bg-surface p-6">
            <h3 class="text-lg font-semibold">{"Sentence Transformation"}</h3>
            <p class="text-sm text-muted">{"Choose an authored grammar pattern, or practice all available patterns. Search accepts only patterns with curated transformation questions."}</p>
            <div class="space-y-2">
                <label for="transformation-pattern" class="block text-sm font-semibold">{"Grammar pattern"}</label>
                <div class="flex flex-wrap gap-2">
                    <input id="transformation-pattern" data-transformation-pattern-search="" data-transformation-pattern-status={if !(*search).is_empty() && pattern_index_for_label(&search).is_none() {"unsupported"} else {"valid"}} type="text" list="transformation-pattern-options"
                        value={if (*search).is_empty() { selected_label.clone() } else { (*search).clone() }}
                        oninput={on_search} placeholder="All authored patterns (type to search)"
                        autocomplete="off" class="min-w-0 flex-1 rounded-lg border border-line bg-raised px-3 py-2"
                        aria-describedby="transformation-pattern-help" />
                    <button type="button" data-transformation-pattern-all="" onclick={choose_all} class="rounded-lg border border-line px-3 py-2">{"All patterns"}</button>
                </div>
                <datalist id="transformation-pattern-options">
                    {for ITEMS.iter().map(|item| html! {<option key={item.id} value={item.target_structure} />})}
                </datalist>
                <p id="transformation-pattern-help" data-transformation-pattern-help="" class="text-xs text-muted">
                    {if !(*search).is_empty() && pattern_index_for_label(&search).is_none() {
                        "No authored question matches this text. The previously selected pattern remains active until you choose a supported suggestion or All patterns."
                    } else if props.options.pattern_index == 0 {
                        "Using all three available authored patterns. Select a suggestion to change the saved pattern."
                    } else {
                        "Using the selected authored grammar pattern."
                    }}
                </p>
            </div>
            <fieldset class="space-y-3" data-transformation-answer-mode={match props.options.answer_mode {AnswerMode::Tiles => "tiles", AnswerMode::Typing => "typing", AnswerMode::Both => "both"}}>
                <legend class="text-sm font-semibold">{"Answer method"}</legend>
                {for [(AnswerMode::Tiles, "Word and phrase tiles"), (AnswerMode::Typing, "Type the full sentence"), (AnswerMode::Both, "Both (choose per question)")]
                    .into_iter().map(|(mode, label)| {
                        let on_action = props.on_action.clone();
                        let onchange = Callback::from(move |_: Event| on_action.emit(PracticeSetupAction::SetTransformationAnswerMode(mode)));
                        html! {
                            <label class="flex items-center gap-3 rounded-lg border border-line bg-raised px-4 py-3">
                                <input type="radio" data-transformation-answer-option={match mode {AnswerMode::Tiles => "tiles", AnswerMode::Typing => "typing", AnswerMode::Both => "both"}} name="transformation-answer-method" checked={props.options.answer_mode == mode} {onchange} />
                                <span>{label}</span>
                            </label>
                        }
                    })}
            </fieldset>
            <p class="text-xs text-muted">{"Distractors are extra tiles which are not needed in the accepted sentence. Typing mode has no distractor tiles. Practice is not mastery evidence."}</p>
        </section>
    }
}
