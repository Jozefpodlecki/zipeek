use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum KnowledgeCategory {
    Pinyin,
    Hanzi,
    Vocabulary,
    Grammar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum KnowledgeDimension {
    Reading,
    Recognition,
    Listening,
    Speaking,
    Pronunciation,
    Handwriting,
    Writing,
    GrammarPattern,
}

impl KnowledgeCategory {
    pub fn dimensions(self) -> &'static [KnowledgeDimension] {
        use KnowledgeDimension::*;
        match self {
            Self::Pinyin => &[Reading, Listening, Speaking],
            Self::Hanzi => &[Recognition, Pronunciation, Handwriting],
            Self::Vocabulary => &[Recognition, Listening, Speaking, Writing],
            Self::Grammar => &[GrammarPattern],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct KnowledgeEntityId {
    pub category: KnowledgeCategory,
    pub catalog_key: String,
}

impl KnowledgeEntityId {
    pub fn new(category: KnowledgeCategory, catalog_key: impl Into<String>) -> Option<Self> {
        let catalog_key = catalog_key.into();
        if catalog_key.trim().is_empty() || catalog_key.len() > 256 {
            return None;
        }
        Some(Self {
            category,
            catalog_key,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelapsePhase {
    Acquiring,
    Mastered,
    Recovering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyProgress {
    pub day: i64,
    pub required_successes: u16,
    pub successes: u16,
    pub disqualified: bool,
    pub qualified: bool,
    pub debt_generated: bool,
    pub recovery_streak: u16,
    pub comeback_triggered: bool,
    pub seen_contexts: BTreeSet<String>,
    pub reading_successes: BTreeMap<String, u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeState {
    pub entity: KnowledgeEntityId,
    pub dimension: KnowledgeDimension,
    pub policy_epoch: u64,
    pub chunks: u16,
    pub debt: u16,
    pub phase: RelapsePhase,
    pub day: Option<DailyProgress>,
    pub seen_attempts: BTreeSet<String>,
    pub required_readings: BTreeSet<String>,
    pub observations: Vec<super::AssessmentEvent>,
}

impl KnowledgeState {
    pub fn new(
        entity: KnowledgeEntityId,
        dimension: KnowledgeDimension,
        policy_epoch: u64,
    ) -> Self {
        Self {
            entity,
            dimension,
            policy_epoch,
            chunks: 0,
            debt: 0,
            phase: RelapsePhase::Acquiring,
            day: None,
            seen_attempts: BTreeSet::new(),
            required_readings: BTreeSet::new(),
            observations: Vec::new(),
        }
    }

    pub fn with_required_readings(mut self, readings: impl IntoIterator<Item = String>) -> Self {
        self.required_readings = readings.into_iter().collect();
        self
    }
}
