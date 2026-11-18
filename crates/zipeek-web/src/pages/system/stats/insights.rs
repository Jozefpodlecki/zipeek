use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{context::{use_stats_ui, use_loaded_section}, data::SectionPayload, fixture::EvidenceStatus, sections::DashboardTab, state::{DashboardAction, SkillGap, WordSort}};

#[function_component(InsightExplorer)]
pub(super) fn insight_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::Insights(records) = loaded.as_ref() else { unreachable!("Insights projection expected") };
    let without_listening = records.iter().filter(|record| record.listening == EvidenceStatus::NeedsEvidence).count();
    let without_speaking = records.iter().filter(|record| record.speaking == EvidenceStatus::NeedsEvidence).count();
    let review = records.iter().filter(|record| {
        [record.recognition, record.listening, record.speaking, record.writing, record.sentence].contains(&EvidenceStatus::NeedsReview)
    }).count();
    html! {
        <section data-stats-insights="" class="grid gap-3 md:grid-cols-3">
            {for [
                ("Listening evidence", without_listening, "words have no sample listening observations", "Coverage gap", Some(SkillGap::Listening)),
                ("Speaking evidence", without_speaking, "words have no sample speaking observations", "Coverage gap", Some(SkillGap::Speaking)),
                ("Review candidates", review, "words have a flagged sample observation", "Review", None),
            ].into_iter().map(|(title, count, explanation, kind, skill)| {
                let dashboard = dashboard.clone();
                let action = Callback::from(move |_: MouseEvent| {
                    if let Some(skill) = skill {
                        dashboard.dispatch(DashboardAction::FocusSkill(skill));
                    } else {
                        dashboard.dispatch(DashboardAction::SelectTab(DashboardTab::Hanzi));
                        dashboard.dispatch(DashboardAction::ClearSkillGap);
                        dashboard.dispatch(DashboardAction::WordSort(WordSort::Review));
                    }
                });
                html! {
                    <article key={title} class="flex min-h-44 flex-col rounded-xl border border-line bg-surface p-4">
                        <span class="text-xs text-muted">{kind}</span>
                        <div class="mt-2 text-3xl font-bold tabular-nums">{count}</div>
                        <h2 class="mt-2 font-semibold">{title}</h2>
                        <p class="mt-1 flex-1 text-xs text-muted">{explanation}</p>
                        <button type="button" onclick={action} class="mt-3 inline-flex min-h-9 items-center gap-2 self-start rounded-lg border border-line px-3 text-sm hover:bg-raised">
                            {"View words"}<Icon data={IconData::LUCIDE_ARROW_RIGHT} width="15px" />
                        </button>
                    </article>
                }
            })}
        </section>
    }
}
