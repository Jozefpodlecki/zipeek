use crate::{
    AnalysisRequest, EvidenceJournal, RecognitionProjection, RunOutcome, WorkerCoordinator,
    WorkerError, WorkerHealth,
};

pub const WORKER_PROTOCOL_VERSION: u16 = 1;
const MAX_CORRELATION_ID_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerMessage {
    pub version: u16,
    pub correlation_id: String,
    pub command: WorkerCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerCommand {
    SetEnabled {
        enabled: bool,
    },
    Enqueue {
        request: AnalysisRequest,
        now_utc_ms: i64,
    },
    RunNext {
        now_utc_ms: i64,
    },
    RetryFailed {
        job_id: String,
        now_utc_ms: i64,
    },
    GetHealth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerResponse {
    pub version: u16,
    pub correlation_id: String,
    pub result: Result<WorkerReply, ProtocolError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerReply {
    Configured(WorkerHealth),
    Enqueued {
        inserted: bool,
        health: WorkerHealth,
    },
    Ran {
        outcome: RunOutcome,
        health: WorkerHealth,
        projection: Option<RecognitionProjection>,
    },
    Retried(WorkerHealth),
    Health(WorkerHealth),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    UnsupportedVersion(u16),
    InvalidCorrelationId,
    InvalidTimestamp,
    InvalidJobId,
    Coordinator(WorkerError),
}

#[derive(Debug)]
pub struct WorkerEndpoint {
    coordinator: WorkerCoordinator,
}

impl WorkerEndpoint {
    pub fn new(enabled: bool) -> Self {
        Self {
            coordinator: WorkerCoordinator::new(enabled),
        }
    }

    pub fn handle<J>(&mut self, message: WorkerMessage, journal: &J) -> WorkerResponse
    where
        J: EvidenceJournal,
        J::Error: std::fmt::Debug,
    {
        let result = self.dispatch(
            message.version,
            &message.correlation_id,
            message.command,
            journal,
        );
        WorkerResponse {
            version: WORKER_PROTOCOL_VERSION,
            correlation_id: message.correlation_id,
            result,
        }
    }

    fn dispatch<J>(
        &mut self,
        version: u16,
        correlation_id: &str,
        command: WorkerCommand,
        journal: &J,
    ) -> Result<WorkerReply, ProtocolError>
    where
        J: EvidenceJournal,
        J::Error: std::fmt::Debug,
    {
        if version != WORKER_PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }
        if correlation_id.is_empty()
            || correlation_id.len() > MAX_CORRELATION_ID_BYTES
            || !correlation_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(ProtocolError::InvalidCorrelationId);
        }
        match command {
            WorkerCommand::SetEnabled { enabled } => {
                self.coordinator.set_enabled(enabled);
                Ok(WorkerReply::Configured(self.coordinator.health()))
            }
            WorkerCommand::Enqueue {
                request,
                now_utc_ms,
            } => {
                if now_utc_ms < 0 {
                    return Err(ProtocolError::InvalidTimestamp);
                }
                let inserted = self
                    .coordinator
                    .enqueue(request, now_utc_ms)
                    .map_err(ProtocolError::Coordinator)?;
                Ok(WorkerReply::Enqueued {
                    inserted,
                    health: self.coordinator.health(),
                })
            }
            WorkerCommand::RunNext { now_utc_ms } => {
                if now_utc_ms < 0 {
                    return Err(ProtocolError::InvalidTimestamp);
                }
                let outcome = self.coordinator.run_next(journal, now_utc_ms);
                let projection = if matches!(&outcome, RunOutcome::Completed(_)) {
                    self.coordinator.latest_projection().cloned()
                } else {
                    None
                };
                Ok(WorkerReply::Ran {
                    outcome,
                    health: self.coordinator.health(),
                    projection,
                })
            }
            WorkerCommand::RetryFailed { job_id, now_utc_ms } => {
                if now_utc_ms < 0 {
                    return Err(ProtocolError::InvalidTimestamp);
                }
                if job_id.is_empty() || job_id.len() > MAX_CORRELATION_ID_BYTES {
                    return Err(ProtocolError::InvalidJobId);
                }
                self.coordinator
                    .retry_failed(&job_id, now_utc_ms)
                    .map_err(ProtocolError::Coordinator)?;
                Ok(WorkerReply::Retried(self.coordinator.health()))
            }
            WorkerCommand::GetHealth => Ok(WorkerReply::Health(self.coordinator.health())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RecognitionEvent;

    struct EmptyCommittedJournal;

    impl EvidenceJournal for EmptyCommittedJournal {
        type Error = &'static str;

        fn read_all(&self) -> Result<Vec<RecognitionEvent>, Self::Error> {
            Ok(Vec::new())
        }

        fn append_atomic(&mut self, _: &[RecognitionEvent]) -> Result<(), Self::Error> {
            Err("read-only")
        }
    }

    fn message(id: &str, command: WorkerCommand) -> WorkerMessage {
        WorkerMessage {
            version: WORKER_PROTOCOL_VERSION,
            correlation_id: id.into(),
            command,
        }
    }

    fn request() -> AnalysisRequest {
        AnalysisRequest {
            id: "job-1".into(),
            as_of_utc_ms: 100,
        }
    }

    #[test]
    fn rejects_unknown_versions_without_mutation() {
        let mut endpoint = WorkerEndpoint::new(true);
        let mut msg = message(
            "c1",
            WorkerCommand::Enqueue {
                request: request(),
                now_utc_ms: 100,
            },
        );
        msg.version = 2;
        let result = endpoint.handle(msg, &EmptyCommittedJournal);
        assert_eq!(result.version, WORKER_PROTOCOL_VERSION);
        assert_eq!(result.correlation_id, "c1");
        assert_eq!(result.result, Err(ProtocolError::UnsupportedVersion(2)));
        assert_eq!(
            endpoint
                .handle(
                    message("c2", WorkerCommand::GetHealth),
                    &EmptyCommittedJournal
                )
                .result,
            Ok(WorkerReply::Health(WorkerCoordinator::new(true).health()))
        );
    }

    #[test]
    fn rejects_invalid_correlation_ids_and_timestamps() {
        let mut endpoint = WorkerEndpoint::new(true);
        for id in ["", "contains space", "a/b"] {
            assert_eq!(
                endpoint
                    .handle(
                        message(id, WorkerCommand::GetHealth),
                        &EmptyCommittedJournal
                    )
                    .result,
                Err(ProtocolError::InvalidCorrelationId)
            );
        }
        assert_eq!(
            endpoint
                .handle(
                    message("valid", WorkerCommand::RunNext { now_utc_ms: -1 }),
                    &EmptyCommittedJournal
                )
                .result,
            Err(ProtocolError::InvalidTimestamp)
        );
    }

    #[test]
    fn successful_run_only_emits_committed_projection() {
        let mut endpoint = WorkerEndpoint::new(true);
        assert!(matches!(
            endpoint
                .handle(
                    message(
                        "q",
                        WorkerCommand::Enqueue {
                            request: request(),
                            now_utc_ms: 100
                        }
                    ),
                    &EmptyCommittedJournal
                )
                .result,
            Ok(WorkerReply::Enqueued { inserted: true, .. })
        ));
        let result = endpoint.handle(
            message("r", WorkerCommand::RunNext { now_utc_ms: 100 }),
            &EmptyCommittedJournal,
        );
        match result.result.unwrap() {
            WorkerReply::Ran {
                outcome: RunOutcome::Completed(id),
                projection: Some(projection),
                health,
            } => {
                assert_eq!(id, "job-1");
                assert_eq!(projection.accepted_attempts, 0);
                assert_eq!(health.last_success_utc_ms, Some(100));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn disabled_endpoint_never_runs_queued_work() {
        let mut endpoint = WorkerEndpoint::new(true);
        endpoint.handle(
            message(
                "q",
                WorkerCommand::Enqueue {
                    request: request(),
                    now_utc_ms: 100,
                },
            ),
            &EmptyCommittedJournal,
        );
        endpoint.handle(
            message("off", WorkerCommand::SetEnabled { enabled: false }),
            &EmptyCommittedJournal,
        );
        assert!(matches!(
            endpoint
                .handle(
                    message("run", WorkerCommand::RunNext { now_utc_ms: 100 }),
                    &EmptyCommittedJournal
                )
                .result,
            Ok(WorkerReply::Ran {
                outcome: RunOutcome::Disabled,
                projection: None,
                ..
            })
        ));
        endpoint.handle(
            message("on", WorkerCommand::SetEnabled { enabled: true }),
            &EmptyCommittedJournal,
        );
        assert!(matches!(
            endpoint
                .handle(
                    message("run2", WorkerCommand::RunNext { now_utc_ms: 100 }),
                    &EmptyCommittedJournal
                )
                .result,
            Ok(WorkerReply::Ran {
                outcome: RunOutcome::Completed(_),
                ..
            })
        ));
    }

    #[test]
    fn repeated_enqueue_is_idempotent() {
        let mut endpoint = WorkerEndpoint::new(true);
        let cmd = WorkerCommand::Enqueue {
            request: request(),
            now_utc_ms: 100,
        };
        endpoint.handle(message("one", cmd.clone()), &EmptyCommittedJournal);
        assert!(matches!(
            endpoint
                .handle(message("two", cmd), &EmptyCommittedJournal)
                .result,
            Ok(WorkerReply::Enqueued {
                inserted: false,
                ..
            })
        ));
    }
}
