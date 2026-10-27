use std::collections::BTreeSet;

use rand::{rngs::ChaCha12Rng, seq::IndexedRandom};

use super::{
    CandidateView,
    labels::{answer_key, eligible},
};
use crate::{Word, WordId, practice::ExerciseVariant};

const MAX_SLIDE: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewAttempt {
    pub view: CandidateView,
    pub candidate_count: usize,
    pub window_start: Option<usize>,
    pub correct: Vec<WordId>,
    pub partner_draws: Vec<WordId>,
    pub reserve_draws: Vec<WordId>,
    pub reserves: Vec<WordId>,
    pub rejections: usize,
}

pub(crate) struct Selection<'a> {
    pub correct: Vec<&'a Word>,
    pub reserves: Vec<&'a Word>,
}

pub(crate) fn attempt<'a>(
    words: &'a [Word],
    view: CandidateView,
    variant: ExerciseVariant,
    offset: u64,
    seed: Option<&'a Word>,
    rng: &mut ChaCha12Rng,
) -> (Option<Selection<'a>>, ViewAttempt) {
    let mut trace = ViewAttempt {
        view,
        candidate_count: words.len(),
        window_start: None,
        correct: vec![],
        partner_draws: vec![],
        reserve_draws: vec![],
        reserves: vec![],
        rejections: 0,
    };
    let needed = if variant.is_mcq() { 3 } else { 8 };
    if words.len() < needed {
        return (None, trace);
    }
    let correct = if let Some(seed) = seed {
        seeded_correct(words, seed, variant, rng, &mut trace)
    } else {
        window(words, view, variant, offset, &mut trace)
    };
    let Some(correct) = correct else {
        return (None, trace);
    };
    trace.correct = correct.iter().map(|word| word.id).collect();
    let mut keys: BTreeSet<_> = correct
        .iter()
        .map(|word| answer_key(word, variant))
        .collect();
    let mut reserves = Vec::new();
    let target = if variant.is_mcq() { 2 } else { 4 };
    let cap = words.len().min(50);
    while reserves.len() < target && trace.rejections < cap {
        let Some(candidate) = words.sample(rng, 1).next() else {
            break;
        };
        trace.reserve_draws.push(candidate.id);
        if correct.iter().any(|word| word.id == candidate.id)
            || !eligible(candidate, variant)
            || !keys.insert(answer_key(candidate, variant))
        {
            trace.rejections += 1;
        } else {
            reserves.push(candidate);
        }
    }
    trace.reserves = reserves.iter().map(|word| word.id).collect();
    let selected = (reserves.len() == target).then_some(Selection { correct, reserves });
    (selected, trace)
}

fn seeded_correct<'a>(
    words: &'a [Word],
    seed: &'a Word,
    variant: ExerciseVariant,
    rng: &mut ChaCha12Rng,
    trace: &mut ViewAttempt,
) -> Option<Vec<&'a Word>> {
    let target = if variant.is_mcq() { 1 } else { 4 };
    let mut correct = vec![seed];
    let mut keys = BTreeSet::from([answer_key(seed, variant)]);
    let mut partners: Vec<_> = words.iter().filter(|word| word.id != seed.id).collect();
    while correct.len() < target && !partners.is_empty() {
        let candidate = *partners.as_slice().sample(rng, 1).next()?;
        trace.partner_draws.push(candidate.id);
        partners.retain(|word| word.id != candidate.id);
        if eligible(candidate, variant) && keys.insert(answer_key(candidate, variant)) {
            correct.push(candidate);
        }
    }
    (correct.len() == target).then_some(correct)
}

fn window<'a>(
    words: &'a [Word],
    view: CandidateView,
    variant: ExerciseVariant,
    offset: u64,
    trace: &mut ViewAttempt,
) -> Option<Vec<&'a Word>> {
    let count = if variant.is_mcq() { 1 } else { 4 };
    let last = words.len().checked_sub(count)?;
    let mut start = if view == CandidateView::Strict {
        usize::try_from(offset)
            .ok()
            .filter(|start| *start <= last)
            .unwrap_or(0)
    } else {
        0
    };
    let cap = if variant.is_mcq() {
        1
    } else {
        words.len().min(MAX_SLIDE)
    };
    for _ in 0..cap {
        let correct = &words[start..start + count];
        let keys: BTreeSet<_> = correct
            .iter()
            .map(|word| answer_key(word, variant))
            .collect();
        if correct.iter().all(|word| eligible(word, variant)) && keys.len() == count {
            trace.window_start = Some(start);
            return Some(correct.iter().collect());
        }
        start = if start < last { start + 1 } else { 0 };
    }
    None
}
