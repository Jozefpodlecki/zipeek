use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use super::progress::completion_percentage;
use crate::{
    route::Route,
    services::{
        practice_history::CompletionDisposition,
        practice_session_store::{ExerciseOutcome, PracticeSession},
    },
};

pub(super) fn render(
    saved: &PracticeSession,
    history_result: &Option<Result<CompletionDisposition, String>>,
    retry_history: &Callback<MouseEvent>,
) -> Html {
    let completed = saved
        .steps
        .iter()
        .filter(|step| {
            matches!(
                step.outcome.as_ref(),
                Some(ExerciseOutcome::Completed | ExerciseOutcome::Scored { .. })
            )
        })
        .count();
    let total = saved.steps.len();
    let skipped = total - completed;
    let progress_percent = completion_percentage(completed, total);
    html! {
        <section
            class="mx-auto flex w-full max-w-lg flex-col items-center gap-5 px-4 py-6 text-center sm:gap-6 sm:py-10"
            data-practice-local-summary=""
            aria-labelledby="practice-complete-heading"
        >
            <header class="flex flex-col items-center gap-3">
                <div
                    class="flex size-16 items-center justify-center rounded-full border border-brand-action/30 bg-brand-action/15 text-amber-300 shadow-sm"
                    aria-hidden="true"
                >
                    <Icon data={IconData::LUCIDE_TROPHY} width="32" height="32" />
                </div>
                <p class="text-xs font-semibold uppercase tracking-[0.16em] text-muted">
                    {"Practice complete"}
                </p>
                <h1
                    id="practice-complete-heading"
                    class="text-3xl font-bold tracking-tight text-ink sm:text-4xl"
                >
                    {if skipped == 0 { "Excellent work!" } else { "Session finished!" }}
                </h1>
                <p class="max-w-sm text-sm leading-6 text-muted">
                    {if skipped == 0 {
                        format!("You finished all {total} exercises. Keep the momentum going.")
                    } else {
                        format!("You completed {completed} of {total} exercises. Every step counts.")
                    }}
                </p>
            </header>
            <div class="w-full space-y-4 rounded-2xl border border-line bg-surface/95 p-4 text-left shadow-sm sm:p-5">
                <div class="flex items-center justify-between gap-4">
                    <h2 class="text-sm font-semibold text-ink">{"Session progress"}</h2>
                    <p class="text-sm font-semibold text-ink" data-practice-completion-count="">
                        {format!("{completed} of {total} exercises")}
                    </p>
                </div>
                <div
                    class="h-2 overflow-hidden rounded-full bg-line"
                    role="progressbar"
                    aria-label="Exercises completed"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow={progress_percent.to_string()}
                >
                    <div
                        class="h-full rounded-full bg-brand-action motion-safe:transition-[width] motion-safe:duration-500"
                        style={format!("width: {progress_percent}%")}
                    />
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <div class="rounded-xl border border-line/70 bg-accent-soft/40 p-3 sm:p-4">
                        <div class="flex items-center gap-2">
                            <Icon data={IconData::LUCIDE_CHECK} width="19" height="19" />
                            <p class="text-2xl font-bold tabular-nums text-ink" data-practice-completed="">{completed}</p>
                        </div>
                        <p class="mt-1 text-xs text-muted">{"Completed"}</p>
                    </div>
                    <div class="rounded-xl border border-line/70 bg-accent-soft/40 p-3 sm:p-4">
                        <div class="flex items-center gap-2">
                            <Icon data={IconData::LUCIDE_MINUS} width="19" height="19" />
                            <p class="text-2xl font-bold tabular-nums text-ink" data-practice-skipped="">{skipped}</p>
                        </div>
                        <p class="mt-1 text-xs text-muted">{"Skipped"}</p>
                    </div>
                </div>
            </div>
            <aside
                class="flex min-h-12 w-full items-start gap-2 px-1 text-left text-xs leading-5 text-muted"
                role="status"
                aria-live="polite"
                aria-label="Learning history notice"
                data-practice-history-notice=""
            >
                <Icon data={IconData::LUCIDE_INFO} width="16" height="16" />
                <div class="min-w-0 flex-1 space-y-2">
                    {
                        match history_result {
                            None => html! {
                                <p>{"Finishing learning-history handling. Your temporary session is retained until the operation finishes."}</p>
                            },
                            Some(Ok(CompletionDisposition::Recorded)) => html! {
                                <p>{"Learning outcomes were saved to your personal database."}</p>
                            },
                            Some(Ok(CompletionDisposition::NotRecorded)) => html! {
                                <p>{"Learning history is off. This session was not saved."}</p>
                            },
                            Some(Err(error)) => html! {
                                <>
                                    <p role="alert">{format!("Your session is still available for recovery: {error}")}</p>
                                    <button
                                        type="button"
                                        class="rounded-lg border border-line px-3 py-2 font-semibold text-ink focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
                                        onclick={(*retry_history).clone()}
                                        data-practice-history-retry=""
                                    >
                                        {"Retry saving results"}
                                    </button>
                                </>
                            },
                        }
                    }
                </div>
            </aside>
            <nav class="flex w-full flex-col gap-3 sm:flex-row" aria-label="After practice">
                <Link<Route>
                    to={Route::PracticeSetup}
                    classes="inline-flex min-h-12 flex-1 items-center justify-center rounded-xl bg-brand-action px-5 py-3 font-semibold text-white shadow-sm transition-colors hover:bg-brand-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent motion-reduce:transition-none"
                >
                    {"Practice again"}
                </Link<Route>>
                <Link<Route>
                    to={Route::Browse}
                    classes="inline-flex min-h-12 flex-1 items-center justify-center rounded-xl border border-line bg-surface px-5 py-3 font-semibold text-ink transition-colors hover:bg-accent-soft focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent motion-reduce:transition-none"
                >
                    {"Explore vocabulary"}
                </Link<Route>>
            </nav>
        </section>
    }
}
