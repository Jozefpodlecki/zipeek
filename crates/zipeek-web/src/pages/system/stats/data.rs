use std::{collections::BTreeMap, future::Future, pin::Pin, rc::Rc};

use yew::Reducible;

use super::{
    fixture::{EvidenceStatus, SAMPLE_RECORDS, SAMPLE_SESSIONS},
    grammar::GRAMMAR_SAMPLES,
    sections::DashboardTab,
};

pub(super) type SectionFuture = Pin<Box<dyn Future<Output = Result<SectionPayload, DataError>>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DataError {
    Unavailable,
    RejectedProjection,
    RequestFailed(String),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("Statistics data is unavailable."),
            Self::RejectedProjection => formatter.write_str("The section received an incompatible data response."),
            Self::RequestFailed(message) => formatter.write_str(message),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct WordRecord {
    pub id: String,
    pub hanzi: String,
    pub pinyin: String,
    pub meaning: String,
    pub encounters: u32,
    pub sessions: u32,
    pub days: u32,
    pub recognition: EvidenceStatus,
    pub listening: EvidenceStatus,
    pub speaking: EvidenceStatus,
    pub writing: EvidenceStatus,
    pub sentence: EvidenceStatus,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct GrammarRecord {
    pub id: String,
    pub pattern: String,
    pub description: String,
    pub example: String,
    pub attempts: u32,
    pub course: String,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct SessionRecord {
    pub id: String,
    pub date: String,
    pub activity_id: String,
    pub activity_title: String,
    pub category: String,
    pub observed: u32,
    pub correct: Option<u32>,
    pub focus: String,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum SectionPayload {
    Overview { words: Rc<Vec<WordRecord>>, sessions: Rc<Vec<SessionRecord>> },
    Words(Rc<Vec<WordRecord>>),
    Grammar(Rc<Vec<GrammarRecord>>),
    Skills(Rc<Vec<WordRecord>>),
    History(Rc<Vec<SessionRecord>>),
    Insights(Rc<Vec<WordRecord>>),
    Settings,
}

impl SectionPayload {
    fn tab(&self) -> DashboardTab {
        match self {
            Self::Overview { .. } => DashboardTab::Overview,
            Self::Words(_) => DashboardTab::Hanzi,
            Self::Grammar(_) => DashboardTab::Grammar,
            Self::Skills(_) => DashboardTab::Skills,
            Self::History(_) => DashboardTab::History,
            Self::Insights(_) => DashboardTab::Insights,
            Self::Settings => DashboardTab::Settings,
        }
    }
}

pub(super) trait StatisticsSource {
    fn load(&self, tab: DashboardTab) -> SectionFuture;
}

#[derive(Default)]
pub(super) struct DemoStatisticsSource;

impl StatisticsSource for DemoStatisticsSource {
    fn load(&self, tab: DashboardTab) -> SectionFuture {
        Box::pin(async move { Ok(demo_section(tab)) })
    }
}

fn demo_words() -> Rc<Vec<WordRecord>> {
    Rc::new(SAMPLE_RECORDS.iter().map(|row| WordRecord {
        id: row.hanzi.to_owned(), hanzi: row.hanzi.to_owned(),
        pinyin: row.pinyin.to_owned(), meaning: row.meaning.to_owned(),
        encounters: u32::from(row.encounters), sessions: u32::from(row.sessions), days: u32::from(row.days),
        recognition: row.recognition, listening: row.listening, speaking: row.speaking,
        writing: row.writing, sentence: row.sentence,
    }).collect())
}

fn demo_sessions() -> Rc<Vec<SessionRecord>> {
    Rc::new(SAMPLE_SESSIONS.iter().map(|row| {
        let activity = row.activity();
        SessionRecord {
            id: row.id.to_owned(), date: row.date.to_owned(), activity_id: row.activity_id.to_owned(),
            activity_title: activity.title.to_owned(), category: activity.category.to_owned(),
            observed: u32::from(row.observed), correct: row.correct.map(u32::from), focus: row.focus.to_owned(),
        }
    }).collect())
}

#[cfg(test)]
pub(super) fn demo_words_for_tests() -> Rc<Vec<WordRecord>> {
    demo_words()
}

#[cfg(test)]
pub(super) fn demo_sessions_for_tests() -> Rc<Vec<SessionRecord>> {
    demo_sessions()
}

#[cfg(test)]
pub(super) fn demo_grammar_for_tests() -> Rc<Vec<GrammarRecord>> {
    if let SectionPayload::Grammar(records) = demo_section(DashboardTab::Grammar) {
        records
    } else {
        unreachable!()
    }
}

fn demo_section(tab: DashboardTab) -> SectionPayload {
    match tab {
        DashboardTab::Overview => SectionPayload::Overview { words: demo_words(), sessions: demo_sessions() },
        DashboardTab::Hanzi => SectionPayload::Words(demo_words()),
        DashboardTab::Grammar => SectionPayload::Grammar(Rc::new(GRAMMAR_SAMPLES.iter().map(|row| GrammarRecord {
            id: row.id.to_owned(), pattern: row.pattern.to_owned(), description: row.description.to_owned(),
            example: row.example.to_owned(), attempts: u32::from(row.attempts), course: row.course.to_owned(),
        }).collect())),
        DashboardTab::Skills => SectionPayload::Skills(demo_words()),
        DashboardTab::History => SectionPayload::History(demo_sessions()),
        DashboardTab::Insights => SectionPayload::Insights(demo_words()),
        DashboardTab::Settings => SectionPayload::Settings,
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum SectionLoad {
    Loading { generation: u64 },
    Ready(Rc<SectionPayload>),
    Failed { generation: u64, error: DataError },
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct StatisticsDataState {
    sections: BTreeMap<&'static str, SectionLoad>,
}

impl StatisticsDataState {
    pub fn get(&self, tab: DashboardTab) -> Option<&SectionLoad> {
        self.sections.get(tab.key())
    }
}

pub(super) enum DataAction {
    Start { tab: DashboardTab, generation: u64 },
    Loaded { tab: DashboardTab, generation: u64, payload: SectionPayload },
    Failed { tab: DashboardTab, generation: u64, error: DataError },
    Invalidate { tab: DashboardTab, generation: u64 },
    Reset,
}

impl Reducible for StatisticsDataState {
    type Action = DataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            DataAction::Reset => next.sections.clear(),
            DataAction::Start { tab, generation } => {
                next.sections.insert(tab.key(), SectionLoad::Loading { generation });
            }
            DataAction::Loaded { tab, generation, payload } => {
                if !matches!(next.get(tab), Some(SectionLoad::Loading { generation: active }) if *active == generation) {
                    return self;
                }
                if payload.tab() != tab {
                    next.sections.insert(tab.key(), SectionLoad::Failed {
                        generation, error: DataError::RejectedProjection,
                    });
                } else {
                    next.sections.insert(tab.key(), SectionLoad::Ready(Rc::new(payload)));
                }
            }
            DataAction::Failed { tab, generation, error } => {
                if !matches!(next.get(tab), Some(SectionLoad::Loading { generation: active }) if *active == generation) {
                    return self;
                }
                next.sections.insert(tab.key(), SectionLoad::Failed { generation, error });
            }
            DataAction::Invalidate { tab, generation } => {
                if matches!(next.get(tab), Some(SectionLoad::Loading { generation: active }) if *active == generation) {
                    next.sections.remove(tab.key());
                }
            }
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tabs_produce_typed_projections() {
        for tab in DashboardTab::ALL {
            let data = demo_section(tab);
            match (tab, data) {
                (DashboardTab::Overview, SectionPayload::Overview { words, sessions }) => {
                    assert_eq!(words.len(), 30);
                    assert_eq!(sessions.len(), 72);
                }
                (DashboardTab::Hanzi, SectionPayload::Words(words)) => assert_eq!(words.len(), 30),
                (DashboardTab::Grammar, SectionPayload::Grammar(patterns)) => assert_eq!(patterns.len(), 16),
                (DashboardTab::Skills, SectionPayload::Skills(_))
                | (DashboardTab::History, SectionPayload::History(_))
                | (DashboardTab::Insights, SectionPayload::Insights(_))
                | (DashboardTab::Settings, SectionPayload::Settings) => {}
                _ => panic!("Incorrect section projection"),
            }
        }
    }

    #[test]
    fn stale_results_and_errors_do_not_override_current_data() {
        let state = Rc::new(StatisticsDataState::default());
        let state = state.reduce(DataAction::Start { tab: DashboardTab::Hanzi, generation: 1 });
        let state = state.reduce(DataAction::Start { tab: DashboardTab::Hanzi, generation: 2 });
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::Hanzi, generation: 1, payload: SectionPayload::Words(demo_words()),
        });
        assert!(matches!(state.get(DashboardTab::Hanzi), Some(SectionLoad::Loading { generation: 2 })));
        let state = state.reduce(DataAction::Failed {
            tab: DashboardTab::Hanzi, generation: 1, error: DataError::RequestFailed("stale".into()),
        });
        assert!(matches!(state.get(DashboardTab::Hanzi), Some(SectionLoad::Loading { generation: 2 })));
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::Hanzi, generation: 2, payload: SectionPayload::Words(demo_words()),
        });
        assert!(matches!(state.get(DashboardTab::Hanzi), Some(SectionLoad::Ready(_))));
    }

    #[test]
    fn failed_section_can_be_retried_without_changing_other_sections() {
        let state = Rc::new(StatisticsDataState::default());
        let state = state.reduce(DataAction::Start { tab: DashboardTab::Overview, generation: 1 });
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::Overview, generation: 1,
            payload: SectionPayload::Overview { words: demo_words(), sessions: demo_sessions() },
        });
        let state = state.reduce(DataAction::Start { tab: DashboardTab::History, generation: 2 });
        let state = state.reduce(DataAction::Failed {
            tab: DashboardTab::History, generation: 2,
            error: DataError::Unavailable,
        });
        assert!(matches!(state.get(DashboardTab::History), Some(SectionLoad::Failed { .. })));
        let state = state.reduce(DataAction::Start { tab: DashboardTab::History, generation: 3 });
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::History, generation: 3, payload: SectionPayload::History(demo_sessions()),
        });
        assert!(matches!(state.get(DashboardTab::History), Some(SectionLoad::Ready(_))));
        assert!(matches!(state.get(DashboardTab::Overview), Some(SectionLoad::Ready(_))));
    }

    #[test]
    fn empty_section_payload_is_valid_data_not_an_error() {
        let state = Rc::new(StatisticsDataState::default());
        let state = state.reduce(DataAction::Start { tab: DashboardTab::Hanzi, generation: 5 });
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::Hanzi, generation: 5,
            payload: SectionPayload::Words(Rc::new(Vec::new())),
        });
        assert!(matches!(state.get(DashboardTab::Hanzi), Some(SectionLoad::Ready(payload))
            if matches!(payload.as_ref(), SectionPayload::Words(records) if records.is_empty())));
    }

    #[test]
    fn incompatible_section_is_rejected_before_rendering() {
        let state = Rc::new(StatisticsDataState::default());
        let state = state.reduce(DataAction::Start { tab: DashboardTab::History, generation: 7 });
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::History, generation: 7,
            payload: SectionPayload::Words(demo_words()),
        });
        assert!(matches!(state.get(DashboardTab::History), Some(SectionLoad::Failed {
            error: DataError::RejectedProjection, ..
        })));
    }

    #[test]
    fn reset_discards_pending_loads() {
        let state = Rc::new(StatisticsDataState::default());
        let state = state.reduce(DataAction::Start { tab: DashboardTab::History, generation: 11 });
        let state = state.reduce(DataAction::Reset);
        let state = state.reduce(DataAction::Loaded {
            tab: DashboardTab::History, generation: 11, payload: SectionPayload::History(demo_sessions()),
        });
        assert!(state.get(DashboardTab::History).is_none());
    }
}
