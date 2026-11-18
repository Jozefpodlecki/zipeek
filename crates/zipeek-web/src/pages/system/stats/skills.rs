use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{
    context::{use_stats_ui, use_loaded_section},
    data::{SectionPayload, WordRecord},
    fixture::EvidenceStatus,
    state::{DashboardAction, SkillGap},
};

fn status_for(record: &WordRecord, skill: SkillGap) -> EvidenceStatus {
    match skill {
        SkillGap::Recognition => record.recognition,
        SkillGap::Listening => record.listening,
        SkillGap::Speaking => record.speaking,
        SkillGap::Writing => record.writing,
        SkillGap::Sentence => record.sentence,
    }
}

#[function_component(SkillsExplorer)]
pub(super) fn skills_explorer() -> Html {
    let dashboard = use_stats_ui().0;
    let loaded = use_loaded_section();
    let SectionPayload::Skills(records) = loaded.as_ref() else { unreachable!("Skills projection expected") };
    html! {
        <section data-stats-skills="" class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {for SkillGap::ALL.into_iter().map(|skill| {
                let observed = records.iter().filter(|record| status_for(record, skill) == EvidenceStatus::Observed).count();
                let review = records.iter().filter(|record| status_for(record, skill) == EvidenceStatus::NeedsReview).count();
                let missing = records.len() - observed - review;
                let view = {
                    let dashboard = dashboard.clone();
                    Callback::from(move |_: MouseEvent| dashboard.dispatch(DashboardAction::FocusSkill(skill)))
                };
                let icon = match skill {
                    SkillGap::Recognition => IconData::LUCIDE_BOOK_OPEN,
                    SkillGap::Listening => IconData::LUCIDE_CLOCK,
                    SkillGap::Speaking => IconData::LUCIDE_MESSAGE_CIRCLE,
                    SkillGap::Writing => IconData::LUCIDE_PENCIL,
                    SkillGap::Sentence => IconData::LUCIDE_LAYERS,
                };
                html! {
                    <article key={skill.label()} class="flex min-h-44 flex-col rounded-xl border border-line bg-surface p-4">
                        <div class="flex items-center gap-2"><Icon data={icon} width="19px" /><h2 class="font-semibold">{skill.label()}</h2></div>
                        <p class="mt-3 text-lg font-semibold tabular-nums">{format!("{} of {} observed", observed, records.len())}</p>
                        <div class="mt-3 flex h-2 overflow-hidden rounded-full bg-raised" role="img" aria-label={format!("{} observed, {} need review, {} have no observations", observed, review, missing)}>
                            <div class="bg-accent" style={format!("width: {}%", observed * 100 / records.len().max(1))} />
                            <div class="bg-amber-500" style={format!("width: {}%", review * 100 / records.len().max(1))} />
                        </div>
                        <p class="mt-2 text-xs text-muted">{format!("{} to review · {} without observations", review, missing)}</p>
                        <button type="button" onclick={view} class="mt-auto inline-flex min-h-9 items-center gap-2 self-start rounded-lg border border-line px-3 text-sm hover:bg-raised">
                            {"Inspect gap"}<Icon data={IconData::LUCIDE_ARROW_RIGHT} width="15px" />
                        </button>
                    </article>
                }
            })}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_skill_has_mutually_exclusive_counts() {
        let records = super::super::data::demo_words_for_tests();
        for skill in SkillGap::ALL {
            let observed = records.iter().filter(|item| status_for(item,skill) == EvidenceStatus::Observed).count();
            let review = records.iter().filter(|item| status_for(item,skill) == EvidenceStatus::NeedsReview).count();
            let missing = records.iter().filter(|item| status_for(item,skill) == EvidenceStatus::NeedsEvidence).count();
            assert_eq!(observed+review+missing,records.len());
        }
    }
}
