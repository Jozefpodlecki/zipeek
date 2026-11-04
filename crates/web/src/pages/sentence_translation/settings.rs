use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use zipeek_ai::STARTER_SETS;
use crate::services::word_sets;
use zipeek_exercises::translation::DirectionSelection;

use super::state::TranslationState;

pub(super) fn render(state: &TranslationState) -> Html {
    let on_set = {
        let state = state.clone();
        Callback::from(move |event: Event| {
            let value = event.target_unchecked_into::<HtmlSelectElement>().value();
            state.reset_configuration();
            state.set_id.set(value);
        })
    };
    let on_endpoint = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            state.reset_configuration();
            state.endpoint.set(value);
        })
    };
    let on_model = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            state.reset_configuration();
            state.model.set(value);
        })
    };
    let on_direction = {
        let state = state.clone();
        Callback::from(move |event: Event| {
            let next = match event.target_unchecked_into::<HtmlSelectElement>().value().as_str() {
                "zh-to-en" => DirectionSelection::ChineseToEnglish,
                "en-to-zh" => DirectionSelection::EnglishToChinese,
                _ => DirectionSelection::Mixed,
            };
            state.reset_configuration();
            state.direction_selection.set(next);
        })
    };
    html! {
        <div class="grid gap-4 sm:grid-cols-2">
            <label class="flex flex-col gap-2 text-sm font-semibold">{"Vocabulary source"}
                <select data-word-set="" data-translation-source="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*state.set_id).clone()} onchange={on_set}>
                    {if state.set_id.starts_with("filters:") { html! {<option value={(*state.set_id).clone()}>{"Custom Practice filters"}</option>} } else {Html::default()}}
                    <optgroup label="HSK levels">
                        {for (1..=6).map(|level| html! { <option value={format!("hsk:2:{level}")}>{format!("HSK 2.0 · {level}")}</option> })}
                        {for (1..=7).map(|level| html! { <option value={format!("hsk:3:{level}")}>{if level == 7 { "HSK 3.0 · 7–9".to_owned() } else { format!("HSK 3.0 · {level}") }}</option> })}
                    </optgroup>
                    <optgroup label="My Word Sets">
                        {if let Ok(sets) = word_sets::read() { html! { <>{for sets.iter().enumerate().map(|(index,set)| html! { <option value={format!("saved:{index}")}>{format!("{} ({} words)", set.name, set.word_ids.len())}</option> })}</> } } else { Html::default() }}
                    </optgroup>
                    <optgroup label="Sample topics">
                        {for STARTER_SETS.iter().map(|set| html! { <option value={format!("starter:{}", set.id)}>{set.name}</option> })}
                    </optgroup>
                </select>
            </label>
            {if state.set_id.starts_with("filters:") { html! {<p class="text-sm text-muted sm:col-span-2">{"Vocabulary filters were selected in Practice setup. Return to Practice to change HSK bands or categories."}</p>} } else {Html::default()}}
            <label class="flex flex-col gap-2 text-sm font-semibold">{"LM Studio model ID"}
                <input data-ai-model="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*state.model).clone()} oninput={on_model} placeholder="Loaded model identifier" />
            </label>
            <label class="flex flex-col gap-2 text-sm font-semibold">{"Translation direction"}
                <select data-translation-direction="" class="min-h-11 rounded-lg border border-line bg-surface px-3" onchange={on_direction}>
                    <option value="mixed" selected={*state.direction_selection == DirectionSelection::Mixed}>{"Mixed (balanced)"}</option>
                    <option value="zh-to-en" selected={*state.direction_selection == DirectionSelection::ChineseToEnglish}>{"Chinese → English"}</option>
                    <option value="en-to-zh" selected={*state.direction_selection == DirectionSelection::EnglishToChinese}>{"English → Chinese"}</option>
                </select>
            </label>
            <label class="flex flex-col gap-2 text-sm font-semibold sm:col-span-2">{"Local API base URL"}
                <input data-ai-endpoint="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*state.endpoint).clone()} oninput={on_endpoint} />
            </label>
        </div>
    }
}
