use std::collections::BTreeSet;

use super::{FixedRounds, PracticeFilter, PracticeMode};
use crate::{HskLevel, WordId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryWarning {
    pub field: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedPracticeQuery {
    pub mode: PracticeMode,
    pub filter: PracticeFilter,
    pub exclusions: BTreeSet<WordId>,
    pub offset: u64,
    pub seed: Option<WordId>,
    pub warnings: Vec<QueryWarning>,
}

impl ParsedPracticeQuery {
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let fields: std::collections::BTreeMap<_, _> = pairs.into_iter().collect();
        let mut query = Self::default();
        let mut get = |field: &'static str, accepts: &dyn Fn(&str) -> bool| {
            fields.get(field).copied().filter(|value| {
                let valid = accepts(value);
                if !valid {
                    query.warnings.push(QueryWarning {
                        field,
                        value: (*value).to_owned(),
                    });
                }
                valid
            })
        };
        let mode = get("mode", &|v| matches!(v, "endless" | "fixed"));
        let rounds = if mode == Some("fixed") {
            match get("rounds", &|v| matches!(v, "5" | "10" | "20")) {
                Some("5") => FixedRounds::Five,
                Some("20") => FixedRounds::Twenty,
                _ => FixedRounds::Ten,
            }
        } else {
            FixedRounds::Ten
        };
        query.mode = if mode == Some("fixed") {
            PracticeMode::Fixed(rounds)
        } else {
            PracticeMode::Endless
        };
        let hsk2 = get("hsk", &|v| matches!(v, "2.0" | "3.0")) == Some("2.0");
        let level =
            get("level", &|v| parse_level(v, hsk2).is_some()).and_then(|v| parse_level(v, hsk2));
        query.filter.level =
            level.unwrap_or_else(|| parse_level("1", hsk2).unwrap_or(query.filter.level));
        query.offset = get("offset", &|v| unsigned(v).is_some())
            .and_then(unsigned)
            .unwrap_or(0);
        query.seed = get("seed", &|v| word_id(v).is_some()).and_then(word_id);
        if let Some(tokens) = fields.get("pos") {
            for token in tokens.split(',').filter(|token| !token.is_empty()) {
                match token.parse() {
                    Ok(pos) => query.filter.part_of_speech.push(pos),
                    Err(_) => query.warnings.push(QueryWarning {
                        field: "pos",
                        value: token.to_owned(),
                    }),
                }
            }
            query.filter.part_of_speech.sort_by_key(|pos| pos.as_str());
            query.filter.part_of_speech.dedup();
        }
        if let Some(tokens) = fields.get("exclude") {
            for token in tokens.split(',').filter(|token| !token.is_empty()) {
                if let Some(id) = word_id(token) {
                    query.exclusions.insert(id);
                } else {
                    query.warnings.push(QueryWarning {
                        field: "exclude",
                        value: token.to_owned(),
                    });
                }
            }
        }
        if matches!(query.mode, PracticeMode::Fixed(_)) && fields.contains_key("seed") {
            query.warnings.push(QueryWarning {
                field: "seed",
                value: fields["seed"].to_owned(),
            });
            query.seed = None;
        }
        query
    }
}

fn unsigned(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn word_id(value: &str) -> Option<WordId> {
    i64::try_from(unsigned(value)?)
        .ok()
        .and_then(|id| WordId::try_from(id).ok())
}

fn parse_level(value: &str, hsk2: bool) -> Option<HskLevel> {
    let value = if value == "7-9" && !hsk2 {
        7
    } else {
        i32::try_from(unsigned(value)?).ok()?
    };
    if hsk2 {
        value.try_into().ok().map(HskLevel::Hsk2)
    } else {
        value.try_into().ok().map(HskLevel::Hsk3)
    }
}
