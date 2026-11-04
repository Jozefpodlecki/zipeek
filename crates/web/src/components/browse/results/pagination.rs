use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct BrowsePaginationProps {
    pub offset: u32,
    pub has_next: bool,
    pub on_previous: Callback<MouseEvent>,
    pub on_next: Callback<MouseEvent>,
}

#[function_component(BrowsePagination)]
pub(super) fn browse_pagination(props: &BrowsePaginationProps) -> Html {
    html! {
        <nav data-component="browse-pagination" class="flex items-center justify-end gap-3" aria-label="Browse pages">
            <button
                type="button"
                data-browse-previous=""
                class="grid size-10 place-items-center rounded-lg border border-line disabled:opacity-50 hover:border-accent"
                disabled={props.offset == 0}
                aria-label="Previous page"
                title="Previous page"
                onclick={props.on_previous.clone()}
            >
                <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="20px" height="20px" />
            </button>
            <span data-browse-page="" class="text-sm text-muted" aria-live="polite">{format!("Page {}", props.offset / crate::reducers::browse::PAGE_SIZE + 1)}</span>
            <button
                type="button"
                data-browse-next=""
                class="grid size-10 place-items-center rounded-lg border border-line disabled:opacity-50 hover:border-accent"
                disabled={!props.has_next}
                aria-label="Next page"
                title="Next page"
                onclick={props.on_next.clone()}
            >
                <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="20px" height="20px" />
            </button>
        </nav>
    }
}
