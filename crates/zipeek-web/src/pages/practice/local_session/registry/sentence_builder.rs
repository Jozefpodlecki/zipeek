use super::super::super::sentence_builder::SentenceBuilder;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct SentenceBuilderDefinition;

impl ExerciseDefinition for SentenceBuilderDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["sentence-builder"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let saved = context.saved;
        let step = context.step;
        let on_finish = context.completed_callback();
        let on_assessed = context.assessment_callback();
        let Some(options) = saved
            .exercise_configurations
            .get(&step.configuration_id)
            .and_then(|config| match config {
                crate::services::practice_session_store::ExerciseConfiguration::SentenceBuilder(
                    options,
                ) => Some(*options),
                _ => None,
            })
        else {
            return html! { <p role="alert">{"Sentence Builder configuration is missing or invalid."}</p> };
        };
        html! {
            <SentenceBuilder
                key={step.id.clone()}
                seed={step.instance_seed}
                prompt_index={zipeek_exercises::sentence_builder::prompt_index_for_session(
                    saved.scheduling_seed,
                    saved.steps.iter().take_while(|previous| previous.id != step.id)
                        .filter(|previous| previous.exercise_id == "sentence-builder").count(),
                )}
                options={options}
                on_completed={on_finish}
                on_assessed={Some(on_assessed)}
            />
        }
    }
}
