use super::super::bulk_selection::use_bulk_selection;
use std::rc::Rc;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::super::{
    context::{StatsSourceContext, use_stats_ui},
    controls::{ChoiceOption, ChoiceSelect, SelectionCheckbox},
    data::GrammarRecord,
    query::{GrammarQuery, PageRequest, QueryFuture},
    query_hook::use_query_page,
    state::{DashboardAction, GrammarRegion, GrammarSort, GrammarView, MAX_FILTERED_SELECTION},
};

pub(crate) fn matching_patterns<'a>(
    records: &'a [GrammarRecord],
    query: &str,
) -> Vec<&'a GrammarRecord> {
    let search = query.trim().to_lowercase();
    records
        .iter()
        .filter(|item| {
            search.is_empty()
                || item.pattern.to_lowercase().contains(&search)
                || item.description.to_lowercase().contains(&search)
                || item.course.to_lowercase().contains(&search)
                || item.id.to_lowercase().contains(&search)
        })
        .collect()
}

use super::detail::GrammarDetail;
use super::items::{GrammarCard, GrammarTableRow};
use super::view::GrammarViewControl;

#[function_component(GrammarExplorer)]
pub(crate) fn grammar_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let source = use_context::<StatsSourceContext>()
        .expect("Statistics query source is required")
        .0;
    let page = dashboard.grammar_page;
    let page_size = dashboard.grammar_page_size;
    let query = GrammarQuery {
        search: dashboard.grammar_query.clone(),
        region: dashboard.grammar_region,
        sort: dashboard.grammar_sort,
        page: PageRequest::new(page, page_size).expect("Validated grammar page size"),
    };
    let loader: Rc<dyn Fn(GrammarQuery) -> QueryFuture<GrammarRecord>> =
        Rc::new(move |query| source.grammar(query));
    let (loaded_page, loading, failure, retry) = use_query_page(query.clone(), loader);
    let records = loaded_page
        .as_ref()
        .map_or(&[][..], |page| page.records.as_slice());
    let total = loaded_page.as_ref().map_or(0, |page| page.total);
    let pages = total.div_ceil(page_size).max(1);
    let (range_start, range_end) = loaded_page.as_ref().map_or((0, 0), |page| page.range());
    let visible = records;
    {
        let dashboard = dashboard.clone();
        use_effect_with(
            (total, page, loading, failure.is_none()),
            move |(total, page, loading, successful)| {
                if !*loading
                    && *successful
                    && *total > 0
                    && *page > 0
                    && *page >= total.div_ceil(page_size)
                {
                    dashboard.dispatch(DashboardAction::GrammarPage(total.div_ceil(page_size) - 1));
                }
            },
        );
    }
    let visible_ids: Vec<_> = visible.iter().map(|item| item.id.clone()).collect();
    let all_visible = !visible_ids.is_empty()
        && visible_ids
            .iter()
            .all(|id| dashboard.selected_patterns.contains(id));
    let retry_request = Callback::from(move |_: MouseEvent| retry.emit(()));
    let source = use_context::<StatsSourceContext>()
        .expect("Statistics query source is required")
        .0;
    let suggestion_query = GrammarQuery {
        search: dashboard.grammar_query.clone(),
        region: dashboard.grammar_region,
        sort: GrammarSort::TeachingOrder,
        page: PageRequest::new(0, 6).expect("Validated suggestion page size"),
    };
    let suggestion_loader: Rc<dyn Fn(GrammarQuery) -> QueryFuture<GrammarRecord>> = Rc::new({
        let source = source.clone();
        move |query| source.grammar(query)
    });
    let (suggestions, suggestions_loading, _, _) =
        use_query_page(suggestion_query, suggestion_loader);
    let suggestion_open = use_state(|| false);
    let suggestion_index = use_state(|| 0_usize);
    let suggested_patterns: Vec<_> = suggestions
        .as_ref()
        .into_iter()
        .flat_map(|page| page.records.iter())
        .filter(|item| item.pattern != dashboard.grammar_query)
        .take(5)
        .map(|item| (item.id.clone(), item.pattern.clone(), item.course.clone()))
        .collect();
    let active_suggestion_index =
        (*suggestion_index).min(suggested_patterns.len().saturating_sub(1));
    let show_suggestions = *suggestion_open
        && dashboard.grammar_query.trim().chars().count() >= 2
        && !suggestions_loading
        && !suggested_patterns.is_empty();
    let bulk_selection = use_bulk_selection(query.clone());
    let selection_loading = bulk_selection.loading.clone();
    let selection_error = bulk_selection.error.clone();
    let search = {
        let dashboard = dashboard.clone();
        let suggestion_open = suggestion_open.clone();
        let suggestion_index = suggestion_index.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            suggestion_index.set(0);
            suggestion_open.set(true);
            dashboard.dispatch(DashboardAction::GrammarQuery(input.value()));
        })
    };
    let search_focus = {
        let suggestion_open = suggestion_open.clone();
        Callback::from(move |_: FocusEvent| suggestion_open.set(true))
    };
    let search_keydown = {
        let dashboard = dashboard.clone();
        let suggestion_open = suggestion_open.clone();
        let suggestion_index = suggestion_index.clone();
        let patterns = suggested_patterns.clone();
        Callback::from(move |event: KeyboardEvent| {
            if !*suggestion_open || patterns.is_empty() {
                if event.key() == "Escape" {
                    suggestion_open.set(false);
                }
                return;
            }
            match event.key().as_str() {
                "ArrowDown" => {
                    event.prevent_default();
                    suggestion_index.set((*suggestion_index + 1) % patterns.len());
                }
                "ArrowUp" => {
                    event.prevent_default();
                    suggestion_index.set((*suggestion_index + patterns.len() - 1) % patterns.len());
                }
                "Enter" => {
                    event.prevent_default();
                    if let Some((_, pattern, _)) =
                        patterns.get((*suggestion_index).min(patterns.len() - 1))
                    {
                        dashboard.dispatch(DashboardAction::GrammarQuery(pattern.clone()));
                        suggestion_open.set(false);
                    }
                }
                "Escape" | "Tab" => suggestion_open.set(false),
                _ => {}
            }
        })
    };
    let change_region = {
        let dashboard = dashboard.clone();
        Callback::from(move |value: String| {
            let region = match value.as_str() {
                "1" => GrammarRegion::Regions1To20,
                "2" => GrammarRegion::Regions21To40,
                "3" => GrammarRegion::Regions41To60,
                "4" => GrammarRegion::Regions61To80,
                "5" => GrammarRegion::Regions81To100,
                _ => GrammarRegion::All,
            };
            dashboard.dispatch(DashboardAction::GrammarRegion(region));
        })
    };
    let change_page_size = {
        let dashboard = dashboard.clone();
        Callback::from(move |value: String| {
            if let Ok(size) = value.parse::<usize>() {
                dashboard.dispatch(DashboardAction::GrammarPageSize(size));
            }
        })
    };
    let sort_patterns = {
        let dashboard = dashboard.clone();
        let next_sort = dashboard.grammar_sort.toggle_pattern();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::GrammarSort(next_sort))
        })
    };
    let sort_attempts = {
        let dashboard = dashboard.clone();
        let next_sort = dashboard.grammar_sort.toggle_attempts();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::GrammarSort(next_sort))
        })
    };
    let select_matching = {
        let dashboard = dashboard.clone();
        let source = source.clone();
        let query = query.clone();
        let selection = bulk_selection.clone();
        Callback::from(move |_: MouseEvent| {
            let ticket = selection.begin();
            let dashboard = dashboard.clone();
            let source = source.clone();
            let query = query.clone();
            let selection = selection.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = super::super::query::matching_grammar_ids(
                    source.as_ref(),
                    query.clone(),
                    MAX_FILTERED_SELECTION,
                )
                .await
                .map_err(|error| error.to_string());
                if !selection.current(&query, ticket) {
                    return;
                }
                if let Some(ids) = selection.finish(result) {
                    dashboard.dispatch(DashboardAction::SelectMatchingPatterns(ids));
                }
            });
        })
    };
    let toggle_visible = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: ()| {
            dashboard.dispatch(DashboardAction::SelectVisiblePatterns(visible_ids.clone()))
        })
    };
    let previous = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::GrammarPage(page.saturating_sub(1)))
        })
    };
    let next = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::GrammarPage((page + 1).min(pages - 1)))
        })
    };
    let review = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::ReviewSelection(true))
        })
    };
    let clear = {
        let dashboard = dashboard.clone();
        let selection = bulk_selection.clone();
        Callback::from(move |_: MouseEvent| {
            selection.cancel();
            dashboard.dispatch(DashboardAction::ClearSelection);
        })
    };
    let on_view = {
        let dashboard = dashboard.clone();
        Callback::from(move |view: GrammarView| {
            dashboard.dispatch(DashboardAction::GrammarView(view))
        })
    };
    let close_inspector = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: ()| {
            if let Some(id) = dashboard.grammar_detail_id.clone() {
                dashboard.dispatch(DashboardAction::ToggleGrammarDetail(id));
            }
        })
    };
    let inspected_record = visible
        .iter()
        .find(|item| dashboard.grammar_detail_id.as_ref() == Some(&item.id))
        .cloned();
    let toggle_pattern = {
        let dashboard = dashboard.clone();
        Callback::from(move |id: String| dashboard.dispatch(DashboardAction::TogglePattern(id)))
    };
    let inspect_pattern = {
        let dashboard = dashboard.clone();
        Callback::from(move |id: String| {
            dashboard.dispatch(DashboardAction::ToggleGrammarDetail(id))
        })
    };
    html! {
        <section data-stats-grammar="" class="space-y-3">
            if loaded_page.is_some() { <span data-stats-query-ready="grammar" class="sr-only" /> }
            if loading { <div role="status" aria-busy="true" data-stats-query-loading="grammar" class="rounded-lg border border-line bg-raised p-4 text-sm text-muted">{"Loading grammar…"}</div> }
            if let Some(error) = &failure {
                <div role="alert" data-stats-query-error="grammar" class="rounded-lg border border-line bg-surface p-4 text-sm">
                    <span>{error.clone()}</span>
                    <button type="button" onclick={retry_request} class="ml-3 rounded-lg bg-brand-action px-3 py-2 font-semibold text-white">{"Retry"}</button>
                </div>
            }
            if let Some(error) = &*selection_error { <p role="alert" class="text-sm text-muted">{error.clone()}</p> }
            if *selection_loading { <p role="status" class="text-sm text-muted">{"Collecting matching IDs…"}</p> }
            <div class="flex flex-wrap items-end gap-2">
            <div class="relative min-w-48 w-full max-w-md flex-1">
                <label class="block space-y-1 text-xs text-muted">
                    <span class="sr-only">{"Search grammar patterns"}</span>
                    <input type="search" value={dashboard.grammar_query.clone()} oninput={search}
                        onfocus={search_focus} onkeydown={search_keydown}
                        role="combobox" aria-autocomplete="list" aria-controls="grammar-search-options"
                        aria-expanded={show_suggestions.to_string()}
                        aria-activedescendant={if show_suggestions { suggested_patterns.get(active_suggestion_index).map(|(id, _, _)| format!("grammar-option-{id}")) } else { None }}
                        placeholder="Search patterns or meaning" autocomplete="off"
                        class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent" />
                </label>
                if show_suggestions {
                    <div id="grammar-search-options" role="listbox" class="absolute left-0 right-0 top-full z-30 mt-1 max-h-48 overflow-y-auto rounded-lg border border-line bg-surface p-1 shadow-xl" aria-label="Grammar suggestions">
                        {for suggested_patterns.iter().enumerate().map(|(index, (id, pattern, course))| {
                            let dashboard = dashboard.clone();
                            let suggestion_open = suggestion_open.clone();
                            let selection = pattern.clone();
                            let selected = active_suggestion_index == index;
                            let onclick = Callback::from(move |_: MouseEvent| {
                                dashboard.dispatch(DashboardAction::GrammarQuery(selection.clone()));
                                suggestion_open.set(false);
                            });
                            html! {
                                <div id={format!("grammar-option-{id}")} role="option" key={id.clone()}
                                    aria-selected={selected.to_string()} onclick={onclick}
                                    class={classes!("flex", "min-h-9", "items-center", "justify-between", "gap-2", "rounded-md", "px-2", "text-left", "text-sm", "cursor-pointer", if selected { "bg-raised" } else { "hover:bg-raised" })}>
                                    <span class="truncate">{pattern.clone()}</span>
                                    <span class="shrink-0 text-xs text-muted">{course.clone()}</span>
                                </div>
                            }
                        })}
                    </div>
                }
            </div>
            <div class="w-full sm:w-52">
                <ChoiceSelect label="Collection" on_change={change_region}
                    value={match dashboard.grammar_region { GrammarRegion::All => "0", GrammarRegion::Regions1To20 => "1", GrammarRegion::Regions21To40 => "2", GrammarRegion::Regions41To60 => "3", GrammarRegion::Regions61To80 => "4", GrammarRegion::Regions81To100 => "5" }.to_owned()}
                    options={GrammarRegion::ALL.iter().enumerate().map(|(index, region)| ChoiceOption { value: index.to_string(), label: region.label().to_owned() }).collect::<Vec<_>>()} />
            </div>
            <div class="w-24">
                <ChoiceSelect label="Rows" on_change={change_page_size} value={page_size.to_string()}
                    options={[10, 25, 50, 100].iter().map(|size| ChoiceOption { value: size.to_string(), label: size.to_string() }).collect::<Vec<_>>()} />
            </div>
            if total > 0 && !loading && failure.is_none() {
                <div class="flex flex-wrap items-center gap-3 text-xs text-muted">
                    <button type="button" onclick={select_matching} disabled={total > MAX_FILTERED_SELECTION || *selection_loading}
                        aria-label={format!("Select all {} matching patterns", total)} title="Select all matching" class="inline-flex size-9 items-center justify-center rounded-md border border-line bg-surface text-ink disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_CHECK} width="16px" />
                    </button>
                </div>
            }

            <div class="ml-auto flex h-10 shrink-0 items-center gap-2 rounded-lg border border-line bg-surface/95 px-2 shadow-sm" data-stats-selection-toolbar="grammar">
                <span class="min-w-0 truncate text-xs tabular-nums text-muted" role="status" aria-live="polite">
                    {if dashboard.selected_patterns.is_empty() { "Select patterns".to_owned() } else { format!("{} selected", dashboard.selected_patterns.len()) }}
                </span>
                <div class="flex shrink-0 items-center gap-1">
                    <button type="button" aria-label="Clear selected patterns" title="Clear selection" onclick={clear.clone()} disabled={dashboard.selected_patterns.is_empty()}
                        class="inline-flex size-8 items-center justify-center rounded-md border border-line text-muted hover:bg-raised disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_X} width="16px" />
                    </button>
                    <button type="button" aria-label="Review selected patterns" title="Review selection" onclick={review.clone()} disabled={dashboard.selected_patterns.is_empty()}
                        class="inline-flex size-8 items-center justify-center rounded-md bg-brand-action text-white hover:opacity-90 disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="16px" />
                    </button>
                </div>
            </div>
            <GrammarViewControl view={dashboard.grammar_view} total={total} on_view={on_view} />
            </div>
            if dashboard.grammar_view == GrammarView::Cards {
                <div class="grid gap-2 md:grid-cols-2 xl:grid-cols-3" data-stats-grammar-cards="">
                    if total == 0 && !loading && failure.is_none() {
                        <p role="status" class="col-span-full rounded-lg border border-line p-5 text-sm text-muted">{"No matching grammar patterns"}</p>
                    }
                    {for visible.iter().map(|pattern| {
                        let key = pattern.id.clone();
                        html! {
                            <GrammarCard key={key} record={(*pattern).clone()}
                                selected={dashboard.selected_patterns.contains(&pattern.id)}
                                on_toggle={toggle_pattern.clone()} on_inspect={inspect_pattern.clone()} />
                        }
                    })}
                </div>
            } else {
            <div class="overflow-x-auto rounded-xl border border-line bg-surface">
                <table class="w-full min-w-[600px] text-left text-sm">
                    <caption class="sr-only">{"Grammar patterns, practice attempts and details"}</caption>
                    <thead class="sticky top-0 z-10 bg-raised text-xs text-muted"><tr>
                        <th scope="col" class="w-14 min-w-14 px-2 py-2"><SelectionCheckbox label="Select visible grammar patterns" checked={all_visible} on_toggle={toggle_visible} disabled={visible.is_empty()} /></th>
                        <th scope="col" class="px-2 py-2" aria-sort={match dashboard.grammar_sort { GrammarSort::PatternAscending => "ascending", GrammarSort::PatternDescending => "descending", _ => "none" }}>
                            <button type="button" onclick={sort_patterns} aria-label="Sort grammar patterns" class="inline-flex items-center gap-1 rounded focus-visible:outline-2 focus-visible:outline-accent">
                                {"Pattern"}<Icon data={IconData::LUCIDE_ARROW_UP_DOWN} width="13px" />
                            </button>
                        </th>
                        <th scope="col" class="px-2 py-2">{"Teaching group"}</th>
                        <th scope="col" class="px-2 py-2">{"Example"}</th>
                        <th scope="col" class="w-28 px-2 py-2" aria-sort={match dashboard.grammar_sort { GrammarSort::MostAttempts => "descending", GrammarSort::FewestAttempts => "ascending", _ => "none" }}>
                            <button type="button" onclick={sort_attempts} aria-label="Sort grammar attempts" class="inline-flex items-center gap-1 rounded focus-visible:outline-2 focus-visible:outline-accent">
                                {"Attempts"}<Icon data={IconData::LUCIDE_ARROW_UP_DOWN} width="13px" />
                            </button>
                        </th>
                    </tr></thead>
                    <tbody>{for visible.iter().map(|pattern| {
                        let key = pattern.id.clone();
                        html! {
                            <GrammarTableRow key={key} record={(*pattern).clone()}
                                selected={dashboard.selected_patterns.contains(&pattern.id)}
                                detail_open={dashboard.grammar_detail_id.as_ref() == Some(&pattern.id)}
                                on_toggle={toggle_pattern.clone()} on_inspect={inspect_pattern.clone()} />
                        }
                    })}</tbody>
                </table>
                if total == 0 && !loading && failure.is_none() { <p role="status" class="p-6 text-center text-sm text-muted">{"No matching grammar patterns"}</p> }

            </div>
            }
            if let Some(record) = inspected_record {
                <GrammarDetail record={record} on_close={close_inspector} />
            }
            <super::super::pagination::StatsPagination subject="patterns" start={range_start} end={range_end}
                total={total} page={page} pages={pages} disabled={loading || failure.is_some()}
                on_previous={previous} on_next={next} />
        </section>
    }
}
