use super::state::{CloudSetupState, SetupAction};
use yew::prelude::*;
use zipeek_exercises::cloud_match::{GameLength, LifeLimit};

#[derive(Properties, PartialEq)]
pub(super) struct SessionOptionsProps {
    pub setup: UseReducerHandle<CloudSetupState>,
}

#[function_component(SessionOptions)]
pub(super) fn session_options(props: &SessionOptionsProps) -> Html {
    let setup = props.setup.clone();
    html! {
        <>
            <div class="space-y-2" data-cloud-match-game-length="">
                <p class="font-semibold">{"Game length"}</p>
                <div class="grid grid-cols-4 gap-2">
                    {for [(GameLength::Fixed(10), "10"), (GameLength::Fixed(20), "20"), (GameLength::Fixed(30), "30"), (GameLength::Endless, "Endless")].into_iter().map(|(length, label)| {
                        let selected = setup.length == length;
                        let on_select = {
                            let setup = setup.clone();
                            Callback::from(move |_| setup.dispatch(SetupAction::Length(length)))
                        };
                        let length_classes = classes!("rounded-lg", "border", "px-2", "py-3", "text-sm", if selected {"border-brand-action"} else {"border-line"}, if selected {"bg-brand-action"} else {""}, if selected {"text-white"} else {""});
                        html! {<button type="button" aria-pressed={selected.to_string()} class={length_classes} onclick={on_select}>{label}</button>}
                    })}
                </div>
            </div>
            <div class="space-y-2" data-cloud-match-life-limit="">
                <p class="font-semibold">{"Lives"}</p>
                <div class="grid grid-cols-4 gap-2">
                    {for [(LifeLimit::Fixed(1), "1"), (LifeLimit::Fixed(2), "2"), (LifeLimit::Fixed(3), "3"), (LifeLimit::Endless, "Endless")].into_iter().map(|(lives, label)| {
                        let selected = setup.lives == lives;
                        let on_select = {
                            let setup = setup.clone();
                            Callback::from(move |_| setup.dispatch(SetupAction::Lives(lives)))
                        };
                        let lives_classes = classes!("rounded-lg", "border", "px-2", "py-3", "text-sm", if selected {"border-brand-action"} else {"border-line"}, if selected {"bg-brand-action"} else {""}, if selected {"text-white"} else {""});
                        html! {<button type="button" aria-pressed={selected.to_string()} class={lives_classes} onclick={on_select}>{label}</button>}
                    })}
                </div>
            </div>
        </>
    }
}
