use super::super::super::spot_mistake::SpotMistake;
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct SpotMistakeDefinition;

impl ExerciseDefinition for SpotMistakeDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["spot-mistake"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let step = context.step;
        let on_finish = context.completed_callback();
        let on_assessed = context.assessment_callback();
        html! {
            <SpotMistake
                key={step.id.clone()}
                seed={step.instance_seed}
                on_completed={on_finish.clone()}
                on_assessed={Some(on_assessed)}
            />
        }
    }
}
