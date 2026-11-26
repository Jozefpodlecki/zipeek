use std::{future::Future, pin::Pin};

mod collections;
mod source;
mod state;
mod types;

#[cfg(test)]
mod tests;

pub(super) use collections::{GrammarRecords, SessionRecords, WordRecords};
pub(super) use source::{DemoStatisticsSource, StatisticsSource};
#[cfg(test)]
pub(super) use source::{demo_grammar_for_tests, demo_sessions_for_tests, demo_words_for_tests};
pub(super) use state::{DataAction, SectionLoad, StatisticsDataState};
pub(super) use types::{DataError, GrammarRecord, SectionPayload, SessionRecord, WordRecord};

pub(super) type SectionFuture = Pin<Box<dyn Future<Output = Result<SectionPayload, DataError>>>>;
