use super::super::{
    context::{use_loaded_section, use_stats_ui},
    data::SectionPayload,
    state::{DashboardAction, SkillGap},
};
use super::{card::SkillCard, evidence::counts_for, writing::WritingActivity};
use yew::prelude::*;

#[function_component(SkillsExplorer)]
pub(crate) fn skills_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::Skills(records) = loaded.as_ref() else {
        unreachable!("Skills projection expected")
    };
    let total = records.len();
    let writing = counts_for(records, SkillGap::Writing);
    let explore = {
        let dashboard = dashboard.clone();
        Callback::from(move |skill: SkillGap| {
            dashboard.dispatch(DashboardAction::FocusSkill(skill))
        })
    };
    let writing_view = {
        let dashboard = dashboard.clone();
        Callback::from(move |_: MouseEvent| {
            dashboard.dispatch(DashboardAction::FocusSkill(SkillGap::Writing))
        })
    };
    html! {
        <section data-stats-skills="" class="space-y-3">
            <header class="flex flex-wrap items-start justify-between gap-3 rounded-xl border border-line bg-surface px-4 py-3">
                <h2 class="font-semibold">{"Skill activity"}</h2>
                <span class="rounded-full border border-line bg-raised px-3 py-1 text-xs tabular-nums text-muted">{format!("{} words", total)}</span>
            </header>
            <div class="grid gap-3 md:grid-cols-2 2xl:grid-cols-4">
                {for [SkillGap::Recognition, SkillGap::Listening, SkillGap::Speaking, SkillGap::Sentence].into_iter().map(|skill| {
                    let counts = counts_for(records, skill);
                    html! { <SkillCard key={skill.label()} skill={skill} recorded={counts.recorded} review={counts.review} total={counts.total} on_explore={explore.clone()} /> }
                })}
            </div>
            <div class="flex flex-wrap items-center gap-x-5 gap-y-2 rounded-xl border border-line bg-surface px-4 py-3 text-xs text-muted" aria-label="Evidence coverage legend">
                <span class="inline-flex items-center gap-2"><span class="size-2.5 rounded-sm bg-accent" aria-hidden="true" />{"Recorded activity"}</span>
                <span class="inline-flex items-center gap-2"><span class="size-2.5 rounded-sm bg-amber-500" aria-hidden="true" />{"Review marker"}</span>
                <span class="inline-flex items-center gap-2"><span class="size-2.5 rounded-sm bg-raised ring-1 ring-line" aria-hidden="true" />{"No record"}</span>
            </div>
            <WritingActivity recorded={writing.recorded} review={writing.review} total={writing.total} on_explore={writing_view} />
        </section>
    }
}
