use super::{Database, DbError};
use serde_json::{Value, json};
use zipeek_core::{HskLevel, HskSystem, Word};
use zipeek_ai::starter_set;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrowseField {
    #[default]
    All,
    Hanzi,
    Pinyin,
    English,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BrowseSearch {
    pub text: String,
    pub field: BrowseField,
    pub hsk: Option<HskLevel>,
    pub part_of_speech: Vec<String>,
    pub categories: Vec<String>,
    pub word_set: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

impl Database {
    pub async fn browse_search(&self, request: &BrowseSearch) -> Result<Vec<Word>, DbError> {
        let (sql, parameters) = browse_query(request)?;
        self.words(&sql, parameters).await
    }
}

fn browse_query(request: &BrowseSearch) -> Result<(String, Vec<Value>), DbError> {
    if request.limit == 0 || request.limit > 100 {
        return Err(DbError::Decode("Browse page size must be between 1 and 100".to_owned()));
    }
    let text = request.text.trim();
    if text.chars().count() > 128 {
        return Err(DbError::Decode("Browse search is limited to 128 characters".to_owned()));
    }
    let mut sql = String::from("SELECT w.* FROM words AS w WHERE TRUE");
    let mut parameters = Vec::new();

    if !text.is_empty() {
        let hanzi = "(contains(w.simplified, ?) OR contains(coalesce(w.traditional, ''), ?))";
        let pinyin = "(contains(lower(w.pinyin), lower(?)) OR contains(lower(coalesce(w.pinyin_numeric, '')), lower(?)))";
        let english = "EXISTS (SELECT 1 FROM word_senses AS s WHERE s.word_id = w.id AND contains(lower(s.gloss), lower(?)))";
        match request.field {
            BrowseField::All => {
                sql.push_str(" AND (");
                sql.push_str(hanzi);
                sql.push_str(" OR ");
                sql.push_str(pinyin);
                sql.push_str(" OR ");
                sql.push_str(english);
                sql.push(')');
                parameters.extend((0..5).map(|_| json!(text)));
            }
            BrowseField::Hanzi => {
                sql.push_str(" AND ");
                sql.push_str(hanzi);
                parameters.extend((0..2).map(|_| json!(text)));
            }
            BrowseField::Pinyin => {
                sql.push_str(" AND ");
                sql.push_str(pinyin);
                parameters.extend((0..2).map(|_| json!(text)));
            }
            BrowseField::English => {
                sql.push_str(" AND ");
                sql.push_str(english);
                parameters.push(json!(text));
            }
        }
    }

    if let Some(level) = request.hsk {
        let system = match level.system() {
            HskSystem::Hsk2 => "hsk2",
            HskSystem::Hsk3 => "hsk3",
        };
        sql.push_str(" AND EXISTS (SELECT 1 FROM word_hsk_membership AS h WHERE h.word_id = w.id AND h.system = ? AND h.level = ?)");
        parameters.push(json!(system));
        parameters.push(json!(level.as_i32()));
    }

    append_membership_filter(
        &mut sql,
        &mut parameters,
        &request.part_of_speech,
        "word_pos_membership",
        "pos",
    )?;
    append_membership_filter(
        &mut sql,
        &mut parameters,
        &request.categories,
        "sense_categories",
        "category_slug",
    )?;

    if let Some(id) = request.word_set.as_deref() {
        let set = starter_set(id)
            .ok_or_else(|| DbError::Decode("Unknown predefined Word Set".to_owned()))?;
        let placeholders = vec!["?"; set.vocabulary.len()].join(",");
        sql.push_str(&format!(" AND w.simplified IN ({placeholders})"));
        parameters.extend(set.vocabulary.iter().map(|word| json!(word)));
    }

    sql.push_str(" ORDER BY w.id ASC LIMIT ? OFFSET ?");
    parameters.push(json!(request.limit));
    parameters.push(json!(request.offset));
    Ok((sql, parameters))
}

fn append_membership_filter(
    sql: &mut String,
    parameters: &mut Vec<Value>,
    codes: &[String],
    table: &'static str,
    column: &'static str,
) -> Result<(), DbError> {
    if codes.is_empty() {
        return Ok(());
    }
    if codes.len() > 64 || codes.iter().any(|code| {
        code.is_empty()
            || code.len() > 64
            || !code.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_')
    }) {
        return Err(DbError::Decode("Invalid browse filter code".to_owned()));
    }
    let placeholders = vec!["?"; codes.len()].join(",");
    sql.push_str(&format!(
        " AND EXISTS (SELECT 1 FROM {table} AS m WHERE m.word_id = w.id AND m.{column} IN ({placeholders}))"
    ));
    parameters.extend(codes.iter().map(|code| json!(code)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_core::practice::ParsedPracticeQuery;

    fn request() -> BrowseSearch {
        BrowseSearch {
            text: "火车".to_owned(),
            limit: 20,
            ..Default::default()
        }
    }

    #[test]
    fn default_search_binds_all_text_fields_and_preserves_one_row_per_word() {
        let (sql, params) = browse_query(&request()).unwrap();
        assert_eq!(params, vec![json!("火车"); 5].into_iter().chain([json!(20), json!(0)]).collect::<Vec<_>>());
        assert!(sql.contains("FROM word_senses AS s"));
        assert!(sql.contains("EXISTS"));
        assert!(!sql.contains(" JOIN "));
        assert!(sql.ends_with("ORDER BY w.id ASC LIMIT ? OFFSET ?"));
    }

    #[test]
    fn combined_filters_are_and_across_dimensions_or_within_each_dimension() {
        let level = ParsedPracticeQuery::from_pairs([("hsk", "3.0"), ("level", "2")]).filter.level;
        let mut request = request();
        request.field = BrowseField::Hanzi;
        request.hsk = Some(level);
        request.part_of_speech = vec!["verb".into(), "noun".into()];
        request.categories = vec!["travel".into(), "transport".into()];
        request.offset = 40;
        let (sql, params) = browse_query(&request).unwrap();
        assert!(sql.contains("h.system = ? AND h.level = ?"));
        assert!(sql.contains("FROM word_pos_membership"));
        assert!(sql.contains("FROM sense_categories"));
        assert!(!sql.contains("火车"));
        assert_eq!(params, vec![json!("火车"), json!("火车"), json!("hsk3"), json!(2), json!("verb"), json!("noun"), json!("travel"), json!("transport"), json!(20), json!(40)]);
    }

    #[test]
    fn rejects_unbounded_pages_and_invalid_codes() {
        let mut request = request();
        request.limit = 101;
        assert!(browse_query(&request).is_err());
        request.limit = 20;
        request.categories = vec!["travel'); DROP TABLE words; --".into()];
        assert!(browse_query(&request).is_err());
        request.categories.clear();
        request.text = "x".repeat(129);
        assert!(browse_query(&request).is_err());
    }

    #[test]
    fn empty_text_and_field_scoping_are_supported() {
        let mut request = request();
        request.text = " ".into();
        let (sql, params) = browse_query(&request).unwrap();
        assert!(!sql.contains("word_senses"));
        assert_eq!(params, vec![json!(20), json!(0)]);
        request.text = "train".into();
        request.field = BrowseField::English;
        let (sql, params) = browse_query(&request).unwrap();
        assert!(sql.contains("word_senses"));
        assert!(!sql.contains("w.pinyin"));
        assert_eq!(params, vec![json!("train"), json!(20), json!(0)]);
    }
    #[test]
    fn predefined_set_uses_bindings_and_rejects_unknown_ids() {
        let mut request = request();
        request.word_set = Some("travel".to_owned());
        let (sql, params) = browse_query(&request).unwrap();
        assert!(sql.contains("w.simplified IN (?,?,?,?,?,?,?)"));
        assert!(params.contains(&json!("机场")));
        assert!(!sql.contains("机场"));
        request.word_set = Some("unknown".to_owned());
        assert!(browse_query(&request).is_err());
    }

    #[test]
    fn category_membership_has_word_id_and_no_join_multiplication() {
        let mut request = request();
        request.categories = vec!["travel".to_owned()];
        let (sql, _) = browse_query(&request).unwrap();
        assert!(sql.contains("FROM sense_categories AS m WHERE m.word_id = w.id"));
        assert!(!sql.contains(" JOIN "));
    }

}
