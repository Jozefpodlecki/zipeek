use std::{collections::BTreeSet, rc::Rc};

use yew::Reducible;

use super::{
    sections::DashboardTab,
    tiles::{TileId, TileSelection},
};
mod filters;
mod metrics;
pub(super) use filters::{
    GrammarRegion, GrammarSort, GrammarView, HistoryCategory, HistoryView, SkillGap, WordSort,
};
pub(super) use metrics::{
    DashboardPreset, MAX_FILTERED_SELECTION, MetricId, MetricSelection, PAGE_SIZES,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreviewPhase {
    Closed,
    Loading { request_id: u64 },
    Open,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DashboardState {
    pub phase: PreviewPhase,
    pub request_id: u64,
    pub tab: DashboardTab,
    pub evidence_page: usize,
    pub word_page_size: usize,
    pub history_page: usize,
    pub history_week_offset: usize,
    pub history_category: HistoryCategory,
    pub history_view: HistoryView,
    pub history_query: String,
    pub grammar_page: usize,
    pub grammar_page_size: usize,
    pub grammar_sort: GrammarSort,
    pub grammar_view: GrammarView,
    pub grammar_region: GrammarRegion,
    pub word_query: String,
    pub grammar_query: String,
    pub grammar_detail_id: Option<String>,
    pub word_sort: WordSort,
    pub skill_gap: Option<SkillGap>,
    pub selected_words: BTreeSet<String>,
    pub selected_patterns: BTreeSet<String>,
    pub selection_review: bool,
    pub metrics: MetricSelection,
    pub tiles: TileSelection,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            phase: PreviewPhase::Closed,
            request_id: 0,
            tab: DashboardTab::Overview,
            evidence_page: 0,
            word_page_size: 10,
            history_page: 0,
            history_week_offset: 0,
            history_category: HistoryCategory::All,
            history_view: HistoryView::Timeline,
            history_query: String::new(),
            grammar_page: 0,
            grammar_page_size: 10,
            grammar_sort: GrammarSort::TeachingOrder,
            grammar_view: GrammarView::Table,
            grammar_region: GrammarRegion::All,
            word_query: String::new(),
            grammar_query: String::new(),
            grammar_detail_id: None,
            word_sort: WordSort::Alphabetical,
            skill_gap: None,
            selected_words: BTreeSet::new(),
            selected_patterns: BTreeSet::new(),
            selection_review: false,
            metrics: MetricSelection::default(),
            tiles: TileSelection::default(),
        }
    }
}

pub(super) enum DashboardAction {
    RequestPreview,
    PreviewReady(u64),
    ClosePreview,
    SelectTab(DashboardTab),
    EvidencePage(usize),
    HistoryPage(usize),
    HistoryWeek(usize),
    HistoryCategory(HistoryCategory),
    HistoryView(HistoryView),
    HistoryQuery(String),
    ToggleMetric(MetricId),
    ToggleTile(TileId),
    MoveTile(TileId, i8),
    ResetTiles,
    SetMetricPreset(DashboardPreset),
    WordPageSize(usize),
    SelectMatchingWords(Vec<String>),
    SelectMatchingPatterns(Vec<String>),
    WordQuery(String),
    GrammarQuery(String),
    ToggleGrammarDetail(String),
    WordSort(WordSort),
    FocusSkill(SkillGap),
    ClearSkillGap,
    GrammarPage(usize),
    GrammarPageSize(usize),
    GrammarSort(GrammarSort),
    GrammarView(GrammarView),
    GrammarRegion(GrammarRegion),
    ToggleWord(String),
    TogglePattern(String),
    SelectVisibleWords(Vec<String>),
    SelectVisiblePatterns(Vec<String>),
    ClearSelection,
    ReviewSelection(bool),
}

mod transitions;

#[cfg(test)]
mod tests;
