use super::{CachedClassification, ClassificationError};
use crate::import::SourceSense;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use zipeek_core::Word;

pub struct BatchPlan {
    pub cached: Vec<CachedClassification>,
    pub pending: Vec<(Word, SourceSense)>,
}

pub fn plan(
    words: &[Word],
    senses: &[SourceSense],
    cached: Vec<CachedClassification>,
    model: &str,
) -> Result<BatchPlan, ClassificationError> {
    if model.trim().is_empty() {
        return Err(ClassificationError::Invalid("model is empty".into()));
    }
    let words_by_id: BTreeMap<_, _> = words.iter().map(|word| (word.id.as_i32(), word)).collect();
    if words_by_id.len() != words.len() {
        return Err(ClassificationError::Invalid("duplicate word IDs".into()));
    }
    let mut targets = BTreeMap::new();
    for sense in senses {
        let key = (sense.word_id.as_i32(), sense.sense_order);
        let word = words_by_id.get(&key.0).ok_or_else(|| {
            ClassificationError::Invalid(format!("missing word for sense {}:{}", key.0, key.1))
        })?;
        if targets
            .insert(key, ((**word).clone(), sense.clone()))
            .is_some()
        {
            return Err(ClassificationError::Invalid(format!(
                "duplicate source sense {}:{}",
                key.0, key.1
            )));
        }
    }
    let mut valid = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for record in cached {
        let key = (record.word_id, record.sense_order);
        if !seen.insert(key) {
            return Err(ClassificationError::Invalid(format!(
                "duplicate cached sense {}:{}",
                key.0, key.1
            )));
        }
        if record.model != model {
            continue;
        }
        if let Some((_, sense)) = targets.get(&key) {
            if record.validate(sense).is_ok() {
                valid.insert(key, record);
            }
        }
    }
    let pending = targets
        .into_iter()
        .filter_map(|(key, target)| (!valid.contains_key(&key)).then_some(target))
        .collect();
    Ok(BatchPlan {
        cached: valid.into_values().collect(),
        pending,
    })
}

pub(super) fn validate_complete_chunk(
    records: &[CachedClassification],
    chunk: &[(Word, SourceSense)],
    model: &str,
) -> Result<(), ClassificationError> {
    if records.len() != chunk.len() {
        return Err(ClassificationError::Invalid(
            "batch result count differs from request".into(),
        ));
    }
    for (record, (_, sense)) in records.iter().zip(chunk) {
        if record.model != model {
            return Err(ClassificationError::Invalid("batch model mismatch".into()));
        }
        record.validate(sense)?;
    }
    Ok(())
}

pub(super) fn split_front(
    queue: &mut VecDeque<Vec<(Word, SourceSense)>>,
    chunk: Vec<(Word, SourceSense)>,
) {
    let midpoint = chunk.len() / 2;
    let (left, right) = chunk.split_at(midpoint);
    queue.push_front(right.to_vec());
    queue.push_front(left.to_vec());
}
