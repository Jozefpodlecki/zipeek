use crate::contexts::database::Dictionary;
use std::collections::BTreeSet;
use zipeek_core::{Word, WordId};

use super::ranking;

pub(super) async fn load(
    dictionary: &Dictionary,
    mode: &str,
    ids: &[i32],
    submitted: &str,
) -> Result<Vec<Word>, String> {
    let database = dictionary.database().map_err(|error| error.to_string())?;
    if mode == "sets" {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids = ids
            .iter()
            .copied()
            .map(|id| WordId::try_from(i64::from(id)).map_err(|error| error.to_string()))
            .collect::<Result<BTreeSet<_>, _>>()?;
        database
            .words_by_ids(&ids)
            .await
            .map_err(|error| error.to_string())
    } else if submitted.trim().is_empty() {
        Ok(Vec::new())
    } else {
        let mut words = database
            .write_hanzi_unified_search(submitted, &ranking::fold_pinyin(submitted), 60)
            .await
            .map_err(|error| error.to_string())?;
        ranking::rank(&mut words, submitted);
        Ok(words)
    }
}
