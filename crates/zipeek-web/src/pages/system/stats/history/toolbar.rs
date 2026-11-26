use super::super::{
    query::is_week_date,
    state::{HistoryCategory, HistoryView},
};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct HistoryToolbarProps {
    pub query: String,
    pub category: HistoryCategory,
    pub view: HistoryView,
    pub on_search: Callback<InputEvent>,
    pub on_category: Callback<HistoryCategory>,
    pub on_view: Callback<HistoryView>,
    pub on_clear_date: Callback<MouseEvent>,
}

#[function_component(HistoryToolbar)]
pub(super) fn history_toolbar(props: &HistoryToolbarProps) -> Html {
    let is_table = props.view == HistoryView::Table;
    let timeline = {
        let on_view = props.on_view.clone();
        Callback::from(move |_: MouseEvent| on_view.emit(HistoryView::Timeline))
    };
    let table = {
        let on_view = props.on_view.clone();
        Callback::from(move |_: MouseEvent| on_view.emit(HistoryView::Table))
    };
    html! {
            <div class="flex flex-wrap items-center gap-2">
                <div class="relative min-w-48 flex-1 sm:max-w-md">
                    <input type="search" value={props.query.clone()} oninput={props.on_search.clone()}
                        placeholder="Search activities" aria-label="Search history"
                        class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent" />
                </div>
                <div role="group" aria-label="History category" class="flex flex-wrap gap-1 rounded-lg border border-line bg-surface p-1">
                    {for HistoryCategory::ALL.map(|category| {
                        let on_category = props.on_category.clone();
                        let active = props.category == category;
                        let onclick = Callback::from(move |_: MouseEvent| on_category.emit(category));
                        html! {
                            <button key={category.label()} type="button" aria-pressed={active.to_string()} onclick={onclick}
                                title={category.label()}
                                aria-label={format!("Show {} activities", category.label())}
                                class={classes!("inline-flex", "size-9", "items-center", "justify-center", "rounded-md", "focus-visible:outline-2", "focus-visible:outline-accent", if active { "bg-brand-action text-white" } else { "text-ink hover:bg-raised" })}>
                                <Icon data={match category { HistoryCategory::All => IconData::LUCIDE_LAYERS, HistoryCategory::Practice => IconData::LUCIDE_BOOK_OPEN, HistoryCategory::Games => IconData::LUCIDE_GAMEPAD_2 }} width="17px" />
                            </button>
                        }
                    })}
                </div>
                <div role="group" aria-label="History display" class="inline-flex rounded-lg border border-line bg-surface p-1">
                    <button type="button" title="Timeline" aria-label="Timeline" aria-pressed={(!is_table).to_string()} onclick={timeline}
                        class={classes!("inline-flex", "size-9", "items-center", "justify-center", "rounded-md", "focus-visible:outline-2", "focus-visible:outline-accent", if !is_table { "bg-brand-action text-white" } else { "hover:bg-raised" })}><Icon data={IconData::LUCIDE_LIST} width="17px" /></button>
                    <button type="button" title="Table" aria-label="Table" aria-pressed={is_table.to_string()} onclick={table}
                        class={classes!("inline-flex", "size-9", "items-center", "justify-center", "rounded-md", "focus-visible:outline-2", "focus-visible:outline-accent", if is_table { "bg-brand-action text-white" } else { "hover:bg-raised" })}><Icon data={IconData::LUCIDE_TABLE_2} width="17px" /></button>
                </div>
                if is_week_date(&props.query) {
                    <button type="button" onclick={props.on_clear_date.clone()} class="min-h-9 rounded-lg border border-line px-3 text-xs font-medium hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                        {format!("Showing {} · Clear date", props.query)}
                    </button>
                }
            </div>
    }
}
