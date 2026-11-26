use super::super::fixture::EvidenceStatus;
use super::super::sections::DashboardTab;
use super::{GrammarRecords, SessionRecords, WordRecords};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum DataError {
    Unavailable,
    RejectedProjection,
    RequestFailed(String),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("Statistics data is unavailable."),
            Self::RejectedProjection => {
                formatter.write_str("The section received an incompatible data response.")
            }
            Self::RequestFailed(message) => formatter.write_str(message),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(in super::super) struct WordRecord {
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
pub(in super::super) struct GrammarRecord {
    pub id: String,
    pub pattern: String,
    pub description: String,
    pub example: String,
    pub attempts: u32,
    pub course: String,
    pub canonical_id: Option<String>,
    pub course_id: Option<String>,
    pub assessed_attempts: Option<u32>,
    pub teaching_review_status: String,
}

#[derive(Clone, PartialEq, Eq)]
pub(in super::super) struct SessionRecord {
    pub id: String,
    pub date: String,
    pub started_at: String,
    pub duration_minutes: u32,
    pub activity_id: String,
    pub activity_title: String,
    pub category: String,
    pub observed: u32,
    pub correct: Option<u32>,
    pub focus: String,
}

#[derive(Clone, PartialEq, Eq)]
pub(in super::super) enum SectionPayload {
    Overview {
        words: WordRecords,
        sessions: SessionRecords,
    },
    Words(WordRecords),
    Grammar(GrammarRecords),
    Skills(WordRecords),
    History(SessionRecords),
    Insights(WordRecords),
    Settings,
}

impl SectionPayload {
    pub(super) fn tab(&self) -> DashboardTab {
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
