use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::{context::use_loaded_section, data::SectionPayload};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DashboardTab {
    Overview,
    Hanzi,
    Grammar,
    Skills,
    History,
    Insights,
    Settings,
}

impl DashboardTab {
    pub(super) const ALL: [Self; 7] = [Self::Overview, Self::Hanzi, Self::Grammar, Self::Skills, Self::History, Self::Insights, Self::Settings];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Hanzi => "Hanzi & words",
            Self::Grammar => "Grammar",
            Self::Skills => "Skills",
            Self::History => "History",
            Self::Insights => "Insights",
            Self::Settings => "Settings",
        }
    }

    pub(super) const fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Hanzi => "hanzi",
            Self::Grammar => "grammar",
            Self::Skills => "skills",
            Self::History => "history",
            Self::Insights => "insights",
            Self::Settings => "settings",
        }
    }

    pub(super) const fn icon(self) -> IconData {
        match self {
            Self::Overview => IconData::LUCIDE_LAYOUT_GRID,
            Self::Hanzi => IconData::LUCIDE_BOOK_OPEN,
            Self::Grammar => IconData::LUCIDE_BOOK_OPEN,
            Self::Skills => IconData::LUCIDE_LAYERS,
            Self::History => IconData::LUCIDE_CLOCK,
            Self::Insights => IconData::LUCIDE_SEARCH,
            Self::Settings => IconData::LUCIDE_SETTINGS,
        }
    }
}

#[derive(Properties, PartialEq)]
struct MetricProps {
    label: &'static str,
    value: String,
    explanation: &'static str,
}

#[function_component(Metric)]
fn metric(props: &MetricProps) -> Html {
    html! {
        <article class="rounded-xl border border-line bg-surface/95 p-3 shadow-sm">
            <div class="flex items-center justify-between gap-2">
                <h2 class="text-xs font-semibold text-muted">{props.label}</h2>
                <span class="group relative inline-flex">
                    <button type="button" aria-label={format!("About {}", props.label)} title={props.explanation}
                        class="rounded-md p-1 text-muted hover:text-ink focus-visible:outline-2 focus-visible:outline-accent">
                        <Icon data={IconData::LUCIDE_INFO} width="15px" height="15px" />
                    </button>
                    <span class="pointer-events-none absolute right-0 top-full z-10 hidden w-48 rounded-lg border border-line bg-surface p-2 text-xs text-ink shadow-lg group-hover:block group-focus-within:block">{props.explanation}</span>
                </span>
            </div>
            <p class="mt-2 text-xl font-semibold tabular-nums">{&props.value}</p>
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct OverviewCardsProps {
    pub words: bool,
    pub encounters: bool,
    pub sessions: bool,
    pub assessment: bool,
}

#[function_component(OverviewCards)]
pub(super) fn overview_cards(props: &OverviewCardsProps) -> Html {
    let loaded = use_loaded_section();
    let SectionPayload::Overview { words, .. } = loaded.as_ref() else { unreachable!("Overview projection expected") };
    let encounters: u32 = words.iter().map(|row| row.encounters).sum();
    let sessions: u32 = words.iter().map(|row| row.sessions).sum();
    html! {
        <div class="grid grid-cols-2 gap-3 lg:grid-cols-4" data-stats-overview-cards="">
            if props.words { <Metric label="Words encountered" value={words.len().to_string()} explanation="Distinct sample vocabulary identities with recorded fictional activity." /> }
            if props.encounters { <Metric label="Encounters" value={encounters.to_string()} explanation="Sample attempts across vocabulary identities; not a proficiency score." /> }
            if props.sessions { <Metric label="Vocabulary session appearances" value={sessions.to_string()} explanation="The sum of per-word session appearances. One session can contribute more than one appearance." /> }
            if props.assessment { <Metric label="Skills tracked" value="5" explanation="Five illustrative observation categories, not a proficiency assessment." /> }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_tabs_have_unique_keys_and_grammar_is_separate() {
        let mut keys = std::collections::HashSet::new();
        for tab in DashboardTab::ALL {
            assert!(keys.insert(tab.key()));
        }
        assert_eq!(DashboardTab::ALL.len(), 7);
        assert_eq!(DashboardTab::Grammar.key(), "grammar");
        assert_eq!(DashboardTab::ALL.last().map(|tab| tab.key()), Some("settings"));
    }
}
