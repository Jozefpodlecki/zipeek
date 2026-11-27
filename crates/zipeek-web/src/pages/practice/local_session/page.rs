use std::collections::HashMap;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{
    hooks::{use_app_environment, use_dictionary},
    route::Route,
    services::practice_generated,
    services::practice_session_store::{ExerciseOutcome, ExerciseReadiness, PracticeSessionStore},
};

use super::super::creature_progress::CreatureProgress;

use super::{
    LocalPracticeSessionProps, exercise, generation, history, persistence,
    progress::{HanziCompletionPreview, preview_progress},
    restore,
    state::{SessionAction, SessionState},
    summary,
    transition::ExerciseTransition,
};

#[function_component(LocalPracticeSessionPage)]
pub(super) fn local_practice_session_page(props: &LocalPracticeSessionProps) -> Html {
    let environment = use_app_environment();

    let dictionary = use_dictionary();
    let storage = environment.local_storage.clone();
    let session = use_reducer_eq(SessionState::default);
    let hanzi_ready_preview = use_state(|| None::<HanziCompletionPreview>);
    let history_retry = use_state(|| 0_u32);
    let restore_retry = use_state(|| 0_u32);
    let mounted = use_mut_ref(|| true);
    let active_session_id = use_mut_ref(|| props.id.clone());
    *active_session_id.borrow_mut() = props.id.clone();
    {
        let mounted = mounted.clone();
        use_effect_with((), move |_| move || *mounted.borrow_mut() = false);
    }

    {
        let session = session.clone();
        let storage = storage.clone();
        use_effect_with((props.id.clone(), *restore_retry), move |(id, _)| {
            let result = restore::restore_session(&PracticeSessionStore::new(storage.clone()), id);
            session.dispatch(SessionAction::Replace(Some(result)));
        });
    }

    let history_result = history::use_history_finalization(
        session.clone(),
        *history_retry,
        storage.clone(),
        environment.document.clone(),
    );
    let retry_restore = {
        let restore_retry = restore_retry.clone();
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            session.dispatch(SessionAction::Replace(None));
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
        let mounted = mounted.clone();
        let active_session_id = active_session_id.clone();
        use_effect_with(session.snapshot.clone(), move |snapshot| {
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
                                state.dispatch(SessionAction::Replace(Some(Err(error))));
                            } else {
                                state.dispatch(SessionAction::Replace(Some(Ok(started.clone()))));
                                let state = state.clone();
                                let storage = storage.clone();
                                let dictionary = dictionary.clone();
                                let vocabulary = started
                                    .vocabulary
                                    .iter()
                                    .map(|(key, value)| (key.clone(), value.clone()))
                                    .collect::<HashMap<_, _>>();
                                let mounted = mounted.clone();
                                let active_session_id = active_session_id.clone();
                                spawn_local(async move {
                                    let translation_ordinal = started
                                        .steps
                                        .iter()
                                        .take_while(|step| step.id != step_id)
                                        .filter(|step| step.exercise_id == "sentence-translation")
                                        .count();
                                    let matching_pairs = started.exercise_configurations.get("matching")
                                        .and_then(|config| match config {
                                            crate::services::practice_session_store::ExerciseConfiguration::Quiz { pair_count, .. } => Some(*pair_count),
                                            _ => None,
                                        });
                                    let narrative = started.exercise_configurations.get(&exercise_id)
                                        .and_then(|config| match config {
                                            crate::services::practice_session_store::ExerciseConfiguration::Narrative(settings) => Some(*settings),
                                            _ => None,
                                        })
                                        .unwrap_or_default();
                                    let result = practice_generated::prepare(
                                        &exercise_id,
                                        &vocabulary,
                                        seed,
                                        started.translation_direction(),
                                        translation_ordinal,
                                        matching_pairs,
                                        narrative,
                                        &dictionary,
                                        &storage,
                                    )
                                    .await;
                                    if !*mounted.borrow()
                                        || active_session_id.borrow().as_str()
                                            != started.id.as_str()
                                    {
                                        return;
                                    }
                                    let store = PracticeSessionStore::new(storage);
                                    if let Some(result) = generation::reconcile_generation(
                                        &store,
                                        &started.id,
                                        &step_id,
                                        attempt,
                                        result,
                                    ) {
                                        state.dispatch(SessionAction::Replace(Some(result)));
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });
    }

    let on_finish_endless = {
        let storage = storage.clone();
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(Ok(current)) = &session.snapshot else {
                return;
            };
            session.dispatch(SessionAction::Replace(Some(persistence::finish_endless(
                &PracticeSessionStore::new(storage.clone()),
                current,
            ))));
        })
    };
    let content = match &session.snapshot {
        None => html! {
            <section role="status" aria-live="polite"
                class="mx-auto flex max-w-lg flex-col items-center gap-4 rounded-2xl border border-line bg-surface p-8 text-center">
                <span class="loading-spinner" aria-hidden="true" />
                <h2 class="font-semibold">{"Restoring your practice"}</h2>
                <p class="text-sm text-muted">{"Checking your saved session and preparing the next exercise…"}</p>
            </section>
        },
        Some(Err(error)) => html! {
            <super::super::session_unavailable::SessionUnavailable error={error.clone()} on_retry={retry_restore.clone()} />
        },
        Some(Ok(saved)) if saved.is_completed() => {
            summary::render(saved, &*history_result, &retry_history)
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
            // Bind the callbacks to the rendered step. An outgoing exercise can
            // finish an async action after the session has already advanced; that
            // must never complete or retry the next step.
            let on_advance = {
                let session = session.clone();
                let storage = storage.clone();
                let step_id = step.id.clone();
                Callback::from(move |outcome: ExerciseOutcome| {
                    let Some(Ok(current)) = &session.snapshot else {
                        return;
                    };
                    if !persistence::is_current_step(current, &step_id) {
                        return;
                    }
                    session.dispatch(SessionAction::Replace(Some(persistence::complete_step(
                        &PracticeSessionStore::new(storage.clone()),
                        current,
                        &step_id,
                        outcome,
                    ))));
                })
            };
            let on_retry = {
                let session = session.clone();
                let storage = storage.clone();
                let step_id = step.id.clone();
                Callback::from(move |_: MouseEvent| {
                    let Some(Ok(current)) = &session.snapshot else {
                        return;
                    };
                    if !persistence::is_current_step(current, &step_id) {
                        return;
                    }
                    session.dispatch(SessionAction::Replace(Some(persistence::retry_generation(
                        &PracticeSessionStore::new(storage.clone()),
                        current,
                        &step_id,
                    ))));
                })
            };
            let exercise_content = exercise::render_exercise(
                saved,
                step,
                &query,
                on_advance,
                on_retry,
                on_hanzi_ready_change,
            );
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
                    <section key={step.id.clone()} class={if matches!(&step.readiness, ExerciseReadiness::Failed(_)) {"practice-round flex min-h-[50dvh] flex-col items-center justify-center gap-4"} else {"practice-round space-y-6"}} data-practice-local-session="">
                        {exercise_content}
                    </section>
                }
            }
        }
    };

    let progress = match &session.snapshot {
        Some(Ok(saved)) => {
            let active_hanzi_id = saved.current_step().and_then(|step| {
                matches!(
                    step.exercise_id.as_str(),
                    "write-hanzi" | "character-builder"
                )
                .then_some(step.id.as_str())
            });
            let completed = preview_progress(
                saved.current,
                active_hanzi_id,
                (*hanzi_ready_preview).as_ref(),
            );
            Some((
                completed,
                saved.steps.len(),
                saved.session_mode.is_endless(),
            ))
        }
        _ => None,
    };
    let layout_classes = "flex min-h-0 w-full flex-1 flex-col items-center gap-3 sm:gap-4";
    let content_classes = "flex min-h-0 w-full flex-1 flex-col justify-center py-2 sm:py-4";
    let transition_step = match &session.snapshot {
        Some(Ok(saved)) if !saved.is_completed() => saved
            .current_step()
            .map(|step| step.id.clone())
            .unwrap_or_else(|| "completed".to_owned()),
        Some(Ok(_)) => "completed".to_owned(),
        Some(Err(_)) => "unavailable".to_owned(),
        None => "restoring".to_owned(),
    };
    html! {
        <main class="mx-auto flex min-h-dvh w-full max-w-7xl flex-col px-2 pb-6 pt-12 sm:px-5 sm:pb-8 sm:pt-10">
            <span data-practice-leave="">
            {if !matches!(&session.snapshot, Some(Ok(saved)) if saved.is_completed()) { html! {
                <Link<Route> to={Route::PracticeSetup}
                    classes="fixed right-4 top-4 z-30 inline-flex size-11 items-center justify-center rounded-full border border-line bg-surface/95 text-ink shadow-lg backdrop-blur-md transition-[background-color,border-color,transform] duration-200 hover:-translate-x-0.5 hover:border-brand-action hover:bg-accent-soft focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent motion-reduce:transition-none">
                    <Icon data={IconData::LUCIDE_ARROW_LEFT} width="22" height="22" />
                    <span class="sr-only">{"Leave session"}</span>
                </Link<Route>>
            } } else { Html::default() }}
            </span>
            <div class={layout_classes} data-practice-progress-layout="top">
                {if let Some((completed, total, endless)) = progress {
                    html! { <CreatureProgress {completed} {total} {endless} /> }
                } else { Html::default() }}
                <div class={content_classes}>
                    <div class="practice-round mx-auto w-full max-w-4xl" data-practice-exercise-workspace="">
                        <ExerciseTransition step_id={transition_step}>
                            {if let Some(Ok(saved)) = &session.snapshot {
                                if saved.is_completed() {
                                    content.clone()
                                } else {
                                    html! {
                                        <super::frame::ExerciseFrame
                                            position={saved.current + 1}
                                            total={saved.steps.len()}
                                            endless={saved.session_mode.is_endless()}
                                            on_finish_endless={if saved.session_mode.is_endless() && saved.current > 0 { Some(on_finish_endless.clone()) } else { None }}
                                        >
                                            {content.clone()}
                                        </super::frame::ExerciseFrame>
                                    }
                                }
                            } else {
                                content.clone()
                            }}
                        </ExerciseTransition>
                    </div>
                </div>
            </div>
        </main>
    }
}
