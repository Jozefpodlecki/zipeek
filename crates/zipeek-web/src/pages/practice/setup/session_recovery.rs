use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{route::Route, services::practice_session_store::PracticeSession};

#[derive(Properties, PartialEq)]
pub(super) struct PracticeSessionRecoveryProps {
    pub session: Option<PracticeSession>,
    pub warning: Option<String>,
}

#[function_component(PracticeSessionRecovery)]
pub(super) fn practice_session_recovery(props: &PracticeSessionRecoveryProps) -> Html {
    html! {
        <section
            data-component="practice-setup-page"
            data-practice-session-blocked=""
            class="flex w-full flex-col gap-5 rounded-2xl border border-line bg-surface p-6"
        >
            <h1 class="text-2xl font-bold">{"Continue your saved Practice"}</h1>
            <p>{"Only one saved Practice session can be active at a time. Finish or recover the existing session before starting another."}</p>
            {if let Some(session) = &props.session {
                html! {
                    <div class="flex flex-wrap items-center gap-3">
                        <Link<Route>
                            to={Route::LocalPracticeSession { id: session.id.clone() }}
                            classes="rounded-lg bg-brand-action px-4 py-2 font-semibold text-white"
                        >
                            {if session.is_completed() { "Recover results" } else { "Resume practice" }}
                        </Link<Route>>
                        <span class="text-sm text-muted">
                            {if session.is_completed() {
                                "Completed results are awaiting history handling.".to_owned()
                            } else {
                                format!("{} of {} exercises completed", session.current, session.steps.len())
                            }}
                        </span>
                    </div>
                }
            } else { Html::default() }}
            {if let Some(warning) = &props.warning {
                html! { <p role="alert" data-practice-session-warning="">{warning}</p> }
            } else { Html::default() }}
            <p class="text-sm text-muted">
                {"To discard an unfinished session, open it or use the saved-session reminder. Completed results should be recovered first."}
            </p>
        </section>
    }
}
