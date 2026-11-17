use std::{collections::VecDeque, future::Future, path::Path, time::Instant};

use zipeek_core::Word;

use crate::import::SourceSense;

use super::{
    LlmClient,
    classification::{self, CachedClassification, ClassificationError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchSummary {
    pub reused: usize,
    pub generated: usize,
    pub remaining: usize,
    pub deferred: usize,
}

mod planning;
pub use planning::{BatchPlan, plan};
use planning::{split_front, validate_complete_chunk};

pub async fn run_batch<F, Fut>(
    words: &[Word],
    senses: &[SourceSense],
    cache_path: &Path,
    model: &str,
    limit: usize,
    mut classify: F,
) -> Result<BatchSummary, ClassificationError>
where
    F: FnMut(Word, SourceSense) -> Fut,
    Fut: Future<Output = Result<CachedClassification, ClassificationError>>,
{
    let existing = classification::read_cache(cache_path)?;
    let BatchPlan { cached, pending } = plan(words, senses, existing, model)?;
    let reused = cached.len();
    let mut generated = 0;
    let mut remaining = pending.len();
    let total = reused + remaining;
    let started = Instant::now();
    tracing::info!(
        total,
        reused,
        remaining,
        "sense classification batch started"
    );
    let mut deferred_senses = Vec::new();
    let mut failures = Vec::new();
    let mut attempted = 0;
    super::failure_report::write(cache_path, &failures)?;
    for (word, sense) in pending.into_iter().take(limit) {
        attempted += 1;
        match classify(word.clone(), sense.clone()).await {
            Ok(record) => {
                save_record(record, &sense, model, cache_path)?;
                generated += 1;
                remaining -= 1;
                log_progress(
                    started,
                    total,
                    reused,
                    generated,
                    remaining,
                    attempted,
                    failures.len(),
                );
            }
            Err(error)
                if matches!(
                    error,
                    ClassificationError::Invalid(_) | ClassificationError::Json(_)
                ) =>
            {
                tracing::warn!(word_id = sense.word_id.as_i32(), sense_order = sense.sense_order, %error,
                    "classification deferred after bounded retries");
                failures.push(super::failure_report::DeferredSense::new(
                    &sense, model, &error,
                ));
                deferred_senses.push((word, sense));
                super::failure_report::write(cache_path, &failures)?;
                log_progress(
                    started,
                    total,
                    reused,
                    generated,
                    remaining,
                    attempted,
                    failures.len(),
                );
            }
            Err(error) => {
                tracing::error!(word_id = sense.word_id.as_i32(), sense_order = sense.sense_order, %error,
                    "non-recoverable classification failure; checkpoints retained");
                return Err(error);
            }
        }
    }
    if !deferred_senses.is_empty() {
        tracing::info!(
            deferred = deferred_senses.len(),
            "retrying deferred senses after primary pass"
        );
    }
    let mut unresolved = 0usize;
    for (word, sense) in deferred_senses {
        match classify(word, sense.clone()).await {
            Ok(record) => {
                save_record(record, &sense, model, cache_path)?;
                generated += 1;
                remaining -= 1;
                failures.retain(|failure| {
                    !(failure.word_id == sense.word_id.as_i32()
                        && failure.sense_order == sense.sense_order)
                });
                super::failure_report::write(cache_path, &failures)?;
                tracing::info!(
                    word_id = sense.word_id.as_i32(),
                    sense_order = sense.sense_order,
                    "deferred sense recovered on final pass"
                );
            }
            Err(error)
                if matches!(
                    error,
                    ClassificationError::Invalid(_) | ClassificationError::Json(_)
                ) =>
            {
                if let Some(failure) = failures.iter_mut().find(|failure| {
                    failure.word_id == sense.word_id.as_i32()
                        && failure.sense_order == sense.sense_order
                }) {
                    failure.attempts += super::recovery::MAX_CLASSIFICATION_ATTEMPTS;
                    failure.error = error.to_string();
                }
                super::failure_report::write(cache_path, &failures)?;
                unresolved += 1;
            }
            Err(error) => return Err(error),
        }
    }
    if unresolved != 0 {
        tracing::warn!(deferred = unresolved, report = %super::failure_report::path_for(cache_path).display(),
            "classification incomplete; installed database must not change");
    }
    Ok(BatchSummary {
        reused,
        generated,
        remaining,
        deferred: unresolved,
    })
}

fn save_record(
    record: CachedClassification,
    sense: &SourceSense,
    model: &str,
    cache_path: &Path,
) -> Result<(), ClassificationError> {
    if record.model != model {
        return Err(ClassificationError::Invalid(
            "classifier returned a different model".into(),
        ));
    }
    record.validate(sense)?;
    super::cache_store::checkpoint(cache_path, &record)?;
    Ok(())
}

fn log_progress(
    started: Instant,
    total: usize,
    reused: usize,
    generated: usize,
    remaining: usize,
    attempted: usize,
    deferred: usize,
) {
    let elapsed = started.elapsed().as_secs_f64();
    let per_minute = if elapsed > 0.0 {
        attempted as f64 * 60.0 / elapsed
    } else {
        0.0
    };
    let unattempted = total.saturating_sub(reused + attempted);
    let eta_minutes = if per_minute > 0.0 {
        unattempted as f64 / per_minute
    } else {
        0.0
    };
    let processed = reused + attempted;
    let percent = if total > 0 {
        processed as f64 * 100.0 / total as f64
    } else {
        100.0
    };
    tracing::info!(
        processed,
        total,
        generated,
        reused,
        remaining,
        deferred,
        percent,
        senses_per_minute = per_minute,
        eta_minutes,
        "sense classification progress"
    );
}

pub const DEFAULT_REQUEST_SIZE: usize = 20;
pub const MAX_REQUEST_SIZE: usize = 100;

pub async fn run_with_client(
    client: &LlmClient,
    model: &str,
    words: &[Word],
    senses: &[SourceSense],
    cache_path: &Path,
    limit: usize,
    batch_size: usize,
) -> Result<BatchSummary, ClassificationError> {
    if !(1..=MAX_REQUEST_SIZE).contains(&batch_size) {
        return Err(ClassificationError::Invalid(format!(
            "batch size must be between 1 and {MAX_REQUEST_SIZE}"
        )));
    }
    run_batched(
        words,
        senses,
        cache_path,
        model,
        limit,
        batch_size,
        |items| async move { classification::classify_many(client, model, &items).await },
        |word, sense| async move { classification::classify(client, model, &word, &sense).await },
    )
    .await
}

pub async fn run_batched<B, BFut, S, SFut>(
    words: &[Word],
    senses: &[SourceSense],
    cache_path: &Path,
    model: &str,
    limit: usize,
    batch_size: usize,
    mut classify_batch: B,
    mut classify_single: S,
) -> Result<BatchSummary, ClassificationError>
where
    B: FnMut(Vec<(Word, SourceSense)>) -> BFut,
    BFut: Future<Output = Result<Vec<CachedClassification>, ClassificationError>>,
    S: FnMut(Word, SourceSense) -> SFut,
    SFut: Future<Output = Result<CachedClassification, ClassificationError>>,
{
    if !(1..=MAX_REQUEST_SIZE).contains(&batch_size) || limit == 0 {
        return Err(ClassificationError::Invalid(
            "invalid limit or batch size".into(),
        ));
    }
    let existing = classification::read_cache(cache_path)?;
    let BatchPlan { cached, pending } = plan(words, senses, existing, model)?;
    let reused = cached.len();
    let total = reused + pending.len();
    let mut remaining = pending.len();
    let mut generated = 0usize;
    let started = Instant::now();
    let mut attempted_requests = 0usize;
    let mut unresolved = Vec::new();
    let selected: Vec<_> = pending.into_iter().take(limit).collect();
    let mut queue: VecDeque<Vec<(Word, SourceSense)>> = selected
        .chunks(batch_size)
        .map(|chunk| chunk.to_vec())
        .collect();
    let mut failures = Vec::new();
    super::failure_report::write(cache_path, &failures)?;
    tracing::info!(
        reused,
        pending = remaining,
        batch_size,
        requests = queue.len(),
        "batched classification started"
    );
    while let Some(chunk) = queue.pop_front() {
        attempted_requests += 1;
        let result = if chunk.len() == 1 {
            classify_single(chunk[0].0.clone(), chunk[0].1.clone())
                .await
                .map(|record| vec![record])
        } else {
            classify_batch(chunk.clone()).await
        };
        match result {
            Ok(records) => {
                let check = validate_complete_chunk(&records, &chunk, model);
                if let Err(error) = check {
                    if chunk.len() > 1 {
                        tracing::warn!(size = chunk.len(), %error, "invalid batch response; splitting");
                        split_front(&mut queue, chunk);
                        continue;
                    }
                    return Err(error);
                }
                for ((_, sense), record) in chunk.iter().zip(records) {
                    save_record(record, sense, model, cache_path)?;
                    generated += 1;
                    remaining -= 1;
                }
                log_progress(
                    started,
                    total,
                    reused,
                    generated,
                    remaining,
                    generated + failures.len(),
                    failures.len(),
                );
            }
            Err(error)
                if matches!(
                    error,
                    ClassificationError::Invalid(_) | ClassificationError::Json(_)
                ) =>
            {
                if chunk.len() > 1 {
                    tracing::warn!(size = chunk.len(), %error, "malformed batch; splitting into smaller requests");
                    split_front(&mut queue, chunk);
                } else {
                    let (_, sense) = &chunk[0];
                    failures.push(super::failure_report::DeferredSense::new(
                        sense, model, &error,
                    ));
                    unresolved.push((sense.word_id.as_i32(), sense.sense_order));
                    super::failure_report::write(cache_path, &failures)?;
                }
            }
            Err(error) => return Err(error),
        }
    }
    let elapsed_seconds = started.elapsed().as_secs_f64();
    tracing::info!(
        generated,
        requests = attempted_requests,
        elapsed_seconds,
        generated_senses_per_minute = if elapsed_seconds > 0.0 {
            generated as f64 * 60.0 / elapsed_seconds
        } else {
            0.0
        },
        "batched classification throughput"
    );
    if !unresolved.is_empty() {
        tracing::warn!(deferred = unresolved.len(), report = %super::failure_report::path_for(cache_path).display(),
            "classification incomplete; installed database must not change");
    }
    Ok(BatchSummary {
        reused,
        generated,
        remaining,
        deferred: unresolved.len(),
    })
}

#[cfg(test)]
mod tests;

pub fn failure_report_path(cache: &Path) -> std::path::PathBuf {
    super::failure_report::path_for(cache)
}
