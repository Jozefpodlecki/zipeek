#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_review_requires_items_and_bulk_selection_ignores_blank_ids() {
        let state = Rc::new(DashboardState::default());
        let state = state.reduce(DashboardAction::ReviewSelection(true));
        assert!(!state.selection_review);
        let state = state.reduce(DashboardAction::SelectMatchingWords(vec![
            "".into(),
            " ".into(),
            "word-1".into(),
        ]));
        assert_eq!(state.selected_words.len(), 1);
        let state = state.reduce(DashboardAction::ReviewSelection(true));
        assert!(state.selection_review);
    }

    #[test]
    fn grammar_view_toggle_keeps_query_and_selection() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::GrammarQuery("comparison".to_owned()))
            .reduce(DashboardAction::TogglePattern("g001-p01".to_owned()));
        let changed = state.reduce(DashboardAction::GrammarView(GrammarView::Cards));
        assert_eq!(changed.grammar_view, GrammarView::Cards);
        assert_eq!(changed.grammar_query, "comparison");
        assert!(changed.selected_patterns.contains("g001-p01"));
    }

    #[test]
    fn grammar_filters_and_sort_reset_paging_without_erasing_selections() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::TogglePattern("g081-p01".to_owned()))
            .reduce(DashboardAction::GrammarPage(3));
        let state = state.reduce(DashboardAction::GrammarRegion(
            GrammarRegion::Regions81To100,
        ));
        assert_eq!(state.grammar_page, 0);
        assert!(state.selected_patterns.contains("g081-p01"));
        let state = state.reduce(DashboardAction::GrammarPageSize(25));
        assert_eq!(state.grammar_page_size, 25);
        let state = state.reduce(DashboardAction::GrammarSort(GrammarSort::MostAttempts));
        assert_eq!(state.grammar_sort, GrammarSort::MostAttempts);
        assert_eq!(state.grammar_page, 0);
        let state = state.reduce(DashboardAction::GrammarPageSize(500));
        assert_eq!(state.grammar_page_size, 25);
        assert!(state.selected_patterns.contains("g081-p01"));
        assert!(!GrammarRegion::Regions81To100.contains("g001-p01"));
        assert!(GrammarRegion::Regions81To100.contains("g100-p05"));
    }

    #[test]
    fn grammar_detail_survives_selection_but_closes_on_page_and_query_changes() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::ToggleGrammarDetail("g01".into()));
        assert_eq!(state.grammar_detail_id.as_deref(), Some("g01"));
        let state = state.reduce(DashboardAction::TogglePattern("g01".into()));
        assert_eq!(state.grammar_detail_id.as_deref(), Some("g01"));
        let state = state.reduce(DashboardAction::GrammarPage(1));
        assert_eq!(state.grammar_detail_id, None);
        let state = state.reduce(DashboardAction::ToggleGrammarDetail("g03".into()));
        let state = state.reduce(DashboardAction::GrammarQuery("comparison".into()));
        assert!(state.grammar_detail_id.is_none());
    }

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
    fn history_view_and_day_filter_preserve_search_contract() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::HistoryPage(5))
            .reduce(DashboardAction::HistoryView(HistoryView::Table))
            .reduce(DashboardAction::HistoryQuery("2026-11-16".to_owned()));
        assert_eq!(state.history_view, HistoryView::Table);
        assert_eq!(state.history_page, 0);
        assert_eq!(state.history_query, "2026-11-16");
        let reset = state.reduce(DashboardAction::HistoryQuery(String::new()));
        assert!(reset.history_query.is_empty());
        assert_eq!(reset.history_view, HistoryView::Table);
    }

    #[test]
    fn history_week_navigation_resets_date_filter_but_preserves_category_and_view() {
        let state = Rc::new(DashboardState::default())
            .reduce(DashboardAction::HistoryCategory(HistoryCategory::Practice))
            .reduce(DashboardAction::HistoryView(HistoryView::Table))
            .reduce(DashboardAction::HistoryQuery("2026-11-18".to_owned()))
            .reduce(DashboardAction::HistoryPage(3))
            .reduce(DashboardAction::HistoryWeek(1));
        assert_eq!(state.history_week_offset, 1);
        assert_eq!(state.history_page, 0);
        assert!(state.history_query.is_empty());
        assert_eq!(state.history_category, HistoryCategory::Practice);
        assert_eq!(state.history_view, HistoryView::Table);
        let state = state
            .reduce(DashboardAction::HistoryQuery("listening".to_owned()))
            .reduce(DashboardAction::HistoryWeek(2));
        assert_eq!(state.history_query, "listening");
        let state = state.reduce(DashboardAction::HistoryWeek(100));
        assert_eq!(state.history_week_offset, 2);
    }

    #[test]
    fn obsolete_preview_completion_does_not_reopen_closed_page() {
        let initial = Rc::new(DashboardState::default());
        let loading = initial.reduce(DashboardAction::RequestPreview);
        let request = loading.request_id;
        let closed = loading.reduce(DashboardAction::ClosePreview);
        assert_eq!(
            closed.reduce(DashboardAction::PreviewReady(request)).phase,
            PreviewPhase::Closed
        );
    }

    #[test]
    fn repeated_start_is_idempotent_and_new_request_is_fresh() {
        let first = Rc::new(DashboardState::default()).reduce(DashboardAction::RequestPreview);
        let repeated = first.clone().reduce(DashboardAction::RequestPreview);
        assert_eq!(first.request_id, repeated.request_id);
        let new_request = first
            .reduce(DashboardAction::ClosePreview)
            .reduce(DashboardAction::RequestPreview);
        assert_ne!(new_request.request_id, repeated.request_id);
        assert_eq!(
            new_request
                .clone()
                .reduce(DashboardAction::PreviewReady(repeated.request_id))
                .phase,
            new_request.phase
        );
        assert_eq!(
            new_request
                .clone()
                .reduce(DashboardAction::PreviewReady(new_request.request_id))
                .phase,
            PreviewPhase::Open
        );
    }

    #[test]
    fn cancelling_loading_invalidates_completion() {
        let loading = Rc::new(DashboardState::default()).reduce(DashboardAction::RequestPreview);
        let id = loading.request_id;
        let closed = loading.reduce(DashboardAction::ClosePreview);
        assert_eq!(
            closed.reduce(DashboardAction::PreviewReady(id)).phase,
            PreviewPhase::Closed
        );
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
            .reduce(DashboardAction::SelectVisibleWords(vec![
                "学习".to_owned(),
                "老师".to_owned(),
            ]));
        assert_eq!(state.selected_words.len(), 3);
        let state = state.reduce(DashboardAction::SelectVisibleWords(vec![
            "学习".to_owned(),
            "老师".to_owned(),
        ]));
        assert_eq!(state.selected_words.len(), 1);
        assert!(state.selected_words.contains("朋友"));
    }

    #[test]
    fn tile_layout_is_reversible_and_presets_reset_order() {
        let starting = Rc::new(DashboardState::default());
        let changed = starting
            .reduce(DashboardAction::ToggleTile(TileId::GameTypes))
            .reduce(DashboardAction::MoveTile(TileId::GameTypes, -1));
        assert!(changed.tiles.visible.contains(&TileId::GameTypes));
        assert_ne!(changed.tiles.order, TileSelection::default().order);
        let reset = changed.reduce(DashboardAction::ResetTiles);
        assert_eq!(reset.tiles, TileSelection::default());
        assert_eq!(reset.metrics, MetricSelection::default());
        let detailed = reset.reduce(DashboardAction::SetMetricPreset(DashboardPreset::Detailed));
        assert_eq!(detailed.tiles.visible.len(), 20);
        let closed = detailed.reduce(DashboardAction::ClosePreview);
        assert_eq!(closed.tiles, TileSelection::default());
    }

    #[test]
    fn metric_selection_is_reversible() {
        let initial = Rc::new(DashboardState::default());
        let changed = initial
            .clone()
            .reduce(DashboardAction::ToggleMetric(MetricId::Words));
        assert!(!changed.metrics.words);
        assert!(
            changed
                .reduce(DashboardAction::ToggleMetric(MetricId::Words))
                .metrics
                .words
        );
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
        let oversized = (0..=MAX_FILTERED_SELECTION)
            .map(|n| format!("w{n}"))
            .collect();
        let rejected = original
            .clone()
            .reduce(DashboardAction::SelectMatchingWords(oversized));
        assert_eq!(rejected.selected_words.len(), 1);
        let chosen = original.reduce(DashboardAction::SelectMatchingWords(vec!["new".into()]));
        assert!(chosen.selected_words.contains("previous"));
        assert!(chosen.selected_words.contains("new"));
    }

    #[test]
    fn all_selection_entry_points_reject_blank_record_ids() {
        let state = Rc::new(DashboardState::default());
        let unchanged = state
            .clone()
            .reduce(DashboardAction::ToggleWord("  ".into()));
        assert!(Rc::ptr_eq(&state, &unchanged));
        let unchanged = state
            .clone()
            .reduce(DashboardAction::TogglePattern(String::new()));
        assert!(Rc::ptr_eq(&state, &unchanged));
        let words = state
            .clone()
            .reduce(DashboardAction::SelectVisibleWords(vec![
                " ".into(),
                "word-1".into(),
                "".into(),
            ]));
        assert_eq!(words.selected_words.len(), 1);
        assert!(words.selected_words.contains("word-1"));
        let patterns = words.reduce(DashboardAction::SelectVisiblePatterns(vec![
            "  ".into(),
            "pattern-1".into(),
        ]));
        assert_eq!(patterns.selected_patterns.len(), 1);
        assert!(patterns.selected_patterns.contains("pattern-1"));
        let cleared = patterns.reduce(DashboardAction::SelectVisiblePatterns(vec![" ".into()]));
        assert!(cleared.selected_patterns.contains("pattern-1"));
        let unchanged = cleared
            .clone()
            .reduce(DashboardAction::ToggleGrammarDetail(" ".into()));
        assert!(Rc::ptr_eq(&cleared, &unchanged));
    }
}
