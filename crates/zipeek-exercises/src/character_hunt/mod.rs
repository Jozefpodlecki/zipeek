mod gameplay;
mod generation;
mod model;

use model::Answer;
pub use model::{
    Candidate, CharacterHunt, Difficulty, FoundWord, HuntError, HuntSessionStats, TapResult, Tile,
};

#[cfg(test)]
mod tests;
