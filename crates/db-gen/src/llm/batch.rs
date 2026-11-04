use std::{collections::{BTreeMap, BTreeSet}, future::Future, path::Path, time::Instant};

use zipeek_core::Word;

use crate::import::SourceSense;

use super::{LlmClient, classification::{self, CachedClassification, ClassificationError}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchSummary {
    pub reused: usize,
    pub generated: usize,
    pub remaining: usize,
    pub deferred: usize,
}

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
        if targets.insert(key, ((**word).clone(), sense.clone())).is_some() {
            return Err(ClassificationError::Invalid(format!("duplicate source sense {}:{}", key.0, key.1)));
        }
    }
    let mut valid = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for record in cached {
        let key = (record.word_id, record.sense_order);
        if !seen.insert(key) {
            return Err(ClassificationError::Invalid(format!("duplicate cached sense {}:{}", key.0, key.1)));
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
    let pending = targets.into_iter().filter_map(|(key, target)| {
        (!valid.contains_key(&key)).then_some(target)
    }).collect();
    Ok(BatchPlan { cached: valid.into_values().collect(), pending })
}

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
    tracing::info!(total, reused, remaining, "sense classification batch started");
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
                log_progress(started, total, reused, generated, remaining, attempted, failures.len());
            }
            Err(error) if matches!(error, ClassificationError::Invalid(_) | ClassificationError::Json(_)) => {
                tracing::warn!(word_id = sense.word_id.as_i32(), sense_order = sense.sense_order, %error,
                    "classification deferred after bounded retries");
                failures.push(super::failure_report::DeferredSense::new(&sense, model, &error));
                deferred_senses.push((word, sense));
                super::failure_report::write(cache_path, &failures)?;
                log_progress(started, total, reused, generated, remaining, attempted, failures.len());
            }
            Err(error) => {
                tracing::error!(word_id = sense.word_id.as_i32(), sense_order = sense.sense_order, %error,
                    "non-recoverable classification failure; checkpoints retained");
                return Err(error);
            }
        }
    }
    if !deferred_senses.is_empty() {
        tracing::info!(deferred = deferred_senses.len(), "retrying deferred senses after primary pass");
    }
    let mut unresolved = 0usize;
    for (word, sense) in deferred_senses {
        match classify(word, sense.clone()).await {
            Ok(record) => {
                save_record(record, &sense, model, cache_path)?;
                generated += 1;
                remaining -= 1;
                failures.retain(|failure| !(failure.word_id == sense.word_id.as_i32()
                    && failure.sense_order == sense.sense_order));
                super::failure_report::write(cache_path, &failures)?;
                tracing::info!(word_id = sense.word_id.as_i32(), sense_order = sense.sense_order,
                    "deferred sense recovered on final pass");
            }
            Err(error) if matches!(error, ClassificationError::Invalid(_) | ClassificationError::Json(_)) => {
                if let Some(failure) = failures.iter_mut().find(|failure|
                    failure.word_id == sense.word_id.as_i32() && failure.sense_order == sense.sense_order) {
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
    Ok(BatchSummary { reused, generated, remaining, deferred: unresolved })
}

fn save_record(
    record: CachedClassification,
    sense: &SourceSense,
    model: &str,
    cache_path: &Path,
) -> Result<(), ClassificationError> {
    if record.model != model {
        return Err(ClassificationError::Invalid("classifier returned a different model".into()));
    }
    record.validate(sense)?;
    super::cache_store::checkpoint(cache_path, &record)?;
    Ok(())
}

fn log_progress(
    started: Instant, total: usize, reused: usize, generated: usize,
    remaining: usize, attempted: usize, deferred: usize,
) {
    let elapsed = started.elapsed().as_secs_f64();
    let per_minute = if elapsed > 0.0 { attempted as f64 * 60.0 / elapsed } else { 0.0 };
    let unattempted = total.saturating_sub(reused + attempted);
    let eta_minutes = if per_minute > 0.0 { unattempted as f64 / per_minute } else { 0.0 };
    let processed = reused + attempted;
    let percent = if total > 0 { processed as f64 * 100.0 / total as f64 } else { 100.0 };
    tracing::info!(processed, total, generated, reused, remaining, deferred, percent,
        senses_per_minute = per_minute, eta_minutes, "sense classification progress");
}

pub async fn run_with_client(
    client: &LlmClient,
    model: &str,
    words: &[Word],
    senses: &[SourceSense],
    cache_path: &Path,
    limit: usize,
) -> Result<BatchSummary, ClassificationError> {
    run_batch(words, senses, cache_path, model, limit, |word, sense| async move {
        classification::classify(client, model, &word, &sense).await
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_core::WordId;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn fixtures() -> (Vec<Word>, Vec<SourceSense>) {
        let id = WordId::try_from(7).unwrap();
        (vec![Word {
            id,
            simplified: "车票".into(),
            traditional: None,
            pinyin: "chē piào".into(),
            pinyin_numeric: None,
            english: Some("ticket".into()),
            part_of_speech: vec![],
            hsk2_level: None,
            hsk3_level: None,
            frequency: None,
        }], vec![SourceSense {
            word_id: id,
            sense_order: 1,
            gloss: "ticket".into(),
            source: "hsk-primary",
            source_ref: 1,
            source_gloss_order: 1,
        }])
    }

    #[test]
    fn pending_targets_are_deterministic() {
        let (words, senses) = fixtures();
        let first = plan(&words, &senses, vec![], "local").unwrap();
        assert_eq!(first.pending.len(), 1);
        assert_eq!(first.pending[0].1.gloss, "ticket");
        assert!(first.cached.is_empty());
    }

    #[test]
    fn duplicate_senses_and_missing_words_are_rejected() {
        let (words, senses) = fixtures();
        assert!(plan(&words, &[senses[0].clone(), senses[0].clone()], vec![], "local").is_err());
        assert!(plan(&[], &senses, vec![], "local").is_err());
        assert!(plan(&words, &senses, vec![], "  ").is_err());
    }

    #[test]
    fn successful_inference_is_saved_and_reused() {
        let (words, senses) = fixtures();
        let path = std::env::temp_dir().join(format!(
            "zipeek-batch-{}-{}.jsonl", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let calls = Arc::new(AtomicUsize::new(0));
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let initial = calls.clone();
        let first = runtime.block_on(run_batch(&words, &senses, &path, "local", 10, move |word, sense| {
            initial.fetch_add(1, Ordering::SeqCst);
            async move {
                let text = r#"{"pos":["noun"],"categories":["travel"]}"#;
                let labels = classification::parse_model_response(text)?;
                Ok(CachedClassification {
                    word_id: word.id.as_i32(),
                    sense_order: sense.sense_order,
                    input_digest: classification::digest_for_test(&sense),
                    model: "local".into(),
                    prompt_version: classification::PROMPT_VERSION.into(),
                    taxonomy_version: classification::TAXONOMY_VERSION,
                    labels,
                })
            }
        })).unwrap();
        assert_eq!(first.generated, 1);
        assert_eq!(first.remaining, 0);
        assert_eq!(classification::read_cache(&path).unwrap().len(), 1);
        let second = runtime.block_on(run_batch(&words, &senses, &path, "local", 10, |_word, _sense| async {
            Err(ClassificationError::Invalid("unexpected invocation".into()))
        })).unwrap();
        assert_eq!(second.reused, 1);
        assert_eq!(second.generated, 0);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir_all(crate::llm::cache_store::records_directory(&path)).unwrap();
    }

    #[test]
    fn deferred_sense_does_not_block_later_senses_and_report_is_cleared_on_resume() {
        let (words, mut senses) = fixtures();
        let mut next = senses[0].clone();
        next.sense_order = 2;
        next.gloss = "fare".into();
        senses.push(next);
        let path = std::env::temp_dir().join(format!(
            "zipeek-deferred-{}-{}.jsonl", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let attempts = calls.clone();
        let first = runtime.block_on(run_batch(&words, &senses, &path, "local", 2, move |word, sense| {
            attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if sense.sense_order == 1 {
                    return Err(ClassificationError::Invalid("empty final text".into()));
                }
                Ok(CachedClassification {
                    word_id: word.id.as_i32(), sense_order: sense.sense_order,
                    input_digest: classification::digest_for_test(&sense),
                    model: "local".into(), prompt_version: classification::PROMPT_VERSION.into(),
                    taxonomy_version: classification::TAXONOMY_VERSION,
                    labels: classification::parse_model_response(r#"{"pos":["noun"],"categories":[]}"#)?,
                })
            }
        })).unwrap();
        assert_eq!(first.generated, 1);
        assert_eq!(first.remaining, 1);
        assert_eq!(first.deferred, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        let report = failure_report_path(&path);
        let content = std::fs::read_to_string(&report).unwrap();
        assert!(content.contains("empty final text"));
        assert!(content.contains("\"attempts\":6"));
        let second = runtime.block_on(run_batch(&words, &senses, &path, "local", 2, |word, sense| async move {
            Ok(CachedClassification {
                word_id: word.id.as_i32(), sense_order: sense.sense_order,
                input_digest: classification::digest_for_test(&sense),
                model: "local".into(), prompt_version: classification::PROMPT_VERSION.into(),
                taxonomy_version: classification::TAXONOMY_VERSION,
                labels: classification::parse_model_response(r#"{"pos":["noun"],"categories":[]}"#)?,
            })
        })).unwrap();
        assert_eq!(second.reused, 1);
        assert_eq!(second.generated, 1);
        assert_eq!(second.deferred, 0);
        assert!(!report.exists());
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir_all(crate::llm::cache_store::records_directory(&path)).unwrap();
    }

    #[test]
    fn transport_error_stops_immediately() {
        let (words, senses) = fixtures();
        let path = std::env::temp_dir().join(format!(
            "zipeek-transport-{}-{}.jsonl", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = runtime.block_on(run_batch(&words, &senses, &path, "local", 1, |_word, _sense| async {
            Err(ClassificationError::Io(std::io::Error::other("disk error")))
        }));
        assert!(matches!(result, Err(ClassificationError::Io(_))));
        assert!(!path.exists());
    }

    #[test]
    fn failed_inference_preserves_successful_checkpoint_for_resume() {
        let (words, mut senses) = fixtures();
        let mut another = senses[0].clone();
        another.sense_order = 2;
        another.gloss = "fare".into();
        senses.push(another);
        let path = std::env::temp_dir().join(format!(
            "zipeek-resume-{}-{}.jsonl", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let first = runtime.block_on(run_batch(&words, &senses, &path, "local", 10, |word, sense| async move {
            if sense.sense_order == 2 {
                return Err(ClassificationError::Invalid("simulated interruption".into()));
            }
            Ok(CachedClassification {
                word_id: word.id.as_i32(),
                sense_order: sense.sense_order,
                input_digest: classification::digest_for_test(&sense),
                model: "local".into(),
                prompt_version: classification::PROMPT_VERSION.into(),
                taxonomy_version: classification::TAXONOMY_VERSION,
                labels: classification::parse_model_response(r#"{"pos":["noun"],"categories":[]}"#)?,
            })
        }));
        assert_eq!(first.unwrap().deferred, 1);
        assert_eq!(classification::read_cache(&path).unwrap().len(), 1);
        let next = runtime.block_on(run_batch(&words, &senses, &path, "local", 10, |word, sense| async move {
            Ok(CachedClassification {
                word_id: word.id.as_i32(),
                sense_order: sense.sense_order,
                input_digest: classification::digest_for_test(&sense),
                model: "local".into(),
                prompt_version: classification::PROMPT_VERSION.into(),
                taxonomy_version: classification::TAXONOMY_VERSION,
                labels: classification::parse_model_response(r#"{"pos":["noun"],"categories":[]}"#)?,
            })
        })).unwrap();
        assert_eq!(next.reused, 1);
        assert_eq!(next.generated, 1);
        assert_eq!(next.remaining, 0);
        assert_eq!(classification::read_cache(&path).unwrap().len(), 2);
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir_all(crate::llm::cache_store::records_directory(&path)).unwrap();
    }

}

pub fn failure_report_path(cache: &Path) -> std::path::PathBuf {
    super::failure_report::path_for(cache)
}
