use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{
    context::{use_stats_ui, use_loaded_section},
    data::{SectionPayload, SessionRecord},
    fixture::SAMPLE_SESSIONS,
    state::{DashboardAction, HistoryCategory},
};

const PAGE_SIZE: usize = 8;
const RECENT_DAYS: [&str; 7] = [
    "2026-11-12", "2026-11-13", "2026-11-14", "2026-11-15",
    "2026-11-16", "2026-11-17", "2026-11-18",
];

fn recent_activity_counts(sessions: &[SessionRecord]) -> [usize; 7] {
    let mut counts = [0; 7];
    for session in sessions {
        if let Some(index) = RECENT_DAYS.iter().position(|date| *date == session.date.as_str()) {
            counts[index] += 1;
        }
    }
    counts
}

fn filtered_sessions<'a>(sessions: &'a [SessionRecord], query: &str, category: HistoryCategory) -> Vec<&'a SessionRecord> {
    let query = query.trim().to_lowercase();
    sessions
        .iter()
        .rev()
        .filter(|session| {
            category.matches(&session.category)
                && (query.is_empty()
                    || session.activity_title.to_lowercase().contains(&query)
                    || session.activity_id.contains(&query)
                    || session.focus.contains(&query)
                    || session.date.contains(&query))
        })
        .collect()
}

#[derive(Properties, PartialEq)]
struct HistoryEntryProps {
    session: SessionRecord,
}

#[function_component(HistoryEntry)]
fn history_entry(props: &HistoryEntryProps) -> Html {
    let expanded = use_state(|| false);
    let session = &props.session;
    let on_toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_: MouseEvent| expanded.set(!*expanded))
    };
    let outcome = session.correct.map_or_else(
        || format!("{} attempts", session.observed),
        |correct| format!("{correct} / {}", session.observed),
    );
    let icon = if session.category == "Games" { IconData::LUCIDE_GAMEPAD_2 } else { IconData::LUCIDE_BOOK_OPEN };
    html! {
        <li class="border-b border-line last:border-b-0" data-stats-history-activity={session.activity_id.clone()}>
            <button type="button" aria-expanded={expanded.to_string()} onclick={on_toggle}
                class="flex min-h-14 w-full items-center gap-3 px-3 py-2 text-left transition-colors hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent sm:px-4">
                <span class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-raised text-accent">
                    <Icon data={icon} width="18px" height="18px" />
                </span>
                <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm font-semibold">{session.activity_title.clone()}</span>
                    <span class="block truncate text-xs text-muted">{session.date.clone()}{" · "}{session.focus.clone()}</span>
                </span>
                <span class="hidden rounded-md border border-line px-2 py-1 text-xs text-muted sm:inline-flex">{session.category.clone()}</span>
                <span class="shrink-0 text-right text-xs font-medium tabular-nums">{outcome}</span>
                <Icon data={if *expanded {IconData::LUCIDE_CHEVRON_DOWN} else {IconData::LUCIDE_CHEVRON_RIGHT}} width="16px" height="16px" />
            </button>
            if *expanded {
                <div class="grid gap-3 border-t border-line bg-raised/60 px-4 py-3 text-sm sm:grid-cols-3">
                    <div><span class="block text-xs text-muted">{"Exercise"}</span><span class="font-medium">{session.activity_title.clone()}</span></div>
                    <div><span class="block text-xs text-muted">{"Attempts"}</span><span class="font-medium tabular-nums">{session.observed}</span></div>
                    <div><span class="block text-xs text-muted">{"Outcome"}</span>
                        <span class="font-medium">{session.correct.map_or("Unscored practice".to_owned(), |correct| format!("{correct} correct"))}</span>
                    </div>
                </div>
            }
        </li>
    }
}

#[function_component(HistoryExplorer)]
pub(super) fn history_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::History(sessions) = loaded.as_ref() else { unreachable!("History projection expected") };
    let matches = filtered_sessions(sessions, &dashboard.history_query, dashboard.history_category);
    let pages = matches.len().div_ceil(PAGE_SIZE).max(1);
    let page = dashboard.history_page.min(pages - 1);
    let start = page * PAGE_SIZE;
    let visible = matches.iter().skip(start).take(PAGE_SIZE);
    let previous = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::HistoryPage(page.saturating_sub(1))))
    };
    let next = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::HistoryPage((page + 1).min(pages - 1))))
    };
    let on_search = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            dashboard.dispatch(DashboardAction::HistoryQuery(input.value()));
        })
    };
    let recent = recent_activity_counts(sessions);
    let busiest = (*recent.iter().max().unwrap_or(&1)).max(1);
    html! {
        <section data-stats-history="" class="space-y-3">
            <div class="flex flex-wrap items-end justify-between gap-3">
                <label class="min-w-48 flex-1 space-y-1 text-xs text-muted">
                    <span>{"Find an activity"}</span>
                    <input type="search" value={dashboard.history_query.clone()} oninput={on_search}
                        placeholder="Exercise, date or Hanzi" aria-label="Search history"
                        class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent" />
                </label>
                <div role="group" aria-label="History category" class="flex flex-wrap gap-1 rounded-lg border border-line bg-surface p-1">
                    {for HistoryCategory::ALL.map(|category| {
                        let dashboard = dashboard.clone();
                        let active = dashboard.history_category == category;
                        let onclick = Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::HistoryCategory(category)));
                        html! {
                            <button key={category.label()} type="button" aria-pressed={active.to_string()} onclick={onclick}
                                class={classes!("min-h-9", "rounded-md", "px-3", "text-sm", "font-semibold", "focus-visible:outline-2", "focus-visible:outline-accent", if active { "bg-brand-action text-white" } else { "text-ink hover:bg-raised" })}>
                                {category.label()}
                            </button>
                        }
                    })}
                </div>
            </div>
            <div class="grid grid-cols-7 gap-2" aria-label="Sample session counts over seven days">
                {for recent.into_iter().enumerate().map(|(index, count)| {
                    let day = RECENT_DAYS[index];
                    let height = count * 100 / busiest;
                    html! {
                        <div key={day} class="flex flex-col items-center gap-1 rounded-lg border border-line bg-surface p-2"
                            aria-label={format!("{}: {} demo sessions", day, count)}>
                            <span class="text-xs text-muted">{&day[8..]}</span>
                            <div class="flex h-9 w-full items-end rounded bg-raised p-1" aria-hidden="true">
                                <div class="w-full rounded-sm bg-accent" style={format!("height: {}%", height)}/>
                            </div>
                        </div>
                    }
                })}
            </div>
            <div class="overflow-hidden rounded-xl border border-line bg-surface">
                <ol class="min-h-[29rem]" aria-label="Sample session history">
                    {for visible.map(|session| html! { <HistoryEntry key={session.id.clone()} session={(**session).clone()} /> })}
                    if matches.is_empty() {
                        <li role="status" class="p-6 text-center text-sm text-muted">{"No matching activities"}</li>
                    }
                </ol>
            </div>
            <div class="flex items-center justify-between gap-3 text-xs text-muted">
                <span class="tabular-nums">{format!("{}-{} of {} sessions", if matches.is_empty() {0} else {start+1}, (start+PAGE_SIZE).min(matches.len()), matches.len())}</span>
                <div class="flex items-center gap-2">
                    <button type="button" aria-label="Previous history page" disabled={page==0} onclick={previous} class="rounded-md border border-line p-2 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent"><Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px" /></button>
                    <span class="tabular-nums">{format!("{} / {}", page+1, pages)}</span>
                    <button type="button" aria-label="Next history page" disabled={page+1==pages} onclick={next} class="rounded-md border border-line p-2 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent"><Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px" /></button>
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_preserve_all_practice_and_game_categories() {
        let sessions = super::super::data::demo_sessions_for_tests();
        assert_eq!(filtered_sessions(&sessions, "", HistoryCategory::All).len(), SAMPLE_SESSIONS.len());
        assert!(filtered_sessions(&sessions, "", HistoryCategory::Practice).iter().all(|item| item.category == "Practice"));
        assert!(filtered_sessions(&sessions, "", HistoryCategory::Games).iter().all(|item| item.category == "Games"));
        assert!(filtered_sessions(&sessions, "pinyin", HistoryCategory::Practice).iter().any(|item| item.activity_id == "type-pinyin"));
        assert!(filtered_sessions(&sessions, "none-available", HistoryCategory::All).is_empty());
    }

    #[test]
    fn recent_activity_is_a_count_not_an_unqualified_success_rate() {
        let sessions = super::super::data::demo_sessions_for_tests();
        let counts = recent_activity_counts(&sessions);
        let expected = SAMPLE_SESSIONS.iter().filter(|item| RECENT_DAYS.contains(&item.date)).count();
        assert_eq!(counts.iter().sum::<usize>(), expected);
        assert!(counts.iter().all(|count| *count > 0));
    }
}
