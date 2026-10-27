use crate::{HskSystem, WordId};

use super::{ExerciseVariant, PracticeFilter};

pub fn canonical_seed_input(
    variant: ExerciseVariant,
    filter: &PracticeFilter,
    offset: u64,
    seed: Option<WordId>,
) -> String {
    format!(
        "zipeek-round-v1|{}|{}|{}|{}|{}|{}",
        variant.seed_token(),
        match filter.level.system() {
            HskSystem::Hsk2 => "hsk2",
            HskSystem::Hsk3 => "hsk3",
        },
        filter.level.as_i32(),
        filter.pos_tokens(),
        offset,
        seed.map(|id| id.as_i32().to_string())
            .unwrap_or_else(|| "none".to_owned())
    )
}

pub fn fnv1a64(input: &str) -> u64 {
    input
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
}
