use std::collections::{HashSet, VecDeque};

use crate::{EvidenceJournal, RecognitionProjection, project_committed};

const MAX_PENDING_JOBS: usize = 256;
const MAX_ATTEMPTS: u8 = 3;
const RETRY_BASE_MS: i64 = 1_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisRequest {
    pub id: String,
    pub as_of_utc_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerPhase {
    Disabled,
    Idle,
    Running,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerHealth {
    pub phase: WorkerPhase,
    pub pending_jobs: usize,
    pub failed_jobs: usize,
    pub last_success_utc_ms: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerError {
    Disabled,
    InvalidRequest,
    QueueFull,
    ConflictingRequest(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Disabled,
    Idle,
    Deferred,
    Completed(String),
    RetryScheduled(String),
    PermanentlyFailed(String),
}

#[derive(Debug, Clone)]
struct QueuedJob {
    request: AnalysisRequest,
    attempts: u8,
    due_at_utc_ms: i64,
}

#[derive(Debug, Default)]
pub struct WorkerCoordinator {
    enabled: bool,
    running: bool,
    pending: VecDeque<QueuedJob>,
    known: std::collections::HashMap<String, AnalysisRequest>,
    failed: HashSet<String>,
    last_success_utc_ms: Option<i64>,
    last_error: Option<String>,
    latest_projection: Option<RecognitionProjection>,
}

impl WorkerCoordinator {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enqueue(
        &mut self,
        request: AnalysisRequest,
        now_utc_ms: i64,
    ) -> Result<bool, WorkerError> {
        if !self.enabled {
            return Err(WorkerError::Disabled);
        }
        if request.id.trim().is_empty()
            || request.as_of_utc_ms < 0
            || now_utc_ms < 0
            || request.as_of_utc_ms > now_utc_ms
        {
            return Err(WorkerError::InvalidRequest);
        }
        if let Some(existing) = self.known.get(&request.id) {
            return if existing == &request {
                Ok(false)
            } else {
                Err(WorkerError::ConflictingRequest(request.id))
            };
        }
        if self.pending.len() >= MAX_PENDING_JOBS {
            return Err(WorkerError::QueueFull);
        }
        self.known.insert(request.id.clone(), request.clone());
        self.pending.push_back(QueuedJob {
            request,
            attempts: 0,
            due_at_utc_ms: now_utc_ms,
        });
        Ok(true)
    }

    pub fn retry_failed(&mut self, id: &str, now_utc_ms: i64) -> Result<(), WorkerError> {
        if !self.enabled {
            return Err(WorkerError::Disabled);
        }
        if now_utc_ms < 0 || !self.failed.contains(id) {
            return Err(WorkerError::InvalidRequest);
        }
        if self.pending.len() >= MAX_PENDING_JOBS {
            return Err(WorkerError::QueueFull);
        }
        let request = self
            .known
            .get(id)
            .cloned()
            .ok_or(WorkerError::InvalidRequest)?;
        self.failed.remove(id);
        self.pending.push_back(QueuedJob {
            request,
            attempts: 0,
            due_at_utc_ms: now_utc_ms,
        });
        Ok(())
    }

    pub fn health(&self) -> WorkerHealth {
        let phase = if !self.enabled {
            WorkerPhase::Disabled
        } else if self.running {
            WorkerPhase::Running
        } else if !self.failed.is_empty() {
            WorkerPhase::Failed
        } else {
            WorkerPhase::Idle
        };
        WorkerHealth {
            phase,
            pending_jobs: self.pending.len(),
            failed_jobs: self.failed.len(),
            last_success_utc_ms: self.last_success_utc_ms,
            last_error: self.last_error.clone(),
        }
    }

    pub fn latest_projection(&self) -> Option<&RecognitionProjection> {
        self.latest_projection.as_ref()
    }

    pub fn run_next<J>(&mut self, journal: &J, now_utc_ms: i64) -> RunOutcome
    where
        J: EvidenceJournal,
        J::Error: std::fmt::Debug,
    {
        if !self.enabled {
            return RunOutcome::Disabled;
        }
        if self.running {
            return RunOutcome::Deferred;
        }
        if self.pending.is_empty() {
            return RunOutcome::Idle;
        }
        let due_index = self
            .pending
            .iter()
            .position(|job| job.due_at_utc_ms <= now_utc_ms);
        let Some(index) = due_index else {
            return RunOutcome::Deferred;
        };
        let Some(mut job) = self.pending.remove(index) else {
            return RunOutcome::Idle;
        };
        self.running = true;
        let result = project_committed(journal, job.request.as_of_utc_ms);
        self.running = false;
        match result {
            Ok(projection) => {
                let id = job.request.id;
                self.latest_projection = Some(projection);
                self.last_success_utc_ms = Some(now_utc_ms);
                self.last_error = None;
                RunOutcome::Completed(id)
            }
            Err(error) => {
                job.attempts += 1;
                self.last_error = Some(format!("{error:?}"));
                if job.attempts >= MAX_ATTEMPTS {
                    let id = job.request.id;
                    self.failed.insert(id.clone());
                    RunOutcome::PermanentlyFailed(id)
                } else {
                    let id = job.request.id.clone();
                    let backoff = RETRY_BASE_MS * (1_i64 << (job.attempts - 1));
                    job.due_at_utc_ms = now_utc_ms.saturating_add(backoff);
                    self.pending.push_back(job);
                    RunOutcome::RetryScheduled(id)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RecognitionEvent;

    #[derive(Default)]
    struct TestJournal {
        fail: bool,
    }

    impl EvidenceJournal for TestJournal {
        type Error = &'static str;

        fn read_all(&self) -> Result<Vec<RecognitionEvent>, Self::Error> {
            if self.fail {
                Err("offline")
            } else {
                Ok(Vec::new())
            }
        }

        fn append_atomic(&mut self, _: &[RecognitionEvent]) -> Result<(), Self::Error> {
            Err("read-only")
        }
    }

    fn request(id: &str) -> AnalysisRequest {
        AnalysisRequest {
            id: id.to_owned(),
            as_of_utc_ms: 10_000,
        }
    }

    #[test]
    fn only_committed_evidence_updates_projection() {
        let mut worker = WorkerCoordinator::new(true);
        assert_eq!(worker.enqueue(request("one"), 20_000), Ok(true));
        assert_eq!(
            worker.run_next(&TestJournal::default(), 20_000),
            RunOutcome::Completed("one".into())
        );
        assert_eq!(worker.latest_projection().unwrap().accepted_attempts, 0);
        assert_eq!(worker.health().last_success_utc_ms, Some(20_000));
    }

    #[test]
    fn duplicate_jobs_are_idempotent_but_conflicts_rejected() {
        let mut worker = WorkerCoordinator::new(true);
        assert_eq!(worker.enqueue(request("one"), 20_000), Ok(true));
        assert_eq!(worker.enqueue(request("one"), 20_000), Ok(false));
        let mut conflict = request("one");
        conflict.as_of_utc_ms += 1;
        assert_eq!(
            worker.enqueue(conflict, 20_000),
            Err(WorkerError::ConflictingRequest("one".into()))
        );
        assert_eq!(worker.health().pending_jobs, 1);
    }

    #[test]
    fn retries_are_bounded_and_report_failure() {
        let mut worker = WorkerCoordinator::new(true);
        worker.enqueue(request("one"), 20_000).unwrap();
        let journal = TestJournal { fail: true };
        assert_eq!(
            worker.run_next(&journal, 20_000),
            RunOutcome::RetryScheduled("one".into())
        );
        assert_eq!(worker.run_next(&journal, 20_999), RunOutcome::Deferred);
        assert_eq!(
            worker.run_next(&journal, 21_000),
            RunOutcome::RetryScheduled("one".into())
        );
        assert_eq!(
            worker.run_next(&journal, 23_000),
            RunOutcome::PermanentlyFailed("one".into())
        );
        assert_eq!(worker.health().phase, WorkerPhase::Failed);
        assert_eq!(worker.health().failed_jobs, 1);
        assert!(worker.latest_projection().is_none());
    }

    #[test]
    fn disabling_pauses_jobs_and_preserves_queue() {
        let mut worker = WorkerCoordinator::new(true);
        worker.enqueue(request("one"), 20_000).unwrap();
        worker.set_enabled(false);
        assert_eq!(
            worker.run_next(&TestJournal::default(), 20_000),
            RunOutcome::Disabled
        );
        assert_eq!(worker.health().pending_jobs, 1);
        assert_eq!(worker.health().phase, WorkerPhase::Disabled);
        worker.set_enabled(true);
        assert_eq!(
            worker.run_next(&TestJournal::default(), 20_000),
            RunOutcome::Completed("one".into())
        );
    }

    #[test]
    fn rejected_requests_do_not_mutate_queue() {
        let mut worker = WorkerCoordinator::new(true);
        assert_eq!(
            worker.enqueue(request(""), 20_000),
            Err(WorkerError::InvalidRequest)
        );
        assert_eq!(
            worker.enqueue(request("one"), -1),
            Err(WorkerError::InvalidRequest)
        );
        assert_eq!(
            worker.enqueue(request("one"), 9_999),
            Err(WorkerError::InvalidRequest)
        );
        assert_eq!(worker.health().pending_jobs, 0);
    }
    #[test]
    fn deferred_retry_does_not_block_new_due_job() {
        let mut worker = WorkerCoordinator::new(true);
        worker.enqueue(request("first"), 20_000).unwrap();
        assert_eq!(
            worker.run_next(&TestJournal { fail: true }, 20_000),
            RunOutcome::RetryScheduled("first".into())
        );
        worker.enqueue(request("second"), 20_100).unwrap();
        assert_eq!(
            worker.run_next(&TestJournal::default(), 20_100),
            RunOutcome::Completed("second".into())
        );
    }

    #[test]
    fn failed_job_can_be_requeued_explicitly() {
        let mut worker = WorkerCoordinator::new(true);
        worker.enqueue(request("one"), 20_000).unwrap();
        let failure = TestJournal { fail: true };
        worker.run_next(&failure, 20_000);
        worker.run_next(&failure, 21_000);
        worker.run_next(&failure, 23_000);
        assert_eq!(worker.retry_failed("one", 25_000), Ok(()));
        assert_eq!(worker.health().failed_jobs, 0);
        assert_eq!(
            worker.run_next(&TestJournal::default(), 25_000),
            RunOutcome::Completed("one".into())
        );
    }
}
