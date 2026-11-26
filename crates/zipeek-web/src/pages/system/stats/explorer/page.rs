use super::super::bulk_selection::use_bulk_selection;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::super::{
    context::{StatsSourceContext, use_stats_ui},
    controls::{ChoiceOption, ChoiceSelect, SelectionCheckbox},
    data::WordRecord,
    fixture::EvidenceStatus,
    query::{PageRequest, QueryFuture, WordQuery},
    query_hook::use_query_page,
    state::{DashboardAction, MAX_FILTERED_SELECTION, PAGE_SIZES, SkillGap, WordSort},
};

use super::row::VocabularyRow;

#[function_component(VocabularyExplorer)]
pub(crate) fn vocabulary_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let source = use_context::<StatsSourceContext>()
        .expect("Statistics query source is required")
        .0;
    let page_size = dashboard.word_page_size;
    let requested_page = dashboard.evidence_page;
    let query = WordQuery {
        search: dashboard.word_query.clone(),
        sort: dashboard.word_sort,
        gap: dashboard.skill_gap,
        page: PageRequest::new(requested_page, page_size).expect("Validated vocabulary page size"),
    };
    let loader: Rc<dyn Fn(WordQuery) -> QueryFuture<WordRecord>> =
        Rc::new(move |query| source.words(query));
    let (loaded_page, loading, failure, retry) = use_query_page(query.clone(), loader);
    let records = loaded_page
        .as_ref()
        .map_or(&[][..], |page| page.records.as_slice());
    let total = loaded_page.as_ref().map_or(0, |page| page.total);
    let pages = total.div_ceil(page_size).max(1);
    let page = requested_page;
    let (range_start, range_end) = loaded_page.as_ref().map_or((0, 0), |page| page.range());
    let visible_ids = records.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let all_visible = !visible_ids.is_empty()
        && visible_ids
            .iter()
            .all(|id| dashboard.selected_words.contains(id));
    let visible = records;
    {
        let dashboard = dashboard.clone();
        use_effect_with(
            (total, page, page_size, loading, failure.is_none()),
            move |(total, page, size, loading, successful)| {
                if !*loading
                    && *successful
                    && *total > 0
                    && *page > 0
                    && *page >= total.div_ceil(*size)
                {
                    dashboard.dispatch(DashboardAction::EvidencePage(total.div_ceil(*size) - 1));
                }
            },
        );
    }
    let retry_request = Callback::from(move |_: MouseEvent| retry.emit(()));
    let source = use_context::<StatsSourceContext>()
        .expect("Statistics query source is required")
        .0;
    let bulk_selection = use_bulk_selection(query.clone());
    let selection_loading = bulk_selection.loading.clone();
    let selection_error = bulk_selection.error.clone();
    let query_change = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            dashboard.dispatch(DashboardAction::WordQuery(input.value()));
        })
    };
    let sort_review = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::WordSort(WordSort::Review))
        })
    };
    let sort_hanzi = {
        let dashboard = dashboard.clone();
        let reverse = dashboard.word_sort == WordSort::Alphabetical;
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::WordSort(if reverse {
                WordSort::AlphabeticalReverse
            } else {
                WordSort::Alphabetical
            }))
        })
    };
    let sort_seen = {
        let dashboard = dashboard.clone();
        let reverse = dashboard.word_sort == WordSort::Encounters;
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::WordSort(if reverse {
                WordSort::FewestEncounters
            } else {
                WordSort::Encounters
            }))
        })
    };
    let page_size_change = {
        let dashboard = dashboard.clone();
        Callback::from(move |value: String| {
            if let Ok(size) = value.parse::<usize>() {
                dashboard.dispatch(DashboardAction::WordPageSize(size));
            }
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
                let result = super::super::query::matching_word_ids(
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
                    dashboard.dispatch(DashboardAction::SelectMatchingWords(ids));
                }
            });
        })
    };
    let select_visible = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: ()| {
            dashboard.dispatch(DashboardAction::SelectVisibleWords(visible_ids.clone()))
        })
    };
    let previous = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::EvidencePage(page.saturating_sub(1)))
        })
    };
    let next = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::EvidencePage((page + 1).min(pages - 1)))
        })
    };
    let clear_gap = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ClearSkillGap))
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
    let toggle_word = {
        let dashboard = dashboard.clone();
        Callback::from(move |id: String| dashboard.dispatch(DashboardAction::ToggleWord(id)))
    };
    html! {
        <section data-stats-vocabulary-explorer="" class="min-w-0 space-y-3">
            if loaded_page.is_some() { <span data-stats-query-ready="words" class="sr-only" /> }
            if loading { <div role="status" aria-busy="true" data-stats-query-loading="words" class="rounded-lg border border-line bg-raised p-4 text-sm text-muted">{"Loading vocabulary…"}</div> }
            if let Some(error) = &failure {
                <div role="alert" data-stats-query-error="words" class="rounded-lg border border-line bg-surface p-4 text-sm">
                    <span>{error.clone()}</span>
                    <button type="button" onclick={retry_request} class="ml-3 rounded-lg bg-brand-action px-3 py-2 font-semibold text-white">{"Retry"}</button>
                </div>
            }
            if let Some(error) = &*selection_error { <p role="alert" class="text-sm text-muted">{error.clone()}</p> }
            if *selection_loading { <p role="status" class="text-sm text-muted">{"Collecting matching IDs…"}</p> }
            <div class="flex flex-wrap items-end gap-3">
                <div class="relative min-w-48 w-full max-w-md flex-1">
                    <label class="block space-y-1 text-xs text-muted">
                        <span class="sr-only">{"Search vocabulary"}</span>
                        <input type="search" value={dashboard.word_query.clone()} oninput={query_change} placeholder="Hanzi, Pinyin or meaning" autocomplete="off"
                            class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent" />
                    </label>
                    if dashboard.word_query.trim().chars().count() >= 2 && !loading && failure.is_none() {
                        <div class="absolute left-0 right-0 top-full z-30 mt-1 max-h-48 overflow-y-auto rounded-lg border border-line bg-surface p-1 shadow-xl" aria-label="Vocabulary search suggestions">
                            {for records.iter().filter(|item| item.hanzi != dashboard.word_query).take(5).map(|item| {
                                let dashboard = dashboard.clone();
                                let hanzi = item.hanzi.clone();
                                let onclick = Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::WordQuery(hanzi.clone())));
                                html! { <button type="button" key={item.id.clone()} onclick={onclick} class="flex min-h-9 w-full items-center justify-between gap-2 rounded-md px-2 text-left text-sm hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                                    <span lang="zh-Hans" class="shrink-0 font-semibold">{item.hanzi.clone()}</span><span class="truncate text-xs text-muted">{item.meaning.clone()}</span>
                                </button> }
                            })}
                        </div>
                    }
                </div>
                <button type="button" title="Review first" aria-label="Sort review first" aria-pressed={(dashboard.word_sort == WordSort::Review).to_string()}
                    onclick={sort_review} class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line bg-surface px-3 text-xs text-ink hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_FILTER} width="16px" />{"Review"}
                </button>
                <div class="w-24">
                    <ChoiceSelect label="Rows" on_change={page_size_change} value={page_size.to_string()}
                        options={PAGE_SIZES.iter().map(|size| ChoiceOption { value: size.to_string(), label: size.to_string() }).collect::<Vec<_>>()} />
                </div>
            if total > 0 && !loading && failure.is_none() {
                <div class="flex flex-wrap items-center gap-3 text-xs text-muted">
                    <button type="button" onclick={select_matching} disabled={total > MAX_FILTERED_SELECTION || *selection_loading}
                        aria-label={format!("Select all {} matching words", total)} title="Select all matching" class="inline-flex size-9 items-center justify-center rounded-md border border-line bg-surface text-ink disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_CHECK} width="16px" />
                    </button>
                    if total > MAX_FILTERED_SELECTION {
                        <span>{format!("Limit selection to {} matching words or fewer using search and filters", MAX_FILTERED_SELECTION)}</span>
                    }
                </div>
            }

            <div class="ml-auto flex h-10 shrink-0 items-center gap-2 rounded-lg border border-line bg-surface/95 px-2 shadow-sm" data-stats-selection-toolbar="words">
                <span class="min-w-0 truncate text-xs tabular-nums text-muted" role="status" aria-live="polite">
                    {if dashboard.selected_words.is_empty() { "Select words".to_owned() } else { format!("{} selected", dashboard.selected_words.len()) }}
                </span>
                <div class="flex shrink-0 items-center gap-1">
                    <button type="button" aria-label="Clear selected words" title="Clear selection" onclick={clear.clone()} disabled={dashboard.selected_words.is_empty()}
                        class="inline-flex size-8 items-center justify-center rounded-md border border-line text-muted hover:bg-raised disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_X} width="16px" />
                    </button>
                    <button type="button" aria-label="Review selected words" title="Review selection" onclick={review.clone()} disabled={dashboard.selected_words.is_empty()}
                        class="inline-flex size-8 items-center justify-center rounded-md bg-brand-action text-white hover:opacity-90 disabled:opacity-30 focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="16px" />
                    </button>
                </div>
            </div>
            </div>
            if let Some(gap) = dashboard.skill_gap {
                <div class="flex items-center gap-2 text-sm">
                    <span class="rounded-full border border-line bg-raised px-3 py-1">{format!("{} · needs observation or review", gap.label())}</span>
                    <button type="button" onclick={clear_gap} class="rounded-lg border border-line px-3 py-1">{"Clear filter"}</button>
                </div>
            }
            <div class="overflow-x-auto rounded-xl border border-line bg-surface">
                <table class="w-full min-w-[780px] table-fixed text-left text-sm">
                    <caption class="sr-only">{"Demo vocabulary evidence; select rows for a review draft"}</caption>
                    <thead class="sticky top-0 z-10 bg-raised text-xs text-muted"><tr>
                        <th scope="col" class="w-14 min-w-14 px-2 py-2">
                            <SelectionCheckbox label="Select visible vocabulary" checked={all_visible} on_toggle={select_visible} disabled={visible.is_empty()} />
                        </th>
                        <th scope="col" class="w-[24%] px-2 py-2" aria-sort={match dashboard.word_sort { WordSort::Alphabetical => "ascending", WordSort::AlphabeticalReverse => "descending", _ => "none" }}>
                            <button type="button" onclick={sort_hanzi} aria-label="Sort words by Hanzi" class="inline-flex items-center gap-1 rounded focus-visible:outline-2 focus-visible:outline-accent">{"Word"}<Icon data={IconData::LUCIDE_ARROW_DOWN} width="13px" /></button>
                        </th>
                        <th scope="col" class="w-16 px-2 py-2" aria-sort={match dashboard.word_sort { WordSort::Encounters => "descending", WordSort::FewestEncounters => "ascending", _ => "none" }}>
                            <button type="button" onclick={sort_seen} aria-label="Sort words by encounters" class="inline-flex items-center gap-1 rounded focus-visible:outline-2 focus-visible:outline-accent">{"Seen"}<Icon data={IconData::LUCIDE_ARROW_DOWN} width="13px" /></button>
                        </th>
                        <th scope="col" class="w-16 px-2 py-2">{"Days"}</th>
                        <th scope="col" class="px-2 py-2">{"Recognition"}</th>
                        <th scope="col" class="px-2 py-2">{"Listen"}</th>
                        <th scope="col" class="px-2 py-2">{"Speak"}</th>
                        <th scope="col" class="px-2 py-2">{"Write"}</th>
                        <th scope="col" class="px-2 py-2">{"Use"}</th>
                    </tr></thead>
                    <tbody>
                        {for visible.iter().map(|record| {
                            let key = record.id.clone();
                            html! {
                                <VocabularyRow key={key} record={(*record).clone()}
                                    selected={dashboard.selected_words.contains(&record.id)} on_toggle={toggle_word.clone()} />
                            }
                        })}
                    </tbody>
                </table>
                if total == 0 && !loading && failure.is_none() { <div role="status" class="p-6 text-center text-sm text-muted">{"No matching vocabulary"}</div> }

            </div>
            <super::super::pagination::StatsPagination subject="words" start={range_start} end={range_end}
                total={total} page={page} pages={pages} disabled={loading || failure.is_some()}
                on_previous={previous} on_next={next} />

        </section>
    }
}
