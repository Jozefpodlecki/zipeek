use std::collections::BTreeSet;

use super::BuildError;
use crate::{Word, WordId, practice::PracticeFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateView {
    Strict,
    HskWidened,
    AnyPos,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CandidatePool {
    pub strict: Vec<Word>,
    pub hsk_widened: Vec<Word>,
    pub hsk_widened_any_pos: Vec<Word>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPool {
    pub filter: PracticeFilter,
    pub pool: CandidatePool,
}

impl CandidatePool {
    pub(crate) fn views(&self) -> [(CandidateView, &[Word]); 3] {
        [
            (CandidateView::Strict, &self.strict),
            (CandidateView::HskWidened, &self.hsk_widened),
            (CandidateView::AnyPos, &self.hsk_widened_any_pos),
        ]
    }

    pub(crate) fn validate(&self, exclusions: &BTreeSet<WordId>) -> Result<(), BuildError> {
        for (view, words) in self.views() {
            validate_order(words, view)?;
            if let Some(word) = words.iter().find(|word| exclusions.contains(&word.id)) {
                return Err(BuildError::InvalidPool {
                    view,
                    id: word.id,
                    reason: "excluded ID present in supplied pool",
                });
            }
        }
        for (smaller, larger, view) in [
            (&self.strict, &self.hsk_widened, CandidateView::HskWidened),
            (
                &self.hsk_widened,
                &self.hsk_widened_any_pos,
                CandidateView::AnyPos,
            ),
        ] {
            for word in smaller {
                if larger
                    .binary_search_by_key(&word.id, |row| row.id)
                    .ok()
                    .is_none_or(|index| larger[index] != *word)
                {
                    return Err(BuildError::InvalidPool {
                        view,
                        id: word.id,
                        reason: "views must be nested with identical rows",
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_set(&self, selected: &BTreeSet<WordId>) -> Result<(), BuildError> {
        for (view, words) in self.views() {
            for word in words {
                if !selected.contains(&word.id) {
                    return Err(BuildError::InvalidPool {
                        view,
                        id: word.id,
                        reason: "row is not in the selected Word Set snapshot",
                    });
                }
            }
            if words != self.strict.as_slice() {
                if let Some(word) = words.first().or_else(|| self.strict.first()) {
                    return Err(BuildError::InvalidPool {
                        view,
                        id: word.id,
                        reason: "Word Set practice must not widen the candidate pool",
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_filter(&self, filter: &PracticeFilter) -> Result<(), BuildError> {
        for (view, words) in self.views() {
            for word in words {
                let level = match filter.level {
                    crate::HskLevel::Hsk2(_) => word.hsk2_level.map(|level| level.as_i32()),
                    crate::HskLevel::Hsk3(_) => word.hsk3_level.map(|level| level.as_i32()),
                };
                let distance = if view == CandidateView::Strict { 0 } else { 1 };
                let level_matches =
                    level.is_some_and(|level| (level - filter.level.as_i32()).abs() <= distance);
                let pos_matches = view == CandidateView::AnyPos
                    || filter.part_of_speech.is_empty()
                    || word
                        .part_of_speech
                        .iter()
                        .any(|pos| filter.part_of_speech.contains(pos));
                if !level_matches || !pos_matches {
                    return Err(BuildError::InvalidPool {
                        view,
                        id: word.id,
                        reason: "row does not match the supplied HSK/PoS view context",
                    });
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn validate_order(words: &[Word], view: CandidateView) -> Result<(), BuildError> {
    if let Some(pair) = words.windows(2).find(|pair| pair[0].id >= pair[1].id) {
        return Err(BuildError::InvalidPool {
            view,
            id: pair[1].id,
            reason: "IDs must be unique and ordered ascending",
        });
    }
    Ok(())
}

#[cfg(test)]
mod word_set_tests {
    use super::*;

    fn word(id: i64) -> Word {
        Word {
            id: WordId::try_from(id).unwrap(),
            simplified: format!("字{id}"),
            traditional: None,
            pinyin: "zì".to_owned(),
            pinyin_numeric: None,
            english: Some(format!("word {id}")),
            part_of_speech: vec![],
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        }
    }

    #[test]
    fn word_sets_accept_mixed_levels_but_reject_extra_rows_and_widening() {
        let selected = BTreeSet::from([WordId::try_from(1).unwrap(), WordId::try_from(2).unwrap()]);
        let pool = CandidatePool {
            strict: vec![word(1), word(2)],
            hsk_widened: vec![word(1), word(2)],
            hsk_widened_any_pos: vec![word(1), word(2)],
        };
        assert!(pool.validate(&BTreeSet::new()).is_ok());
        assert!(pool.validate_set(&selected).is_ok());
        assert!(
            pool.validate_set(&BTreeSet::from([WordId::try_from(1).unwrap()]))
                .is_err()
        );
        let mut widened = pool;
        widened.hsk_widened_any_pos.push(word(3));
        assert!(
            widened
                .validate_set(&BTreeSet::from([
                    WordId::try_from(1).unwrap(),
                    WordId::try_from(2).unwrap(),
                    WordId::try_from(3).unwrap(),
                ]))
                .is_err()
        );
    }
}
