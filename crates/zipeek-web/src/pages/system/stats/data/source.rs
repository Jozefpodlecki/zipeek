use super::super::{
    query::{
        ActivityBucket, ActivityQuery, GrammarQuery, HistoryQuery, QueryError, QueryFuture,
        StatisticsQuerySource, WordQuery, query_activity, query_grammar, query_history,
        query_words,
    },
    sections::DashboardTab,
    simulator::{demo_grammar, demo_sessions, demo_words},
};
use super::{
    DataError, GrammarRecord, GrammarRecords, SectionFuture, SectionPayload, SessionRecord,
    SessionRecords, WordRecord, WordRecords,
};

pub(in super::super) trait StatisticsSource: StatisticsQuerySource {
    fn load(&self, tab: DashboardTab) -> SectionFuture;
}

#[derive(Default)]
pub(in super::super) struct DemoStatisticsSource;

impl StatisticsSource for DemoStatisticsSource {
    fn load(&self, tab: DashboardTab) -> SectionFuture {
        Box::pin(async move { Ok(demo_section(tab)) })
    }
}

impl StatisticsQuerySource for DemoStatisticsSource {
    fn words(&self, query: WordQuery) -> QueryFuture<WordRecord> {
        Box::pin(async move { query_words(&demo_words(), &query) })
    }

    fn grammar(&self, query: GrammarQuery) -> QueryFuture<GrammarRecord> {
        Box::pin(async move {
            let SectionPayload::Grammar(records) = demo_section(DashboardTab::Grammar) else {
                return Err(QueryError::Source(DataError::RejectedProjection));
            };
            query_grammar(&records, &query)
        })
    }

    fn history(&self, query: HistoryQuery) -> QueryFuture<SessionRecord> {
        Box::pin(async move { query_history(&demo_sessions(), &query) })
    }

    fn activity(&self, query: ActivityQuery) -> QueryFuture<ActivityBucket> {
        Box::pin(async move { query_activity(&demo_sessions(), &query) })
    }
}

#[cfg(test)]
pub(in super::super) fn demo_words_for_tests() -> WordRecords {
    demo_words()
}

#[cfg(test)]
pub(in super::super) fn demo_sessions_for_tests() -> SessionRecords {
    demo_sessions()
}

#[cfg(test)]
pub(in super::super) fn demo_grammar_for_tests() -> GrammarRecords {
    if let SectionPayload::Grammar(records) = demo_section(DashboardTab::Grammar) {
        records
    } else {
        unreachable!()
    }
}

pub(super) fn demo_section(tab: DashboardTab) -> SectionPayload {
    match tab {
        DashboardTab::Overview => SectionPayload::Overview {
            words: demo_words(),
            sessions: demo_sessions(),
        },
        DashboardTab::Hanzi => SectionPayload::Words(demo_words()),
        DashboardTab::Grammar => SectionPayload::Grammar(demo_grammar()),
        DashboardTab::Skills => SectionPayload::Skills(demo_words()),
        DashboardTab::History => SectionPayload::History(demo_sessions()),
        DashboardTab::Insights => SectionPayload::Insights(demo_words()),
        DashboardTab::Settings => SectionPayload::Settings,
    }
}
