use std::collections::BTreeSet;

use super::{
    ExerciseVariant, ParsedPracticeQuery, PracticeFilter, PracticeMode, QueryWarning,
    canonical_seed_input,
};
use crate::{HskSystem, Word, WordId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedDisposition {
    Unseeded,
    Ready(WordId),
    RecoveryRequired(WordId),
}

impl SeedDisposition {
    pub const fn original_id(self) -> Option<WordId> {
        match self {
            Self::Unseeded => None,
            Self::Ready(id) | Self::RecoveryRequired(id) => Some(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPracticeQuery {
    pub mode: PracticeMode,
    pub filter: PracticeFilter,
    pub exclusions: BTreeSet<WordId>,
    pub offset: u64,
    pub seed: SeedDisposition,
    pub warnings: Vec<QueryWarning>,
}

impl ParsedPracticeQuery {
    pub fn normalize<'a>(
        &self,
        variant: ExerciseVariant,
        mut find: impl FnMut(WordId) -> Option<&'a Word>,
    ) -> NormalizedPracticeQuery {
        let mut warnings = self.warnings.clone();
        let exclusions = self
            .exclusions
            .iter()
            .copied()
            .filter(|id| {
                let exists = find(*id).is_some();
                if !exists {
                    warnings.push(QueryWarning {
                        field: "exclude",
                        value: id.to_string(),
                    });
                }
                exists
            })
            .collect::<BTreeSet<_>>();
        let seed = if matches!(self.mode, PracticeMode::Fixed(_)) {
            SeedDisposition::Unseeded
        } else if let Some(id) = self.seed {
            match find(id) {
                Some(word) if !exclusions.contains(&id) => {
                    if word.hsk2_level.is_none() && word.hsk3_level.is_none()
                        || variant.requires_english()
                            && word.english.as_ref().is_none_or(|v| v.trim().is_empty())
                    {
                        SeedDisposition::RecoveryRequired(id)
                    } else {
                        SeedDisposition::Ready(id)
                    }
                }
                _ => {
                    warnings.push(QueryWarning {
                        field: "seed",
                        value: id.to_string(),
                    });
                    SeedDisposition::Unseeded
                }
            }
        } else {
            SeedDisposition::Unseeded
        };
        NormalizedPracticeQuery {
            mode: self.mode,
            filter: self.filter.clone(),
            exclusions,
            offset: if seed.original_id().is_some() {
                0
            } else {
                self.offset
            },
            seed,
            warnings,
        }
    }
}

impl NormalizedPracticeQuery {
    pub fn canonical_seed_input(&self, variant: ExerciseVariant) -> String {
        canonical_seed_input(variant, &self.filter, self.offset, self.seed.original_id())
    }

    pub fn query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = vec![(
            "mode".to_owned(),
            match self.mode {
                PracticeMode::Endless => "endless",
                PracticeMode::Fixed(_) => "fixed",
            }
            .to_owned(),
        )];
        if let PracticeMode::Fixed(rounds) = self.mode {
            pairs.push(("rounds".to_owned(), rounds.count().to_string()));
        }
        if let Some(id) = self.seed.original_id() {
            pairs.push(("seed".to_owned(), id.to_string()));
        }
        pairs.push((
            "hsk".to_owned(),
            match self.filter.level.system() {
                HskSystem::Hsk2 => "2.0",
                HskSystem::Hsk3 => "3.0",
            }
            .to_owned(),
        ));
        pairs.push(("level".to_owned(), self.filter.level.as_i32().to_string()));
        let pos = self.filter.pos_tokens();
        if !pos.is_empty() {
            pairs.push(("pos".to_owned(), pos));
        }
        if !self.exclusions.is_empty() {
            pairs.push((
                "exclude".to_owned(),
                self.exclusions
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if self.seed == SeedDisposition::Unseeded {
            pairs.push(("offset".to_owned(), self.offset.to_string()));
        }
        pairs
    }
}
