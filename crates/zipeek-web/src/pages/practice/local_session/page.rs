use std::collections::HashMap;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{
    appearance::Appearance,
    hooks::{use_app_environment, use_dictionary},
    route::Route,
    services::practice_generated,
    services::practice_session_store::{
        ExerciseOutcome, ExerciseReadiness, PracticeSession, PracticeSessionError,
        PracticeSessionStore,
    },
};

use super::super::creature_progress::{
    CreatureOrientation, CreatureProgress, creature_orientation,
};

use super::{
    LocalPracticeSessionProps, exercise, generation, history, persistence,
    progress::{HanziCompletionPreview, preview_progress},
    restore, summary,
};

#[function_component(LocalPracticeSessionPage)]
pub(super) fn local_practice_session_page(props: &LocalPracticeSessionProps) -> Html {
    let environment = use_app_environment();
    let appearance =
        use_context::<Appearance>().expect("Appearance provider missing from Practice session");
    let horizontal_progress =
        creature_orientation(appearance.preset) == CreatureOrientation::Horizontal;
    let dictionary = use_dictionary();
    let storage = environment.local_storage.clone();
    let session = use_state(|| None::<Result<PracticeSession, PracticeSessionError>>);
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
            session.set(Some(result));
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
        let mounted = mounted.clone();
        let active_session_id = active_session_id.clone();
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
                                    let result = practice_generated::prepare(
                                        &exercise_id,
                                        &vocabulary,
                                        seed,
                                        started.translation_direction(),
                                        translation_ordinal,
                                        matching_pairs,
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
                                        state.set(Some(result));
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
            session.set(Some(persistence::retry_generation(
                &PracticeSessionStore::new(storage.clone()),
                current,
            )));
        })
    };

    let on_advance = {
        let session = session.clone();
        let storage = storage.clone();
        Callback::from(move |outcome: ExerciseOutcome| {
            let Some(Ok(current)) = &*session else {
                return;
            };
            session.set(Some(persistence::complete_step(
                &PracticeSessionStore::new(storage.clone()),
                current,
                outcome,
            )));
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
            session.set(Some(persistence::finish_endless(
                &PracticeSessionStore::new(storage.clone()),
                current,
            )));
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
            let exercise_content = exercise::render_exercise(
                saved,
                step,
                &query,
                on_finish.clone(),
                on_scored.clone(),
                on_retry.clone(),
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
            let active_hanzi_id = saved
                .current_step()
                .and_then(|step| (step.exercise_id == "write-hanzi").then_some(step.id.as_str()));
            let completed = preview_progress(
                saved.current,
                active_hanzi_id,
                (*hanzi_ready_preview).as_ref(),
            );
            Some((completed, saved.steps.len(), saved.endless_translation))
        }
        _ => None,
    };
    let layout_classes = if horizontal_progress {
        "flex w-full flex-1 flex-col items-center justify-start gap-3 sm:gap-4"
    } else {
        "flex min-h-0 flex-1 flex-row items-stretch gap-2 sm:gap-6 lg:gap-10"
    };
    let content_classes = if horizontal_progress {
        "order-1 flex w-full flex-none flex-col justify-center py-2 sm:py-4"
    } else {
        "flex min-h-0 min-w-0 flex-1 flex-col justify-center overflow-y-auto py-4 sm:py-6"
    };
    html! {
        <main class="mx-auto flex min-h-dvh w-full max-w-7xl flex-col px-2 pb-6 pt-12 sm:px-5 sm:pb-8 sm:pt-10">
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
