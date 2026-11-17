use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct WordSetManagerProps {
    pub renamed_set: String,
    pub confirm_delete: bool,
    pub on_renamed_set: Callback<InputEvent>,
    pub on_rename: Callback<MouseEvent>,
    pub on_delete: Callback<MouseEvent>,
}

#[function_component(WordSetManager)]
pub(super) fn word_set_manager(props: &WordSetManagerProps) -> Html {
    html! {
        <div data-component="word-set-manager" data-manage-word-set="" class="space-y-2 border-t border-line pt-3">
            <div class="flex items-center justify-between gap-2">
                <details class="min-w-0 flex-1" data-word-set-rename-tools="">
                    <summary class="inline-flex min-h-10 cursor-pointer items-center gap-2 rounded-lg border border-line px-3 text-sm font-semibold hover:bg-raised" title="Rename selected set">
                        <Icon data={IconData::LUCIDE_PENCIL} width="16px" height="16px" />
                        {"Rename"}
                    </summary>
                    <div class="mt-2 space-y-2">
                        <label class="block text-xs text-muted" for="rename-word-set-name">{"New set name"}</label>
                        <input id="rename-word-set-name" data-rename-word-set="" class="min-h-11 w-full rounded-lg border border-line bg-surface px-3 text-sm" maxlength="64" value={props.renamed_set.clone()} oninput={props.on_renamed_set.clone()} placeholder="New name" />
                        <button type="button" data-confirm-rename-word-set="" disabled={props.renamed_set.trim().is_empty()} class="min-h-10 rounded-lg bg-brand-action px-3 text-sm font-semibold text-white disabled:opacity-50" onclick={props.on_rename.clone()}> {"Save name"} </button>
                    </div>
                </details>
                <button type="button" data-delete-word-set="" class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line px-3 text-sm text-red-400 hover:bg-red-500/10" aria-label={if props.confirm_delete { "Confirm deleting selected Word Set" } else { "Delete selected Word Set" }} title="Delete selected set" onclick={props.on_delete.clone()}>
                    <Icon data={IconData::LUCIDE_TRASH_2} width="16px" height="16px" />
                    {if props.confirm_delete { "Confirm deletion" } else { "Delete" }}
                </button>
            </div>
            {if props.confirm_delete { html! { <p role="alert" class="text-xs text-red-400">{"Click Confirm deletion to permanently remove this set."}</p> } } else { Html::default() }}
        </div>
    }
}
