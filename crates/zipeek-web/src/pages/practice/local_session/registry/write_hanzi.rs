use super::super::super::write_hanzi_exercise::WriteHanziExercise;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct WriteHanziDefinition;

impl ExerciseDefinition for WriteHanziDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["write-hanzi"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let saved = context.saved;
        let step = context.step;
        let query = context.query.clone();
        let on_finish = context.completed_callback();
        let on_hanzi_ready_change = context.on_hanzi_ready_change.clone();
        let Some(options) = saved
            .exercise_configurations
            .get(&step.configuration_id)
            .and_then(|config| match config {
                crate::services::practice_session_store::ExerciseConfiguration::WriteHanzi(
                    options,
                ) => Some(*options),
                _ => None,
            })
        else {
            return html! { <p role="alert">{"Write Hanzi configuration is missing or invalid."}</p> };
        };
        html! {
            <WriteHanziExercise
                key={step.id.clone()}
                options={options}
                vocabulary={query}
                seed={step.instance_seed}
                on_completed={on_finish}
                on_ready_change={on_hanzi_ready_change}
            />
        }
    }
}
