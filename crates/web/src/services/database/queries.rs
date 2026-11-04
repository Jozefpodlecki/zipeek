use super::{Database, DbError, row::WordRow};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use zipeek_core::{
    HskLevel, HskSystem, Word, WordId, practice::PracticeFilter, round::CandidatePool,
};

#[derive(Deserialize)]
struct SpotlightCategory {
    category: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SpotlightDefinition {
    pub definition: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct BrowseRelation {
    pub relation_type: String,
    pub target_simplified: String,
    pub source_annotation: String,
}

impl Database {
    pub async fn word_relations(&self, word_id: WordId) -> Result<Vec<BrowseRelation>, DbError> {
        self.query(
            "SELECT relation_type, target_simplified, source_annotation FROM sense_relations WHERE word_id = ? ORDER BY sense_order, relation_order",
            vec![json!(word_id.as_i32())],
        ).await
    }

    pub async fn word_display_definition(&self, word_id: WordId) -> Result<Option<String>, DbError> {
        let rows: Vec<SpotlightDefinition> = self.query(
            "SELECT CASE WHEN EXISTS (SELECT 1 FROM sense_display WHERE word_id = ?) THEN (SELECT string_agg(definition, '|' ORDER BY sense_order) FROM sense_display WHERE word_id = ? AND definition IS NOT NULL) ELSE (SELECT english FROM words WHERE id = ?) END AS definition",
            vec![json!(word_id.as_i32()), json!(word_id.as_i32()), json!(word_id.as_i32())],
        ).await?;
        match rows.as_slice() {
            [row] => Ok(row.definition.clone()),
            _ => Err(DbError::Decode("Expected one spotlight definition row".to_owned())),
        }
    }

    pub async fn word_categories(&self, word_id: WordId) -> Result<Vec<String>, DbError> {
        let rows: Vec<SpotlightCategory> = self.query(
            "SELECT DISTINCT c.label AS category FROM sense_categories AS sc JOIN categories AS c ON c.slug = sc.category_slug WHERE sc.word_id = ? ORDER BY c.label",
            vec![json!(word_id.as_i32())],
        ).await?;
        Ok(rows.into_iter().map(|row| row.category).collect())
    }

    pub async fn translation_words_for_hsk(&self, level: HskLevel) -> Result<Vec<Word>, DbError> {
        let column = match level.system() {
            HskSystem::Hsk2 => "hsk2_level",
            HskSystem::Hsk3 => "hsk3_level",
        };
        self.words(
            &format!("SELECT * FROM words WHERE {column} = ? AND length(simplified) BETWEEN 1 AND 8 ORDER BY id ASC LIMIT 24"),
            vec![json!(level.as_i32())],
        ).await
    }

    pub async fn words_by_ids(&self, ids: &BTreeSet<WordId>) -> Result<Vec<Word>, DbError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = vec!["?"; ids.len()].join(",");
        self.words(
            &format!("SELECT * FROM words WHERE id IN ({placeholders}) ORDER BY id ASC"),
            ids.iter().map(|id| json!(id.as_i32())).collect(),
        )
        .await
    }

    pub(super) async fn words(
        &self,
        sql: &str,
        parameters: Vec<Value>,
    ) -> Result<Vec<Word>, DbError> {
        self.query::<Vec<WordRow>>(sql, parameters)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    pub async fn word_by_id(&self, id: WordId) -> Result<Option<Word>, DbError> {
        Ok(self
            .words(
                "SELECT * FROM words WHERE id = ? ORDER BY id ASC",
                vec![json!(id.as_i32())],
            )
            .await?
            .into_iter()
            .next())
    }

    pub async fn lookup_hanzi(&self, text: &str) -> Result<Vec<Word>, DbError> {
        self.words(
            "SELECT * FROM words WHERE simplified = ? OR traditional = ? ORDER BY id ASC",
            vec![json!(text), json!(text)],
        )
        .await
    }

    pub async fn lookup_pinyin(&self, text: &str) -> Result<Vec<Word>, DbError> {
        self.words(
            "SELECT * FROM words WHERE pinyin = ? OR pinyin_numeric = ? ORDER BY id ASC",
            vec![json!(text), json!(text)],
        )
        .await
    }

    pub async fn count_words(&self, exclusions: &BTreeSet<WordId>) -> Result<u32, DbError> {
        #[derive(Deserialize)]
        struct Count {
            count: u32,
        }
        let mut sql = "SELECT CAST(count(*) AS INTEGER) AS count FROM words WHERE (hsk2_level IS NOT NULL OR hsk3_level IS NOT NULL)".to_owned();
        let mut parameters = Vec::new();
        exclude(&mut sql, &mut parameters, exclusions);
        let counts: Vec<Count> = self.query(&sql, parameters).await?;
        match counts.as_slice() {
            [count] => Ok(count.count),
            _ => Err(DbError::Decode(
                "Expected one dictionary count row".to_owned(),
            )),
        }
    }

    pub async fn word_at_offset(
        &self,
        offset: u32,
        exclusions: &BTreeSet<WordId>,
    ) -> Result<Option<Word>, DbError> {
        let mut sql =
            "SELECT * FROM words WHERE (hsk2_level IS NOT NULL OR hsk3_level IS NOT NULL)"
                .to_owned();
        let mut parameters = Vec::new();
        exclude(&mut sql, &mut parameters, exclusions);
        sql.push_str(" ORDER BY id ASC LIMIT 1 OFFSET ?");
        parameters.push(json!(offset));
        Ok(self.words(&sql, parameters).await?.into_iter().next())
    }

    pub async fn hsk_union(&self, exclusions: &BTreeSet<WordId>) -> Result<Vec<Word>, DbError> {
        let mut sql =
            "SELECT * FROM words WHERE (hsk2_level IS NOT NULL OR hsk3_level IS NOT NULL)"
                .to_owned();
        let mut parameters = Vec::new();
        exclude(&mut sql, &mut parameters, exclusions);
        sql.push_str(" ORDER BY id ASC");
        self.words(&sql, parameters).await
    }

    pub async fn candidate_pool_for_set(
        &self,
        ids: &BTreeSet<WordId>,
        exclusions: &BTreeSet<WordId>,
    ) -> Result<CandidatePool, DbError> {
        let kept = ids.difference(exclusions).copied().collect::<BTreeSet<_>>();
        let words = self.words_by_ids(&kept).await?;
        Ok(exact_set_pool(words))
    }

    pub async fn candidate_pool(
        &self,
        filter: &PracticeFilter,
        exclusions: &BTreeSet<WordId>,
    ) -> Result<CandidatePool, DbError> {
        let strict = self
            .filtered_words(filter, exclusions, false, false)
            .await?;
        let hsk_widened = self.filtered_words(filter, exclusions, true, false).await?;
        let hsk_widened_any_pos = self.filtered_words(filter, exclusions, true, true).await?;
        Ok(CandidatePool {
            strict,
            hsk_widened,
            hsk_widened_any_pos,
        })
    }

    async fn filtered_words(
        &self,
        filter: &PracticeFilter,
        exclusions: &BTreeSet<WordId>,
        widen: bool,
        any_pos: bool,
    ) -> Result<Vec<Word>, DbError> {
        let (sql, parameters) = filtered_query(filter, exclusions, widen, any_pos);
        self.words(&sql, parameters).await
    }

    pub async fn page(
        &self,
        filter: &PracticeFilter,
        exclusions: &BTreeSet<WordId>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Word>, DbError> {
        let (mut sql, mut parameters) = filtered_query(filter, exclusions, false, false);
        sql.push_str(" LIMIT ? OFFSET ?");
        parameters.extend([json!(limit), json!(offset)]);
        self.words(&sql, parameters).await
    }
}

fn exact_set_pool(words: Vec<Word>) -> CandidatePool {
    CandidatePool {
        strict: words.clone(),
        hsk_widened: words.clone(),
        hsk_widened_any_pos: words,
    }
}

fn filtered_query(
    filter: &PracticeFilter,
    exclusions: &BTreeSet<WordId>,
    widen: bool,
    any_pos: bool,
) -> (String, Vec<Value>) {
    let column = match filter.level.system() {
        HskSystem::Hsk2 => "hsk2_level",
        HskSystem::Hsk3 => "hsk3_level",
    };
    let level = filter.level.as_i32();
    let maximum = if matches!(filter.level, HskLevel::Hsk2(_)) {
        6
    } else {
        7
    };
    let delta = i32::from(widen);
    let mut sql = format!("SELECT * FROM words WHERE {column} BETWEEN ? AND ?");
    let mut parameters = vec![
        json!((level - delta).max(1)),
        json!((level + delta).min(maximum)),
    ];
    if !any_pos && !filter.part_of_speech.is_empty() {
        sql.push_str(" AND (");
        let predicates = filter
            .part_of_speech
            .iter()
            .map(|pos| {
                parameters.push(json!(pos.as_str()));
                "list_contains(string_split(coalesce(part_of_speech, ''), ','), ?)"
            })
            .collect::<Vec<_>>();
        sql.push_str(&predicates.join(" OR "));
        sql.push(')');
    }
    exclude(&mut sql, &mut parameters, exclusions);
    sql.push_str(" ORDER BY id ASC");
    (sql, parameters)
}

fn exclude(sql: &mut String, parameters: &mut Vec<Value>, exclusions: &BTreeSet<WordId>) {
    if !exclusions.is_empty() {
        sql.push_str(" AND id NOT IN (");
        sql.push_str(&vec!["?"; exclusions.len()].join(","));
        sql.push(')');
        parameters.extend(exclusions.iter().map(|id| json!(id.as_i32())));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_core::practice::ParsedPracticeQuery;

    #[test]
    fn selected_set_never_widens_into_other_vocabulary() {
        let words = vec![Word {
            id: WordId::try_from(4).unwrap(),
            simplified: "水".into(),
            traditional: None,
            pinyin: "shuǐ".into(),
            pinyin_numeric: None,
            english: Some("water".into()),
            part_of_speech: vec![],
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        }];
        let pool = exact_set_pool(words.clone());
        assert_eq!(pool.strict, words);
        assert_eq!(pool.hsk_widened, words);
        assert_eq!(pool.hsk_widened_any_pos, words);
    }

    #[test]
    fn widening_clamps_in_the_selected_system() {
        for (system, level, expected) in [
            ("2.0", "1", [1, 2]),
            ("2.0", "6", [5, 6]),
            ("3.0", "1", [1, 2]),
            ("3.0", "7", [6, 7]),
        ] {
            let query = ParsedPracticeQuery::from_pairs([("hsk", system), ("level", level)]);
            let (sql, parameters) = filtered_query(&query.filter, &BTreeSet::new(), true, false);
            assert_eq!(parameters, expected.map(|value| json!(value)));
            assert!(sql.contains(if system == "2.0" {
                "hsk2_level"
            } else {
                "hsk3_level"
            }));
            assert!(!sql.contains("NOT IN"));
            assert!(sql.ends_with("ORDER BY id ASC"));
        }
    }

    #[test]
    fn pos_and_exclusions_are_bound_before_ordering() {
        let query = ParsedPracticeQuery::from_pairs([("pos", "verb,noun")]);
        let exclusions = BTreeSet::from([
            WordId::try_from(991).unwrap(),
            WordId::try_from(992).unwrap(),
        ]);
        let (sql, parameters) = filtered_query(&query.filter, &exclusions, false, false);
        assert!(!sql.contains("991"));
        assert!(sql.contains("list_contains(string_split"));
        assert!(sql.ends_with("AND id NOT IN (?,?) ORDER BY id ASC"));
        assert_eq!(
            parameters,
            vec![
                json!(1),
                json!(1),
                json!("noun"),
                json!("verb"),
                json!(991),
                json!(992)
            ]
        );
        let (any_pos, parameters) = filtered_query(&query.filter, &exclusions, true, true);
        assert!(!any_pos.contains("list_contains"));
        assert_eq!(parameters, vec![json!(1), json!(2), json!(991), json!(992)]);
    }
}
