use super::state::{CloudSetupState, SetupAction};
use yew::prelude::*;
use zipeek_exercises::cloud_match::MatchMode;

#[derive(Properties, PartialEq)]
pub(super) struct MatchingOptionsProps {
    pub setup: UseReducerHandle<CloudSetupState>,
}

#[function_component(MatchingOptions)]
pub(super) fn matching_options(props: &MatchingOptionsProps) -> Html {
    let setup = props.setup.clone();
    html! {
        <div class="space-y-2">
            <p class="font-semibold">{"Matching mode"}</p>
            <div class="grid gap-2 sm:grid-cols-3">
                {for [
                    (MatchMode::HanziMeaning, "Hanzi ↔ Meaning"),
                    (MatchMode::HanziPinyin, "Hanzi ↔ Pinyin"),
                    (MatchMode::PinyinMeaning, "Pinyin ↔ Meaning"),
                ].into_iter().map(|(mode, label)| {
                    let selected = setup.mode == mode;
                    let on_select = {
                        let setup = setup.clone();
                        Callback::from(move |_| setup.dispatch(SetupAction::Mode(mode)))
                    };
                    let mode_classes = classes!("rounded-lg", "border", "px-3", "py-3", "text-sm", if selected {"border-brand-action"} else {"border-line"}, if selected {"bg-brand-action"} else {""}, if selected {"text-white"} else {""});
                    html! {<button type="button" aria-pressed={selected.to_string()} class={mode_classes} onclick={on_select}>{label}</button>}
                })}
            </div>
        </div>
    }
}
