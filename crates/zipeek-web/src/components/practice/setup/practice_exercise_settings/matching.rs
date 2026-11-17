use yew::prelude::*;

use crate::reducers::practice_setup_flow::PracticeSetupAction;
use zipeek_exercises::practice_draft::PracticeDraft;

use super::super::practice_format_settings::PracticeFormatSettings;

#[derive(Properties, PartialEq)]
pub(super) struct MatchingSettingsProps {
    pub draft: PracticeDraft,
    pub family: &'static str,
    pub pair_count: u8,
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
    html! {
        <PracticeFormatSettings
            draft={props.draft.clone()}
            family={props.family.to_owned()}
            on_toggle={on_toggle}
            on_column_order={on_order}
            pair_count={props.pair_count}
            on_pair_count={on_pairs}
        />
    }
}
