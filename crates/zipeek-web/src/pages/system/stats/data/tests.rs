use super::super::fixture::SAMPLE_ACTIVITIES;
use super::super::sections::DashboardTab;
use super::super::simulator::{demo_sessions, demo_words};
use super::source::demo_section;
use super::*;
use std::rc::Rc;
use yew::Reducible;

#[test]
fn all_tabs_produce_typed_projections() {
    for tab in DashboardTab::ALL {
        let data = demo_section(tab);
        match (tab, data) {
            (DashboardTab::Overview, SectionPayload::Overview { words, sessions }) => {
                assert_eq!(words.len(), 240);
                assert_eq!(sessions.len(), 292);
            }
            (DashboardTab::Hanzi, SectionPayload::Words(words)) => assert_eq!(words.len(), 240),
            (DashboardTab::Grammar, SectionPayload::Grammar(patterns)) => {
                assert_eq!(patterns.len(), 550)
            }
            (DashboardTab::Skills, SectionPayload::Skills(_))
            | (DashboardTab::History, SectionPayload::History(_))
            | (DashboardTab::Insights, SectionPayload::Insights(_))
            | (DashboardTab::Settings, SectionPayload::Settings) => {}
            _ => panic!("Incorrect section projection"),
        }
    }
}

#[test]
fn demo_history_preserves_activity_scoring_and_unique_identities() {
    let rows = demo_sessions_for_tests();
    let mut ids = std::collections::BTreeSet::new();
    for row in rows.iter() {
        assert!(ids.insert(&row.id));
        assert!(row.started_at.starts_with(&row.date));
        assert!(row.duration_minutes > 0);
        let activity = SAMPLE_ACTIVITIES
            .iter()
            .find(|item| item.id == row.activity_id)
            .unwrap();
        assert_eq!(row.correct.is_some(), activity.can_show_score);
        assert!(row.correct.is_none_or(|correct| correct <= row.observed));
    }
    let activity_ids: std::collections::BTreeSet<_> =
        rows.iter().map(|item| item.activity_id.as_str()).collect();
    assert_eq!(activity_ids.len(), SAMPLE_ACTIVITIES.len());
}

#[test]
fn stale_results_and_errors_do_not_override_current_data() {
    let state = Rc::new(StatisticsDataState::default());
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::Hanzi,
        generation: 1,
    });
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::Hanzi,
        generation: 2,
    });
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::Hanzi,
        generation: 1,
        payload: SectionPayload::Words(demo_words()),
    });
    assert!(matches!(
        state.get(DashboardTab::Hanzi),
        Some(SectionLoad::Loading { generation: 2 })
    ));
    let state = state.reduce(DataAction::Failed {
        tab: DashboardTab::Hanzi,
        generation: 1,
        error: DataError::RequestFailed("stale".into()),
    });
    assert!(matches!(
        state.get(DashboardTab::Hanzi),
        Some(SectionLoad::Loading { generation: 2 })
    ));
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::Hanzi,
        generation: 2,
        payload: SectionPayload::Words(demo_words()),
    });
    assert!(matches!(
        state.get(DashboardTab::Hanzi),
        Some(SectionLoad::Ready(_))
    ));
}

#[test]
fn failed_section_can_be_retried_without_changing_other_sections() {
    let state = Rc::new(StatisticsDataState::default());
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::Overview,
        generation: 1,
    });
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::Overview,
        generation: 1,
        payload: SectionPayload::Overview {
            words: demo_words(),
            sessions: demo_sessions(),
        },
    });
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::History,
        generation: 2,
    });
    let state = state.reduce(DataAction::Failed {
        tab: DashboardTab::History,
        generation: 2,
        error: DataError::Unavailable,
    });
    assert!(matches!(
        state.get(DashboardTab::History),
        Some(SectionLoad::Failed { .. })
    ));
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::History,
        generation: 3,
    });
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::History,
        generation: 3,
        payload: SectionPayload::History(demo_sessions()),
    });
    assert!(matches!(
        state.get(DashboardTab::History),
        Some(SectionLoad::Ready(_))
    ));
    assert!(matches!(
        state.get(DashboardTab::Overview),
        Some(SectionLoad::Ready(_))
    ));
}

#[test]
fn empty_section_payload_is_valid_data_not_an_error() {
    let state = Rc::new(StatisticsDataState::default());
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::Hanzi,
        generation: 5,
    });
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::Hanzi,
        generation: 5,
        payload: SectionPayload::Words(WordRecords::new(Vec::new())),
    });
    assert!(
        matches!(state.get(DashboardTab::Hanzi), Some(SectionLoad::Ready(payload))
            if matches!(payload.as_ref(), SectionPayload::Words(records) if records.is_empty()))
    );
}

#[test]
fn incompatible_section_is_rejected_before_rendering() {
    let state = Rc::new(StatisticsDataState::default());
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::History,
        generation: 7,
    });
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::History,
        generation: 7,
        payload: SectionPayload::Words(demo_words()),
    });
    assert!(matches!(
        state.get(DashboardTab::History),
        Some(SectionLoad::Failed {
            error: DataError::RejectedProjection,
            ..
        })
    ));
}

#[test]
fn reset_discards_pending_loads() {
    let state = Rc::new(StatisticsDataState::default());
    let state = state.reduce(DataAction::Start {
        tab: DashboardTab::History,
        generation: 11,
    });
    let state = state.reduce(DataAction::Reset);
    let state = state.reduce(DataAction::Loaded {
        tab: DashboardTab::History,
        generation: 11,
        payload: SectionPayload::History(demo_sessions()),
    });
    assert!(state.get(DashboardTab::History).is_none());
}
