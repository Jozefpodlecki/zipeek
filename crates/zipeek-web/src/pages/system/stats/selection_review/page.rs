use super::super::{
    context::use_stats_ui,
    dialog_focus::{trap_tab, use_dialog_initial_focus},
    state::DashboardAction,
};
use super::{
    resolve::{SelectedItem, resolve_selection},
    row::SelectedItemRow,
};
use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(SelectionReview)]
pub(crate) fn selection_review() -> Html {
    let dashboard = use_stats_ui().0;
    let close_ref = use_node_ref();
    let done_ref = use_node_ref();
    use_dialog_initial_focus(close_ref.clone());
    let close = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::ReviewSelection(false))
        })
    };
    let escape = {
        let dashboard = dashboard.clone();
        let first = close_ref.clone();
        let last = done_ref.clone();
        Callback::from(move |event: KeyboardEvent| {
            trap_tab(&event, &first, &last);
            if event.key() == "Escape" {
                event.prevent_default();
                dashboard.dispatch(DashboardAction::ReviewSelection(false));
            }
        })
    };
    let remove = {
        let dashboard = dashboard.clone();
        Callback::from(move |item: SelectedItem| {
            dashboard.dispatch(if item.grammar {
                DashboardAction::TogglePattern(item.id)
            } else {
                DashboardAction::ToggleWord(item.id)
            })
        })
    };
    let clear = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ClearSelection))
    };
    let (items, unresolved) =
        resolve_selection(&dashboard.selected_words, &dashboard.selected_patterns);
    let selected_count = dashboard.selected_words.len() + dashboard.selected_patterns.len();
    html! {
        <div class="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-6" data-stats-selection-review="">
            <button type="button" tabindex="-1" aria-label="Dismiss selection review" onclick={close.clone()}
                class="absolute inset-0 h-full w-full bg-black/70" />
            <section role="dialog" aria-modal="true" aria-labelledby="stats-selection-heading" aria-describedby="stats-selection-help"
                onkeydown={escape} class="relative flex max-h-[min(85vh,42rem)] w-full max-w-xl flex-col overflow-hidden rounded-xl border border-line bg-surface shadow-2xl">
                <header class="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
                    <div class="min-w-0">
                        <h2 id="stats-selection-heading" class="text-base font-semibold">{"Selected learning material"}</h2>
                        <p class="text-xs tabular-nums text-muted">{format!("{} items · {} vocabulary · {} grammar", selected_count, dashboard.selected_words.len(), dashboard.selected_patterns.len())}</p>
                    </div>
                    <button type="button" ref={close_ref} aria-label="Close selection review" onclick={close.clone()}
                        class="inline-flex size-9 shrink-0 items-center justify-center rounded-lg border border-line hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_X} width="17px" />
                    </button>
                </header>
                <ul class="min-h-0 flex-1 overflow-y-auto" aria-label="Selected entries">
                    {for items.into_iter().map(|item| {
                        let key = item.id.clone();
                        html! { <SelectedItemRow key={key} item={item} on_remove={remove.clone()} /> }
                    })}
                    if unresolved > 0 {
                        <li class="p-3 text-xs text-muted">{format!("{} selected items could not be resolved in the preview catalogue.", unresolved)}</li>
                    }
                    if selected_count == 0 {
                        <li class="p-5 text-center text-sm text-muted">{"No items selected"}</li>
                    }
                </ul>
                <footer class="space-y-3 border-t border-line p-4">
                    <p id="stats-selection-help" class="text-xs text-muted">{"Preview items are not verified Practice or Word Set identities. Transfer is unavailable until their catalogue mapping is validated."}</p>
                    <div class="flex flex-wrap items-center justify-between gap-2">
                        <button type="button" onclick={clear} disabled={selected_count == 0}
                            class="min-h-10 rounded-lg border border-line px-3 text-sm hover:bg-raised disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent">{"Clear all"}</button>
                        <button type="button" ref={done_ref} onclick={close} class="min-h-10 rounded-lg bg-brand-action px-4 text-sm font-semibold text-white focus-visible:outline-2 focus-visible:outline-accent">{"Done"}</button>
                    </div>
                </footer>
            </section>
        </div>
    }
}
