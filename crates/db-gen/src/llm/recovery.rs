use std::future::Future;

use tracing::warn;

use super::classification::{ClassificationError, ClassificationLabels};

pub const MAX_CLASSIFICATION_ATTEMPTS: usize = 3;

fn is_recoverable(error: &ClassificationError) -> bool {
    matches!(error, ClassificationError::Invalid(_) | ClassificationError::Json(_))
}

pub async fn classify_with_recovery<F, Fut>(
    word_id: i32,
    sense_order: i32,
    mut attempt: F,
) -> Result<ClassificationLabels, ClassificationError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ClassificationLabels, ClassificationError>>,
{
    let mut number = 0;
    loop {
        number += 1;
        match attempt().await {
            Ok(labels) => {
                if number > 1 {
                    tracing::info!(word_id, sense_order, attempts = number, "classification recovered after invalid model output");
                }
                return Ok(labels);
            }
            Err(error) => {
                if !is_recoverable(&error) || number == MAX_CLASSIFICATION_ATTEMPTS {
                    tracing::error!(word_id, sense_order, attempt = number, max_attempts = MAX_CLASSIFICATION_ATTEMPTS, %error, "classification attempts exhausted or non-retryable failure");
                    return Err(error);
                }
                warn!(word_id, sense_order, attempt = number, max_attempts = MAX_CLASSIFICATION_ATTEMPTS, %error, "invalid classification response; retrying same sense");
                tokio::time::sleep(std::time::Duration::from_millis(250 * number as u64)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    use super::*;

    fn valid() -> ClassificationLabels {
        ClassificationLabels { pos: vec!["noun".into()], categories: vec!["food".into()] }
    }

    #[test]
    fn empty_responses_retry_and_eventually_succeed() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let labels = runtime.block_on(classify_with_recovery(1, 1, || {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 { Err(ClassificationError::Invalid("empty".into())) }
                else { Ok(valid()) }
            }
        })).unwrap();
        assert_eq!(labels, valid());
        assert_eq!(calls.load(Ordering::SeqCst), MAX_CLASSIFICATION_ATTEMPTS);
    }

    #[test]
    fn invalid_responses_are_bounded() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let outcome = runtime.block_on(classify_with_recovery(1, 1, || {
            counter.fetch_add(1, Ordering::SeqCst);
            async { Err(ClassificationError::Invalid("empty".into())) }
        }));
        assert!(matches!(outcome, Err(ClassificationError::Invalid(_))));
        assert_eq!(calls.load(Ordering::SeqCst), MAX_CLASSIFICATION_ATTEMPTS);
    }

    #[test]
    fn non_response_errors_are_not_retried() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let outcome = runtime.block_on(classify_with_recovery(1, 1, || {
            counter.fetch_add(1, Ordering::SeqCst);
            async { Err(ClassificationError::Io(std::io::Error::other("disk unavailable"))) }
        }));
        assert!(matches!(outcome, Err(ClassificationError::Io(_))));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
