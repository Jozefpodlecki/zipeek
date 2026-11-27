use yew::prelude::*;
use zipeek_exercises::practice_draft::{HskBand, VocabularySource};

use super::practice_vocabulary_source::PracticeVocabularySource;
use crate::services::word_sets;

#[derive(Properties, PartialEq)]
pub struct PracticeVocabularyPanelProps {
    pub source: VocabularySource,
    pub hsk_standard: u8,
    pub personal_sets: Result<Vec<word_sets::UserWordSet>, String>,
    pub on_change: Callback<VocabularySource>,
    pub on_toggle_hsk_band: Callback<HskBand>,
    pub on_select_hsk_standard: Callback<u8>,
}

#[function_component(PracticeVocabularyPanel)]
pub fn practice_vocabulary_panel(props: &PracticeVocabularyPanelProps) -> Html {
    let word_set_error = match &props.personal_sets {
        Err(error) => html! {
            <p
                role="alert"
                class="rounded-xl border border-red-600/40 bg-red-950/10 p-3 text-sm text-red-700 dark:text-red-300"
            >
                {format!("Could not load your Word Sets: {error}")}
            </p>
        },
        Ok(_) => Html::default(),
    };

    html! {
        <div
            role="tabpanel"
            data-practice-step-panel="vocabulary"
            class="space-y-5"
        >
            {word_set_error}
            <PracticeVocabularySource
                source={props.source.clone()}
                hsk_standard={props.hsk_standard}
                personal_sets={props.personal_sets.clone().unwrap_or_default()}
                on_change={props.on_change.clone()}
                on_toggle_hsk_band={props.on_toggle_hsk_band.clone()}
                on_select_hsk_standard={props.on_select_hsk_standard.clone()}
            />
        </div>
    }
}
