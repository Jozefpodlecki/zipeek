use std::collections::BTreeSet;

use super::{ExerciseVariant, FixedRounds};
use crate::WordId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffsetAdvance {
    pub offset: u64,
    pub looped: bool,
}

pub fn advance_offset(offset: u64, pool_size: u64, variant: ExerciseVariant) -> OffsetAdvance {
    let step = if variant.is_mcq() { 1 } else { 4 };
    match offset.checked_add(step) {
        Some(next) if next <= pool_size => OffsetAdvance {
            offset: next,
            looped: false,
        },
        _ => OffsetAdvance {
            offset: 0,
            looped: true,
        },
    }
}

pub fn union_exclusions<'a>(
    sets: impl IntoIterator<Item = &'a BTreeSet<WordId>>,
) -> BTreeSet<WordId> {
    sets.into_iter()
        .flat_map(|set| set.iter().copied())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedProgress {
    rounds: FixedRounds,
    completed: u8,
}

impl FixedProgress {
    pub fn new(rounds: FixedRounds) -> Self {
        Self {
            rounds,
            completed: 0,
        }
    }
    pub fn completed(self) -> u8 {
        self.completed
    }
    pub fn is_complete(self) -> bool {
        self.completed == self.rounds.count()
    }
    pub fn finish_round(&mut self) {
        self.completed = self.completed.saturating_add(1).min(self.rounds.count());
    }
}
