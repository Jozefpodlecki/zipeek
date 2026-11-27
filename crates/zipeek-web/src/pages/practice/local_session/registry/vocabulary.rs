use super::super::super::vocabulary_exercises::VocabularyExercisePage;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct VocabularyDefinition;

impl ExerciseDefinition for VocabularyDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["type-pinyin", "character-builder", "word-mosaics"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let saved = context.saved;
        let step = context.step;
        let query = context.query.clone();
        let on_assessed = context.assessment_callback();
        let on_hanzi_ready_change = context.on_hanzi_ready_change.clone();
        let options = match step.exercise_id.as_str() {
            "word-mosaics" => {
                if !matches!(
                    saved.exercise_configurations.get(&step.configuration_id),
                    Some(crate::services::practice_session_store::ExerciseConfiguration::Standard)
                ) {
                    return html! { <p role="alert">{"Word Mosaics configuration is missing or invalid."}</p> };
                }
                crate::models::VocabularyExerciseOptions::WordMosaics
            }
            "type-pinyin" | "character-builder" => {
                let config = saved.exercise_configurations.get(&step.configuration_id);
                let valid = matches!(
                    (step.exercise_id.as_str(), config),
                    ("type-pinyin", Some(crate::services::practice_session_store::ExerciseConfiguration::TypePinyin(_)))
                        | ("character-builder", Some(crate::services::practice_session_store::ExerciseConfiguration::CharacterBuilder(_)))
                );
                if !valid {
                    return html! { <p role="alert">{"Vocabulary exercise configuration is missing or invalid."}</p> };
                }
                let Some(options) = config.and_then(|config| config.vocabulary_options()) else {
                    return html! { <p role="alert">{"Vocabulary exercise configuration is missing or invalid."}</p> };
                };
                options
            }
            _ => return html! { <p role="alert">{"Unsupported vocabulary exercise."}</p> },
        };
        html! {
            <VocabularyExercisePage
                key={step.id.clone()}
                session_query={Some(query)}
                on_assessed={Some(on_assessed)}
                on_review_change={on_hanzi_ready_change}
                instance_seed={step.instance_seed}
                exercise_options={options}
            />
        }
    }
}
