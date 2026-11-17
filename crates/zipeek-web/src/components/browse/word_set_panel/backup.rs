use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct WordSetBackupProps {
    pub disabled: bool,
    pub backup_text: String,
    pub on_export: Callback<MouseEvent>,
    pub on_import: Callback<MouseEvent>,
    pub on_backup_text: Callback<InputEvent>,
}

#[function_component(WordSetBackup)]
pub(super) fn word_set_backup(props: &WordSetBackupProps) -> Html {
    html! {
            <details
                data-component="word-set-backup"
                class="rounded-lg border border-line p-3"
                data-word-set-backup=""
            >
                <summary
                    class="cursor-pointer font-semibold"
                >
                    {"Back up or restore personal Word Sets"}
                </summary>
                <p
                    class="mt-2 text-sm text-muted"
                >
                    {"Export as JSON and copy it somewhere safe. Paste a backup below to merge sets with matching names. Word IDs are specific to the dictionary build; check restored words after a database update."}
                </p>
                <div
                    class="mt-3 flex flex-wrap gap-2"
                >
                    <button
                        type="button"
                        data-export-word-sets=""
                        class="min-h-11 rounded-lg border border-line px-3"
                        disabled={props.disabled}
                        onclick={props.on_export.clone()}
                    >
                        {"Generate backup"}
                    </button>
                    <button
                        type="button"
                        data-import-word-sets=""
                        class="min-h-11 rounded-lg border border-line px-3"
                        disabled={props.disabled || props.backup_text.trim().is_empty()}
                        onclick={props.on_import.clone()}
                    >
                        {"Merge pasted backup"}
                    </button>
                </div>
                <label
                    class="mt-3 flex flex-col gap-1 text-sm"
                >
                    {"Backup JSON"}
                    <textarea
                        data-word-set-backup-json=""
                        class="min-h-36 rounded-lg border border-line bg-surface p-3 font-mono text-xs"
                        value={props.backup_text.clone()}
                        oninput={props.on_backup_text.clone()}
                        spellcheck="false"
                    />
                </label>
            </details>
    }
}
