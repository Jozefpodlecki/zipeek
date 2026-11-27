mod launch;
mod mode;
mod profile;
mod review;

use yew::prelude::*;
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::practice_draft::{PracticeDraft, SessionLength, VocabularySource};

use crate::{models::PracticeExerciseOptions, services::practice_profiles::PracticeProfile};

#[derive(Properties, PartialEq)]
pub(crate) struct PracticeSetupSummaryProps {
    pub draft: PracticeDraft,
    pub variants: Vec<ExerciseVariant>,
    pub plan_rounds: Option<usize>,
    pub can_start: bool,
    pub start_failed: bool,
    pub gate_message: String,
    pub on_length: Callback<SessionLength>,
    pub on_count: Callback<u16>,
    pub on_start: Callback<MouseEvent>,
    pub on_back: Callback<()>,
    pub profile_name: String,
    pub profile_error: Option<String>,
    pub profile_saved_notice: Option<String>,
    pub loaded_profile: Option<PracticeProfile>,
    pub profile_advanced: bool,
    pub profile_options: PracticeExerciseOptions,
    pub on_profile_name: Callback<InputEvent>,
    pub on_save_profile: Callback<MouseEvent>,
}

#[function_component(PracticeSetupSummary)]
pub(crate) fn practice_setup_summary(props: &PracticeSetupSummaryProps) -> Html {
    html! {
        <section
            role="tabpanel"
            data-practice-step-panel="summary"
            class="space-y-5"
        >
            <div
                class="rounded-2xl border border-line bg-surface p-5 space-y-5"
            >
                {launch::render(props)}
                {review::render(props)}
                {mode::render(props)}
                {profile::render(props)}
            </div>
        </section>
    }
}

fn exercise_label(id: &str) -> &str {
    match id {
        "matching" => "Matching",
        "multiple-choice" => "Multiple choice",
        "sentence-translation" => "Sentence translation",
        "story-builder" => "Story Builder",
        "reading-comprehension" => "Reading Comprehension",
        "conversation-practice" => "Conversation Practice",
        "speaking" => "Pronunciation",
        "listening-choice" => "Listening",
        "image-vocabulary" => "Picture vocabulary",
        "character-hunt" => "Character Hunt",
        "type-pinyin" => "Type pinyin",
        "character-builder" => "Character builder",
        "word-mosaics" => "Word mosaics",
        "fill-blank" => "Fill in the Blank",
        "audio-dictation" => "Audio Dictation",
        "sentence-builder" => "Sentence Builder",
        "sentence-transformation" => "Sentence Transformation",
        "spot-mistake" => "Spot the Mistake",
        "write-hanzi" => "Write Hanzi",
        _ => "Unknown exercise",
    }
}

fn format_label(variant: ExerciseVariant) -> &'static str {
    variant.label()
}

fn vocabulary_label(source: &VocabularySource) -> String {
    match source {
        VocabularySource::PersonalSet { reference } => format!("Word Set {reference}"),
        VocabularySource::Dictionary(filters) => {
            let mut parts = Vec::new();
            if !filters.hsk.is_empty() {
                let bands = filters.hsk.len();
                parts.push(format!(
                    "{bands} HSK {}",
                    if bands == 1 { "band" } else { "bands" }
                ));
            }
            if !filters.categories.is_empty() {
                let count = filters.categories.len();
                parts.push(format!(
                    "{count} {}",
                    if count == 1 { "category" } else { "categories" }
                ));
            }
            if !filters.parts_of_speech.is_empty() {
                let count = filters.parts_of_speech.len();
                parts.push(format!(
                    "{count} {}",
                    if count == 1 {
                        "part of speech"
                    } else {
                        "parts of speech"
                    }
                ));
            }
            if parts.is_empty() {
                "Dictionary".to_owned()
            } else {
                format!("Dictionary · {}", parts.join(" · "))
            }
        }
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::exercise_label;
    use zipeek_exercises::definition::DEFINITIONS;

    #[test]
    fn every_registered_exercise_has_a_summary_label() {
        for exercise in DEFINITIONS {
            assert_ne!(
                exercise_label(exercise.id),
                "Unknown exercise",
                "{}",
                exercise.id
            );
        }
    }
}
