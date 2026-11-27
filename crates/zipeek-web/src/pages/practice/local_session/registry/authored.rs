use super::super::super::authored_exercises::{AuthoredExercise, AuthoredExerciseMode};
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct AuthoredDefinition;

impl ExerciseDefinition for AuthoredDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["fill-blank", "audio-dictation"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let step = context.step;
        let on_finish = context.completed_callback();
        let on_assessed = context.assessment_callback();
        {
            let mode = if step.exercise_id == "fill-blank" {
                AuthoredExerciseMode::FillBlank
            } else {
                AuthoredExerciseMode::AudioDictation
            };
            html! {
                <AuthoredExercise
                    key={step.id.clone()}
                    mode={mode}
                    seed={step.instance_seed}
                    on_completed={on_finish}
                    on_assessed={Some(on_assessed)}
                />
            }
        }
    }
}
