use super::toolbar::HistoryToolbar;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::super::{
    context::{StatsSourceContext, use_stats_ui},
    data::SessionRecord,
    query::{
        ActivityBucket, ActivityQuery, HistoryQuery, PageRequest, QueryFuture, activity_week_dates,
        is_week_date,
    },
    query_hook::use_query_page,
    state::{DashboardAction, HistoryCategory, HistoryView},
};

const PAGE_SIZE: usize = 8;

pub(crate) fn display_timestamp(timestamp: &str) -> String {
    let date = timestamp.get(0..10).unwrap_or(timestamp);
    let time = timestamp.get(11..16).unwrap_or("");
    format!("{date} · {time} UTC")
}

fn summary_search(query: &str) -> String {
    if is_week_date(query) {
        String::new()
    } else {
        query.to_owned()
    }
}

use super::day::ActivityDay;
use super::entry::HistoryEntry;
use super::table_row::HistoryTableRow;

#[function_component(HistoryExplorer)]
pub(crate) fn history_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let source = use_context::<StatsSourceContext>()
        .expect("Statistics query source is required")
        .0;
    let page = dashboard.history_page;
    let query = HistoryQuery {
        search: dashboard.history_query.clone(),
        category: dashboard.history_category,
        page: PageRequest::new(page, PAGE_SIZE).expect("Validated history page size"),
    };
    let loader: Rc<dyn Fn(HistoryQuery) -> QueryFuture<SessionRecord>> =
        Rc::new(move |query| source.history(query));
    let week_dates = activity_week_dates(dashboard.history_week_offset);
    let activity_query = ActivityQuery {
        search: summary_search(&dashboard.history_query),
        category: dashboard.history_category,
        dates: week_dates.clone(),
    };
    let activity_source = use_context::<StatsSourceContext>()
        .expect("Statistics query source is required")
        .0;
    let activity_loader: Rc<dyn Fn(ActivityQuery) -> QueryFuture<ActivityBucket>> =
        Rc::new(move |query| activity_source.activity(query));
    let (activity_page, activity_loading, activity_failure, retry_activity) =
        use_query_page(activity_query, activity_loader);
    let (loaded_page, loading, failure, retry) = use_query_page(query, loader);
    let sessions = loaded_page
        .as_ref()
        .map_or(&[][..], |page| page.records.as_slice());
    let total = loaded_page.as_ref().map_or(0, |page| page.total);
    let pages = total.div_ceil(PAGE_SIZE).max(1);
    let (range_start, range_end) = loaded_page.as_ref().map_or((0, 0), |page| page.range());
    let is_table = dashboard.history_view == HistoryView::Table;
    {
        let dashboard = dashboard.clone();
        use_effect_with(
            (total, page, loading, failure.is_none()),
            move |(total, page, loading, successful)| {
                if !*loading
                    && *successful
                    && *total > 0
                    && *page > 0
                    && *page >= total.div_ceil(PAGE_SIZE)
                {
                    dashboard.dispatch(DashboardAction::HistoryPage(total.div_ceil(PAGE_SIZE) - 1));
                }
            },
        );
    }
    let previous_week = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::HistoryWeek(
                dashboard.history_week_offset.saturating_add(1),
            ))
        })
    };
    let next_week = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::HistoryWeek(
                dashboard.history_week_offset.saturating_sub(1),
            ))
        })
    };
    let retry_request = Callback::from(move |_: MouseEvent| retry.emit(()));
    let retry_activity_request = Callback::from(move |_: MouseEvent| retry_activity.emit(()));
    let previous = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::HistoryPage(page.saturating_sub(1)))
        })
    };
    let next = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::HistoryPage((page + 1).min(pages - 1)))
        })
    };
    let clear_date = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::HistoryQuery(String::new()))
        })
    };
    let on_category = {
        let dashboard = dashboard.clone();
        Callback::from(move |category: HistoryCategory| {
            dashboard.dispatch(DashboardAction::HistoryCategory(category))
        })
    };
    let on_view = {
        let dashboard = dashboard.clone();
        Callback::from(move |view: HistoryView| {
            dashboard.dispatch(DashboardAction::HistoryView(view))
        })
    };
    let on_search = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            dashboard.dispatch(DashboardAction::HistoryQuery(input.value()));
        })
    };
    let on_date_select = {
        let dashboard = dashboard.clone();
        Callback::from(move |date: String| {
            let new_query = if dashboard.history_query == date {
                String::new()
            } else {
                date
            };
            dashboard.dispatch(DashboardAction::HistoryQuery(new_query));
        })
    };
    html! {
        <section data-stats-history="" class="space-y-3">
            if loaded_page.is_some() { <span data-stats-query-ready="history" class="sr-only" /> }
            if loading { <div role="status" aria-busy="true" data-stats-query-loading="history" class="rounded-lg border border-line bg-raised p-4 text-sm text-muted">{"Loading history…"}</div> }
            if let Some(error) = &failure {
                <div role="alert" data-stats-query-error="history" class="rounded-lg border border-line bg-surface p-4 text-sm">
                    <span>{error.clone()}</span>
                    <button type="button" onclick={retry_request} class="ml-3 rounded-lg bg-brand-action px-3 py-2 font-semibold text-white">{"Retry"}</button>
                </div>
            }
            <HistoryToolbar query={dashboard.history_query.clone()} category={dashboard.history_category}
                view={dashboard.history_view} on_search={on_search}
                on_category={on_category} on_view={on_view} on_clear_date={clear_date} />
            <div class="flex flex-wrap items-center justify-between gap-2">
                    <div class="flex flex-wrap items-center gap-4 text-xs text-muted" aria-label="Activity categories">
                    <span class="inline-flex items-center gap-1.5"><span aria-hidden="true" class="size-2 rounded-sm bg-accent" />{"Practice"}</span>
                    <span class="inline-flex items-center gap-1.5"><span aria-hidden="true" class="size-2 rounded-sm bg-amber-400/80" />{"Games"}</span>
                    </div>
                    <div class="flex items-center gap-1" role="group" aria-label="Activity week navigation">
                        <button type="button" onclick={previous_week} disabled={dashboard.history_week_offset >= 12}
                            aria-label="Previous activity week" title="Previous week" class="inline-flex size-8 items-center justify-center rounded-md border border-line hover:bg-raised disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                            <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px" />
                        </button>
                        <span class="min-w-36 text-center text-xs tabular-nums text-muted">{format!("{} to {}", &week_dates[0][5..], &week_dates[6][5..])}</span>
                        <button type="button" onclick={next_week} disabled={dashboard.history_week_offset == 0}
                            aria-label="Next activity week" title="Next week" class="inline-flex size-8 items-center justify-center rounded-md border border-line hover:bg-raised disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                            <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px" />
                        </button>
                    </div>
                </div>
            if let Some(activity) = &activity_page {
                <div class="grid grid-cols-7 gap-1" role="group" aria-label="Filter history by activity date">
                    {for activity.records.iter().map(|day| {
                        let key = day.date.clone();
                        html! { <ActivityDay key={key} day={(*day).clone()} selected={dashboard.history_query == day.date} on_select={on_date_select.clone()} /> }
                    })}
                </div>
            } else if activity_loading {
                <div role="status" aria-busy="true" class="h-16 animate-pulse rounded-xl border border-line bg-raised motion-reduce:animate-none">
                    <span class="sr-only">{"Loading activity summary"}</span>
                </div>
            } else if let Some(error) = &activity_failure {
                <div role="alert" class="rounded-lg border border-line p-3 text-sm">
                    <span>{error.clone()}</span>
                    <button type="button" onclick={retry_activity_request} class="ml-3 rounded-lg border border-line px-3 py-2">{"Retry"}</button>
                </div>
            }
            if is_table {
                <div class="overflow-x-auto rounded-xl border border-line bg-surface" data-stats-history-view="table">
                    <table class="w-full min-w-[36rem] border-collapse text-left text-sm">
                        <thead class="bg-raised text-xs text-muted"><tr>
                            <th scope="col" class="px-3 py-2 font-semibold">{"When (UTC)"}</th>
                            <th scope="col" class="px-3 py-2 font-semibold">{"Exercise"}</th>
                            <th scope="col" class="px-3 py-2 font-semibold">{"Category"}</th>
                            <th scope="col" class="px-3 py-2 font-semibold">{"Focus"}</th>
                            <th scope="col" class="px-3 py-2 text-right font-semibold">{"Duration"}</th>
                            <th scope="col" class="px-3 py-2 text-right font-semibold">{"Attempts"}</th>
                            <th scope="col" class="px-3 py-2 text-right font-semibold">{"Result"}</th>
                        </tr></thead>
                        <tbody>
                            {for sessions.iter().map(|session| {
                                let key = session.id.clone();
                                html! { <HistoryTableRow key={key} session={(*session).clone()} /> }
                            })}
                        </tbody>
                    </table>
                    if total == 0 && !loading && failure.is_none() {
                        <p role="status" class="p-6 text-center text-sm text-muted">{"No matching activities"}</p>
                    }
                </div>
            } else {
                <div class="overflow-hidden rounded-xl border border-line bg-surface" data-stats-history-view="timeline">
                    <ol aria-label="Preview session history">
                        {for sessions.iter().map(|session| html! { <HistoryEntry key={session.id.clone()} session={(*session).clone()} /> })}
                        if total == 0 && !loading && failure.is_none() {
                            <li role="status" class="p-6 text-center text-sm text-muted">{"No matching activities"}</li>
                        }
                    </ol>
                </div>
            }
            <super::super::pagination::StatsPagination subject="sessions" start={range_start} end={range_end}
                total={total} page={page} pages={pages} disabled={loading || failure.is_some()}
                on_previous={previous} on_next={next} />
        </section>
    }
}
