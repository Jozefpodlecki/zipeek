use super::super::state::GrammarView;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct GrammarViewControlProps {
    pub view: GrammarView,
    pub total: usize,
    pub on_view: Callback<GrammarView>,
}

#[function_component(GrammarViewControl)]
pub(super) fn grammar_view_control(props: &GrammarViewControlProps) -> Html {
    let show_table = {
        let on_view = props.on_view.clone();
        Callback::from(move |_: MouseEvent| on_view.emit(GrammarView::Table))
    };
    let show_cards = {
        let on_view = props.on_view.clone();
        Callback::from(move |_: MouseEvent| on_view.emit(GrammarView::Cards))
    };
    html! {
        <div class="flex items-center justify-end gap-2 text-xs text-muted">
            <span class="tabular-nums">{format!("{} patterns", props.total)}</span>
            <div class="inline-flex items-center rounded-lg border border-line bg-surface p-1" role="group" aria-label="Grammar layout">
                <button type="button" onclick={show_table} aria-label="Table view" title="Table view"
                    aria-pressed={(props.view == GrammarView::Table).to_string()}
                    class={classes!("inline-flex", "size-9", "items-center", "justify-center", "rounded-md", "focus-visible:outline-2", "focus-visible:outline-accent", if props.view == GrammarView::Table { "bg-brand-action text-white" } else { "hover:bg-raised" })}>
                    <Icon data={IconData::LUCIDE_LIST} width="17px" />
                </button>
                <button type="button" onclick={show_cards} aria-label="Card view" title="Card view"
                    aria-pressed={(props.view == GrammarView::Cards).to_string()}
                    class={classes!("inline-flex", "size-9", "items-center", "justify-center", "rounded-md", "focus-visible:outline-2", "focus-visible:outline-accent", if props.view == GrammarView::Cards { "bg-brand-action text-white" } else { "hover:bg-raised" })}>
                    <Icon data={IconData::LUCIDE_LAYOUT_GRID} width="17px" />
                </button>
            </div>
        </div>
    }
}
