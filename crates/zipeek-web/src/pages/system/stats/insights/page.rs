use super::super::{
    context::{use_loaded_section, use_stats_ui},
    data::SectionPayload,
    sections::DashboardTab,
    state::{DashboardAction, SkillGap, WordSort},
};
use super::{recommendations::summarize, review::ReviewRecommendation, row::InsightRow};
use yew::prelude::*;

#[function_component(InsightExplorer)]
pub(crate) fn insight_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::Insights(records) = loaded.as_ref() else {
        unreachable!("Insights projection expected")
    };
    let summary = summarize(records);
    let on_explore = {
        let dashboard = dashboard.clone();
        Callback::from(move |skill: SkillGap| {
            dashboard.dispatch(DashboardAction::FocusSkill(skill))
        })
    };
    let on_review = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::SelectTab(DashboardTab::Hanzi));
            dashboard.dispatch(DashboardAction::ClearSkillGap);
            dashboard.dispatch(DashboardAction::WordSort(WordSort::Review));
        })
    };
    html! {
        <section data-stats-insights="" class="space-y-3">
            <header class="flex items-center justify-between rounded-xl border border-line bg-surface px-4 py-3">
                <h2 class="font-semibold">{"Next steps"}</h2>
                <span class="text-xs text-muted tabular-nums">{format!("{} words", records.len())}</span>
            </header>
            <div class="overflow-hidden rounded-xl border border-line bg-surface divide-y divide-line">
                {for summary.priorities.into_iter().enumerate().map(|(index, recommendation)| html! {
                    <InsightRow key={recommendation.title} rank={index + 1} title={recommendation.title.to_owned()}
                        count={recommendation.count} explanation={recommendation.explanation.to_owned()}
                        skill={Some(recommendation.skill)} on_explore={on_explore.clone()} />
                })}
                <ReviewRecommendation count={summary.review_count} on_open={on_review} />
            </div>
        </section>
    }
}
