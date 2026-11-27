use super::super::data::GrammarRecord;
use super::super::dialog_focus::{trap_tab, use_dialog_initial_focus};
use crate::hooks::use_app_environment;
use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct GrammarDetailProps {
    pub record: GrammarRecord,
    pub on_close: Callback<()>,
}

#[function_component(GrammarDetail)]
pub(super) fn grammar_detail(props: &GrammarDetailProps) -> Html {
    let record = &props.record;
    let environment = use_app_environment();
    let close_ref = use_node_ref();
    use_dialog_initial_focus(close_ref.clone());
    let close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };
    let onkeydown = {
        let first = close_ref.clone();
        let on_close = props.on_close.clone();
        let document = environment.document.clone();
        Callback::from(move |event: KeyboardEvent| {
            trap_tab(&document, &event, &first, &first);
            if event.key() == "Escape" {
                event.prevent_default();
                on_close.emit(());
            }
        })
    };
    html! {
        <div class="fixed inset-0 z-50 flex justify-end bg-black/60" data-stats-grammar-inspector="">
            <aside role="dialog" aria-modal="true" aria-labelledby="grammar-inspector-heading" onkeydown={onkeydown}
                class="flex h-full w-full max-w-xl flex-col overflow-y-auto border-l border-line bg-surface p-5 shadow-2xl sm:p-6">
                <header class="flex items-center justify-between gap-3 border-b border-line pb-4">
                    <div class="min-w-0">
                        <p class="text-xs text-muted">{"Grammar pattern"}</p>
                        <h2 id="grammar-inspector-heading" class="mt-1 break-words text-xl font-semibold">{record.pattern.clone()}</h2>
                    </div>
                    <button type="button" aria-label="Close grammar details" title="Close details" ref={close_ref} onclick={close}
                        class="inline-flex size-10 shrink-0 items-center justify-center rounded-lg border border-line text-ink hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_X} width="18px" />
                    </button>
                </header>
                <div class="mt-5 space-y-5 text-sm">
                    if record.description != "Grammar pattern" {
                        <section><h3 class="text-xs font-semibold uppercase tracking-wide text-muted">{"Meaning"}</h3>
                            <p class="mt-2">{record.description.clone()}</p></section>
                    }
                    <section>
                        <h3 class="text-xs font-semibold uppercase tracking-wide text-muted">{"Example"}</h3>
                        <p lang="zh-Hans" class="mt-2 rounded-lg border border-line bg-raised p-4 text-lg leading-relaxed">{record.example.clone()}</p>
                    </section>
                    <section class="grid grid-cols-2 gap-3 rounded-lg border border-line p-4">
                        <div><p class="text-xs text-muted">{"Teaching group"}</p><p class="mt-1 font-semibold">{record.course.clone()}</p></div>
                        <div><p class="text-xs text-muted">{"Practice attempts"}</p><p class="mt-1 font-semibold tabular-nums">{record.attempts}</p></div>
                        <div><p class="text-xs text-muted">{"Teaching reference"}</p><p class="mt-1 font-medium">{record.id.clone()}</p></div>
                        <div><p class="text-xs text-muted">{"Editorial status"}</p>
                            <p class="mt-1 font-medium">{if record.teaching_review_status == "pending_linguistic_review" { "Pending review" } else { record.teaching_review_status.as_str() }}</p></div>
                        if let Some(attempts) = record.assessed_attempts {
                            <div><p class="text-xs text-muted">{"Assessed attempts"}</p><p class="mt-1 font-medium">{attempts}</p></div>
                        }
                    </section>
                </div>
            </aside>
        </div>
    }
}
