use super::*;

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
                next.phase = PreviewPhase::Loading {
                    request_id: next.request_id,
                };
                next.tab = DashboardTab::Overview;
                next.evidence_page = 0;
                next.history_page = 0;
                next.history_week_offset = 0;
                next.history_category = HistoryCategory::All;
                next.history_view = HistoryView::Timeline;
                next.history_query.clear();
                next.grammar_page = 0;
                next.grammar_page_size = 10;
                next.grammar_sort = GrammarSort::TeachingOrder;
                next.grammar_region = GrammarRegion::All;
                next.word_query.clear();
                next.skill_gap = None;
                next.grammar_query.clear();
                next.grammar_detail_id = None;
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
                next.metrics = MetricSelection::default();
                next.tiles = TileSelection::default();
            }
            DashboardAction::SelectTab(tab) => {
                next.tab = tab;
                if tab != DashboardTab::Grammar {
                    next.grammar_detail_id = None;
                }
                next.selection_review = false;
            }
            DashboardAction::EvidencePage(page) => next.evidence_page = page,
            DashboardAction::HistoryPage(page) => next.history_page = page,
            DashboardAction::HistoryWeek(week) => {
                if week <= 12 {
                    next.history_week_offset = week;
                    next.history_page = 0;
                    if super::super::query::is_week_date(&next.history_query) {
                        next.history_query.clear();
                    }
                }
            }
            DashboardAction::HistoryView(view) => next.history_view = view,
            DashboardAction::HistoryCategory(category) => {
                next.history_category = category;
                next.history_page = 0;
            }
            DashboardAction::HistoryQuery(query) => {
                next.history_query = query;
                next.history_page = 0;
            }
            DashboardAction::ToggleMetric(metric) => next.metrics.toggle(metric),
            DashboardAction::ToggleTile(tile) => next.tiles.toggle(tile),
            DashboardAction::MoveTile(tile, direction) => next.tiles.move_tile(tile, direction),
            DashboardAction::ResetTiles => {
                next.metrics = MetricSelection::default();
                next.tiles = TileSelection::default();
            }
            DashboardAction::SetMetricPreset(preset) => {
                next.metrics = preset.metrics();
                next.tiles = TileSelection::preset(preset);
            }
            DashboardAction::WordPageSize(size) => {
                if PAGE_SIZES.contains(&size) {
                    next.word_page_size = size;
                    next.evidence_page = 0;
                }
            }
            DashboardAction::SelectMatchingWords(ids) => {
                if !ids.is_empty() && ids.len() <= MAX_FILTERED_SELECTION {
                    next.selected_words
                        .extend(ids.into_iter().filter(|id| !id.trim().is_empty()));
                }
            }
            DashboardAction::SelectMatchingPatterns(ids) => {
                if !ids.is_empty() && ids.len() <= MAX_FILTERED_SELECTION {
                    next.selected_patterns
                        .extend(ids.into_iter().filter(|id| !id.trim().is_empty()));
                }
            }
            DashboardAction::WordQuery(query) => {
                next.word_query = query;
                next.evidence_page = 0;
            }
            DashboardAction::GrammarQuery(query) => {
                next.grammar_detail_id = None;
                next.grammar_query = query;
                next.grammar_page = 0;
            }
            DashboardAction::WordSort(sort) => {
                next.word_sort = sort;
                next.evidence_page = 0;
            }
            DashboardAction::GrammarPageSize(size) => {
                if [10, 25, 50, 100].contains(&size) {
                    next.grammar_page_size = size;
                    next.grammar_page = 0;
                    next.grammar_detail_id = None;
                }
            }
            DashboardAction::GrammarView(view) => next.grammar_view = view,
            DashboardAction::GrammarSort(sort) => {
                next.grammar_sort = sort;
                next.grammar_page = 0;
                next.grammar_detail_id = None;
            }
            DashboardAction::GrammarRegion(region) => {
                next.grammar_region = region;
                next.grammar_page = 0;
                next.grammar_detail_id = None;
            }
            DashboardAction::GrammarPage(page) => {
                next.grammar_page = page;
                next.grammar_detail_id = None;
            }
            DashboardAction::ToggleGrammarDetail(id) => {
                if id.trim().is_empty() {
                    return self;
                }
                next.grammar_detail_id = if next.grammar_detail_id.as_ref() == Some(&id) {
                    None
                } else {
                    Some(id)
                };
            }
            DashboardAction::FocusSkill(skill) => {
                next.skill_gap = Some(skill);
                next.word_query.clear();
                next.evidence_page = 0;
                next.tab = DashboardTab::Hanzi;
            }
            DashboardAction::ClearSkillGap => {
                next.skill_gap = None;
                next.evidence_page = 0;
            }
            DashboardAction::ToggleWord(id) => {
                if id.trim().is_empty() {
                    return self;
                }
                if !next.selected_words.remove(&id) {
                    next.selected_words.insert(id);
                }
            }
            DashboardAction::TogglePattern(id) => {
                if id.trim().is_empty() {
                    return self;
                }
                if !next.selected_patterns.remove(&id) {
                    next.selected_patterns.insert(id);
                }
            }
            DashboardAction::SelectVisibleWords(ids) => {
                let ids = ids
                    .into_iter()
                    .filter(|id| !id.trim().is_empty())
                    .collect::<Vec<_>>();
                if ids.is_empty() {
                    return self;
                }
                let all_selected = ids.iter().all(|id| next.selected_words.contains(id));
                for id in ids {
                    if all_selected {
                        next.selected_words.remove(&id);
                    } else {
                        next.selected_words.insert(id);
                    }
                }
            }
            DashboardAction::SelectVisiblePatterns(ids) => {
                let ids = ids
                    .into_iter()
                    .filter(|id| !id.trim().is_empty())
                    .collect::<Vec<_>>();
                if ids.is_empty() {
                    return self;
                }
                let all_selected = ids.iter().all(|id| next.selected_patterns.contains(id));
                for id in ids {
                    if all_selected {
                        next.selected_patterns.remove(&id);
                    } else {
                        next.selected_patterns.insert(id);
                    }
                }
            }
            DashboardAction::ClearSelection => {
                next.selected_words.clear();
                next.selected_patterns.clear();
                next.selection_review = false;
            }
            DashboardAction::ReviewSelection(visible) => {
                next.selection_review = visible
                    && (!next.selected_words.is_empty() || !next.selected_patterns.is_empty());
            }
        }
        Rc::new(next)
    }
}
