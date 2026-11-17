use super::state::{CloudSetupState, SetupAction};
use yew::prelude::*;
use zipeek_exercises::cloud_match::Difficulty;

#[derive(Properties, PartialEq)]
pub(super) struct DifficultySelectorProps {
    pub setup: UseReducerHandle<CloudSetupState>,
}

#[function_component(DifficultySelector)]
pub(super) fn difficulty_selector(props: &DifficultySelectorProps) -> Html {
    let setup = props.setup.clone();
    html! {
        <div class="space-y-2">
            <p class="font-semibold">{"Difficulty"}</p>
            <div class="grid grid-cols-3 gap-2">
                {for [(Difficulty::Calm, "Calm"), (Difficulty::Normal, "Normal"), (Difficulty::Storm, "Storm")].into_iter().map(|(difficulty, label)| {
                    let selected = setup.difficulty == difficulty;
                    let on_select = {
                        let setup = setup.clone();
                        Callback::from(move |_| setup.dispatch(SetupAction::Difficulty(difficulty)))
                    };
                    let difficulty_classes = classes!("rounded-lg", "border", "px-3", "py-3", if selected {"border-brand-action"} else {"border-line"}, if selected {"bg-brand-action"} else {""}, if selected {"text-white"} else {""});
                    html! {<button type="button" aria-pressed={selected.to_string()} class={difficulty_classes} onclick={on_select}>{label}</button>}
                })}
            </div>
        </div>
    }
}
