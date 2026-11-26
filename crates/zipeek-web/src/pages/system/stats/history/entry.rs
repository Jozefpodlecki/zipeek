use super::{super::data::SessionRecord, display_timestamp};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct HistoryEntryProps {
    pub session: SessionRecord,
}

#[function_component(HistoryEntry)]
pub(super) fn history_entry(props: &HistoryEntryProps) -> Html {
    let expanded = use_state(|| false);
    let session = &props.session;
    let on_toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_: MouseEvent| expanded.set(!*expanded))
    };
    let result = session.correct.map(|correct| {
        let ratio = if session.observed == 0 {
            0
        } else {
            correct.saturating_mul(100) / session.observed
        };
        (correct, ratio)
    });
    let icon = if session.category == "Games" {
        IconData::LUCIDE_GAMEPAD_2
    } else {
        IconData::LUCIDE_BOOK_OPEN
    };
    html! {
        <li class="border-b border-line last:border-b-0" data-stats-history-activity={session.activity_id.clone()}>
            <button type="button" aria-expanded={expanded.to_string()} onclick={on_toggle}
                class="flex min-h-12 w-full items-center gap-2 px-3 py-1.5 text-left transition-colors hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent sm:px-4">
                <span class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-raised text-accent">
                    <Icon data={icon} width="18px" height="18px" />
                </span>
                <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm font-semibold">{session.activity_title.clone()}</span>
                    <span class="block truncate text-xs text-muted">{display_timestamp(&session.started_at)}{" · "}{session.duration_minutes}{" min · "}{session.focus.clone()}</span>
                </span>
                <span class="hidden shrink-0 text-xs text-muted md:inline">{if session.category == "Games" { "Game" } else { "Practice" }}</span>
                <span class="w-20 shrink-0 text-right sm:w-24">
                    if let Some((correct, ratio)) = result {
                        <span class="block text-xs font-medium tabular-nums" aria-label={format!("{} correct out of {} attempts", correct, session.observed)}>{format!("{correct}/{}", session.observed)}</span>
                        <span class="mt-1 block h-1.5 overflow-hidden rounded-full bg-raised" aria-hidden="true"><span class="block h-full rounded-full bg-accent" style={format!("width: {ratio}%")} /></span>
                    } else {
                        <span class="text-xs text-muted">{"Unscored"}</span>
                    }
                </span>
                <Icon data={if *expanded {IconData::LUCIDE_CHEVRON_DOWN} else {IconData::LUCIDE_CHEVRON_RIGHT}} width="16px" height="16px" />
            </button>
            if *expanded {
                <div class="grid gap-3 border-t border-line bg-raised/60 px-4 py-3 text-sm sm:grid-cols-3">
                    <div><span class="block text-xs text-muted">{"Exercise"}</span><span class="font-medium">{session.activity_title.clone()}</span></div>
                    <div><span class="block text-xs text-muted">{"Duration and attempts"}</span><span class="font-medium tabular-nums">{format!("{} min · {} attempts", session.duration_minutes, session.observed)}</span></div>
                    <div><span class="block text-xs text-muted">{"Outcome"}</span>
                        <span class="font-medium">{session.correct.map_or("Unscored practice".to_owned(), |correct| format!("{correct} correct"))}</span>
                    </div>
                </div>
            }
        </li>
    }
}
