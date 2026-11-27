use super::super::super::local_quiz_session::{LocalQuizMode, LocalQuizSession};
use super::{ExerciseDefinition, ExerciseRenderContext};
use yew::prelude::*;

pub(super) struct QuizDefinition;

impl ExerciseDefinition for QuizDefinition {
    fn ids(&self) -> &'static [&'static str] {
        &["matching", "multiple-choice"]
    }

    fn render(&self, context: &ExerciseRenderContext<'_>) -> Html {
        let saved = context.saved;
        let step = context.step;
        let query = context.query.clone();
        let on_finish = context.completed_callback();
        let on_assessed = context.assessment_callback();
        {
            let mode = if step.exercise_id == "matching" {
                LocalQuizMode::Matching
            } else {
                LocalQuizMode::MultipleChoice
            };
            let Some(pair_count) = saved
                .exercise_configurations
                .get(&step.configuration_id)
                .and_then(|config| match config {
                    crate::services::practice_session_store::ExerciseConfiguration::Quiz {
                        pair_count,
                        ..
                    } => Some(*pair_count),
                    _ => None,
                })
            else {
                return html! { <p role="alert">{"Quiz configuration is missing or invalid."}</p> };
            };
            html! {
                <LocalQuizSession
                    key={step.id.clone()}
                    mode={mode}
                    vocabulary={query.clone()}
                    seed={step.instance_seed}
                    quiz_format={step.quiz_format.clone().unwrap_or_default()}
                    reverse_matching={step.reverse_matching}
                    show_pinyin={saved.exercise_configurations.get(&step.configuration_id).and_then(|config| match config {
                        crate::services::practice_session_store::ExerciseConfiguration::Quiz { show_pinyin, .. } => Some(*show_pinyin),
                        _ => None,
                    }).unwrap_or(true)}
                    generated_words={step.generated_instance.as_ref().and_then(|raw|
                        crate::services::practice_generated::GeneratedInstance::decode(raw, "matching").ok()
                    ).and_then(|instance| match instance {
                        crate::services::practice_generated::GeneratedInstance::Matching { words } => Some(words),
                        _ => None,
                    })}
                    pair_count={pair_count}
                    on_completed={on_finish.clone()}
                    on_assessed={Some(on_assessed)}
                />
            }
        }
    }
}
