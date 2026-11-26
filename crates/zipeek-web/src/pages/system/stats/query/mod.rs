use std::{future::Future, pin::Pin};

use super::{
    data::{DataError, GrammarRecord, SessionRecord, WordRecord},
    fixture::EvidenceStatus,
    state::{GrammarRegion, GrammarSort, HistoryCategory, SkillGap, WordSort},
};

pub(super) const MAX_QUERY_PAGE_SIZE: usize = 100;
pub(super) const MAX_QUERY_OFFSET: usize = 1_000_000;
pub(super) const MAX_QUERY_LENGTH: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PageRequest {
    pub offset: usize,
    pub limit: usize,
}

impl PageRequest {
    pub fn new(page: usize, limit: usize) -> Result<Self, QueryError> {
        if !(1..=MAX_QUERY_PAGE_SIZE).contains(&limit) {
            return Err(QueryError::InvalidPageSize);
        }
        let offset = page.checked_mul(limit).ok_or(QueryError::InvalidOffset)?;
        if offset > MAX_QUERY_OFFSET {
            return Err(QueryError::InvalidOffset);
        }
        Ok(Self { offset, limit })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum QueryError {
    InvalidPageSize,
    InvalidOffset,
    InvalidSearch,
    Source(DataError),
    TooManyMatches,
    InconsistentPage,
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPageSize => formatter.write_str("Invalid Statistics page size"),
            Self::InvalidOffset => formatter.write_str("Statistics page offset is too large"),
            Self::InvalidSearch => formatter.write_str("Statistics search is too long"),
            Self::Source(error) => write!(formatter, "{error}"),
            Self::TooManyMatches => formatter
                .write_str("Too many matching records. Narrow your filters before selecting all."),
            Self::InconsistentPage => formatter
                .write_str("The Statistics source returned an inconsistent page. Please retry."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct QueryPage<T> {
    pub records: Vec<T>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

impl<T> QueryPage<T> {
    pub fn page_count(&self) -> usize {
        self.total.div_ceil(self.limit.max(1)).max(1)
    }

    pub fn range(&self) -> (usize, usize) {
        if self.records.is_empty() {
            (0, 0)
        } else {
            (
                self.offset.saturating_add(1),
                self.offset.saturating_add(self.records.len()),
            )
        }
    }

    pub fn validate(&self, requested: PageRequest) -> Result<(), QueryError> {
        let expected = self
            .total
            .saturating_sub(requested.offset)
            .min(requested.limit);
        if !(1..=MAX_QUERY_PAGE_SIZE).contains(&requested.limit)
            || requested.offset > MAX_QUERY_OFFSET
            || self.offset != requested.offset
            || self.limit != requested.limit
            || self.records.len() != expected
        {
            return Err(QueryError::InconsistentPage);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WordQuery {
    pub search: String,
    pub sort: WordSort,
    pub gap: Option<SkillGap>,
    pub page: PageRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GrammarQuery {
    pub search: String,
    pub region: GrammarRegion,
    pub sort: GrammarSort,
    pub page: PageRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HistoryQuery {
    pub search: String,
    pub category: HistoryCategory,
    pub page: PageRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActivityQuery {
    pub search: String,
    pub category: HistoryCategory,
    pub dates: [String; 7],
}

pub(super) trait QueryRequest {
    fn page_request(&self) -> Option<PageRequest>;
}

impl QueryRequest for WordQuery {
    fn page_request(&self) -> Option<PageRequest> {
        Some(self.page)
    }
}

impl QueryRequest for GrammarQuery {
    fn page_request(&self) -> Option<PageRequest> {
        Some(self.page)
    }
}

impl QueryRequest for HistoryQuery {
    fn page_request(&self) -> Option<PageRequest> {
        Some(self.page)
    }
}

impl QueryRequest for ActivityQuery {
    fn page_request(&self) -> Option<PageRequest> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActivityBucket {
    pub date: String,
    pub count: usize,
    pub practice: usize,
    pub games: usize,
}

pub(super) const RECENT_DAYS: [&str; 7] = [
    "2026-11-12",
    "2026-11-13",
    "2026-11-14",
    "2026-11-15",
    "2026-11-16",
    "2026-11-17",
    "2026-11-18",
];

pub(super) fn is_week_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

pub(super) fn activity_week_dates(weeks_before_latest: usize) -> [String; 7] {
    use chrono::{Duration, NaiveDate};
    let latest = NaiveDate::parse_from_str(RECENT_DAYS[6], "%Y-%m-%d")
        .expect("Static preview activity date must be valid");
    let end = latest - Duration::days((weeks_before_latest as i64) * 7);
    std::array::from_fn(|index| {
        (end - Duration::days((6 - index) as i64))
            .format("%Y-%m-%d")
            .to_string()
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum QueryLoad<T> {
    Idle,
    Loading { generation: u64 },
    Ready { generation: u64, page: QueryPage<T> },
    Failed { generation: u64, error: QueryError },
}

impl<T> Default for QueryLoad<T> {
    fn default() -> Self {
        Self::Idle
    }
}

impl<T> QueryLoad<T> {
    pub fn start(&mut self, generation: u64) {
        *self = Self::Loading { generation };
    }

    pub fn complete(&mut self, generation: u64, result: Result<QueryPage<T>, QueryError>) {
        if !matches!(self, Self::Loading { generation: active } if *active == generation) {
            return;
        }
        *self = match result {
            Ok(page) => Self::Ready { generation, page },
            Err(error) => Self::Failed { generation, error },
        };
    }

    pub fn invalidate(&mut self) {
        *self = Self::Idle;
    }
}

pub(super) type QueryFuture<T> = Pin<Box<dyn Future<Output = Result<QueryPage<T>, QueryError>>>>;

pub(super) trait StatisticsQuerySource {
    fn words(&self, query: WordQuery) -> QueryFuture<WordRecord>;
    fn grammar(&self, query: GrammarQuery) -> QueryFuture<GrammarRecord>;
    fn history(&self, query: HistoryQuery) -> QueryFuture<SessionRecord>;
    fn activity(&self, query: ActivityQuery) -> QueryFuture<ActivityBucket>;
}

mod selectors;
pub(super) use selectors::{
    matching_grammar_ids, matching_word_ids, query_activity, query_grammar, query_history,
    query_words,
};

#[cfg(test)]
mod tests;
