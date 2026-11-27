use yew::prelude::*;

use crate::reducers::practice_setup_flow::PracticeSetupAction;
use zipeek_exercises::practice_draft::PracticeDraft;

use super::super::practice_format_settings::PracticeFormatSettings;

#[derive(Properties, PartialEq)]
pub(super) struct MatchingSettingsProps {
    pub draft: PracticeDraft,
    pub family: &'static str,
    pub pair_count: u8,
    pub ai_generated: bool,
    pub show_pinyin: bool,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(MatchingSettings)]
pub(super) fn matching_settings(props: &MatchingSettingsProps) -> Html {
    let on_toggle = {
        let on_action = props.on_action.clone();
        Callback::from(move |variant| {
            on_action.emit(PracticeSetupAction::ToggleFormat(variant));
        })
    };
    let on_order = {
        let on_action = props.on_action.clone();
        Callback::from(move |order| {
            on_action.emit(PracticeSetupAction::SetMatchingColumnOrder(order));
        })
    };
    let on_pairs = {
        let on_action = props.on_action.clone();
        Callback::from(move |count| {
            on_action.emit(PracticeSetupAction::SetMatchingPairs(count));
        })
    };
    let on_source = {
        let on_action = props.on_action.clone();
        Callback::from(move |enabled: bool| {
            on_action.emit(PracticeSetupAction::SetMatchingAiGenerated(enabled))
        })
    };
    let on_pinyin = {
        let on_action = props.on_action.clone();
        Callback::from(move |enabled: bool| {
            on_action.emit(PracticeSetupAction::SetMatchingShowPinyin(enabled))
        })
    };
    html! { <div class="space-y-4">
        {if props.family == "matching" { html! {
            <fieldset class="space-y-2 rounded-xl border border-line bg-surface p-4" data-matching-source="">
                <legend class="font-semibold">{"Question source"}</legend>
                <div class="flex flex-wrap gap-2">
                    {for [(false, "Existing vocabulary"), (true, "AI-generated")].into_iter().map(|(value, name)| {
                        let callback = on_source.clone();
                        let choose = Callback::from(move |_: MouseEvent| callback.emit(value));
                        html! {<button type="button" key={name} onclick={choose}
                            data-matching-ai={value.to_string()}
                            aria-pressed={(props.ai_generated == value).to_string()}
                            class={classes!("rounded-lg", "border-2", "px-4", "py-3", "font-medium", if props.ai_generated == value {"border-brand-action bg-accent-soft"} else {"border-line hover:bg-raised"})}>{name}</button>}
                    })}
                </div>
                {if props.ai_generated { html! {<p class="text-sm text-muted">{"Questions are generated asynchronously and saved with the session. AI content may contain mistakes; no personal word lists are sent to the model."}</p>} } else { Html::default() }}
            </fieldset>
        } } else { Html::default() }}
        {if props.family == "matching" { html! {
            <fieldset class="space-y-2 rounded-xl border border-line bg-surface p-4" data-matching-pinyin="">
                <legend class="font-semibold">{"Pinyin under Hanzi"}</legend>
                <div class="flex flex-wrap gap-2">
                    {for [(true, "Show"), (false, "Hide")].into_iter().map(|(value, name)| {
                        let callback = on_pinyin.clone();
                        let choose = Callback::from(move |_: MouseEvent| callback.emit(value));
                        html! { <button type="button" key={name} onclick={choose} aria-pressed={(props.show_pinyin == value).to_string()}
                            class={classes!("rounded-lg", "border-2", "px-4", "py-2", if props.show_pinyin == value {"border-brand-action bg-accent-soft"} else {"border-line hover:bg-raised"})}>{name}</button> }
                    })}
                </div>
            </fieldset>
        }} else { Html::default() }}
        <PracticeFormatSettings
            draft={props.draft.clone()}
            family={props.family.to_owned()}
            on_toggle={on_toggle}
            on_column_order={on_order}
            pair_count={props.pair_count}
            on_pair_count={on_pairs}
        />
        </div>
    }
}
