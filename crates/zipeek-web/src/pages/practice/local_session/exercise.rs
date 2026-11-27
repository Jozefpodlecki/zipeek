use std::collections::HashMap;

use super::registry;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{
    route::Route,
    services::practice_session_store::{
        ExerciseOutcome, ExerciseReadiness, PracticeSession, PracticeStep,
    },
};

pub(super) fn render_exercise(
    saved: &PracticeSession,
    step: &PracticeStep,
    query: &HashMap<String, String>,
    on_outcome: Callback<ExerciseOutcome>,
    on_retry: Callback<MouseEvent>,
    on_hanzi_ready_change: Callback<bool>,
) -> Html {
    let query = query.clone();
    let exercise_content = match &step.readiness {
        ExerciseReadiness::Ready => registry::render_registered(&registry::ExerciseRenderContext {
            saved,
            step,
            query: &query,
            on_outcome: &on_outcome,
            on_hanzi_ready_change: &on_hanzi_ready_change,
        }),
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
