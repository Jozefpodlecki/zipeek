use super::resolve::SelectedItem;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct SelectedItemRowProps {
    pub item: SelectedItem,
    pub on_remove: Callback<SelectedItem>,
}

#[function_component(SelectedItemRow)]
pub(super) fn selected_item_row(props: &SelectedItemRowProps) -> Html {
    let on_remove = props.on_remove.clone();
    let item = props.item.clone();
    let remove = Callback::from(move |_: MouseEvent| on_remove.emit(item.clone()));
    html! {
        <li class="flex min-w-0 items-center gap-3 border-b border-line px-3 py-2 last:border-0">
            <span class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-raised text-accent">
                <Icon data={if props.item.grammar { IconData::LUCIDE_BOOK_OPEN } else { IconData::LUCIDE_BOOK_OPEN }} width="17px" />
            </span>
            <span class="min-w-0 flex-1">
                <span class="block truncate text-sm font-semibold" lang={if props.item.grammar { "en" } else { "zh-Hans" }}>{props.item.title.clone()}</span>
                <span class="block truncate text-xs text-muted">{props.item.detail.clone()}</span>
            </span>
            <button type="button" aria-label={format!("Remove {} from selection", props.item.title)} title="Remove" onclick={remove}
                class="inline-flex size-9 shrink-0 items-center justify-center rounded-lg text-muted hover:bg-raised hover:text-ink focus-visible:outline-2 focus-visible:outline-accent">
                <Icon data={IconData::LUCIDE_X} width="16px" />
            </button>
        </li>
    }
}
