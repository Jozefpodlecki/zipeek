use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use zipeek_core::WordId;

use super::{Database, DbError};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DictionaryMeaning {
    pub sense_order: i32,
    pub definition: String,
    pub categories: Vec<String>,
    pub qualifiers: Vec<MeaningQualifier>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct MeaningQualifier {
    pub kind: String,
    pub code: String,
    pub annotation: String,
}

impl MeaningQualifier {
    pub fn label(&self) -> &str {
        match (self.kind.as_str(), self.code.as_str()) {
            ("region", "taiwan") => "Taiwan",
            ("usage", "loanword") => "Loanword",
            ("usage", "figurative") => "Figurative",
            ("register", "colloquial") => "Colloquial",
            ("register", "archaic") => "Archaic",
            ("register", "literary") => "Literary",
            ("register", "slang") => "Slang",
            _ => &self.code,
        }
    }
}

#[derive(Deserialize)]
struct MeaningRow {
    sense_order: i32,
    definition: String,
}

#[derive(Deserialize)]
struct MetadataRow {
    sense_order: i32,
    kind: String,
    code: String,
    label: String,
}

pub(super) fn assemble_meanings(
    rows: Vec<(i32, String)>,
    metadata: Vec<(i32, String, String, String)>,
) -> Vec<DictionaryMeaning> {
    let mut senses = BTreeMap::<i32, DictionaryMeaning>::new();
    for (sense_order, definition) in rows {
        if sense_order <= 0 || definition.trim().is_empty() {
            continue;
        }
        senses.entry(sense_order).or_insert(DictionaryMeaning {
            sense_order,
            definition,
            categories: Vec::new(),
            qualifiers: Vec::new(),
        });
    }
    for (sense_order, kind, code, label) in metadata {
        let Some(sense) = senses.get_mut(&sense_order) else {
            continue;
        };
        if kind == "category" {
            if !sense.categories.contains(&label) {
                sense.categories.push(label);
            }
        } else if !sense
            .qualifiers
            .iter()
            .any(|existing| existing.kind == kind && existing.code == code)
        {
            sense.qualifiers.push(MeaningQualifier {
                kind,
                code,
                annotation: label,
            });
        }
    }
    senses.into_values().collect()
}

impl Database {
    pub async fn word_meanings(&self, word_id: WordId) -> Result<Vec<DictionaryMeaning>, DbError> {
        let params = vec![json!(word_id.as_i32())];
        let rows: Vec<MeaningRow> = self.query(
            "SELECT sense_order, definition FROM sense_display WHERE word_id = ? AND definition IS NOT NULL ORDER BY sense_order",
            params.clone(),
        ).await?;
        let metadata: Vec<MetadataRow> = self.query(
            r#"SELECT sc.sense_order, 'category' AS kind, sc.category_slug AS code, c.label AS label
FROM sense_categories AS sc JOIN categories AS c ON c.slug = sc.category_slug WHERE sc.word_id = ?
UNION ALL
SELECT sq.sense_order, sq.qualifier_kind AS kind, sq.qualifier_code AS code, sq.source_annotation AS label
FROM sense_qualifiers AS sq WHERE sq.word_id = ?
ORDER BY sense_order, kind, code"#,
            vec![json!(word_id.as_i32()), json!(word_id.as_i32())],
        ).await?;
        Ok(assemble_meanings(
            rows.into_iter()
                .map(|r| (r.sense_order, r.definition))
                .collect(),
            metadata
                .into_iter()
                .map(|r| (r.sense_order, r.kind, r.code, r.label))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_empty_senses_and_orphaned_metadata_are_not_shown() {
        let senses = assemble_meanings(
            vec![(0, "invalid".into()), (1, "  ".into()), (2, "valid".into())],
            vec![
                (0, "region".into(), "taiwan".into(), "(Tw)".into()),
                (17, "category".into(), "money".into(), "Money".into()),
            ],
        );
        assert_eq!(senses.len(), 1);
        assert_eq!(senses[0].sense_order, 2);
        assert!(senses[0].qualifiers.is_empty());
    }

    #[test]
    fn duplicate_qualifiers_do_not_repeat_badges() {
        let senses = assemble_meanings(
            vec![(3, "a gloss".into())],
            vec![
                (3, "region".into(), "taiwan".into(), "(Tw)".into()),
                (3, "region".into(), "taiwan".into(), "(Taiwan)".into()),
            ],
        );
        assert_eq!(senses[0].qualifiers.len(), 1);
        assert_eq!(senses[0].qualifiers[0].annotation, "(Tw)");
    }

    #[test]
    fn attaches_metadata_to_the_correct_sense() {
        let senses = assemble_meanings(
            vec![(1, "to fasten".into()), (7, "code".into())],
            vec![
                (
                    7,
                    "category".into(),
                    "money".into(),
                    "Money and payments".into(),
                ),
                (7, "region".into(), "taiwan".into(), "(Tw)".into()),
            ],
        );
        assert_eq!(senses.len(), 2);
        assert!(senses[0].categories.is_empty());
        assert_eq!(senses[1].sense_order, 7);
        assert_eq!(senses[1].qualifiers[0].label(), "Taiwan");
    }
}
