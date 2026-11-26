use crate::{hooks::use_app_environment, services::app_reset};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(ResetSettings)]
pub(super) fn reset_settings() -> Html {
    let environment = use_app_environment();
    let error = use_state(|| None::<String>);
    let busy = use_state(|| false);
    let pending = use_mut_ref(|| false);
    let on_reset = {
        let window = environment.window.clone();
        let error = error.clone();
        let busy = busy.clone();
        let pending = pending.clone();
        Callback::from(move |_| {
            if *pending.borrow() {
                return;
            }
            let confirmation = window.confirm_with_message("Permanently clear all Zipeek preferences, progress, Word Sets, exclusions, saved sessions and personal snapshots? You will return to the landing page. Dictionary files, models and exported backups will be kept.");
            match confirmation {
                Ok(true) => {
                    *pending.borrow_mut() = true;
                    busy.set(true);
                    error.set(None);
                    if let Err(message) = app_reset::request(&window) {
                        error.set(Some(message));
                        *pending.borrow_mut() = false;
                        busy.set(false);
                    }
                }
                Ok(false) => {}
                Err(_) => error.set(Some(
                    "Could not open the reset confirmation. No data was cleared.".into(),
                )),
            }
        })
    };
    html! {
        <section class="space-y-4 rounded-2xl border border-red-500/35 bg-surface p-5 sm:p-6" aria-labelledby="reset-app-heading" data-component="reset-settings">
            <div class="flex items-center gap-2">
                <Icon data={IconData::LUCIDE_TRASH_2} width="20px" />
                <h2 id="reset-app-heading" class="text-xl font-semibold">{"Reset app"}</h2>
            </div>
            <p class="text-sm text-muted">{"Permanently clear your preferences, learning progress, Word Sets, exclusions, saved sessions and personal snapshots. Zipeek will reopen at the landing page so you can start again."}</p>
            <p class="text-sm text-muted">{"Dictionary files, model files and exported backups are kept. Close other Zipeek tabs and windows first. This cannot be undone."}</p>
            <button type="button" onclick={on_reset} disabled={*busy} data-reset-app="" class="inline-flex min-h-11 items-center gap-2 rounded-lg border border-red-500/50 px-4 py-2 font-semibold text-ink hover:bg-red-500/10 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:opacity-50">
                <Icon data={IconData::LUCIDE_TRASH_2} width="18px" />
                {if *busy { "Preparing reset..." } else { "Clear all data and start over" }}
            </button>
            if let Some(message) = &*error {
                <p role="alert">{message}</p>
            }
        </section>
    }
}
