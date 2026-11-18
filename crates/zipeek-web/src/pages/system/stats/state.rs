use std::{collections::BTreeSet, rc::Rc};

use yew::Reducible;

use super::sections::DashboardTab;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreviewPhase {
    Closed,
    Loading { request_id: u64 },
    Open,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MetricId {
    Words,
    Encounters,
    Sessions,
    Coverage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MetricSelection {
    pub words: bool,
    pub encounters: bool,
    pub sessions: bool,
    pub coverage: bool,
}

impl Default for MetricSelection {
    fn default() -> Self {
        Self { words: true, encounters: true, sessions: false, coverage: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DashboardPreset {
    Focused,
    Activity,
    Detailed,
}

impl DashboardPreset {
    pub const fn metrics(self) -> MetricSelection {
        match self {
            Self::Focused => MetricSelection { words: true, encounters: true, sessions: false, coverage: false },
            Self::Activity => MetricSelection { words: true, encounters: false, sessions: true, coverage: false },
            Self::Detailed => MetricSelection { words: true, encounters: true, sessions: true, coverage: true },
        }
    }
}

pub(super) const PAGE_SIZES: [usize; 4] = [10, 25, 50, 100];
pub(super) const MAX_FILTERED_SELECTION: usize = 250;

impl MetricSelection {
    pub fn is_enabled(self, metric: MetricId) -> bool {
        match metric {
            MetricId::Words => self.words,
            MetricId::Encounters => self.encounters,
            MetricId::Sessions => self.sessions,
            MetricId::Coverage => self.coverage,
        }
    }

    fn toggle(&mut self, metric: MetricId) {
        let value = match metric {
            MetricId::Words => &mut self.words,
            MetricId::Encounters => &mut self.encounters,
            MetricId::Sessions => &mut self.sessions,
            MetricId::Coverage => &mut self.coverage,
        };
        *value = !*value;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DashboardState {
    pub phase: PreviewPhase,
    pub request_id: u64,
    pub tab: DashboardTab,
    pub evidence_page: usize,
    pub word_page_size: usize,
    pub history_page: usize,
    pub history_category: HistoryCategory,
    pub history_query: String,
    pub grammar_page: usize,
    pub word_query: String,
    pub grammar_query: String,
    pub word_sort: WordSort,
    pub skill_gap: Option<SkillGap>,
    pub selected_words: BTreeSet<String>,
    pub selected_patterns: BTreeSet<String>,
    pub selection_review: bool,
    pub metrics: MetricSelection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum HistoryCategory {
    #[default]
    All,
    Practice,
    Games,
}

impl HistoryCategory {
    pub const ALL: [Self; 3] = [Self::All, Self::Practice, Self::Games];

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All activity",
            Self::Practice => "Practice",
            Self::Games => "Games",
        }
    }

    pub fn matches(self, category: &str) -> bool {
        match self {
            Self::All => true,
            Self::Practice => category == "Practice",
            Self::Games => category == "Games",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SkillGap {
    Recognition,
    Listening,
    Speaking,
    Writing,
    Sentence,
}

impl SkillGap {
    pub const ALL: [Self; 5] = [Self::Recognition, Self::Listening, Self::Speaking, Self::Writing, Self::Sentence];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Recognition => "Recognition",
            Self::Listening => "Listening",
            Self::Speaking => "Speaking",
            Self::Writing => "Writing",
            Self::Sentence => "Sentence use",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum WordSort {
    #[default]
    Alphabetical,
    Encounters,
    Review,
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
            history_category: HistoryCategory::All,
            history_query: String::new(),
            grammar_page: 0,
            word_query: String::new(),
            grammar_query: String::new(),
            word_sort: WordSort::Alphabetical,
            skill_gap: None,
            selected_words: BTreeSet::new(),
            selected_patterns: BTreeSet::new(),
            selection_review: false,
            metrics: MetricSelection::default(),
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
    HistoryCategory(HistoryCategory),
    HistoryQuery(String),
    ToggleMetric(MetricId),
    SetMetricPreset(DashboardPreset),
    WordPageSize(usize),
    SelectMatchingWords(Vec<String>),
    SelectMatchingPatterns(Vec<String>),
    WordQuery(String),
    GrammarQuery(String),
    WordSort(WordSort),
    FocusSkill(SkillGap),
    ClearSkillGap,
    GrammarPage(usize),
    ToggleWord(String),
    TogglePattern(String),
    SelectVisibleWords(Vec<String>),
    SelectVisiblePatterns(Vec<String>),
    ClearSelection,
    ReviewSelection(bool),
}

impl Reducible for DashboardState {
    type Action = DashboardAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            DashboardAction::RequestPreview => {
                if next.phase != PreviewPhase::Closed {
                    return self;
                }
                next.request_id = next.request_id.wrapping_add(1);
                next.phase = PreviewPhase::Loading { request_id: next.request_id };
                next.tab = DashboardTab::Overview;
                next.evidence_page = 0;
                next.history_page = 0;
                next.history_category = HistoryCategory::All;
                next.history_query.clear();
                next.grammar_page = 0;
                next.word_query.clear();
                next.skill_gap = None;
                next.grammar_query.clear();
                next.selected_words.clear();
                next.selected_patterns.clear();
                next.selection_review = false;
            }
            DashboardAction::PreviewReady(id) => {
                if next.phase != (PreviewPhase::Loading { request_id: id }) {
                    return self;
                }
                next.phase = PreviewPhase::Open;
            }
            DashboardAction::ClosePreview => {
                next.request_id = next.request_id.wrapping_add(1);
                next.phase = PreviewPhase::Closed;
            }
            DashboardAction::SelectTab(tab) => {
                next.tab = tab;
                next.selection_review = false;
            },
            DashboardAction::EvidencePage(page) => next.evidence_page = page,
            DashboardAction::HistoryPage(page) => next.history_page = page,
            DashboardAction::HistoryCategory(category) => {
                next.history_category = category;
                next.history_page = 0;
            }
            DashboardAction::HistoryQuery(query) => {
                next.history_query = query;
                next.history_page = 0;
            }
            DashboardAction::ToggleMetric(metric) => next.metrics.toggle(metric),
            DashboardAction::SetMetricPreset(preset) => next.metrics = preset.metrics(),
            DashboardAction::WordPageSize(size) => {
                if PAGE_SIZES.contains(&size) {
                    next.word_page_size = size;
                    next.evidence_page = 0;
                }
            }
            DashboardAction::SelectMatchingWords(ids) => {
                if !ids.is_empty() && ids.len() <= MAX_FILTERED_SELECTION {
                    next.selected_words.extend(ids);
                }
            }
            DashboardAction::SelectMatchingPatterns(ids) => {
                if !ids.is_empty() && ids.len() <= MAX_FILTERED_SELECTION {
                    next.selected_patterns.extend(ids);
                }
            }
            DashboardAction::WordQuery(query) => {
                next.word_query = query;
                next.evidence_page = 0;
            }
            DashboardAction::GrammarQuery(query) => {
                next.grammar_query = query;
                next.grammar_page = 0;
            }
            DashboardAction::WordSort(sort) => {
                next.word_sort = sort;
                next.evidence_page = 0;
            }
            DashboardAction::GrammarPage(page) => next.grammar_page = page,
            DashboardAction::FocusSkill(skill) => {
                next.skill_gap = Some(skill);
                next.word_query.clear();
                next.evidence_page = 0;
                next.tab = DashboardTab::Hanzi;
            }
            DashboardAction::ClearSkillGap => {
                next.skill_gap = None;
                next.evidence_page = 0;
            },
            DashboardAction::ToggleWord(id) => {
                if !next.selected_words.remove(&id) { next.selected_words.insert(id); }
            }
            DashboardAction::TogglePattern(id) => {
                if !next.selected_patterns.remove(&id) { next.selected_patterns.insert(id); }
            }
            DashboardAction::SelectVisibleWords(ids) => {
                let all_selected = ids.iter().all(|id| next.selected_words.contains(id));
                for id in ids {
                    if all_selected { next.selected_words.remove(&id); }
                    else { next.selected_words.insert(id); }
                }
            }
            DashboardAction::SelectVisiblePatterns(ids) => {
                let all_selected = ids.iter().all(|id| next.selected_patterns.contains(id));
                for id in ids {
                    if all_selected { next.selected_patterns.remove(&id); }
                    else { next.selected_patterns.insert(id); }
                }
            }
            DashboardAction::ClearSelection => {
                next.selected_words.clear();
                next.selected_patterns.clear();
                next.selection_review = false;
            }
            DashboardAction::ReviewSelection(visible) => next.selection_review = visible,
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_filters_reset_pagination_without_changing_other_sections() {
        let state = Rc::new(DashboardState::default()).reduce(DashboardAction::HistoryPage(3));
        let state = state.reduce(DashboardAction::HistoryCategory(HistoryCategory::Games));
        assert_eq!(state.history_page, 0);
        assert_eq!(state.history_category, HistoryCategory::Games);
        let state = state.reduce(DashboardAction::HistoryQuery("pinyin".to_owned()));
        assert_eq!(state.history_query, "pinyin");
        assert_eq!(state.history_page, 0);
    }

    #[test]
    fn obsolete_preview_completion_does_not_reopen_closed_page() {
        let initial = Rc::new(DashboardState::default());
        let loading = initial.reduce(DashboardAction::RequestPreview);
        let request = loading.request_id;
        let closed = loading.reduce(DashboardAction::ClosePreview);
        assert_eq!(closed.reduce(DashboardAction::PreviewReady(request)).phase, PreviewPhase::Closed);
    }

    #[test]
    fn repeated_start_is_idempotent_and_new_request_is_fresh() {
        let first = Rc::new(DashboardState::default()).reduce(DashboardAction::RequestPreview);
        let repeated = first.clone().reduce(DashboardAction::RequestPreview);
        assert_eq!(first.request_id, repeated.request_id);
        let new_request = first.reduce(DashboardAction::ClosePreview).reduce(DashboardAction::RequestPreview);
        assert_ne!(new_request.request_id, repeated.request_id);
        assert_eq!(new_request.clone().reduce(DashboardAction::PreviewReady(repeated.request_id)).phase, new_request.phase);
        assert_eq!(new_request.clone().reduce(DashboardAction::PreviewReady(new_request.request_id)).phase, PreviewPhase::Open);
    }

    #[test]
    fn cancelling_loading_invalidates_completion() {
        let loading = Rc::new(DashboardState::default()).reduce(DashboardAction::RequestPreview);
        let id = loading.request_id;
        let closed = loading.reduce(DashboardAction::ClosePreview);
        assert_eq!(closed.reduce(DashboardAction::PreviewReady(id)).phase, PreviewPhase::Closed);
    }

    #[test]
    fn selection_survives_pages_and_query_resets_page() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::ToggleWord("学习".to_owned()))
            .reduce(DashboardAction::EvidencePage(4))
            .reduce(DashboardAction::WordQuery("学".to_owned()));
        assert!(state.selected_words.contains("学习"));
        assert_eq!(state.evidence_page, 0);
        assert_eq!(state.word_query, "学");
        let cleared = state.reduce(DashboardAction::ClearSelection);
        assert!(cleared.selected_words.is_empty());
    }

    #[test]
    fn visible_toggle_does_not_clear_off_page_selection() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::ToggleWord("朋友".to_owned()))
            .reduce(DashboardAction::SelectVisibleWords(vec!["学习".to_owned(), "老师".to_owned()]));
        assert_eq!(state.selected_words.len(), 3);
        let state = state.reduce(DashboardAction::SelectVisibleWords(vec!["学习".to_owned(), "老师".to_owned()]));
        assert_eq!(state.selected_words.len(), 1);
        assert!(state.selected_words.contains("朋友"));
    }

    #[test]
    fn metric_selection_is_reversible() {
        let initial = Rc::new(DashboardState::default());
        let changed = initial.clone().reduce(DashboardAction::ToggleMetric(MetricId::Words));
        assert!(!changed.metrics.words);
        assert!(changed.reduce(DashboardAction::ToggleMetric(MetricId::Words)).metrics.words);
    }
    #[test]
    fn presets_and_metric_toggles_remain_consistent() {
        let selected = Rc::new(DashboardState::default())
            .reduce(DashboardAction::SetMetricPreset(DashboardPreset::Detailed));
        assert!(selected.metrics.words && selected.metrics.coverage && selected.metrics.sessions);
        let changed = selected.reduce(DashboardAction::ToggleMetric(MetricId::Coverage));
        assert!(!changed.metrics.coverage);
        let focused = changed.reduce(DashboardAction::SetMetricPreset(DashboardPreset::Focused));
        assert_eq!(focused.metrics, MetricSelection::default());
    }

    #[test]
    fn page_size_is_bounded_and_resets_pagination() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::EvidencePage(3))
            .reduce(DashboardAction::WordPageSize(25));
        assert_eq!(state.word_page_size, 25);
        assert_eq!(state.evidence_page, 0);
        let invalid = state.reduce(DashboardAction::WordPageSize(50_000));
        assert_eq!(invalid.word_page_size, 25);
    }

    #[test]
    fn filtered_bulk_selection_is_explicit_bounded_and_preserves_prior_selection() {
        let original = Rc::new(DashboardState::default())
            .reduce(DashboardAction::ToggleWord("previous".into()));
        let oversized = (0..=MAX_FILTERED_SELECTION).map(|n| format!("w{n}")).collect();
        let rejected = original.clone().reduce(DashboardAction::SelectMatchingWords(oversized));
        assert_eq!(rejected.selected_words.len(), 1);
        let chosen = original.reduce(DashboardAction::SelectMatchingWords(vec!["new".into()]));
        assert!(chosen.selected_words.contains("previous"));
        assert!(chosen.selected_words.contains("new"));
    }

}
