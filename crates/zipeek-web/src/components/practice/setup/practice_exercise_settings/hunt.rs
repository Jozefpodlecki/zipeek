use yew::prelude::*;

use crate::reducers::practice_setup_flow::PracticeSetupAction;
use zipeek_exercises::character_hunt::Difficulty as HuntDifficulty;

use super::super::character_hunt_settings::CharacterHuntSettings;

#[derive(Properties, PartialEq)]
pub(super) struct HuntSettingsProps {
    pub selected: HuntDifficulty,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(HuntSettings)]
pub(super) fn hunt_settings(props: &HuntSettingsProps) -> Html {
    let on_change = {
        let on_action = props.on_action.clone();
        Callback::from(move |difficulty| {
            on_action.emit(PracticeSetupAction::SetHuntDifficulty(difficulty));
        })
    };
    html! { <CharacterHuntSettings selected={props.selected} on_change={on_change} /> }
}
