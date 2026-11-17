use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use zipeek_core::WordId;

fn fixtures() -> (Vec<Word>, Vec<SourceSense>) {
    let id = WordId::try_from(7).unwrap();
    (
        vec![Word {
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
        }],
        vec![SourceSense {
            word_id: id,
            sense_order: 1,
            gloss: "ticket".into(),
            source: "hsk-primary",
            source_ref: 1,
            source_gloss_order: 1,
        }],
    )
}

#[test]
fn malformed_batch_splits_and_saves_each_sense() {
    let (words, mut senses) = fixtures();
    for order in 2..=4 {
        let mut sense = senses[0].clone();
        sense.sense_order = order;
        sense.gloss = format!("definition {order}");
        senses.push(sense);
    }
    let path = std::env::temp_dir().join(format!(
        "zipeek-batched-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let batch_calls = Arc::new(AtomicUsize::new(0));
    let calls = batch_calls.clone();
    let result = runtime
        .block_on(run_batched(
            &words,
            &senses,
            &path,
            "local",
            4,
            4,
            move |items| {
                calls.fetch_add(1, Ordering::SeqCst);
                async move {
                    if items.len() == 4 {
                        return Err(ClassificationError::Invalid("truncated batch".into()));
                    }
                    Ok(items
                        .iter()
                        .map(|(_, sense)| CachedClassification {
                            word_id: sense.word_id.as_i32(),
                            sense_order: sense.sense_order,
                            input_digest: classification::digest_for_test(sense),
                            model: "local".into(),
                            prompt_version: classification::PROMPT_VERSION.into(),
                            taxonomy_version: classification::TAXONOMY_VERSION,
                            labels: classification::parse_model_response(
                                r#"{"pos":["noun"],"categories":[]}"#,
                            )
                            .unwrap(),
                        })
                        .collect())
                }
            },
            |word, sense| async move {
                Ok(CachedClassification {
                    word_id: word.id.as_i32(),
                    sense_order: sense.sense_order,
                    input_digest: classification::digest_for_test(&sense),
                    model: "local".into(),
                    prompt_version: classification::PROMPT_VERSION.into(),
                    taxonomy_version: classification::TAXONOMY_VERSION,
                    labels: classification::parse_model_response(
                        r#"{"pos":["noun"],"categories":[]}"#,
                    )
                    .unwrap(),
                })
            },
        ))
        .unwrap();
    assert_eq!(result.generated, 4);
    assert_eq!(result.remaining, 0);
    assert_eq!(batch_calls.load(Ordering::SeqCst), 3);
    assert_eq!(classification::read_cache(&path).unwrap().len(), 4);
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir_all(crate::llm::cache_store::records_directory(&path)).unwrap();
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
    assert!(
        plan(
            &words,
            &[senses[0].clone(), senses[0].clone()],
            vec![],
            "local"
        )
        .is_err()
    );
    assert!(plan(&[], &senses, vec![], "local").is_err());
    assert!(plan(&words, &senses, vec![], "  ").is_err());
}

#[test]
fn successful_inference_is_saved_and_reused() {
    let (words, senses) = fixtures();
    let path = std::env::temp_dir().join(format!(
        "zipeek-batch-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let calls = Arc::new(AtomicUsize::new(0));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let initial = calls.clone();
    let first = runtime
        .block_on(run_batch(
            &words,
            &senses,
            &path,
            "local",
            10,
            move |word, sense| {
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
            },
        ))
        .unwrap();
    assert_eq!(first.generated, 1);
    assert_eq!(first.remaining, 0);
    assert_eq!(classification::read_cache(&path).unwrap().len(), 1);
    let second = runtime
        .block_on(run_batch(
            &words,
            &senses,
            &path,
            "local",
            10,
            |_word, _sense| async {
                Err(ClassificationError::Invalid("unexpected invocation".into()))
            },
        ))
        .unwrap();
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
        "zipeek-deferred-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let attempts = calls.clone();
    let first = runtime
        .block_on(run_batch(
            &words,
            &senses,
            &path,
            "local",
            2,
            move |word, sense| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async move {
                    if sense.sense_order == 1 {
                        return Err(ClassificationError::Invalid("empty final text".into()));
                    }
                    Ok(CachedClassification {
                        word_id: word.id.as_i32(),
                        sense_order: sense.sense_order,
                        input_digest: classification::digest_for_test(&sense),
                        model: "local".into(),
                        prompt_version: classification::PROMPT_VERSION.into(),
                        taxonomy_version: classification::TAXONOMY_VERSION,
                        labels: classification::parse_model_response(
                            r#"{"pos":["noun"],"categories":[]}"#,
                        )?,
                    })
                }
            },
        ))
        .unwrap();
    assert_eq!(first.generated, 1);
    assert_eq!(first.remaining, 1);
    assert_eq!(first.deferred, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    let report = failure_report_path(&path);
    let content = std::fs::read_to_string(&report).unwrap();
    assert!(content.contains("empty final text"));
    assert!(content.contains("\"attempts\":6"));
    let second = runtime
        .block_on(run_batch(
            &words,
            &senses,
            &path,
            "local",
            2,
            |word, sense| async move {
                Ok(CachedClassification {
                    word_id: word.id.as_i32(),
                    sense_order: sense.sense_order,
                    input_digest: classification::digest_for_test(&sense),
                    model: "local".into(),
                    prompt_version: classification::PROMPT_VERSION.into(),
                    taxonomy_version: classification::TAXONOMY_VERSION,
                    labels: classification::parse_model_response(
                        r#"{"pos":["noun"],"categories":[]}"#,
                    )?,
                })
            },
        ))
        .unwrap();
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
        "zipeek-transport-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let result = runtime.block_on(run_batch(
        &words,
        &senses,
        &path,
        "local",
        1,
        |_word, _sense| async { Err(ClassificationError::Io(std::io::Error::other("disk error"))) },
    ));
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
        "zipeek-resume-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let first = runtime.block_on(run_batch(
        &words,
        &senses,
        &path,
        "local",
        10,
        |word, sense| async move {
            if sense.sense_order == 2 {
                return Err(ClassificationError::Invalid(
                    "simulated interruption".into(),
                ));
            }
            Ok(CachedClassification {
                word_id: word.id.as_i32(),
                sense_order: sense.sense_order,
                input_digest: classification::digest_for_test(&sense),
                model: "local".into(),
                prompt_version: classification::PROMPT_VERSION.into(),
                taxonomy_version: classification::TAXONOMY_VERSION,
                labels: classification::parse_model_response(
                    r#"{"pos":["noun"],"categories":[]}"#,
                )?,
            })
        },
    ));
    assert_eq!(first.unwrap().deferred, 1);
    assert_eq!(classification::read_cache(&path).unwrap().len(), 1);
    let next = runtime
        .block_on(run_batch(
            &words,
            &senses,
            &path,
            "local",
            10,
            |word, sense| async move {
                Ok(CachedClassification {
                    word_id: word.id.as_i32(),
                    sense_order: sense.sense_order,
                    input_digest: classification::digest_for_test(&sense),
                    model: "local".into(),
                    prompt_version: classification::PROMPT_VERSION.into(),
                    taxonomy_version: classification::TAXONOMY_VERSION,
                    labels: classification::parse_model_response(
                        r#"{"pos":["noun"],"categories":[]}"#,
                    )?,
                })
            },
        ))
        .unwrap();
    assert_eq!(next.reused, 1);
    assert_eq!(next.generated, 1);
    assert_eq!(next.remaining, 0);
    assert_eq!(classification::read_cache(&path).unwrap().len(), 2);
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir_all(crate::llm::cache_store::records_directory(&path)).unwrap();
}
