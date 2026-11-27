use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{
    hooks::use_app_environment,
    route::Route,
    services::practice_session_store::{PracticeSession, PracticeSessionStore},
};

#[derive(Properties, PartialEq)]
pub(super) struct SessionReminderProps {
    pub route: Option<Route>,
}

#[function_component(SessionReminder)]
pub(super) fn session_reminder(props: &SessionReminderProps) -> Html {
    let environment = use_app_environment();
    let dismissed = use_state(|| false);
    let stored = use_state(|| None::<PracticeSession>);
    let error = use_state(|| None::<String>);
    let inventory_warning = use_state(|| None::<String>);
    let invalid_records = use_state(|| 0_usize);
    let reload = use_state(|| 0_u32);
    {
        let window = environment.window.clone();
        let revision = reload.clone();
        use_effect_with((), move |_| {
            let on_change = {
                let revision = revision.clone();
                gloo::events::EventListener::new(
                    &window,
                    crate::services::practice_session_store::SESSION_CHANGE_EVENT,
                    move |_| revision.set((*revision).wrapping_add(1)),
                )
            };
            let on_storage = gloo::events::EventListener::new(&window, "storage", move |_| {
                revision.set((*revision).wrapping_add(1));
            });
            move || drop((on_change, on_storage))
        });
    }

    {
        let stored = stored.clone();
        let error = error.clone();
        let dismissed = dismissed.clone();
        let inventory_warning = inventory_warning.clone();
        let invalid_records = invalid_records.clone();
        let storage = environment.local_storage.clone();
        use_effect_with((props.route.clone(), *reload), move |_| {
            dismissed.set(false);
            match PracticeSessionStore::new(storage).recoverable_sessions() {
                Ok(inventory) => {
                    let warning = (inventory.invalid_records > 0).then(|| {
                        format!(
                            "{} saved practice record(s) could not be restored. The records were not deleted.",
                            inventory.invalid_records,
                        )
                    });
                    invalid_records.set(inventory.invalid_records);
                    stored.set(inventory.sessions.into_iter().next());
                    inventory_warning.set(warning);
                    error.set(None);
                }
                Err(failure) => {
                    stored.set(None);
                    invalid_records.set(0);
                    inventory_warning
                        .set(Some(format!("Could not check saved practice: {failure:?}")));
                    error.set(None);
                }
            }
        });
    }

    let in_session = matches!(
        props.route.as_ref(),
        Some(Route::LocalPracticeSession { .. })
    );
    if in_session || matches!(props.route.as_ref(), Some(Route::PracticeSetup)) || *dismissed {
        return Html::default();
    }
    if (*stored).is_none() && (*inventory_warning).is_none() {
        return Html::default();
    }

    let completed = (*stored)
        .as_ref()
        .is_some_and(PracticeSession::is_completed);
    let resume = (*stored)
        .as_ref()
        .map(|session| Route::LocalPracticeSession {
            id: session.id.clone(),
        });

    let on_remove_session = {
        let stored = stored.clone();
        let error = error.clone();
        let reload = reload.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(id) = (*stored).as_ref().map(|session| session.id.clone()) else {
                return;
            };
            match PracticeSessionStore::new(storage.clone()).discard(&id) {
                Ok(()) => {
                    error.set(None);
                    reload.set((*reload).wrapping_add(1));
                }
                Err(failure) => {
                    error.set(Some(format!(
                        "Could not remove the saved session: {failure:?}"
                    )));
                }
            }
        })
    };

    let on_clear_invalid = {
        let error = error.clone();
        let reload = reload.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |_: MouseEvent| {
            match PracticeSessionStore::new(storage.clone()).clear_invalid_sessions() {
                Ok(_) => {
                    error.set(None);
                    reload.set((*reload).wrapping_add(1));
                }
                Err(failure) => {
                    error.set(Some(format!(
                        "Could not remove broken saved sessions: {failure:?}"
                    )));
                }
            }
        })
    };

    let on_reload = {
        let reload = reload.clone();
        Callback::from(move |_: MouseEvent| reload.set((*reload).wrapping_add(1)))
    };

    let on_dismiss = {
        let dismissed = dismissed.clone();
        Callback::from(move |_: MouseEvent| dismissed.set(true))
    };

    let icon_button_classes = "flex size-11 items-center justify-center rounded-xl border border-line bg-surface text-ink transition hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";

    html! {
        <aside
            data-unfinished-session=""
            aria-label="Saved practice session"
            class="fixed right-5 top-5 z-50 w-[min(26rem,calc(100vw-2rem))] rounded-3xl border border-line bg-surface/95 p-5 shadow-2xl backdrop-blur"
        >
            <div class="flex items-start justify-between gap-3">
                <div class="space-y-1">
                    <h2 class="text-lg font-bold">{if completed { "Practice results awaiting recovery" } else { "Practice session in progress" }}</h2>
                    {if let Some(session) = &*stored {
                        html! { <p class="text-sm text-muted">
                            {if completed {
                                "Your completed session is still stored. Recover the results or remove the saved session.".to_owned()
                            } else {
                                format!("{} / {}", session.current, session.steps.len())
                            }}
                        </p> }
                    } else {
                        Html::default()
                    }}
                </div>
                <button
                    type="button"
                    aria-label="Dismiss session reminder"
                    title="Dismiss"
                    onclick={on_dismiss}
                    class="flex size-9 items-center justify-center rounded-lg text-muted transition hover:bg-raised hover:text-ink"
                >
                    <Icon data={IconData::LUCIDE_X} width="18" height="18" />
                </button>
            </div>

            {if let Some(message) = &*inventory_warning {
                html! { <p role="alert" class="mt-3 text-sm" data-practice-recovery-warning="">{message}</p> }
            } else {
                Html::default()
            }}

            <div class="mt-5 flex items-center justify-between gap-3">
                <div class="flex flex-wrap gap-2">
                    {if let Some(resume) = resume {
                        html! {
                            <Link<Route>
                                to={resume}
                                classes="inline-flex min-h-11 items-center gap-2 rounded-xl bg-brand-action px-4 py-2 font-semibold text-white transition hover:brightness-110 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"
                            >
                                <Icon data={IconData::LUCIDE_PLAY} width="17" height="17" />
                                {if completed { "Recover" } else { "Resume" }}
                            </Link<Route>>
                        }
                    } else {
                        Html::default()
                    }}
                </div>
                <div class="flex items-center gap-2">
                    {if *invalid_records > 0 {
                        html! {
                            <>
                                <button
                                    type="button"
                                    onclick={on_reload.clone()}
                                    class={icon_button_classes}
                                    title="Retry checking saved sessions"
                                    aria-label="Retry checking saved sessions"
                                    data-practice-recovery-refresh=""
                                >
                                    <Icon data={IconData::LUCIDE_REFRESH_CW} width="18" height="18" />
                                </button>
                                <button
                                    type="button"
                                    onclick={on_clear_invalid.clone()}
                                    class={icon_button_classes}
                                    title="Delete broken saved sessions"
                                    aria-label="Delete broken saved sessions"
                                    data-practice-recovery-clear-invalid=""
                                >
                                    <Icon data={IconData::LUCIDE_TRASH_2} width="18" height="18" />
                                </button>
                            </>
                        }
                    } else {
                        Html::default()
                    }}
                    {if (*stored).is_some() {
                        html! {
                            <button
                                type="button"
                                onclick={on_remove_session.clone()}
                                class={icon_button_classes}
                                title="Delete saved session"
                                aria-label="Delete saved session"
                                data-practice-recovery-discard=""
                            >
                                <Icon data={IconData::LUCIDE_TRASH_2} width="18" height="18" />
                            </button>
                        }
                    } else {
                        Html::default()
                    }}
                </div>
            </div>

            {if let Some(message) = &*error {
                html! { <p role="alert" class="mt-3 text-sm">{message}</p> }
            } else {
                Html::default()
            }}
        </aside>
    }
}
