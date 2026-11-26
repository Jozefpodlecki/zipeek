use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DatabaseInfo {
    pub engine: String,
    pub native_engine: String,
    pub storage_compatibility: String,
    pub row_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HistoryRow {
    pub session_id: String,
    pub evidence: String,
}
