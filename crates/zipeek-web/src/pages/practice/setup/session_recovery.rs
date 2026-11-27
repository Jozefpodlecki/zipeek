use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{route::Route, services::practice_session_store::PracticeSession};

#[derive(Properties, PartialEq)]
pub(super) struct PracticeSessionRecoveryProps {
    pub session: Option<PracticeSession>,
    pub warning: Option<String>,
    #[prop_or_default]
    pub has_invalid_records: bool,
    pub on_retry: Callback<MouseEvent>,
    pub on_discard_session: Callback<MouseEvent>,
    pub on_clear_invalid_records: Callback<MouseEvent>,
}

#[function_component(PracticeSessionRecovery)]
pub(super) fn practice_session_recovery(props: &PracticeSessionRecoveryProps) -> Html {
    html! {
        <section
            data-component="practice-setup-page"
            data-practice-session-blocked=""
            class="flex min-h-[calc(100dvh-var(--topbar-height)-2rem)] w-full items-center justify-center px-4 py-6"
        >
            <div class="w-full max-w-xl rounded-2xl border border-line bg-surface/95 p-6 shadow-2xl backdrop-blur sm:p-7">
                <h1 class="text-2xl font-bold">{"Continue practice"}</h1>

                <div class="mt-5 space-y-5">
                    {if let Some(session) = &props.session {
                        if session.is_completed() {
                            html! { <p class="text-sm text-muted">{"Results ready to recover"}</p> }
                        } else if session.session_mode.is_endless() {
                            html! { <p class="text-sm text-muted">{format!("{} completed · Endless", session.current)}</p> }
                        } else {
                            let total = session.steps.len();
                            let completed = session.current.min(total);
                            html! { <div class="space-y-2" aria-label="Practice progress">
                                <div class="flex items-center justify-between text-sm text-muted"><span>{"Progress"}</span><span>{format!("{completed} / {total}")}</span></div>
                                <div class="h-2 overflow-hidden rounded-full bg-raised" role="progressbar" aria-valuemin="0" aria-valuemax={total.to_string()} aria-valuenow={completed.to_string()}>
                                    <div class="h-full rounded-full bg-brand-action" style={format!("width: {}%", if total == 0 { 0 } else { 100 * completed / total })}></div>
                                </div>
                            </div> }
                        }
                    } else { Html::default() }}
                    {if let Some(warning) = &props.warning { html! { <p role="alert" class="text-sm text-red-400">{warning}</p> } } else { Html::default() }}

                    <div class="flex flex-wrap items-center gap-3 lg:justify-end">
                        {if let Some(session) = &props.session {
                            html! {
                                <Link<Route>
                                    to={Route::LocalPracticeSession { id: session.id.clone() }}
                                    classes="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl bg-brand-action px-5 py-3 font-semibold text-white transition hover:brightness-110 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"
                                >
                                    <Icon data={IconData::LUCIDE_PLAY} width="18" height="18" />
                                    {if session.is_completed() { "Recover" } else { "Resume" }}
                                </Link<Route>>
                            }
                        } else {
                            Html::default()
                        }}
                        <button
                            type="button"
                            aria-label="Delete saved session" title="Delete saved session"
                            onclick={props.on_discard_session.clone()}
                            class="inline-flex size-11 items-center justify-center rounded-xl border border-line text-muted transition hover:bg-raised hover:text-ink disabled:opacity-50"
                            disabled={props.session.is_none()}
                        >
                            <Icon data={IconData::LUCIDE_TRASH_2} width="18" height="18" />
                        </button>
                        {if props.has_invalid_records {
                            html! {
                                <button
                                    type="button"
                                    onclick={props.on_clear_invalid_records.clone()}
                                    class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border border-line px-4 py-3 font-semibold transition hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
                                >
                                    <Icon data={IconData::LUCIDE_TRASH_2} width="18" height="18" />
                                    {"Delete broken records"}
                                </button>
                            }
                        } else {
                            Html::default()
                        }}
                        {if props.session.is_none() || props.warning.is_some() { html! { <button
                            type="button"
                            onclick={props.on_retry.clone()}
                            class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border border-line px-4 py-3 font-semibold transition hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
                        >
                            <Icon data={IconData::LUCIDE_REFRESH_CW} width="18" height="18" />
                            {"Retry"}
                        </button> } } else { Html::default() }}
                    </div>
                </div>
            </div>
        </section>
    }
}
