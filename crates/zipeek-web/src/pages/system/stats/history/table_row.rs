use super::{super::data::SessionRecord, display_timestamp};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct HistoryTableRowProps {
    pub session: SessionRecord,
}

#[function_component(HistoryTableRow)]
pub(super) fn history_table_row(props: &HistoryTableRowProps) -> Html {
    let session = &props.session;
    html! {
        <tr class="border-t border-line hover:bg-raised/60">
            <td class="whitespace-nowrap px-3 py-2 text-xs tabular-nums">{display_timestamp(&session.started_at)}</td>
            <td class="px-3 py-2 font-medium">{session.activity_title.clone()}</td>
            <td class="px-3 py-2 text-muted">{session.category.clone()}</td>
            <td class="px-3 py-2">{session.focus.clone()}</td>
            <td class="px-3 py-2 text-right tabular-nums">{format!("{} min", session.duration_minutes)}</td>
            <td class="px-3 py-2 text-right tabular-nums">{session.observed}</td>
            <td class="px-3 py-2 text-right tabular-nums">{session.correct.map_or_else(|| "Not scored".to_owned(), |correct| format!("{correct} correct"))}</td>
        </tr>
    }
}
