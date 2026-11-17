use super::{
    difficulty_selector::DifficultySelector, matching_options::MatchingOptions,
    session_options::SessionOptions, spawn_selector::SpawnSelector, start_controls::StartControls,
    state::CloudSetupState, vocabulary_source::VocabularySource,
};
use crate::services::word_sets::UserWordSet;
use yew::prelude::*;
use zipeek_exercises::cloud_match::CloudPair;

#[derive(Properties, PartialEq)]
pub(crate) struct CloudSetupViewProps {
    pub setup: UseReducerHandle<CloudSetupState>,
    pub pool: Option<Result<Vec<CloudPair>, String>>,
    pub set_names: Vec<UserWordSet>,
    pub setup_error: String,
    pub on_start: Callback<MouseEvent>,
}

#[function_component(CloudSetupView)]
pub(crate) fn cloud_setup_view(props: &CloudSetupViewProps) -> Html {
    html! {
        <section data-cloud-match-setup="" class="mx-auto flex w-full max-w-2xl flex-col gap-5 rounded-2xl border border-line bg-surface p-5 sm:p-7">
            <div class="space-y-1">
                <h2 class="text-xl font-bold">{"Choose your vocabulary"}</h2>
                <p class="text-sm text-muted">{"Select a level or saved Word Set before entering the sky."}</p>
            </div>
            <VocabularySource setup={props.setup.clone()} sets={props.set_names.clone()}/>
            <MatchingOptions setup={props.setup.clone()}/>
            <DifficultySelector setup={props.setup.clone()}/>
            <SpawnSelector setup={props.setup.clone()}/>
            <SessionOptions setup={props.setup.clone()}/>
            <StartControls setup={props.setup.clone()} pool={props.pool.clone()} setup_error={props.setup_error.clone()} on_start={props.on_start.clone()}/>
        </section>
    }
}
