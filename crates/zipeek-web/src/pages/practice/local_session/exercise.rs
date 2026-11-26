use std::collections::HashMap;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use super::super::{
    authored_exercises::{AuthoredExercise, AuthoredExerciseMode},
    generated_session::GeneratedPractice,
    listening_session::ResumableListeningChoice,
    local_quiz_session::{LocalQuizMode, LocalQuizSession},
    sentence_builder::SentenceBuilder,
    sentence_transformation::SentenceTransformation,
    spot_mistake::SpotMistake,
    vocabulary_exercises::VocabularyExercisePage,
    write_hanzi_exercise::WriteHanziExercise,
};
use crate::pages::learning::games::ResumableCharacterHunt;
use crate::{
    route::Route,
    services::practice_session_store::{ExerciseReadiness, PracticeSession, PracticeStep},
};

pub(super) fn render_exercise(
    saved: &PracticeSession,
    step: &PracticeStep,
    query: &HashMap<String, String>,
    on_finish: Callback<()>,
    on_scored: Callback<bool>,
    on_retry: Callback<MouseEvent>,
    on_hanzi_ready_change: Callback<bool>,
) -> Html {
    let query = query.clone();
    let exercise_content = match &step.readiness {
        ExerciseReadiness::Ready => match step.exercise_id.as_str() {
            "type-pinyin" | "character-builder" | "word-mosaics" => {
                html! {
                    <VocabularyExercisePage
                        key={step.id.clone()}
                        session_query={Some(query)}
                        on_completed={Some(on_finish)}
                        instance_seed={step.instance_seed}
                        exercise_options={saved.exercise_configurations.get(&step.configuration_id).and_then(|configuration| configuration.vocabulary_options()).unwrap_or(crate::models::VocabularyExerciseOptions::WordMosaics)}
                    />
                }
            }
            "sentence-translation" | "image-vocabulary" => {
                if let Some(raw) = &step.generated_instance {
                    html! { <GeneratedPractice
                        key={step.id.clone()}
                        raw={raw.clone()}
                        exercise_id={step.exercise_id.clone()}
                        seed={step.instance_seed}
                        on_completed={on_finish.clone()}
                        on_scored={on_scored.clone()}
                    /> }
                } else {
                    html! { <p role="alert">{"Generated exercise content is missing."}</p> }
                }
            }
            "matching" | "multiple-choice" => {
                let mode = if step.exercise_id == "matching" {
                    LocalQuizMode::Matching
                } else {
                    LocalQuizMode::MultipleChoice
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
                        pair_count={saved.exercise_configurations.get(&step.configuration_id).and_then(|config| match config {
                            crate::services::practice_session_store::ExerciseConfiguration::Quiz { pair_count, .. } => Some(*pair_count),
                            _ => None,
                        }).expect("validated Matching session must have pair count")}
                        on_completed={on_finish.clone()}
                    />
                }
            }
            "character-hunt" => html! {
                <ResumableCharacterHunt
                    key={step.id.clone()}
                    vocabulary={query.clone()}
                    seed={step.instance_seed}
                    on_completed={on_finish.clone()}
                />
            },
            "listening-choice" => html! {
                <ResumableListeningChoice
                    key={step.id.clone()}
                    options={saved.exercise_configurations.get(&step.configuration_id).and_then(|config| match config {
                        crate::services::practice_session_store::ExerciseConfiguration::ListeningChoice(options) => Some(*options),
                        _ => None,
                    }).expect("validated Listening Choice configuration must exist")}
                    vocabulary={query.clone()}
                    seed={step.instance_seed}
                    on_completed={on_finish.clone()}
                />
            },
            "speaking" => html! {
                <crate::pages::translation::speaking::SpeakingPractice
                    key={step.id.clone()}
                    seed={step.instance_seed}
                    on_completed={Some(on_finish.clone())}
                />
            },
            "write-hanzi" => html! {
                <WriteHanziExercise
                    key={step.id.clone()}
                    options={saved.exercise_configurations.get(&step.configuration_id).and_then(|config| match config {
                        crate::services::practice_session_store::ExerciseConfiguration::WriteHanzi(options) => Some(*options),
                        _ => None,
                    }).unwrap_or_default()}
                    vocabulary={query.clone()}
                    seed={step.instance_seed}
                    on_completed={on_finish.clone()}
                    on_ready_change={on_hanzi_ready_change}
                />
            },
            "spot-mistake" => html! {
                <SpotMistake
                    key={step.id.clone()}
                    seed={step.instance_seed}
                    on_completed={on_finish.clone()}
                />
            },
            "sentence-transformation" => html! {
                <SentenceTransformation
                    key={step.id.clone()}
                    seed={step.instance_seed}
                    options={saved.exercise_configurations.get(&step.configuration_id).and_then(|config| match config {
                        crate::services::practice_session_store::ExerciseConfiguration::SentenceTransformation(options) => Some(*options),
                        _ => None,
                    }).expect("validated Sentence Transformation configuration must exist")}
                    on_completed={on_finish.clone()}
                />
            },
            "sentence-builder" => html! {
                <SentenceBuilder
                    key={step.id.clone()}
                    seed={step.instance_seed}
                    on_completed={on_finish.clone()}
                />
            },
            "fill-blank" | "audio-dictation" => {
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
                    />
                }
            }
            _ => html! {
                <p
                    role="status"
                    class="text-sm text-muted"
                >
                    {"This exercise cannot run in the resumable session yet. You may skip it without earning a score, or return to setup to choose a supported exercise."}
                </p>
            },
        },
        ExerciseReadiness::Unavailable(reason) => html! {
            <p
                role="status"
            >
                {format!("{reason}. You may skip this step without recording a score.")}
            </p>
        },
        ExerciseReadiness::Failed(message) => html! {
            <section class="mx-auto w-full max-w-xl space-y-5 rounded-2xl border border-line bg-surface px-6 py-7 shadow-lg sm:px-8" data-practice-generation-error="" role="alert">
                <div class="flex items-start gap-3">
                    <span class="mt-0.5 flex size-10 shrink-0 items-center justify-center rounded-xl bg-raised text-accent" aria-hidden="true">
                        <Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="22px" height="22px" />
                    </span>
                    <div class="space-y-2">
                        <h2 class="text-xl font-semibold text-ink">{"We couldn't prepare this exercise"}</h2>
                        <p class="text-sm leading-6 text-muted">{"The AI scene didn't pass the vocabulary validation check. No answer was submitted and no score was recorded."}</p>
                    </div>
                </div>
                <div class="rounded-xl border border-line bg-raised p-4 text-sm leading-6">
                    <p class="mb-1 font-semibold text-ink">{"What went wrong"}</p>
                    <p class="break-words text-muted">{message}</p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <button type="button" class="rounded-lg bg-brand-action px-5 py-3 text-sm font-semibold text-white hover:bg-brand-hover"
                        onclick={on_retry.clone()} disabled={step.generation_attempt >= 3}>
                        {if step.generation_attempt >= 3 {"Retry limit reached"} else {"Generate another scene"}}
                    </button>
                    <Link<Route> to={Route::PracticeSetup} classes="rounded-lg border border-line bg-surface px-4 py-3 text-sm font-semibold hover:bg-raised">{"Change exercise selection"}</Link<Route>>
                    <span class="text-xs text-muted">{format!("Attempt {} of 3", step.generation_attempt)}</span>
                </div>
                <details class="rounded-xl border border-line bg-surface px-4 py-3 text-xs text-muted" data-practice-error-diagnostics="">
                    <summary class="cursor-pointer font-semibold text-ink">{"Technical diagnostics"}</summary>
                    <dl class="mt-3 grid gap-2 break-all">
                        <div><dt class="font-semibold">{"Exercise type"}</dt><dd>{&step.exercise_id}</dd></div>
                        <div><dt class="font-semibold">{"Session step"}</dt><dd>{&step.id}</dd></div>
                        <div><dt class="font-semibold">{"Generation attempts"}</dt><dd>{step.generation_attempt.to_string()}</dd></div>
                        <div><dt class="font-semibold">{"Validation error"}</dt><dd>{message}</dd></div>
                    </dl>
                    <p class="mt-3">{"If this repeats, check AI Translation settings and the model's completion logs. Do not share API keys."}</p>
                </details>
            </section>
        },
        ExerciseReadiness::NeedsGeneration
        | ExerciseReadiness::Queued
        | ExerciseReadiness::Generating => html! {
            <div
                role="status"
                aria-live="polite"
                data-practice-generation-status=""
                class="flex flex-col items-center gap-4 py-6 text-center"
            >
                <span class="loading-spinner" aria-hidden="true" />
                <div class="space-y-2">
                    <h2 class="text-xl font-semibold text-ink">{"Preparing your exercise"}</h2>
                    <p class="max-w-md text-sm leading-6 text-muted">
                        {if step.exercise_id == "image-vocabulary" {
                            "Generating an image for your selected vocabulary. This may take a while."
                        } else {
                            "Creating an exercise using your selected vocabulary. This may take a while."
                        }}
                    </p>
                    <p class="text-xs text-muted">{"Progress within AI generation is not available."}</p>
                </div>
            </div>
        },
    };
    exercise_content
}
