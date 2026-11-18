use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{
    context::{use_stats_ui, use_loaded_section},
    data::{SectionPayload, WordRecord},
    controls::SelectionCheckbox,
    fixture::EvidenceStatus,
    state::{DashboardAction, SkillGap, WordSort, PAGE_SIZES, MAX_FILTERED_SELECTION},
};


pub(super) fn filtered_words<'a>(records: &'a [WordRecord], query: &str, sort: WordSort, gap: Option<SkillGap>) -> Vec<&'a WordRecord> {
    let needle = query.trim().to_lowercase();
    let mut records: Vec<_> = records.iter().filter(|record| {
        needle.is_empty()
            || record.hanzi.contains(&needle)
            || record.pinyin.to_lowercase().contains(&needle)
            || record.meaning.to_lowercase().contains(&needle)
    }).filter(|record| {
        let Some(skill) = gap else { return true; };
        let status = match skill {
            SkillGap::Recognition => record.recognition,
            SkillGap::Listening => record.listening,
            SkillGap::Speaking => record.speaking,
            SkillGap::Writing => record.writing,
            SkillGap::Sentence => record.sentence,
        };
        status != EvidenceStatus::Observed
    }).collect();
    match sort {
        WordSort::Alphabetical => records.sort_by(|a, b| a.hanzi.cmp(&b.hanzi)),
        WordSort::Encounters => records.sort_by(|a, b| b.encounters.cmp(&a.encounters).then(a.hanzi.cmp(&b.hanzi))),
        WordSort::Review => records.sort_by(|a, b| has_review(b).cmp(&has_review(a)).then(a.hanzi.cmp(&b.hanzi))),
    }
    records
}

fn has_review(row: &WordRecord) -> bool {
    [row.recognition, row.listening, row.speaking, row.writing, row.sentence]
        .contains(&EvidenceStatus::NeedsReview)
}

fn evidence_icon(status: EvidenceStatus) -> Html {
    let (symbol, class) = match status {
        EvidenceStatus::Observed => ("✓", "text-accent"),
        EvidenceStatus::NeedsReview => ("!", "text-amber-400"),
        EvidenceStatus::NeedsEvidence => ("–", "text-muted"),
    };
    html! { <span title={status.label()} aria-label={status.label()} class={classes!("inline-flex", "min-w-7", "justify-center", "font-semibold", class)}>{symbol}</span> }
}

#[function_component(VocabularyExplorer)]
pub(super) fn vocabulary_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::Words(words) = loaded.as_ref() else { unreachable!("Vocabulary projection expected") };
    let records = filtered_words(words, &dashboard.word_query, dashboard.word_sort, dashboard.skill_gap);
    let total = records.len();
    let page_size = dashboard.word_page_size;
    let pages = total.div_ceil(page_size).max(1);
    let page = dashboard.evidence_page.min(pages - 1);
    let offset = page * page_size;
    let visible = records.iter().skip(offset).take(page_size).copied().collect::<Vec<_>>();
    let visible_ids = visible.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let all_visible = !visible_ids.is_empty() && visible_ids.iter().all(|id| dashboard.selected_words.contains(id));
    let query_change = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            dashboard.dispatch(DashboardAction::WordQuery(input.value()));
        })
    };
    let sort_change = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: Event| {
            let input: HtmlSelectElement = event.target_unchecked_into();
            let sort = match input.value().as_str() {
                "encounters" => WordSort::Encounters,
                "review" => WordSort::Review,
                _ => WordSort::Alphabetical,
            };
            dashboard.dispatch(DashboardAction::WordSort(sort));
        })
    };
    let page_size_change = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: Event| {
            let input: HtmlSelectElement = event.target_unchecked_into();
            if let Ok(size) = input.value().parse::<usize>() {
                dashboard.dispatch(DashboardAction::WordPageSize(size));
            }
        })
    };
    let select_matching = {
        let dashboard = dashboard.clone();
        let matching = records.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::SelectMatchingWords(matching.clone()));
        })
    };
    let select_visible = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: ()| dashboard.dispatch(DashboardAction::SelectVisibleWords(visible_ids.clone())))
    };
    let previous = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::EvidencePage(page.saturating_sub(1))))
    };
    let next = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::EvidencePage((page + 1).min(pages - 1))))
    };
    let clear_gap = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ClearSkillGap))
    };
    let review = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ReviewSelection(true)))
    };
    let clear = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::ClearSelection))
    };
    html! {
        <section data-stats-vocabulary-explorer="" class="min-w-0 space-y-3">
            <div class="flex flex-wrap items-end gap-3">
                <label class="min-w-48 flex-1 space-y-1 text-xs text-muted">
                    <span>{"Search vocabulary"}</span>
                    <input type="search" value={dashboard.word_query.clone()} oninput={query_change} placeholder="Hanzi, Pinyin or meaning"
                        class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent" />
                </label>
                <label class="space-y-1 text-xs text-muted">
                    <span>{"Sort"}</span>
                    <span class="relative block">
                        <select onchange={sort_change} class="min-h-10 w-full appearance-none rounded-lg border border-line bg-surface px-3 pr-8 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent">
                            <option value="alphabetical" selected={dashboard.word_sort == WordSort::Alphabetical}>{"Hanzi"}</option>
                            <option value="encounters" selected={dashboard.word_sort == WordSort::Encounters}>{"Most encounters"}</option>
                            <option value="review" selected={dashboard.word_sort == WordSort::Review}>{"Review first"}</option>
                        </select>
                        <span aria-hidden="true" class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2"><Icon data={IconData::LUCIDE_CHEVRON_DOWN} width="15px" /></span>
                    </span>
                </label>
                <label class="space-y-1 text-xs text-muted">
                    <span>{"Rows"}</span>
                    <span class="relative block">
                        <select aria-label="Vocabulary rows per page" onchange={page_size_change}
                            class="min-h-10 w-full appearance-none rounded-lg border border-line bg-surface px-3 pr-8 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent">
                            {for PAGE_SIZES.into_iter().map(|size| html! {
                                <option key={size.to_string()} value={size.to_string()} selected={page_size == size}>{size}</option>
                            })}
                        </select>
                        <span aria-hidden="true" class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2"><Icon data={IconData::LUCIDE_CHEVRON_DOWN} width="15px" /></span>
                    </span>
                </label>
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
                    <thead class="bg-raised text-xs text-muted"><tr>
                        <th scope="col" class="w-10 px-3 py-2">
                            <SelectionCheckbox label="Select visible vocabulary" checked={all_visible} on_toggle={select_visible} disabled={visible.is_empty()} />
                        </th>
                        <th scope="col" class="w-[24%] px-2 py-2">{"Word"}</th>
                        <th scope="col" class="w-16 px-2 py-2">{"Seen"}</th>
                        <th scope="col" class="w-16 px-2 py-2">{"Days"}</th>
                        <th scope="col" class="px-2 py-2">{"Recognition"}</th>
                        <th scope="col" class="px-2 py-2">{"Listen"}</th>
                        <th scope="col" class="px-2 py-2">{"Speak"}</th>
                        <th scope="col" class="px-2 py-2">{"Write"}</th>
                        <th scope="col" class="px-2 py-2">{"Use"}</th>
                    </tr></thead>
                    <tbody>
                        {for visible.iter().map(|record| {
                            let id = record.id.clone();
                            let selected = dashboard.selected_words.contains(&id);
                            let toggle = {
                                let dashboard = dashboard.clone();
                                Callback::from(move |_: ()| dashboard.dispatch(DashboardAction::ToggleWord(id.clone())))
                            };
                            html! {
                                <tr key={record.id.clone()} class="h-14 border-t border-line hover:bg-raised/60">
                                    <td class="px-3"><SelectionCheckbox label={format!("Select {}", record.hanzi)} checked={selected} on_toggle={toggle} /></td>
                                    <th scope="row" class="px-2"><span lang="zh-Hans" class="font-semibold">{record.hanzi.clone()}</span><span class="block truncate text-xs font-normal text-muted">{record.pinyin.clone()}{" · "}{record.meaning.clone()}</span></th>
                                    <td class="px-2 tabular-nums">{record.encounters}</td>
                                    <td class="px-2 tabular-nums">{record.days}</td>
                                    <td class="px-2">{evidence_icon(record.recognition)}</td>
                                    <td class="px-2">{evidence_icon(record.listening)}</td>
                                    <td class="px-2">{evidence_icon(record.speaking)}</td>
                                    <td class="px-2">{evidence_icon(record.writing)}</td>
                                    <td class="px-2">{evidence_icon(record.sentence)}</td>
                                </tr>
                            }
                        })}
                    </tbody>
                </table>
                if total == 0 { <div role="status" class="p-6 text-center text-sm text-muted">{"No matching vocabulary"}</div> }
                <div class="min-h-10 border-t border-line" aria-hidden="true" />
            </div>
            <div class="flex items-center justify-between gap-3 text-xs text-muted">
                <span>{format!("{}-{} of {}", if total == 0 {0} else {offset + 1}, (offset + page_size).min(total), total)}</span>
                <div class="flex items-center gap-2">
                    <button type="button" aria-label="Previous vocabulary page" disabled={page == 0} onclick={previous} class="rounded-md border border-line p-2 disabled:opacity-40"><Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px" /></button>
                    <span class="tabular-nums">{format!("{} / {}", page + 1, pages)}</span>
                    <button type="button" aria-label="Next vocabulary page" disabled={page + 1 == pages} onclick={next} class="rounded-md border border-line p-2 disabled:opacity-40"><Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px" /></button>
                </div>
            </div>
            if total > 0 {
                <div class="flex flex-wrap items-center gap-3 text-xs text-muted">
                    <button type="button" onclick={select_matching} disabled={total > MAX_FILTERED_SELECTION}
                        class="min-h-10 rounded-lg border border-line bg-surface px-3 font-semibold text-ink disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-accent">
                        {format!("Select all {} matching words", total)}
                    </button>
                    if total > MAX_FILTERED_SELECTION {
                        <span>{format!("Limit selection to {} matching words or fewer using search and filters", MAX_FILTERED_SELECTION)}</span>
                    }
                </div>
            }
            if !dashboard.selected_words.is_empty() {
                <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-raised p-3">
                    <span class="text-sm font-semibold">{format!("{} words selected", dashboard.selected_words.len())}</span>
                    <div class="flex gap-2">
                        <button type="button" onclick={clear} class="min-h-10 rounded-lg border border-line px-3 text-sm">{"Clear"}</button>
                        <button type="button" onclick={review} class="min-h-10 rounded-lg bg-brand-action px-3 text-sm font-semibold text-white">{"Review selection"}</button>
                    </div>
                </div>
            }
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_and_sort_are_deterministic() {
        let fixture = super::super::data::demo_words_for_tests();
        let found = filtered_words(&fixture, "study", WordSort::Alphabetical, None);
        assert!(found.iter().any(|word| word.hanzi == "学习"));
        let ranked = filtered_words(&fixture, "", WordSort::Encounters, None);
        assert!(ranked.windows(2).all(|pair| pair[0].encounters >= pair[1].encounters));
        assert!(filtered_words(&fixture, "nonexistent-token", WordSort::Review, None).is_empty());
    }
}
