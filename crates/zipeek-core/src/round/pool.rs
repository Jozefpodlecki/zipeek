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
