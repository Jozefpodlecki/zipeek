use super::super::super::sentence_transformation::SentenceTransformation;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct SentenceTransformationDefinition;

impl ExerciseDefinition for SentenceTransformationDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["sentence-transformation"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let saved = context.saved;
        let step = context.step;
        let on_finish = context.completed_callback();
        let on_assessed = context.assessment_callback();
        let Some(options) = saved.exercise_configurations.get(&step.configuration_id).and_then(|config| match config {
            crate::services::practice_session_store::ExerciseConfiguration::SentenceTransformation(options) => Some(*options),
            _ => None,
        }) else {
            return html! { <p role="alert">{"Sentence Transformation configuration is missing or invalid."}</p> };
        };
        html! {
            <SentenceTransformation
                key={step.id.clone()}
                seed={step.instance_seed}
                options={options}
                on_completed={on_finish.clone()}
                on_assessed={Some(on_assessed)}
            />
        }
    }
}
