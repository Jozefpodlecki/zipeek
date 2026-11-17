use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct CatalogRecord {
    pub(crate) id: String,
    pub(crate) parent_id: Option<String>,
    pub(crate) hanzi: String,
    pub(crate) pinyin: String,
    pub(crate) english: String,
    pub(crate) capital_hanzi: Option<String>,
    pub(crate) capital_pinyin: Option<String>,
    pub(crate) kind: String,
    pub(crate) source_names: Vec<String>,
}
