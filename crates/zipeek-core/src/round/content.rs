use rand::{rngs::ChaCha12Rng, seq::SliceRandom};

use super::labels::labels;
use crate::{Word, WordId, practice::ExerciseVariant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub word_id: WordId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundContent {
    Matching {
        left: Vec<Card>,
        right: Vec<Card>,
    },
    Mcq {
        prompt: Card,
        options: Vec<Card>,
        correct_id: WordId,
    },
}

pub(crate) fn shuffled(
    correct: &[&Word],
    reserves: &[&Word],
    variant: ExerciseVariant,
    rng: &mut ChaCha12Rng,
) -> RoundContent {
    let mut left: Vec<_> = correct
        .iter()
        .map(|word| Card {
            word_id: word.id,
            label: labels(word, variant).0.to_owned(),
        })
        .collect();
    let mut right: Vec<_> = correct
        .iter()
        .map(|word| Card {
            word_id: word.id,
            label: labels(word, variant).1.to_owned(),
        })
        .collect();
    if variant.is_mcq() {
        right.extend(reserves.iter().map(|word| Card {
            word_id: word.id,
            label: labels(word, variant).1.to_owned(),
        }));
        right.as_mut_slice().shuffle(rng);
        RoundContent::Mcq {
            prompt: left.remove(0),
            options: right,
            correct_id: correct[0].id,
        }
    } else {
        left.as_mut_slice().shuffle(rng);
        right.as_mut_slice().shuffle(rng);
        RoundContent::Matching { left, right }
    }
}
