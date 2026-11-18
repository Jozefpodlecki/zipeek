use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{context::{use_stats_ui, use_loaded_section}, data::{GrammarRecord, SectionPayload}, controls::SelectionCheckbox, state::{DashboardAction, MAX_FILTERED_SELECTION}};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct GrammarSample {
    pub id: &'static str,
    pub pattern: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub attempts: u16,
    pub course: &'static str,
}

pub(super) const GRAMMAR_SAMPLES: [GrammarSample; 16] = [
    GrammarSample { id: "g01", pattern: "把 + object + verb", description: "Disposal construction", example: "把书放在桌上。", attempts: 12, course: "Sentence structure" },
    GrammarSample { id: "g02", pattern: "被 + agent + verb", description: "Passive construction", example: "门被风吹开了。", attempts: 8, course: "Sentence structure" },
    GrammarSample { id: "g03", pattern: "比 + comparison", description: "Comparison", example: "今天比昨天热。", attempts: 9, course: "Comparisons" },
    GrammarSample { id: "g04", pattern: "越来越 + adjective", description: "Progressive change", example: "天气越来越冷。", attempts: 6, course: "Comparisons" },
    GrammarSample { id: "g05", pattern: "虽然…但是…", description: "Concession", example: "虽然下雨，但是我会去。", attempts: 10, course: "Connectives" },
    GrammarSample { id: "g06", pattern: "因为…所以…", description: "Cause and result", example: "因为下雨，所以我没去。", attempts: 14, course: "Connectives" },
    GrammarSample { id: "g07", pattern: "如果…就…", description: "Conditional", example: "如果有时间，我就来。", attempts: 7, course: "Connectives" },
    GrammarSample { id: "g08", pattern: "不但…而且…", description: "Addition", example: "他不但聪明，而且勤奋。", attempts: 4, course: "Connectives" },
    GrammarSample { id: "g09", pattern: "一边…一边…", description: "Simultaneous actions", example: "她一边听歌一边做饭。", attempts: 8, course: "Aspect and time" },
    GrammarSample { id: "g10", pattern: "正在 + verb", description: "Ongoing action", example: "我正在学习。", attempts: 11, course: "Aspect and time" },
    GrammarSample { id: "g11", pattern: "verb + 过", description: "Past experience", example: "我去过北京。", attempts: 5, course: "Aspect and time" },
    GrammarSample { id: "g12", pattern: "verb + 了", description: "Change or completion", example: "我吃了饭。", attempts: 9, course: "Aspect and time" },
    GrammarSample { id: "g13", pattern: "除了…以外…", description: "Exception and addition", example: "除了中文以外，我还学英语。", attempts: 3, course: "Connectives" },
    GrammarSample { id: "g14", pattern: "跟 + person + 一起", description: "Doing things together", example: "我跟朋友一起学习。", attempts: 10, course: "Sentence structure" },
    GrammarSample { id: "g15", pattern: "连…都…", description: "Emphatic inclusion", example: "连孩子都知道。", attempts: 2, course: "Emphasis" },
    GrammarSample { id: "g16", pattern: "先…再…", description: "Sequence", example: "先洗手，再吃饭。", attempts: 6, course: "Aspect and time" },
];

pub(super) fn matching_patterns<'a>(records: &'a [GrammarRecord], query: &str) -> Vec<&'a GrammarRecord> {
    let search = query.trim().to_lowercase();
    records.iter().filter(|item| {
        search.is_empty()
            || item.pattern.to_lowercase().contains(&search)
            || item.description.to_lowercase().contains(&search)
            || item.course.to_lowercase().contains(&search)
    }).collect()
}

const GRAMMAR_PAGE_SIZE: usize = 8;

#[function_component(GrammarExplorer)]
pub(super) fn grammar_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::Grammar(records) = loaded.as_ref() else { unreachable!("Grammar projection expected") };
    let matches = matching_patterns(records, &dashboard.grammar_query);
    let pages = matches.len().div_ceil(GRAMMAR_PAGE_SIZE).max(1);
    let page = dashboard.grammar_page.min(pages - 1);
    let offset = page * GRAMMAR_PAGE_SIZE;
    let visible: Vec<_> = matches.iter().skip(offset).take(GRAMMAR_PAGE_SIZE).copied().collect();
    let visible_ids: Vec<_> = visible.iter().map(|item| item.id.to_owned()).collect();
    let all_visible = !visible_ids.is_empty() && visible_ids.iter().all(|id| dashboard.selected_patterns.contains(id));
    let search = {
        let dashboard = dashboard.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            dashboard.dispatch(DashboardAction::GrammarQuery(input.value()));
        })
    };
    let select_matching = {
        let dashboard = dashboard.clone();
        let matching = matches.iter().map(|item| item.id.to_owned()).collect::<Vec<_>>();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::SelectMatchingPatterns(matching.clone()));
        })
    };
    let toggle_visible = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: ()| dashboard.dispatch(DashboardAction::SelectVisiblePatterns(visible_ids.clone())))
    };
    let previous = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::GrammarPage(page.saturating_sub(1))))
    };
    let next = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::GrammarPage((page + 1).min(pages - 1))))
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
        <section data-stats-grammar="" class="space-y-3">
            <label class="block max-w-xl space-y-1 text-xs text-muted">
                <span>{"Find a grammar pattern"}</span>
                <input type="search" value={dashboard.grammar_query.clone()} oninput={search} placeholder="Pattern, course or meaning"
                    class="min-h-10 w-full rounded-lg border border-line bg-surface px-3 text-sm text-ink focus-visible:outline-2 focus-visible:outline-accent" />
            </label>
            <div class="overflow-x-auto rounded-xl border border-line bg-surface">
                <table class="w-full min-w-[600px] text-left text-sm">
                    <caption class="sr-only">{"Fictional grammar activity using illustrative grammar patterns"}</caption>
                    <thead class="bg-raised text-xs text-muted"><tr>
                        <th scope="col" class="w-10 px-3 py-2"><SelectionCheckbox label="Select visible grammar patterns" checked={all_visible} on_toggle={toggle_visible} disabled={visible.is_empty()} /></th>
                        <th scope="col" class="px-2 py-2">{"Pattern"}</th>
                        <th scope="col" class="px-2 py-2">{"Course"}</th>
                        <th scope="col" class="px-2 py-2">{"Example"}</th>
                        <th scope="col" class="w-24 px-2 py-2">{"Attempts"}</th>
                    </tr></thead>
                    <tbody>{for visible.iter().map(|pattern| {
                        let id = pattern.id.to_owned();
                        let selected = dashboard.selected_patterns.contains(&id);
                        let toggle = {
                            let dashboard = dashboard.clone();
                            Callback::from(move |_: ()| dashboard.dispatch(DashboardAction::TogglePattern(id.clone())))
                        };
                        html! {
                            <tr key={pattern.id.clone()} class="h-14 border-t border-line hover:bg-raised/60">
                                <td class="px-3"><SelectionCheckbox label={format!("Select {}", pattern.pattern)} checked={selected} on_toggle={toggle} /></td>
                                <th scope="row" class="px-2"><span class="font-semibold">{pattern.pattern.clone()}</span><span class="block text-xs font-normal text-muted">{pattern.description.clone()}</span></th>
                                <td class="px-2 text-muted">{pattern.course.clone()}</td>
                                <td class="px-2" lang="zh-Hans">{pattern.example.clone()}</td>
                                <td class="px-2 tabular-nums">{pattern.attempts}</td>
                            </tr>
                        }
                    })}</tbody>
                </table>
                if matches.is_empty() { <p role="status" class="p-6 text-center text-sm text-muted">{"No matching grammar patterns"}</p> }
                <div class="min-h-10 border-t border-line" aria-hidden="true" />
            </div>
            <div class="flex items-center justify-between gap-2 text-xs text-muted">
                <span>{format!("{}-{} of {}", if matches.is_empty() {0} else {offset+1}, (offset+GRAMMAR_PAGE_SIZE).min(matches.len()), matches.len())}</span>
                <div class="flex items-center gap-2">
                    <button type="button" aria-label="Previous grammar page" disabled={page==0} onclick={previous} class="rounded-md border border-line p-2 disabled:opacity-40"><Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px"/></button>
                    <span>{format!("{} / {}", page+1, pages)}</span>
                    <button type="button" aria-label="Next grammar page" disabled={page+1==pages} onclick={next} class="rounded-md border border-line p-2 disabled:opacity-40"><Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px"/></button>
                </div>
            </div>
            if !matches.is_empty() {
                <div class="flex flex-wrap items-center gap-3 text-xs text-muted">
                    <button type="button" onclick={select_matching} disabled={matches.len() > MAX_FILTERED_SELECTION}
                        class="min-h-10 rounded-lg border border-line bg-surface px-3 font-semibold text-ink disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-accent">
                        {format!("Select all {} matching patterns", matches.len())}
                    </button>
                </div>
            }
            if !dashboard.selected_patterns.is_empty() {
                <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-raised p-3">
                    <span class="text-sm font-semibold">{format!("{} patterns selected", dashboard.selected_patterns.len())}</span>
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
    fn grammar_fixture_ids_are_unique() {
        let ids = GRAMMAR_SAMPLES.iter().map(|item| item.id).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), GRAMMAR_SAMPLES.len());
        assert!(matching_patterns(&super::super::data::demo_grammar_for_tests(), "conditional").iter().any(|item| item.id == "g07"));
    }
}
