use super::state::{CloudSetupState, SetupAction};
use yew::prelude::*;
use zipeek_exercises::cloud_match::SpawnMode;

#[derive(Properties, PartialEq)]
pub(super) struct SpawnSelectorProps {
    pub setup: UseReducerHandle<CloudSetupState>,
}

#[function_component(SpawnSelector)]
pub(super) fn spawn_selector(props: &SpawnSelectorProps) -> Html {
    let setup = props.setup.clone();
    let on_hint = {
        let setup = setup.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            setup.dispatch(SetupAction::HighlightPartner(input.checked()));
        })
    };
    html! {
        <>
            <div class="space-y-2" data-cloud-match-spawn-mode="">
                <p class="font-semibold">{"Cloud arrival"}</p>
                <div class="grid grid-cols-2 gap-2">
                    {for [(SpawnMode::Waves, "Waves"), (SpawnMode::Continuous, "Continuous")].into_iter().map(|(spawn, label)| {
                        let selected = setup.spawn == spawn;
                        let on_select = {
                            let setup = setup.clone();
                            Callback::from(move |_| setup.dispatch(SetupAction::Spawn(spawn)))
                        };
                        let spawn_classes = classes!("rounded-lg", "border", "px-3", "py-3", if selected {"border-brand-action"} else {"border-line"}, if selected {"bg-brand-action"} else {""}, if selected {"text-white"} else {""});
                        html! {<button type="button" aria-pressed={selected.to_string()} class={spawn_classes} onclick={on_select}>{label}</button>}
                    })}
                </div>
                <p class="text-xs text-muted">
                    {if setup.spawn == SpawnMode::Continuous {
                        "A new pair enters when you match one. Existing clouds continue drifting."
                    } else {
                        "Match three pairs before the next wave enters."
                    }}
                </p>
            </div>
            <div class="space-y-2" data-cloud-match-hint-option="">
                <label class="flex cursor-pointer items-start gap-3 rounded-lg border border-line bg-raised px-4 py-3">
                    <input type="checkbox" class="mt-1 size-4 accent-blue-500" checked={setup.highlight_partner} onchange={on_hint}/>
                    <span class="space-y-1">
                        <span class="block font-semibold">{"Highlight matching cloud"}</span>
                        <span class="block text-xs text-muted">{"When you select a cloud, its matching partner glows until your next choice. A learner aid; no automatic match."}</span>
                    </span>
                </label>
            </div>
        </>
    }
}
