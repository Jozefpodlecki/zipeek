use super::super::pairs::make_pairs;
use crate::contexts::database::Dictionary;
use std::collections::BTreeSet;
use zipeek_core::WordId;
use zipeek_exercises::cloud_match::CloudPair;

pub(super) async fn load_pairs(
    dictionary: &Dictionary,
    source: &str,
    standard: u8,
    level: u8,
    ids: Vec<i32>,
) -> Result<Vec<CloudPair>, String> {
    let db = dictionary.database().map_err(|error| error.to_string())?;
    let words = if source == "hsk" {
        db.game_words_for_hsk(standard, level)
            .await
            .map_err(|error| error.to_string())?
    } else {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let keys = ids
            .into_iter()
            .map(|id| WordId::try_from(i64::from(id)).map_err(|error| error.to_string()))
            .collect::<Result<BTreeSet<_>, _>>()?;
        db.words_by_ids(&keys)
            .await
            .map_err(|error| error.to_string())?
    };
    Ok(make_pairs(words))
}
