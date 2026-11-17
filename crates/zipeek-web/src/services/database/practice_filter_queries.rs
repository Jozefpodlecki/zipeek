use super::{Database, DbError};
use serde::Deserialize;
use serde_json::{Value, json};
use zipeek_core::Word;
use zipeek_exercises::practice_draft::DictionaryFilters;

const ELIGIBLE_MEANING: &str = "(SELECT d.definition FROM sense_display AS d WHERE d.word_id = w.id AND d.sense_kind = 'lexical' AND d.definition IS NOT NULL ORDER BY d.sense_order LIMIT 1)";

fn eligible_word_projection() -> String {
    format!("w.* REPLACE ({ELIGIBLE_MEANING} AS english)")
}

impl Database {
    pub(super) async fn practice_eligible_from_query(
        &self,
        sql: &str,
        parameters: Vec<Value>,
    ) -> Result<Vec<Word>, DbError> {
        self.words(
            &format!("SELECT {} FROM ({sql}) AS w WHERE {ELIGIBLE_MEANING} IS NOT NULL ORDER BY w.id ASC", eligible_word_projection()),
            parameters,
        ).await
    }

    pub async fn practice_eligible_words_by_ids(
        &self,
        ids: &std::collections::BTreeSet<zipeek_core::WordId>,
    ) -> Result<Vec<Word>, DbError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = vec!["?"; ids.len()].join(",");
        self.words(
            &format!(
                "SELECT {} FROM words AS w WHERE w.id IN ({placeholders}) AND {ELIGIBLE_MEANING} IS NOT NULL ORDER BY w.id ASC",
                eligible_word_projection()
            ),
            ids.iter().map(|id| json!(id.as_i32())).collect(),
        ).await
    }

    pub async fn practice_eligible_words_for_filters(
        &self,
        filters: &DictionaryFilters,
        limit: u32,
    ) -> Result<Vec<Word>, DbError> {
        let (predicate, parameters) = practice_filter_predicate(filters)?;
        if !(1..=128).contains(&limit) {
            return Err(DbError::Decode(
                "Practice vocabulary sample must contain 1–128 words".to_owned(),
            ));
        }
        let sql = format!(
            "SELECT {} FROM words AS w WHERE {predicate} AND {ELIGIBLE_MEANING} IS NOT NULL ORDER BY w.id ASC LIMIT ?",
            eligible_word_projection()
        );
        let mut parameters = parameters;
        parameters.push(json!(limit));
        self.words(&sql, parameters).await
    }
    pub async fn practice_exact_pool_for_filters(
        &self,
        filters: &DictionaryFilters,
        exclusions: &std::collections::BTreeSet<zipeek_core::WordId>,
    ) -> Result<Vec<Word>, DbError> {
        let (predicate, mut parameters) = practice_filter_predicate(filters)?;
        let mut sql = format!(
            "SELECT {} FROM words AS w WHERE {predicate} AND {ELIGIBLE_MEANING} IS NOT NULL",
            eligible_word_projection()
        );
        if !exclusions.is_empty() {
            let placeholders = vec!["?"; exclusions.len()].join(",");
            sql.push_str(&format!(" AND w.id NOT IN ({placeholders})"));
            parameters.extend(exclusions.iter().map(|id| json!(id.as_i32())));
        }
        sql.push_str(" ORDER BY w.id ASC");
        self.words(&sql, parameters).await
    }

    pub async fn practice_words_for_filters(
        &self,
        filters: &DictionaryFilters,
        limit: u32,
    ) -> Result<Vec<Word>, DbError> {
        let (sql, parameters) = practice_filter_query(filters, limit)?;
        self.words(&sql, parameters).await
    }

    pub async fn crossword_words_for_filters(
        &self,
        filters: &DictionaryFilters,
    ) -> Result<Vec<Word>, DbError> {
        let (sql, parameters) = crossword_filter_query(filters)?;
        self.words(&sql, parameters).await
    }

    pub async fn character_hunt_words_for_filters(
        &self,
        filters: &DictionaryFilters,
    ) -> Result<Vec<Word>, DbError> {
        let (sql, parameters) = character_hunt_query(filters)?;
        self.words(&sql, parameters).await
    }

    pub async fn practice_filter_count(&self, filters: &DictionaryFilters) -> Result<u32, DbError> {
        #[derive(Deserialize)]
        struct CountRow {
            count: u32,
        }
        let (predicate, parameters) = practice_filter_predicate(filters)?;
        let rows: Vec<CountRow> = self
            .query(
                &format!(
                    r#"SELECT CAST(count(*) AS INTEGER) AS count
FROM words AS w
WHERE {predicate} AND {ELIGIBLE_MEANING} IS NOT NULL"#
                ),
                parameters,
            )
            .await?;
        match rows.as_slice() {
            [row] => Ok(row.count),
            _ => Err(DbError::Decode(
                "Expected one filtered vocabulary count".to_owned(),
            )),
        }
    }

    pub async fn practice_category_suggestions(
        &self,
        prefix: &str,
    ) -> Result<Vec<String>, DbError> {
        let trimmed = prefix.trim();
        if trimmed.chars().count() > 64 {
            return Err(DbError::Decode(
                "Category search is limited to 64 characters".to_owned(),
            ));
        }
        #[derive(Deserialize)]
        struct CategoryRow {
            category_slug: String,
        }
        let rows: Vec<CategoryRow> = self
            .query(
                r#"SELECT DISTINCT
    sca.category_slug
FROM sense_categories AS sca
WHERE starts_with(sca.category_slug, lower(?))
ORDER BY sca.category_slug
LIMIT 64"#,
                vec![json!(trimmed)],
            )
            .await?;
        Ok(rows.into_iter().map(|row| row.category_slug).collect())
    }
}

fn crossword_filter_query(filters: &DictionaryFilters) -> Result<(String, Vec<Value>), DbError> {
    let (predicate, parameters) = practice_filter_predicate(filters)?;
    let sql = format!(
        r#"SELECT w.*
FROM words AS w
WHERE {predicate}
    AND length(w.simplified) BETWEEN 2 AND 5
ORDER BY hash(w.id)
LIMIT 1024"#
    );
    Ok((sql, parameters))
}

fn character_hunt_query(filters: &DictionaryFilters) -> Result<(String, Vec<Value>), DbError> {
    let (predicate, parameters) = practice_filter_predicate(filters)?;
    let sql = format!(
        r#"SELECT w.*
FROM words AS w
WHERE {predicate}
    AND length(w.simplified) BETWEEN 2 AND 3
ORDER BY w.id ASC
LIMIT 1024"#
    );
    Ok((sql, parameters))
}

fn practice_filter_query(
    filters: &DictionaryFilters,
    limit: u32,
) -> Result<(String, Vec<Value>), DbError> {
    if !(1..=128).contains(&limit) {
        return Err(DbError::Decode(
            "Practice vocabulary sample must contain 1–128 words".to_owned(),
        ));
    }
    let (predicate, mut parameters) = practice_filter_predicate(filters)?;
    parameters.push(json!(limit));
    Ok((
        format!(
            r#"SELECT w.*
FROM words AS w
WHERE {predicate}
ORDER BY w.id ASC
LIMIT ?"#
        ),
        parameters,
    ))
}

fn practice_filter_predicate(filters: &DictionaryFilters) -> Result<(String, Vec<Value>), DbError> {
    if filters.hsk.is_empty() && filters.categories.is_empty() && filters.parts_of_speech.is_empty()
    {
        return Err(DbError::Decode(
            "Choose an HSK band, category or part of speech".to_owned(),
        ));
    }
    if filters.hsk.len() > 16 {
        return Err(DbError::Decode("Too many HSK bands".to_owned()));
    }
    validate_codes(&filters.categories)?;
    validate_codes(&filters.parts_of_speech)?;

    let mut sql = String::from("TRUE");
    let mut parameters = Vec::new();
    if !filters.hsk.is_empty() {
        let predicates = vec!["(h.system = ? AND h.level = ?)"; filters.hsk.len()].join(" OR ");
        sql.push_str(&format!(
            r#" AND EXISTS (
    SELECT 1
    FROM word_hsk_membership AS h
    WHERE h.word_id = w.id
        AND ({predicates})
)"#
        ));
        for band in &filters.hsk {
            let system = match band.system {
                2 => "hsk2",
                3 => "hsk3",
                _ => return Err(DbError::Decode("Invalid HSK system".to_owned())),
            };
            let maximum = if band.system == 2 { 6 } else { 7 };
            if band.level == 0 || band.level > maximum {
                return Err(DbError::Decode("Invalid HSK level".to_owned()));
            }
            parameters.push(json!(system));
            parameters.push(json!(band.level));
        }
    }
    if !filters.categories.is_empty() {
        let placeholders = vec!["?"; filters.categories.len()].join(",");
        sql.push_str(&format!(
            r#" AND EXISTS (
    SELECT 1
    FROM word_senses AS s
    JOIN sense_categories AS c
        ON c.word_id = s.word_id
        AND c.sense_order = s.sense_order
    WHERE s.word_id = w.id
        AND c.category_slug IN ({placeholders})
)"#
        ));
        parameters.extend(filters.categories.iter().map(|code| json!(code)));
    }
    if !filters.parts_of_speech.is_empty() {
        let placeholders = vec!["?"; filters.parts_of_speech.len()].join(",");
        sql.push_str(&format!(
            r#" AND EXISTS (
    SELECT 1
    FROM word_pos_membership AS p
    WHERE p.word_id = w.id
        AND p.pos IN ({placeholders})
)"#
        ));
        parameters.extend(filters.parts_of_speech.iter().map(|code| json!(code)));
    }
    Ok((sql, parameters))
}

fn validate_codes(codes: &std::collections::BTreeSet<String>) -> Result<(), DbError> {
    if codes.len() > 64
        || codes.iter().any(|code| {
            code.is_empty()
                || code.len() > 64
                || !code
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
        })
    {
        return Err(DbError::Decode(
            "Invalid practice category or PoS code".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_exercises::practice_draft::HskBand;

    #[test]
    fn multiple_hsk_bands_are_or_and_filter_groups_are_and() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.insert(HskBand::new(2, 6).unwrap());
        filters.hsk.insert(HskBand::new(3, 7).unwrap());
        filters.categories.insert("travel".into());
        filters.categories.insert("shopping".into());
        filters.parts_of_speech.insert("noun".into());
        let (sql, parameters) = practice_filter_query(&filters, 24).unwrap();
        assert!(sql.contains("h.system = ? AND h.level = ?) OR (h.system = ? AND h.level = ?"));
        assert!(sql.contains("JOIN sense_categories AS c"));
        assert!(sql.contains("word_pos_membership AS p"));
        assert!(sql.contains("category_slug IN (?,?)"));
        assert!(!sql.contains("travel"));
        assert_eq!(
            parameters,
            vec![
                json!("hsk2"),
                json!(6),
                json!("hsk3"),
                json!(7),
                json!("shopping"),
                json!("travel"),
                json!("noun"),
                json!(24)
            ]
        );
    }

    #[test]
    fn character_hunt_uses_exact_filter_predicate_and_bounded_dictionary_words() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.insert(HskBand::new(3, 2).unwrap());
        filters.hsk.insert(HskBand::new(3, 3).unwrap());
        filters.add_category("Travel").unwrap();
        filters.parts_of_speech.insert("noun".to_owned());

        let (predicate, expected_parameters) = practice_filter_predicate(&filters).unwrap();
        let (query, parameters) = character_hunt_query(&filters).unwrap();
        assert!(query.contains(&format!("WHERE {predicate}")));
        assert!(query.contains("length(w.simplified) BETWEEN 2 AND 3"));
        assert!(query.contains("ORDER BY w.id ASC\nLIMIT 1024"));
        assert!(!query.contains("travel"));
        assert_eq!(parameters, expected_parameters);
    }

    #[test]
    fn character_hunt_accepts_category_only_source_and_rejects_empty_filters() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.clear();
        filters.add_category("Travel").unwrap();
        let (query, parameters) = character_hunt_query(&filters).unwrap();
        assert!(!query.contains("word_hsk_membership"));
        assert_eq!(parameters, vec![json!("travel")]);
        filters.categories.clear();
        assert!(character_hunt_query(&filters).is_err());
    }

    #[test]
    fn crossword_pool_filters_to_multi_character_hanzi_before_sampling() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.insert(HskBand::new(3, 1).unwrap());
        let (sql, params) = crossword_filter_query(&filters).unwrap();
        assert!(sql.contains("word_hsk_membership"));
        assert!(sql.contains("length(w.simplified) BETWEEN 2 AND 5"));
        assert!(sql.contains("ORDER BY hash(w.id)\nLIMIT 1024"));
        assert_eq!(params, vec![json!("hsk3"), json!(1)]);
    }

    #[test]
    fn category_only_filter_does_not_inject_an_hsk_limit() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.clear();
        filters.add_category("Travel").unwrap();
        let (sql, parameters) = practice_filter_query(&filters, 128).unwrap();
        assert!(!sql.contains("word_hsk_membership"));
        assert_eq!(parameters, vec![json!("travel"), json!(128)]);
    }

    #[test]
    fn count_and_sample_share_identical_filters() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.insert(HskBand::new(3, 7).unwrap());
        filters.add_category("travel").unwrap();
        let (predicate, count_parameters) = practice_filter_predicate(&filters).unwrap();
        let (sample_sql, sample_parameters) = practice_filter_query(&filters, 24).unwrap();
        assert!(sample_sql.contains(&format!("WHERE {predicate}\nORDER BY")));
        assert_eq!(
            sample_parameters[..sample_parameters.len() - 1],
            count_parameters
        );
    }

    #[test]
    fn empty_filter_and_invalid_codes_are_rejected() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.clear();
        assert!(practice_filter_query(&filters, 24).is_err());
        filters
            .categories
            .insert("travel'); DROP TABLE words; --".into());
        assert!(practice_filter_query(&filters, 24).is_err());
        filters.categories.clear();
        filters.parts_of_speech.insert("noun".into());
        assert!(practice_filter_query(&filters, 0).is_err());
        assert!(practice_filter_query(&filters, 129).is_err());
    }
}
