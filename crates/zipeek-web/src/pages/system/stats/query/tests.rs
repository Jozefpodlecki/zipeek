use super::*;

#[cfg(test)]
mod tests {
    use super::super::super::data::{
        demo_grammar_for_tests, demo_sessions_for_tests, demo_words_for_tests,
    };
    use super::super::selectors::needs_review;
    use super::super::*;

    fn word_query(page: usize, limit: usize) -> WordQuery {
        WordQuery {
            search: String::new(),
            sort: WordSort::Alphabetical,
            gap: None,
            page: PageRequest::new(page, limit).unwrap(),
        }
    }

    #[test]
    fn remote_page_contract_rejects_partial_and_misaddressed_responses() {
        let request = PageRequest::new(1, 10).unwrap();
        let good = QueryPage {
            records: vec![1_u8, 2, 3],
            total: 13,
            offset: 10,
            limit: 10,
        };
        assert_eq!(good.validate(request), Ok(()));
        assert_eq!(
            QueryPage {
                offset: 0,
                ..good.clone()
            }
            .validate(request),
            Err(QueryError::InconsistentPage)
        );
        assert_eq!(
            QueryPage {
                records: vec![1_u8],
                ..good.clone()
            }
            .validate(request),
            Err(QueryError::InconsistentPage)
        );
        assert_eq!(
            QueryPage {
                total: 9,
                ..good.clone()
            }
            .validate(request),
            Err(QueryError::InconsistentPage)
        );
        let past_end: QueryPage<u8> = QueryPage {
            records: vec![],
            total: 9,
            offset: 10,
            limit: 10,
        };
        assert_eq!(past_end.validate(request), Ok(()));
    }

    #[test]
    fn page_request_validates_size_and_offset() {
        assert_eq!(PageRequest::new(0, 0), Err(QueryError::InvalidPageSize));
        assert_eq!(PageRequest::new(0, 101), Err(QueryError::InvalidPageSize));
        assert_eq!(
            PageRequest::new(usize::MAX, 100),
            Err(QueryError::InvalidOffset)
        );
        assert_eq!(
            PageRequest::new(10_001, 100),
            Err(QueryError::InvalidOffset)
        );
    }

    #[test]
    fn paged_words_keep_total_and_stable_boundaries() {
        let words = demo_words_for_tests();
        let first = query_words(&words, &word_query(0, 10)).unwrap();
        let last = query_words(&words, &word_query(23, 10)).unwrap();
        assert_eq!(
            (first.total, first.records.len(), first.page_count()),
            (240, 10, 24)
        );
        assert_eq!(last.records.len(), 10);
        assert!(
            first
                .records
                .iter()
                .all(|row| !last.records.iter().any(|other| row.id == other.id))
        );
        let empty = query_words(&words, &word_query(24, 10)).unwrap();
        assert_eq!((empty.total, empty.range()), (240, (0, 0)));
    }

    #[test]
    fn vocabulary_header_sort_directions_are_stable() {
        let words = demo_words_for_tests();
        let mut query = word_query(0, 100);
        query.sort = WordSort::Alphabetical;
        let first = query_words(&words, &query).unwrap();
        query.sort = WordSort::AlphabeticalReverse;
        let reverse = query_words(&words, &query).unwrap();
        assert_eq!(
            first.records.first().map(|row| &row.hanzi),
            reverse.records.last().map(|row| &row.hanzi)
        );
        query.sort = WordSort::Encounters;
        let high = query_words(&words, &query).unwrap();
        query.sort = WordSort::FewestEncounters;
        let low = query_words(&words, &query).unwrap();
        assert_eq!(
            high.records.first().map(|row| row.encounters),
            low.records.last().map(|row| row.encounters)
        );
        assert_eq!(
            high.records.last().map(|row| row.encounters),
            low.records.first().map(|row| row.encounters)
        );
    }

    #[test]
    fn grammar_and_history_queries_filter_before_paging() {
        let grammar = demo_grammar_for_tests();
        let history = demo_sessions_for_tests();
        let grammar_query = GrammarQuery {
            search: "Region 100".into(),
            region: GrammarRegion::All,
            sort: GrammarSort::TeachingOrder,
            page: PageRequest::new(0, 10).unwrap(),
        };
        let matches = query_grammar(&grammar, &grammar_query).unwrap();
        assert!(matches.total > 0);
        assert!(
            matches
                .records
                .iter()
                .all(|row| row.course.to_lowercase().contains("region 100"))
        );
        let history_query = HistoryQuery {
            search: String::new(),
            category: HistoryCategory::Games,
            page: PageRequest::new(0, 25).unwrap(),
        };
        let matches = query_history(&history, &history_query).unwrap();
        assert!(matches.total > 0);
        assert!(matches.records.iter().all(|row| row.category == "Games"));
    }

    #[test]
    fn query_state_rejects_superseded_completion_and_retries() {
        let mut state: QueryLoad<WordRecord> = QueryLoad::default();
        state.start(1);
        state.start(2);
        let demo = demo_words_for_tests();
        let page = query_words(&demo, &word_query(0, 10)).unwrap();
        state.complete(1, Ok(page.clone()));
        assert!(matches!(state, QueryLoad::Loading { generation: 2 }));
        state.complete(2, Err(QueryError::InvalidSearch));
        assert!(matches!(state, QueryLoad::Failed { generation: 2, .. }));
        state.start(3);
        state.complete(3, Ok(page));
        assert!(matches!(state, QueryLoad::Ready { generation: 3, .. }));
        state.invalidate();
        assert!(matches!(state, QueryLoad::Idle));
    }

    #[test]
    fn empty_and_single_record_pages_have_correct_ranges() {
        let rows = demo_words_for_tests();
        let query = WordQuery {
            search: "definitely-unmatched-phrase".into(),
            ..word_query(0, 10)
        };
        let page = query_words(&rows, &query).unwrap();
        assert_eq!(
            (page.total, page.page_count(), page.range()),
            (0, 1, (0, 0))
        );
        let only = vec![rows[0].clone()];
        let page = query_words(&only, &word_query(0, 10)).unwrap();
        assert_eq!(
            (page.total, page.page_count(), page.range()),
            (1, 1, (1, 1))
        );
        assert!(matches!(
            query_words(
                &rows,
                &WordQuery {
                    search: "x".repeat(257),
                    ..word_query(0, 10)
                }
            ),
            Err(QueryError::InvalidSearch)
        ));
    }

    #[test]
    fn skill_gap_and_review_order_are_deterministic() {
        let rows = demo_words_for_tests();
        let mut query = word_query(0, 100);
        query.gap = Some(SkillGap::Speaking);
        let gap = query_words(&rows, &query).unwrap();
        assert!(
            gap.records
                .iter()
                .all(|row| row.speaking != EvidenceStatus::Observed)
        );
        query.gap = None;
        query.sort = WordSort::Review;
        let review = query_words(&rows, &query).unwrap();
        assert!(
            review
                .records
                .windows(2)
                .all(|pair| needs_review(&pair[0]) || !needs_review(&pair[1]))
        );
    }

    #[test]
    fn chinese_and_case_insensitive_search_are_supported() {
        let words = demo_words_for_tests();
        let target = &words[0];
        let hanzi = query_words(
            &words,
            &WordQuery {
                search: target.hanzi.clone(),
                ..word_query(0, 10)
            },
        )
        .unwrap();
        assert!(hanzi.records.iter().any(|word| word.hanzi == target.hanzi));
        let latin = query_words(
            &words,
            &WordQuery {
                search: target.meaning.to_uppercase(),
                ..word_query(0, 10)
            },
        )
        .unwrap();
        assert!(latin.records.iter().any(|word| word.id == target.id));
    }

    #[test]
    fn ten_thousand_items_page_without_duplicate_ids_or_unstable_sorting() {
        let template = demo_words_for_tests();
        let mut rows = Vec::with_capacity(10_000);
        for index in 0..10_000 {
            let mut row = template[index % template.len()].clone();
            row.id = format!("word-{index:05}");
            rows.push(row);
        }
        let first = query_words(&rows, &word_query(0, 100)).unwrap();
        let second = query_words(&rows, &word_query(1, 100)).unwrap();
        let last = query_words(&rows, &word_query(99, 100)).unwrap();
        assert_eq!((first.total, first.page_count()), (10_000, 100));
        assert_eq!(last.records.len(), 100);
        let mut ids = std::collections::BTreeSet::new();
        for item in first
            .records
            .iter()
            .chain(second.records.iter())
            .chain(last.records.iter())
        {
            assert!(ids.insert(item.id.as_str()));
        }
        assert!(first == query_words(&rows, &word_query(0, 100)).unwrap());
    }
    #[test]
    fn grammar_region_search_sort_and_page_size_compose() {
        let rows = demo_grammar_for_tests();
        let mut request = GrammarQuery {
            search: String::new(),
            region: GrammarRegion::Regions81To100,
            sort: GrammarSort::TeachingOrder,
            page: PageRequest::new(0, 25).unwrap(),
        };
        let first = query_grammar(&rows, &request).unwrap();
        assert_eq!(first.total, 100);
        assert_eq!(first.records.len(), 25);
        assert!(
            first
                .records
                .iter()
                .all(|item| request.region.contains(&item.id))
        );
        request.sort = GrammarSort::MostAttempts;
        let by_attempts = query_grammar(&rows, &request).unwrap();
        assert!(
            by_attempts
                .records
                .windows(2)
                .all(|pair| pair[0].attempts >= pair[1].attempts)
        );
        request.sort = GrammarSort::PatternAscending;
        let ascending = query_grammar(&rows, &request).unwrap();
        assert!(
            ascending
                .records
                .windows(2)
                .all(|pair| pair[0].pattern <= pair[1].pattern)
        );
        request.search = "中文".to_owned();
        let matching = query_grammar(&rows, &request).unwrap();
        assert!(
            matching
                .records
                .iter()
                .all(|item| item.example.contains("中文") || item.pattern.contains("中文"))
        );
        request.region = GrammarRegion::Regions1To20;
        request.search = "no pattern has this exact phrase".to_owned();
        assert_eq!(query_grammar(&rows, &request).unwrap().total, 0);
    }

    #[test]
    fn select_all_uses_bounded_page_queries_and_stable_id_order() {
        let source = super::super::super::data::DemoStatisticsSource;
        let query = word_query(2, 10);
        let ids = futures::executor::block_on(matching_word_ids(&source, query, 250)).unwrap();
        assert_eq!(ids.len(), 240);
        assert_eq!(
            ids.len(),
            ids.iter().collect::<std::collections::BTreeSet<_>>().len()
        );
        let grammar_query = GrammarQuery {
            search: "Region 100".to_owned(),
            region: GrammarRegion::All,
            sort: GrammarSort::TeachingOrder,
            page: PageRequest::new(1, 8).unwrap(),
        };
        let grammar_ids =
            futures::executor::block_on(matching_grammar_ids(&source, grammar_query, 250)).unwrap();
        assert!(!grammar_ids.is_empty());
        assert!(grammar_ids.len() <= 250);
        assert_eq!(
            grammar_ids.len(),
            grammar_ids
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
    }

    #[test]
    fn select_all_rejects_large_matching_sets_before_creating_selection() {
        let source = super::super::super::data::DemoStatisticsSource;
        let result = futures::executor::block_on(matching_word_ids(&source, word_query(0, 10), 10));
        assert_eq!(result, Err(QueryError::TooManyMatches));
    }

    #[test]
    fn activity_summary_uses_all_filtered_sessions_not_a_paginated_slice() {
        let sessions = demo_sessions_for_tests();
        let query = ActivityQuery {
            search: String::new(),
            category: HistoryCategory::All,
            dates: RECENT_DAYS.map(str::to_owned),
        };
        let summary = query_activity(&sessions, &query).unwrap();
        let count = sessions
            .iter()
            .filter(|session| RECENT_DAYS.contains(&session.date.as_str()))
            .count();
        assert_eq!(summary.records.len(), 7);
        assert_eq!(
            summary.records.iter().map(|day| day.count).sum::<usize>(),
            count
        );
        let games = query_activity(
            &sessions,
            &ActivityQuery {
                category: HistoryCategory::Games,
                ..query
            },
        )
        .unwrap();
        assert!(games.records.iter().map(|day| day.count).sum::<usize>() <= count);
    }

    #[test]
    fn session_query_supports_activity_id_without_full_dataset_reads() {
        let sessions = demo_sessions_for_tests();
        let query = HistoryQuery {
            search: "province-quest".to_owned(),
            category: HistoryCategory::All,
            page: PageRequest::new(0, 8).unwrap(),
        };
        let page = query_history(&sessions, &query).unwrap();
        assert!(page.total > 0);
        assert!(
            page.records
                .iter()
                .all(|session| session.activity_id.contains("province-quest"))
        );
    }
}

#[cfg(test)]
mod activity_week_navigation_tests {
    use super::*;

    #[test]
    fn preview_activity_weeks_are_contiguous_and_nonoverlapping() {
        let current = activity_week_dates(0);
        let previous = activity_week_dates(1);
        assert_eq!(current[0], "2026-11-12");
        assert_eq!(current[6], "2026-11-18");
        assert_eq!(previous[6], "2026-11-11");
        assert_eq!(previous[0], "2026-11-05");
        assert!(previous.iter().all(|date| !current.contains(date)));
    }

    #[test]
    fn date_search_recognizes_iso_calendar_dates() {
        assert!(is_week_date("2026-11-18"));
        assert!(!is_week_date("listening"));
        assert!(!is_week_date("2026-99-99"));
    }
}

#[cfg(test)]
mod r593_page_boundary_tests {
    use super::*;

    #[test]
    fn malformed_pages_are_safe_and_rejected() {
        let page = QueryPage::<u8> {
            records: vec![],
            total: 0,
            offset: 0,
            limit: 0,
        };
        assert_eq!(page.page_count(), 1);
        assert_eq!(
            page.validate(PageRequest {
                offset: 0,
                limit: 0
            }),
            Err(QueryError::InconsistentPage)
        );
        let overflowing = QueryPage {
            records: vec![1_u8],
            total: usize::MAX,
            offset: usize::MAX,
            limit: 1,
        };
        assert_eq!(overflowing.range(), (usize::MAX, usize::MAX));
    }
}
