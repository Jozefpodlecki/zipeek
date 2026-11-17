use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::{route::Route, services::practice_session_store::PracticeSessionError};

#[derive(Properties, PartialEq)]
pub(super) struct SessionUnavailableProps {
    pub error: PracticeSessionError,
    pub on_retry: Callback<MouseEvent>,
}

fn recovery_message(error: &PracticeSessionError) -> (&'static str, &'static str, bool) {
    match error {
        PracticeSessionError::InvalidStoredSession => (
            "This saved session can no longer be opened",
            "Its saved format may have changed, or the session data may be damaged. You can start a new session from Practice setup. Your personal Word Sets are separate from this session.",
            false,
        ),
        PracticeSessionError::NotFound | PracticeSessionError::InvalidId => (
            "This session was not found",
            "The session link may be old, or its browser data may have been cleared. Create a new session from Practice setup.",
            false,
        ),
        PracticeSessionError::Storage(_) => (
            "Saved sessions are temporarily unavailable",
            "The browser could not read or save your practice session. Check that browser storage is available, then try again.",
            true,
        ),
        PracticeSessionError::OutOfDate | PracticeSessionError::WrongSession => (
            "This session has changed",
            "Another tab or a newer session revision may have replaced this copy. Try loading it again, or return to Practice setup.",
            true,
        ),
        _ => (
            "This practice session cannot continue",
            "The saved session could not be restored safely. Try again, or return to Practice setup to begin a new session.",
            true,
        ),
    }
}

#[function_component(SessionUnavailable)]
pub(super) fn session_unavailable(props: &SessionUnavailableProps) -> Html {
    let (title, detail, retry_recommended) = recovery_message(&props.error);
    html! {
        <section data-component="practice-session-unavailable" data-session-error={match &props.error {
            PracticeSessionError::InvalidStoredSession => "invalid-stored-session",
            PracticeSessionError::NotFound => "not-found",
            PracticeSessionError::InvalidId => "invalid-id",
            PracticeSessionError::Storage(_) => "storage",
            PracticeSessionError::OutOfDate => "out-of-date",
            _ => "other",
        }} class="mx-auto flex min-h-[min(70vh,36rem)] w-full max-w-2xl items-center justify-center px-4 py-10">
            <div class="w-full rounded-2xl border border-line bg-surface p-6 shadow-lg sm:p-10" role="alert" aria-labelledby="session-recovery-heading">
                <div class="mb-6 flex size-14 items-center justify-center rounded-xl border border-line bg-raised text-accent" aria-hidden="true">
                    <Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="26" height="26" />
                </div>
                <p class="mb-2 text-xs font-semibold uppercase tracking-wider text-muted">{"Practice session"}</p>
                <h1 id="session-recovery-heading" class="text-2xl font-bold text-ink sm:text-3xl">{title}</h1>
                <p class="mt-4 max-w-prose text-sm leading-7 text-muted">{detail}</p>
                <div class="mt-8 flex flex-wrap items-center gap-3">
                    <Link<Route> to={Route::PracticeSetup} classes="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl bg-brand-action px-5 py-3 text-sm font-semibold text-white no-underline hover:opacity-90" >
                        <Icon data={IconData::LUCIDE_ARROW_LEFT} width="17" height="17" />
                        {"Back to Practice setup"}
                    </Link<Route>>
                    if retry_recommended {
                        <button type="button" onclick={props.on_retry.clone()} data-practice-session-restore-retry=""
                            class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border border-line bg-raised px-5 py-3 text-sm font-semibold text-ink hover:bg-surface">
                            <Icon data={IconData::LUCIDE_REFRESH_CW} width="17" height="17" />
                            {"Try again"}
                        </button>
                    }
                </div>
                <p class="mt-6 border-t border-line pt-4 text-xs text-muted">{"This screen does not delete your saved data."}</p>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_saved_session_does_not_suggest_retry() {
        let (title, _, retry) = recovery_message(&PracticeSessionError::InvalidStoredSession);
        assert!(title.contains("saved session"));
        assert!(!retry);
    }

    #[test]
    fn temporary_storage_failure_allows_retry() {
        let (_, _, retry) = recovery_message(&PracticeSessionError::Storage("unavailable".into()));
        assert!(retry);
    }
}
