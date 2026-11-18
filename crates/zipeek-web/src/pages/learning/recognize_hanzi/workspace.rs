use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::state::InputKind;

#[derive(Properties, PartialEq)]
pub(super) struct WorkspacePanelProps {
    pub canvas: NodeRef,
    pub file_input: NodeRef,
    pub input: InputKind,
    pub can_undo: bool,
    pub busy: bool,
    pub on_undo: Callback<MouseEvent>,
    pub on_clear: Callback<MouseEvent>,
    pub on_upload: Callback<MouseEvent>,
    pub children: Children,
}

#[function_component(WorkspacePanel)]
pub(super) fn workspace_panel(props: &WorkspacePanelProps) -> Html {
    let has_input = props.input.has_input();
    html! {
        <section class="min-w-0 overflow-hidden rounded-2xl border border-line bg-surface shadow-sm" aria-label="Hanzi recognition workspace">
            <div class="flex items-center justify-between gap-2 px-3 py-2 sm:px-4">
                <div class="flex items-center gap-2">
                    <input ref={props.file_input.clone()} type="file" accept="image/png,image/jpeg,image/webp" class="sr-only" aria-label="Choose an image of Chinese writing" />
                    <button type="button" onclick={props.on_upload.clone()} title="Upload an image" aria-label="Upload an image" class="inline-flex min-h-11 items-center gap-2 rounded-lg border border-line bg-raised px-3 text-sm font-medium text-ink hover:bg-surface focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">
                        <Icon data={IconData::LUCIDE_IMAGE} width="18px" height="18px" />
                        <span>{"Upload"}</span>
                    </button>
                </div>
                <div class="flex items-center gap-2">
                    <button type="button" title="Undo last stroke" aria-label="Undo last stroke" onclick={props.on_undo.clone()} disabled={!props.can_undo || props.busy} class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-raised text-ink hover:bg-surface focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action disabled:cursor-not-allowed disabled:opacity-40">
                        <Icon data={IconData::LUCIDE_UNDO_2} width="19px" height="19px" />
                    </button>
                    <button type="button" title="Clear drawing or image" aria-label="Clear drawing or image" onclick={props.on_clear.clone()} disabled={!has_input} class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-raised text-ink hover:bg-surface focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action disabled:cursor-not-allowed disabled:opacity-40">
                        <Icon data={IconData::LUCIDE_TRASH_2} width="19px" height="19px" />
                    </button>
                </div>
            </div>
            <div class="relative mx-3 mb-3 overflow-hidden rounded-xl border border-line bg-raised text-ink sm:mx-4 sm:mb-4">
                <canvas ref={props.canvas.clone()} tabindex="0" width="768" height="320" class="block h-auto w-full touch-none bg-raised text-ink focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand-action" aria-label="Draw Chinese characters or paste an image with Control V" data-recognition-canvas="" />
                if !has_input {
                    <span aria-hidden="true" class="pointer-events-none absolute inset-0 flex items-center justify-center text-muted/40">
                        <Icon data={IconData::LUCIDE_PENCIL} width="32px" height="32px" />
                    </span>
                }
            </div>
            <div class="flex flex-col items-stretch gap-3 border-t border-line px-3 py-3 sm:flex-row sm:items-start sm:justify-end sm:px-4">
                {for props.children.iter()}
            </div>
        </section>
    }
}
