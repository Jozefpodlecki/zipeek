use super::super::super::listening_session::ResumableListeningChoice;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct ListeningChoiceDefinition;

impl ExerciseDefinition for ListeningChoiceDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["listening-choice"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let saved = context.saved;
        let step = context.step;
        let query = context.query.clone();
        let on_finish = context.completed_callback();
        let on_assessed = context.assessment_callback();
        let Some(options) = saved
            .exercise_configurations
            .get(&step.configuration_id)
            .and_then(|config| match config {
                crate::services::practice_session_store::ExerciseConfiguration::ListeningChoice(
                    options,
                ) => Some(*options),
                _ => None,
            })
        else {
            return html! { <p role="alert">{"Listening Choice configuration is missing or invalid."}</p> };
        };
        html! {
            <ResumableListeningChoice
                key={step.id.clone()}
                options={options}
                vocabulary={query.clone()}
                seed={step.instance_seed}
                on_completed={on_finish.clone()}
                on_assessed={Some(on_assessed)}
            />
        }
    }
}
