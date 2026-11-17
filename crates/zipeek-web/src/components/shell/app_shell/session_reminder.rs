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
    let reload = use_state(|| 0_u32);

    {
        let stored = stored.clone();
        let error = error.clone();
        let dismissed = dismissed.clone();
        let inventory_warning = inventory_warning.clone();
        let storage = environment.local_storage.clone();
        use_effect_with((props.route.clone(), *reload), move |_| {
            dismissed.set(false);
            match PracticeSessionStore::new(storage).recoverable_sessions() {
                Ok(inventory) => {
                    stored.set(inventory.sessions.into_iter().next());
                    inventory_warning.set((inventory.invalid_records > 0).then(|| {
                        format!("{} saved practice record(s) could not be restored. The records were not deleted.", inventory.invalid_records)
                    }));
                    error.set(None);
                }
                Err(failure) => {
                    stored.set(None);
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
    if in_session || *dismissed {
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
    let on_end_session = {
        let stored = stored.clone();
        let error = error.clone();
        let storage = environment.local_storage.clone();
        let id = (*stored)
            .as_ref()
            .map(|session| session.id.clone())
            .unwrap_or_default();
        Callback::from(
            move |_| match PracticeSessionStore::new(storage.clone()).discard(&id) {
                Ok(()) => {
                    stored.set(None);
                    error.set(None);
                }
                Err(failure) => {
                    error.set(Some(format!("Could not end the session: {failure:?}")));
                }
            },
        )
    };
    let on_reload = {
        let reload = reload.clone();
        Callback::from(move |_| reload.set(reload.wrapping_add(1)))
    };
    let on_dismiss = {
        let dismissed = dismissed.clone();
        Callback::from(move |_| dismissed.set(true))
    };

    html! {
        <aside
            data-unfinished-session=""
            aria-label="Saved practice session"
            class="fixed bottom-5 right-5 z-50 w-[min(24rem,calc(100vw-2rem))] rounded-2xl border border-line bg-surface p-5 shadow-2xl"
        >
            <div class="flex items-start justify-between gap-3">
                <h2 class="font-bold">{if completed { "Practice results awaiting recovery" } else { "Practice session in progress" }}</h2>
                <button
                    type="button"
                    aria-label="Dismiss session reminder"
                    onclick={on_dismiss}
                    class="text-muted hover:text-ink"
                >
                    <Icon data={IconData::LUCIDE_X} width="18" height="18" />
                </button>
            </div>
            {if let Some(session) = &*stored {
                html! { <p class="mt-2 text-sm text-muted">
                    {if completed {
                        "Your completed session is still stored. Open it to retry learning-history handling before leaving it behind.".to_owned()
                    } else {
                        format!("{} of {} exercises completed", session.current, session.steps.len())
                    }}
                </p> }
            } else { Html::default() }}
            {if let Some(message) = &*inventory_warning {
                html! { <p role="alert" class="mt-2 text-sm" data-practice-recovery-warning="">{message}</p> }
            } else { Html::default() }}
            <div class="mt-4 flex flex-wrap gap-2">
                {if let Some(resume) = resume {
                    html! {
                        <Link<Route>
                            to={resume}
                            classes="rounded-lg bg-brand-action px-4 py-2 font-semibold text-white"
                        >
                            {if completed { "Recover results" } else { "Resume practice" }}
                        </Link<Route>>
                    }
                } else { Html::default() }}
                {if !completed && (*stored).is_some() {
                    html! { <button
                        type="button"
                        onclick={on_end_session}
                        class="rounded-lg border border-line px-4 py-2 font-semibold"
                    >
                        {"End session"}
                    </button> }
                } else { Html::default() }}
                {if (*inventory_warning).is_some() {
                    html! { <button type="button" onclick={on_reload}
                        class="rounded-lg border border-line px-4 py-2 font-semibold"
                        data-practice-recovery-refresh="">
                        {"Retry checking sessions"}
                    </button> }
                } else { Html::default() }}
            </div>
            {if let Some(message) = &*error {
                html! { <p role="alert" class="mt-2 text-sm">{message}</p> }
            } else {
                Html::default()
            }}
        </aside>
    }
}
