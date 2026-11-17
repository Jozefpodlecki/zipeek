use crate::services::personal_data::{self, PersonalDataService};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct BackupPanelProps {
    pub message: UseStateHandle<String>,
}

#[function_component(BackupPanel)]
pub(super) fn backup_panel(props: &BackupPanelProps) -> Html {
    let environment = crate::hooks::use_app_environment();
    let personal_data = PersonalDataService::new(
        environment.local_storage.clone(),
        environment.document.clone(),
        environment.navigator.clone(),
    );
    let message = props.message.clone();
    let backup_text = use_state(String::new);
    let import_confirmation = use_state(String::new);
    let restore_revision = use_mut_ref(|| 0u64);
    let export = {
        let personal_data = personal_data.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let result = personal_data
                .export_backup()
                .and_then(|raw| personal_data.download_backup(&raw));
            message.set(match result {
                Ok(()) => "Backup download requested. Keep the file somewhere safe.".to_owned(),
                Err(error) => personal_data::error_message(error),
            });
        })
    };

    let on_backup_text = {
        let backup_text = backup_text.clone();
        let import_confirmation = import_confirmation.clone();
        let restore_revision = restore_revision.clone();
        Callback::from(move |event: InputEvent| {
            *restore_revision.borrow_mut() += 1;
            backup_text.set(event.target_unchecked_into::<HtmlTextAreaElement>().value());
            import_confirmation.set(String::new());
        })
    };

    let choose_backup_file = {
        let personal_data = personal_data.clone();
        let backup_text = backup_text.clone();
        let message = message.clone();
        let import_confirmation = import_confirmation.clone();
        let restore_revision = restore_revision.clone();
        Callback::from(move |_| {
            let revision = {
                let mut current = restore_revision.borrow_mut();
                *current += 1;
                *current
            };
            let backup_text = backup_text.clone();
            let message = message.clone();
            let import_confirmation = import_confirmation.clone();
            let personal_data = personal_data.clone();
            let restore_revision = restore_revision.clone();
            import_confirmation.set(String::new());
            spawn_local(async move {
                let result = personal_data.pick_backup_file().await;
                if *restore_revision.borrow() != revision {
                    return;
                }
                match result {
                    Ok(raw) if !raw.is_empty() => match personal_data.preview_backup(&raw) {
                        Ok(summary) => {
                            backup_text.set(raw);
                            import_confirmation.set(String::new());
                            message.set(format!(
                                "File validated: {summary}. Review before typing REPLACE."
                            ));
                        }
                        Err(error) => {
                            backup_text.set(String::new());
                            message.set(personal_data::error_message(error));
                        }
                    },
                    Ok(_) => {}
                    Err(error) => message.set(personal_data::error_message(error)),
                }
            });
        })
    };

    let preview = {
        let personal_data = personal_data.clone();
        let message = message.clone();
        let backup_text = backup_text.clone();
        Callback::from(move |_| {
            message.set(match personal_data.preview_backup(&backup_text) {
                Ok(text) => text,
                Err(error) => personal_data::error_message(error),
            })
        })
    };

    let import = {
        let personal_data = personal_data.clone();
        let message = message.clone();
        let backup_text = backup_text.clone();
        let import_confirmation = import_confirmation.clone();
        Callback::from(move |_| {
            if *import_confirmation != "REPLACE" || backup_text.trim().is_empty() {
                return;
            }
            message.set(match personal_data.import_backup(&backup_text) {
                Ok(text) => text,
                Err(error) => personal_data::error_message(error),
            });
            import_confirmation.set(String::new());
        })
    };

    let on_import_confirm = {
        let import_confirmation = import_confirmation.clone();
        Callback::from(move |event: InputEvent| {
            import_confirmation.set(event.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };

    let load_snapshot = {
        let personal_data = personal_data.clone();
        let backup_text = backup_text.clone();
        let message = message.clone();
        let import_confirmation = import_confirmation.clone();
        let restore_revision = restore_revision.clone();
        Callback::from(move |_| {
            let revision = {
                let mut current = restore_revision.borrow_mut();
                *current += 1;
                *current
            };
            let backup_text = backup_text.clone();
            let message = message.clone();
            let import_confirmation = import_confirmation.clone();
            let personal_data = personal_data.clone();
            let restore_revision = restore_revision.clone();
            import_confirmation.set(String::new());
            spawn_local(async move {
                let result = personal_data.read_snapshot().await;
                if *restore_revision.borrow() != revision {
                    return;
                }
                match result {
                    Ok(text) => {
                        backup_text.set(text);
                        import_confirmation.set(String::new());
                        message.set("Private snapshot loaded into the restore preview. Type REPLACE to apply it.".to_owned());
                    }
                    Err(error) => message.set(personal_data::error_message(error)),
                }
            });
        })
    };

    let save_snapshot = {
        let personal_data = personal_data.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let message = message.clone();
            let personal_data = personal_data.clone();
            spawn_local(async move {
                message.set(match personal_data.save_snapshot().await {
                    Ok(text) => text,
                    Err(error) => personal_data::error_message(error),
                });
            });
        })
    };
    html! {
        <>
            <div
                class="space-y-3 rounded-xl border border-line p-4"
            >
                <h3
                    class="font-semibold"
                >
                    {"Portable backup"}
                </h3>
                <p
                    class="text-sm text-muted"
                >
                    {"Downloads a JSON backup of Word Sets, completed Pinyin lessons and excluded words. It does not include exercise histories or writing attempts, which Zipeek does not yet persist."}
                </p>
                <div
                    class="flex flex-wrap gap-2"
                >
                    <button
                        type="button"
                        data-personal-export=""
                        class="rounded-lg bg-brand-action px-4 py-3 font-semibold text-white"
                        onclick={export}
                    >
                    {"Download backup"}
                    </button><button
                        type="button"
                        data-personal-opfs-snapshot=""
                        class="rounded-lg border border-line px-4 py-3"
                        onclick={save_snapshot}
                    >
                        {"Save private browser snapshot"}
                    </button><button
                        type="button"
                        data-personal-load-snapshot=""
                        class="rounded-lg border border-line px-4 py-3"
                        onclick={load_snapshot}
                    >
                        {"Load private snapshot"}
                    </button>
                    </div>
                <p
                    class="text-xs text-muted"
                >
                    {"A private snapshot stays on this browser and may be erased when site data is cleared. Download backups for protection against losing the device."}
                </p>
            </div>
            <div
                class="space-y-3 rounded-xl border border-line p-4"
            >
                <h3
                    class="font-semibold"
                >
                    {"Restore backup"}
                </h3>
                <p
                    class="text-sm text-muted"
                >
                    {"Select a JSON backup file or paste its contents below. Preview it first. Restore replaces only the categories present in the backup and keeps unrelated data unchanged. Word Set IDs refer to the dictionary build used when they were saved; review mappings after replacing the dictionary."}
                </p>
                <button
                    type="button"
                    data-personal-select-backup-file=""
                    class="rounded-lg border border-line px-4 py-2"
                    onclick={choose_backup_file}
                >
                    {"Select JSON backup file"}
                </button>
                <textarea
                    data-personal-import-text=""
                    value={(*backup_text).clone()}
                    oninput={on_backup_text}
                    class="min-h-28 w-full rounded-lg border border-line bg-raised p-3 font-mono text-xs"
                    placeholder="Paste personal backup JSON…"
                />
                <button
                    type="button"
                    data-personal-preview=""
                    class="rounded-lg border border-line px-4 py-2"
                    onclick={preview}
                >
                    {"Preview backup"}
                </button>
                <label
                    class="block text-sm"
                    for="personal-import-confirm"
                >
                    {"Type REPLACE to enable restore"}
                </label>
                <textarea
                    id="personal-import-confirm"
                    data-personal-import-confirm=""
                    rows="1"
                    value={(*import_confirmation).clone()}
                    oninput={on_import_confirm}
                    class="w-full rounded-lg border border-line bg-raised p-3"
                />
                <button
                    type="button"
                    data-personal-import=""
                    disabled={*import_confirmation != "REPLACE" || backup_text.trim().is_empty()}
                    class="rounded-lg border border-line px-4 py-3 disabled:opacity-50"
                    onclick={import}
                >
                    {"Replace categories from backup"}
                </button>
            </div>
        </>
    }
}
