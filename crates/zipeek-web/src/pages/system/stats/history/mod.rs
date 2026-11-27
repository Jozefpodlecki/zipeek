mod day;
mod entry;
mod page;
mod table_row;
mod toolbar;
pub(super) use page::{HistoryExplorer, display_timestamp};

#[cfg(test)]
mod tests {
    use super::super::{
        data::demo_sessions_for_tests,
        query::{RECENT_DAYS, query_activity, query_history},
    };
    use super::super::{
        query::{ActivityQuery, HistoryQuery, PageRequest},
        state::HistoryCategory,
    };
    use super::page::{PAGE_SIZE, summary_search};
    use super::*;

    #[test]
    fn date_selection_preserves_weekly_context_and_filters_session_page() {
        let sessions = demo_sessions_for_tests();
        let chosen = "2026-11-16";
        assert!(summary_search(chosen).is_empty());
        assert_eq!(summary_search("pinyin"), "pinyin");
        let page = query_history(
            &sessions,
            &HistoryQuery {
                search: chosen.to_owned(),
                category: HistoryCategory::All,
                page: PageRequest::new(0, PAGE_SIZE).unwrap(),
            },
        )
        .unwrap();
        assert!(page.total > 0);
        assert!(page.records.iter().all(|session| session.date == chosen));
        let week = query_activity(
            &sessions,
            &ActivityQuery {
                search: summary_search(chosen),
                category: HistoryCategory::All,
                dates: RECENT_DAYS.map(str::to_owned),
            },
        )
        .unwrap();
        assert!(
            week.records
                .iter()
                .any(|day| day.date == chosen && day.count > 0)
        );
        assert!(
            week.records
                .iter()
                .any(|day| day.date != chosen && day.count > 0)
        );
        assert!(
            week.records
                .iter()
                .all(|day| day.practice + day.games == day.count)
        );
    }

    #[test]
    fn history_sessions_have_full_timestamps_and_diverse_recent_exercises() {
        let sessions = demo_sessions_for_tests();
        assert!(sessions.len() > 120);
        assert!(
            sessions
                .iter()
                .all(|session| session.started_at.starts_with(&session.date))
        );
        assert!(
            sessions
                .iter()
                .all(|session| session.started_at.ends_with('Z'))
        );
        assert!(sessions.iter().all(|session| session.duration_minutes > 0));
        let newest = query_history(
            &sessions,
            &HistoryQuery {
                search: String::new(),
                category: HistoryCategory::All,
                page: PageRequest::new(0, 24).unwrap(),
            },
        )
        .unwrap();
        let categories: std::collections::BTreeSet<_> = newest
            .records
            .iter()
            .map(|session| session.category.as_str())
            .collect();
        assert!(categories.contains("Practice"));
        assert!(categories.contains("Games"));
        assert!(
            newest
                .records
                .windows(2)
                .all(|pair| pair[0].started_at >= pair[1].started_at)
        );
    }

    #[test]
    fn history_filtering_and_activity_summary_are_query_driven() {
        let sessions = demo_sessions_for_tests();
        let query = HistoryQuery {
            search: "pinyin".to_owned(),
            category: HistoryCategory::Practice,
            page: PageRequest::new(0, PAGE_SIZE).unwrap(),
        };
        let page = query_history(&sessions, &query).unwrap();
        assert!(
            page.records
                .iter()
                .all(|session| session.category == "Practice")
        );
        assert!(
            page.records
                .iter()
                .any(|session| session.activity_id == "type-pinyin")
        );
        let summary = query_activity(
            &sessions,
            &ActivityQuery {
                search: String::new(),
                category: HistoryCategory::All,
                dates: RECENT_DAYS.map(str::to_owned),
            },
        )
        .unwrap();
        assert_eq!(
            summary.records.iter().map(|day| day.count).sum::<usize>(),
            sessions
                .iter()
                .filter(|session| RECENT_DAYS.contains(&session.date.as_str()))
                .count()
        );
    }
}
