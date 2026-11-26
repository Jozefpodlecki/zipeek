use yew::prelude::*;
use yew_icons::{Icon, IconData};

mod backup;
mod deletion;
mod history;
mod personal_storage_categories;

#[function_component(PersonalStoragePanel)]
pub fn personal_storage_panel() -> Html {
    let message = use_state(String::new);
    html! {
        <section data-component="personal-storage" class="rounded-xl border border-line bg-surface p-4 sm:p-5 space-y-3" aria-labelledby="manage-learning-heading">
            <div class="flex items-center gap-2">
                <Icon data={IconData::LUCIDE_DATABASE} width="18px" />
                <h3 id="manage-learning-heading" class="font-semibold">{"Manage learning data"}</h3>
            </div>
            <details class="rounded-lg border border-line bg-raised p-3" data-learning-backup-section="">
                <summary class="cursor-pointer font-medium focus-visible:outline-2 focus-visible:outline-accent">{"Backup and restore"}</summary>
                <p class="mt-2 text-xs text-muted">{"The portable JSON backup excludes the Practice outcomes database. Browser-local snapshots are managed separately."}</p>
                <div class="mt-3"><backup::BackupPanel message={message.clone()} /></div>
            </details>
            <details class="rounded-lg border border-red-500/35 p-3" data-learning-delete-section="">
                <summary class="cursor-pointer font-medium focus-visible:outline-2 focus-visible:outline-accent">{"Delete learning data"}</summary>
                <div class="mt-3 space-y-4">
                    <deletion::DeletionPanel message={message.clone()} />
                    <history::PersonalLearningHistoryPanel />
                </div>
            </details>
            if !message.is_empty() {
                <p role="status" data-personal-feedback="" class="rounded-lg bg-raised p-3 text-sm">{(*message).clone()}</p>
            }
        </section>
    }
}
