use yew::prelude::*;
use zipeek_exercises::ai_narrative::NarrativeSettings;

use crate::reducers::practice_setup_flow::PracticeSetupAction;

#[derive(Properties, PartialEq)]
pub(super) struct NarrativeSettingsProps {
    pub exercise_id: &'static str,
    pub settings: NarrativeSettings,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(NarrativeSettingsPanel)]
pub(super) fn narrative_settings(props: &NarrativeSettingsProps) -> Html {
    let dialogue = props.exercise_id == "conversation-practice";
    let value = if dialogue {
        props.settings.dialogue_turns
    } else {
        props.settings.passage_sentences
    };
    html! {
        <section role="tabpanel" class="space-y-5 rounded-2xl border border-line bg-surface p-5" data-narrative-settings="">
            <fieldset class="space-y-2">
                <legend class="font-medium">{if dialogue { "Dialogue rounds" } else { "Passage sentences" }}</legend>
                <p class="text-sm text-muted">{if dialogue { "Choose 5 to 10 conversational turns." } else { "Generate a multiline passage of 5 to 10 complete sentences." }}</p>
                <div class="flex flex-wrap gap-2">
                    {for (5_u8..=10).map(|count| {
                        let on_action = props.on_action.clone();
                        let onclick = Callback::from(move |_: MouseEvent| {
                            on_action.emit(if dialogue { PracticeSetupAction::SetDialogueTurns(count) } else { PracticeSetupAction::SetPassageSentences(count) });
                        });
                        html! { <button type="button" aria-pressed={(value == count).to_string()} {onclick}
                            class={classes!("min-h-11", "min-w-11", "rounded-lg", "border", "px-3", "py-2", if value == count { "border-accent bg-accent-soft text-ink" } else { "border-line hover:border-accent" })}>
                            {count}
                        </button> }
                    })}
                </div>
            </fieldset>
            <fieldset class="space-y-2">
                <legend class="font-medium">{"Grammar focus"}</legend>
                <div class="flex flex-wrap gap-2">
                    {for std::iter::once((0_u8, "Automatic".to_owned())).chain((1..=3).filter_map(|i| {
                        zipeek_exercises::sentence_transformation::item_for_seed(0, i)
                            .map(|item| (i, item.target_structure.to_owned()))
                    })).map(|(index, label)| {
                        let on_action = props.on_action.clone();
                        let onclick = Callback::from(move |_: MouseEvent| on_action.emit(PracticeSetupAction::SetNarrativeGrammar(index)));
                        html! { <button type="button" aria-pressed={(props.settings.grammar_pattern_index == index).to_string()} {onclick}
                            class={classes!("min-h-11", "rounded-lg", "border", "px-3", "py-2", if props.settings.grammar_pattern_index == index { "border-accent bg-accent-soft text-ink" } else { "border-line hover:border-accent" })}>
                            {label}
                        </button> }
                    })}
                </div>
            </fieldset>
        </section>
    }
}
