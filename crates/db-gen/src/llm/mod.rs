pub mod classification;
pub mod prompts;
pub mod batch;
mod recovery;
mod cache_store;
mod failure_report;
pub(crate) mod client;
mod error;
mod streaming;
mod usage;

pub use client::LlmClient;
pub use error::{LlmError, LlmResult};
pub use streaming::{classify, consume_progress, consume_silent};
pub use usage::TurnUsage;