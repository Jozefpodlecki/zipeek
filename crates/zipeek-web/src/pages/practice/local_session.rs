use std::collections::HashMap;
use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{
    appearance::Appearance,
    hooks::{use_app_environment, use_dictionary},
    pages::learning::games::ResumableCharacterHunt,
    route::Route,
    services::practice_session_store::{
        ExerciseOutcome, ExerciseReadiness, PracticeSession, PracticeSessionError,
        PracticeSessionStore,
    },
    services::{
        practice_generated,
        practice_history::{self, CompletionDisposition},
    },
};

use super::{
    authored_exercises::{AuthoredExercise, AuthoredExerciseMode},
    creature_progress::{CreatureOrientation, CreatureProgress, creature_orientation},
    generated_session::GeneratedPractice,
    listening_session::ResumableListeningChoice,
    local_quiz_session::{LocalQuizMode, LocalQuizSession},
    sentence_builder::SentenceBuilder,
    sentence_transformation::SentenceTransformation,
    spot_mistake::SpotMistake,
    vocabulary_exercises::VocabularyExercisePage,
    write_hanzi_exercise::WriteHanziExercise,
};

#[derive(Properties, PartialEq)]
pub struct LocalPracticeSessionProps {
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HanziCompletionPreview {
    step_id: String,
    ready: bool,
}

fn preview_progress(
    completed: usize,
    step_id: Option<&str>,
    preview: Option<&HanziCompletionPreview>,
) -> usize {
    completed
        + usize::from(step_id.is_some_and(|id| {
            preview.is_some_and(|pending| pending.step_id == id && pending.ready)
        }))
}

fn completion_percentage(completed: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        completed.saturating_mul(100) / total
    }
}

#[function_component(LocalPracticeSession)]
pub fn local_practice_session(props: &LocalPracticeSessionProps) -> Html {
    let environment = use_app_environment();
    let appearance =
        use_context::<Appearance>().expect("Appearance provider missing from Practice session");
    let horizontal_progress =
        creature_orientation(appearance.preset) == CreatureOrientation::Horizontal;
    let dictionary = use_dictionary();
    let storage = environment.local_storage.clone();
    let session = use_state(|| None::<Result<PracticeSession, PracticeSessionError>>);
    let hanzi_ready_preview = use_state(|| None::<HanziCompletionPreview>);
    let history_result = use_state(|| None::<Result<CompletionDisposition, String>>);
    let history_retry = use_state(|| 0_u32);
    let restore_retry = use_state(|| 0_u32);

    {
        let session = session.clone();
        let storage = storage.clone();
        use_effect_with((props.id.clone(), *restore_retry), move |(id, _)| {
            let store = PracticeSessionStore::new(storage.clone());
            let result = store.load(id).and_then(|mut saved| {
                if !saved.is_completed() && saved.recover_interrupted_generation() {
                    store.save(&saved)?;
                }
                if !saved.is_completed() && saved.start_current()? {
                    saved.revision += 1;
                    store.save(&saved)?;
                }
                Ok(saved)
            });
            session.set(Some(result));
        });
    }

    {
        let storage = storage.clone();
        let document = environment.document.clone();
        let history_result = history_result.clone();
        use_effect_with(
            ((*session).clone(), *history_retry),
            move |(snapshot, _)| {
                let active = Rc::new(Cell::new(true));
                if let Some(Ok(saved)) = snapshot {
                    if saved.is_completed() {
                        let saved = saved.clone();
                        let storage = storage.clone();
                        let document = document.clone();
                        let history_result = history_result.clone();
                        history_result.set(None);
                        let active_for_result = active.clone();
                        spawn_local(async move {
                            let result = practice_history::finalize_completed(storage, document, saved).await;
                            if active_for_result.get() {
                                history_result.set(Some(result));
                            }
                        });
                    }
                }
                move || active.set(false)
            },
        );
    }
    let retry_restore = {
        let restore_retry = restore_retry.clone();
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            session.set(None);
            restore_retry.set(restore_retry.wrapping_add(1));
        })
    };
    let retry_history = {
        let history_retry = history_retry.clone();
        Callback::from(move |_: MouseEvent| {
            history_retry.set(history_retry.wrapping_add(1));
        })
    };
    {
        let state = session.clone();
        let storage = storage.clone();
        let dictionary = dictionary.clone();
        use_effect_with((*session).clone(), move |snapshot| {
            if let Some(Ok(saved)) = snapshot {
                let already_generating = saved
                    .steps
                    .iter()
                    .any(|step| matches!(&step.readiness, ExerciseReadiness::Generating));
                if !already_generating {
                    if let Some(next) = saved.pending_generation().next() {
                        let mut started = saved.clone();
                        let step_id = next.id.clone();
                        let exercise_id = next.exercise_id.clone();
                        let seed = next.instance_seed;
                        if let Ok(attempt) = started.start_generation(&step_id) {
                            let store = PracticeSessionStore::new(storage.clone());
                            if let Err(error) = store.save(&started) {
                                state.set(Some(Err(error)));
                            } else {
                                state.set(Some(Ok(started.clone())));
                                let state = state.clone();
                                let storage = storage.clone();
                                let dictionary = dictionary.clone();
                                let vocabulary = started
                                    .vocabulary
                                    .iter()
                                    .map(|(key, value)| (key.clone(), value.clone()))
                                    .collect::<HashMap<_, _>>();
                                spawn_local(async move {
                                    let translation_ordinal = started
                                        .steps
                                        .iter()
                                        .take_while(|step| step.id != step_id)
                                        .filter(|step| step.exercise_id == "sentence-translation")
                                        .count();
                                    let result = practice_generated::prepare(
                                        &exercise_id,
                                        &vocabulary,
                                        seed,
                                        started.translation_direction(),
                                        translation_ordinal,
                                        &dictionary,
                                        &storage,
                                    )
                                    .await;
                                    let store = PracticeSessionStore::new(storage);
                                    let mut current = match store.load(&started.id) {
                                        Ok(current) => current,
                                        Err(PracticeSessionError::NotFound) => return,
                                        Err(error) => {
                                            state.set(Some(Err(error)));
                                            return;
                                        }
                                    };
                                    let update = match result {
                                        Ok(instance) => current
                                            .generation_succeeded(&step_id, attempt, instance),
                                        Err(message) => {
                                            current.generation_failed(&step_id, attempt, message)
                                        }
                                    };
                                    if update.is_ok() && !current.is_completed() {
                                        if let Err(error) = current.start_current() {
                                            state.set(Some(Err(error)));
                                            return;
                                        }
                                    }
                                    match update {
                                        Ok(()) => match store.save(&current) {
                                            Ok(()) => state.set(Some(Ok(current))),
                                            Err(PracticeSessionError::OutOfDate) => {
                                                state.set(Some(store.load(&started.id)));
                                            }
                                            Err(error) => state.set(Some(Err(error))),
                                        },
                                        Err(
                                            PracticeSessionError::StaleGeneration
                                            | PracticeSessionError::NotReady
                                            | PracticeSessionError::AlreadyFinished,
                                        ) => {
                                            state.set(Some(Ok(current)));
                                        }
                                        Err(error) => state.set(Some(Err(error))),
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });
    }

    let on_retry = {
        let session = session.clone();
        let storage = storage.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(Ok(current)) = &*session else {
                return;
            };
            let mut updated = current.clone();
            let Some(step) = updated.current_step() else {
                return;
            };
            let id = step.id.clone();
            let result = updated
                .retry_generation(&id)
                .and_then(|()| PracticeSessionStore::new(storage.clone()).save(&updated));
            match result {
                Ok(()) => session.set(Some(Ok(updated))),
                Err(error) => session.set(Some(Err(error))),
            }
        })
    };

    let on_advance = {
        let session = session.clone();
        let storage = storage.clone();
        Callback::from(move |outcome: ExerciseOutcome| {
            let Some(Ok(current)) = &*session else {
                return;
            };
            let mut updated = current.clone();
            let Some(step) = updated.current_step() else {
                return;
            };
            let step_id = step.id.clone();
            let result = updated.complete(&step_id, outcome).and_then(|()| {
                if updated.is_completed() {
                    updated.stamp_completed_at(chrono::Utc::now().timestamp_millis() as u64)?;
                } else {
                    updated.start_current()?;
                }
                let store = PracticeSessionStore::new(storage.clone());
                store.record_progress(&updated)?;
                Ok(())
            });
            match result {
                Ok(()) => session.set(Some(Ok(updated))),
                Err(error) => session.set(Some(Err(error))),
            }
        })
    };
    let on_finish = {
        let on_advance = on_advance.clone();
        Callback::from(move |()| {
            on_advance.emit(ExerciseOutcome::Completed);
        })
    };
    let on_scored = {
        let on_advance = on_advance.clone();
        Callback::from(move |correct: bool| {
            on_advance.emit(ExerciseOutcome::Scored {
                earned: u16::from(correct),
                possible: 1,
            });
        })
    };
    let on_finish_endless = {
        let storage = storage.clone();
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(Ok(current)) = &*session else {
                return;
            };
            let mut updated = current.clone();
            let result = updated.finish_endless_translation().and_then(|()| {
                updated.stamp_completed_at(chrono::Utc::now().timestamp_millis() as u64)?;
                let store = PracticeSessionStore::new(storage.clone());
                store.record_progress(&updated)
            });
            match result {
                Ok(()) => session.set(Some(Ok(updated))),
                Err(error) => session.set(Some(Err(error))),
            }
        })
    };
    let content = match &*session {
        None => html! {
            <section role="status" aria-live="polite"
                class="mx-auto flex max-w-lg flex-col items-center gap-4 rounded-2xl border border-line bg-surface p-8 text-center">
                <span class="loading-spinner" aria-hidden="true" />
                <h2 class="font-semibold">{"Restoring your practice"}</h2>
                <p class="text-sm text-muted">{"Checking your saved session and preparing the next exercise…"}</p>
            </section>
        },
        Some(Err(error)) => html! {
            <super::session_unavailable::SessionUnavailable error={error.clone()} on_retry={retry_restore.clone()} />
        },
        Some(Ok(saved)) if saved.is_completed() => {
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
                                match &*history_result {
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
                                                onclick={retry_history.clone()}
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
        Some(Ok(saved)) => {
            let step = saved
                .current_step()
                .expect("incomplete validated session must have an active step");
            let mut query = saved
                .vocabulary
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<HashMap<_, _>>();
            query.insert("rounds".to_owned(), "1".to_owned());
            let on_finish = on_finish.clone();
            let on_hanzi_ready_change = {
                let hanzi_ready_preview = hanzi_ready_preview.clone();
                let step_id = step.id.clone();
                Callback::from(move |ready: bool| {
                    let next = Some(HanziCompletionPreview {
                        step_id: step_id.clone(),
                        ready,
                    });
                    if *hanzi_ready_preview != next {
                        hanzi_ready_preview.set(next);
                    }
                })
            };
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
            let is_preparing = matches!(
                &step.readiness,
                ExerciseReadiness::NeedsGeneration
                    | ExerciseReadiness::Queued
                    | ExerciseReadiness::Generating
            );
            if is_preparing {
                html! {
                    <section class="mx-auto w-full max-w-xl rounded-2xl border border-line bg-surface/95 px-6 py-8 shadow-lg sm:px-10 sm:py-10"
                        data-practice-local-session="" data-practice-session-preparing="">
                        {exercise_content}
                    </section>
                }
            } else {
                html! {
                    <section class={if matches!(&step.readiness, ExerciseReadiness::Failed(_)) {"flex min-h-[50dvh] flex-col items-center justify-center gap-4"} else {"space-y-6"}} data-practice-local-session="">
                        {if saved.endless_translation && saved.current > 0 {
                            html! { <button type="button"
                                class="rounded-lg border border-line px-4 py-2 text-sm"
                                onclick={on_finish_endless.clone()}
                                data-practice-finish-endless="">
                                {"Finish practice"}
                            </button> }
                        } else { Html::default() }}
                        {exercise_content}
                    </section>
                }
            }
        }
    };

    let progress = match &*session {
        Some(Ok(saved)) => {
            let active_hanzi_id = saved.current_step().and_then(|step| {
                (step.exercise_id == "write-hanzi").then_some(step.id.as_str())
            });
            let completed = preview_progress(saved.current, active_hanzi_id, (*hanzi_ready_preview).as_ref());
            Some((completed, saved.steps.len(), saved.endless_translation))
        },
        _ => None,
    };
    let layout_classes = if horizontal_progress {
        "flex min-h-0 flex-1 flex-col items-center justify-center gap-2 sm:gap-3"
    } else {
        "flex min-h-0 flex-1 flex-row items-stretch gap-2 sm:gap-6 lg:gap-10"
    };
    let content_classes = if horizontal_progress {
        "order-1 flex min-h-0 w-full flex-1 flex-col justify-center overflow-y-auto py-2 sm:py-4"
    } else {
        "flex min-h-0 min-w-0 flex-1 flex-col justify-center overflow-y-auto py-4 sm:py-6"
    };
    html! {
        <main class="mx-auto flex h-full min-h-0 w-full max-w-7xl flex-col px-2 pt-12 sm:px-5 sm:pt-10">
            <span data-practice-leave="">
                <Link<Route> to={Route::PracticeSetup}
                    classes="fixed right-4 top-4 z-30 inline-flex size-11 items-center justify-center rounded-full border border-line bg-surface/95 text-ink shadow-lg backdrop-blur-md transition-[background-color,border-color,transform] duration-200 hover:-translate-x-0.5 hover:border-brand-action hover:bg-accent-soft focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent motion-reduce:transition-none">
                    <Icon data={IconData::LUCIDE_ARROW_LEFT} width="22" height="22" />
                    <span class="sr-only">{"Leave session"}</span>
                </Link<Route>>
            </span>
            <div class={layout_classes} data-practice-progress-layout={if horizontal_progress {"horizontal"} else {"vertical"}}>
                {if let Some((completed, total, endless)) = progress {
                    html! { <CreatureProgress {completed} {total} {endless} /> }
                } else { Html::default() }}
                <div class={content_classes}>
                    {content}
                </div>
            </div>
        </main>
    }
}

#[cfg(test)]
mod hanzi_progress_tests {
    use super::{HanziCompletionPreview, completion_percentage, preview_progress};

    #[test]
    fn completion_percentage_handles_empty_and_partial_sessions() {
        assert_eq!(completion_percentage(0, 0), 0);
        assert_eq!(completion_percentage(0, 5), 0);
        assert_eq!(completion_percentage(3, 5), 60);
        assert_eq!(completion_percentage(5, 5), 100);
    }

    #[test]
    fn previews_finished_hanzi_only_for_the_active_step() {
        let pending = HanziCompletionPreview {
            step_id: "step-2".to_owned(),
            ready: true,
        };
        assert_eq!(preview_progress(1, Some("step-2"), Some(&pending)), 2);
        assert_eq!(preview_progress(1, Some("step-3"), Some(&pending)), 1);
        assert_eq!(preview_progress(1, None, Some(&pending)), 1);
        let not_ready = HanziCompletionPreview {
            ready: false,
            ..pending
        };
        assert_eq!(preview_progress(1, Some("step-2"), Some(&not_ready)), 1);
        assert_eq!(preview_progress(1, Some("step-2"), None), 1);
    }
}
