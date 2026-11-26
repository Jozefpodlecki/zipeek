use super::super::dialog_focus::{trap_tab, use_dialog_initial_focus};
use super::activity::{ACTIVITY_WEEKS, WEEKS};
use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct ActivityBreakdownProps {
    pub categories: [(u32, u32); WEEKS],
    pub on_close: Callback<()>,
}

#[function_component(ActivityBreakdown)]
pub(super) fn activity_breakdown(props: &ActivityBreakdownProps) -> Html {
    let categories = props.categories;
    let close_ref = use_node_ref();
    use_dialog_initial_focus(close_ref.clone());
    let close_breakdown = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };
    let onkeydown = {
        let first = close_ref.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |event: KeyboardEvent| {
            trap_tab(&event, &first, &first);
            if event.key() == "Escape" {
                event.prevent_default();
                on_close.emit(());
            }
        })
    };
    html! {
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-4" data-stats-activity-details="">
                        <section role="dialog" aria-modal="true" aria-labelledby="stats-activity-details-heading" onkeydown={onkeydown} class="w-full max-w-lg rounded-xl border border-line bg-surface p-5 shadow-2xl">
                            <header class="mb-3 flex items-center justify-between">
                                <h3 id="stats-activity-details-heading" class="font-semibold">{"Weekly activity"}</h3>
                                <button type="button" aria-label="Close activity breakdown" ref={close_ref} onclick={close_breakdown} class="inline-flex size-10 items-center justify-center rounded-lg border border-line hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent"><Icon data={IconData::LUCIDE_X} width="17px" /></button>
                            </header>
                            <div class="max-h-[60vh] overflow-y-auto">
                                <table class="w-full text-left text-xs tabular-nums">
                                    <caption class="sr-only">{"Completed sessions by week and activity category"}</caption>
                                    <thead class="text-muted"><tr><th scope="col" class="py-2">{"Week of"}</th><th scope="col" class="py-2 text-right">{"Practice"}</th><th scope="col" class="py-2 text-right">{"Games"}</th><th scope="col" class="py-2 text-right">{"Total"}</th></tr></thead>
                                    <tbody>{for categories.iter().enumerate().map(|(index, (practice, games))| html! {
                                        <tr key={index} class="border-t border-line">
                                            <th scope="row" class="py-3 font-medium">{ACTIVITY_WEEKS[index]}</th>
                                            <td class="text-right">{practice}</td><td class="text-right">{games}</td>
                                            <td class="text-right font-semibold">{practice + games}</td>
                                        </tr>
                                    })}</tbody>
                                </table>
                            </div>
                        </section>
                    </div>
        }
}
