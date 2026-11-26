use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct StatsPaginationProps {
    pub subject: AttrValue,
    pub start: usize,
    pub end: usize,
    pub total: usize,
    pub page: usize,
    pub pages: usize,
    pub disabled: bool,
    pub on_previous: Callback<MouseEvent>,
    pub on_next: Callback<MouseEvent>,
}

#[function_component(StatsPagination)]
pub(super) fn stats_pagination(props: &StatsPaginationProps) -> Html {
    html! {
        <div class="flex items-center justify-between gap-3 text-xs text-muted">
            <span class="tabular-nums">{format!("{}-{} of {} {}", props.start, props.end, props.total, props.subject)}</span>
            <div class="flex items-center gap-2">
                <button type="button" aria-label={format!("Previous {} page", props.subject)}
                    disabled={props.disabled || props.page == 0} onclick={props.on_previous.clone()}
                    class="rounded-md border border-line p-2 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px" />
                </button>
                <span class="tabular-nums">{format!("{} / {}", props.page + 1, props.pages)}</span>
                <button type="button" aria-label={format!("Next {} page", props.subject)}
                    disabled={props.disabled || props.page + 1 >= props.pages} onclick={props.on_next.clone()}
                    class="rounded-md border border-line p-2 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px" />
                </button>
            </div>
        </div>
    }
}
