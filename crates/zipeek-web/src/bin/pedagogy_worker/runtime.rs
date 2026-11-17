use zipeek_pedagogy::{
    AnalysisRequest, RecognitionEvent, WorkerCommand, WorkerEndpoint, WorkerMessage, WorkerReply,
};

use super::{
    error::{WorkerErrorCode, WorkerRuntimeError},
    evidence::CommittedSnapshot,
    wire::{CommandDto, Envelope, ProjectionDto, ResponseDto, ResultDto},
};

const MAX_EVIDENCE_EVENTS: usize = 10_000;

pub struct WorkerRuntime(WorkerEndpoint);

impl Default for WorkerRuntime {
    fn default() -> Self {
        Self(WorkerEndpoint::new(false))
    }
}

impl WorkerRuntime {
    pub fn handle_json(&mut self, request_json: &str) -> ResponseDto {
        #[derive(serde::Deserialize)]
        struct CorrelationProbe {
            correlation_id: Option<String>,
        }
        let correlation_id = serde_json::from_str::<CorrelationProbe>(request_json)
            .ok()
            .and_then(|probe| probe.correlation_id);
        match serde_json::from_str::<Envelope>(request_json) {
            Ok(request) => self.handle(request),
            Err(error) => ResponseDto::failure(
                correlation_id,
                WorkerRuntimeError::new(
                    WorkerErrorCode::InvalidRequest,
                    format!("invalid request: {error}"),
                ),
            ),
        }
    }

    pub fn handle(&mut self, request: Envelope) -> ResponseDto {
        let id = request.correlation_id.clone();
        match self.dispatch(request) {
            Ok(result) => ResponseDto::success(id, result),
            Err(error) => ResponseDto::failure(Some(id), error),
        }
    }

    fn dispatch(&mut self, request: Envelope) -> Result<ResultDto, WorkerRuntimeError> {
        use super::error::WorkerErrorCode as Code;
        if request.committed_evidence.len() > MAX_EVIDENCE_EVENTS {
            return Err(WorkerRuntimeError::new(
                Code::EvidenceLimitExceeded,
                "evidence snapshot exceeds 10000 events",
            ));
        }
        if matches!(&request.command, CommandDto::RunNext { .. }) && !request.snapshot_verified {
            return Err(WorkerRuntimeError::new(
                Code::UnverifiedSnapshot,
                "verified committed snapshot required",
            ));
        }
        if !matches!(&request.command, CommandDto::RunNext { .. })
            && !request.committed_evidence.is_empty()
        {
            return Err(WorkerRuntimeError::new(
                Code::UnexpectedEvidence,
                "evidence is accepted only for run_next",
            ));
        }

        let events: Result<Vec<RecognitionEvent>, _> = request
            .committed_evidence
            .into_iter()
            .map(RecognitionEvent::try_from)
            .collect();
        let command = match request.command {
            CommandDto::SetEnabled { enabled } => WorkerCommand::SetEnabled { enabled },
            CommandDto::Enqueue {
                id,
                as_of_utc_ms,
                now_utc_ms,
            } => WorkerCommand::Enqueue {
                request: AnalysisRequest { id, as_of_utc_ms },
                now_utc_ms,
            },
            CommandDto::RunNext { now_utc_ms } => WorkerCommand::RunNext { now_utc_ms },
            CommandDto::RetryFailed { job_id, now_utc_ms } => {
                WorkerCommand::RetryFailed { job_id, now_utc_ms }
            }
            CommandDto::GetHealth => WorkerCommand::GetHealth,
        };
        let events = events?;
        let reply = self.0.handle(
            WorkerMessage {
                version: request.version,
                correlation_id: request.correlation_id,
                command,
            },
            &CommittedSnapshot(events),
        );
        match reply.result {
            Err(error) => Err(WorkerRuntimeError::new(
                Code::ProtocolError,
                format!("{error:?}"),
            )),
            Ok(
                WorkerReply::Configured(health)
                | WorkerReply::Health(health)
                | WorkerReply::Retried(health),
            ) => Ok(ResultDto::Health {
                health: health.into(),
            }),
            Ok(WorkerReply::Enqueued { inserted, health }) => Ok(ResultDto::Enqueued {
                inserted,
                health: health.into(),
            }),
            Ok(WorkerReply::Ran {
                outcome,
                health,
                projection,
            }) => {
                let projection = projection.map(|p| ProjectionDto {
                    accepted_attempts: p.accepted_attempts,
                    supporting_attempts: p.supporting_attempts,
                    acquired_characters: p
                        .states
                        .values()
                        .filter(|s| s.acquired_at_utc_ms.is_some())
                        .count(),
                });
                Ok(ResultDto::Ran {
                    outcome: format!("{outcome:?}"),
                    health: health.into(),
                    projection,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pedagogy_worker::{error::WorkerErrorCode, wire::PhaseDto};

    fn health_request(id: &str) -> String {
        format!(r#"{{"version":1,"correlation_id":"{id}","command":{{"kind":"get_health"}}}}"#)
    }

    #[test]
    fn default_worker_is_disabled_and_result_has_v1_shape() {
        let mut runtime = WorkerRuntime::default();
        let reply = runtime.handle_json(&health_request("check_1"));
        let value = serde_json::to_value(&reply).unwrap();
        assert_eq!(value["result"]["health"]["phase"], "disabled");
        assert_eq!(value["correlation_id"], "check_1");
        assert_eq!(value["version"], 1);
        assert!(value.get("error").is_none());
    }

    #[test]
    fn malformed_requests_keep_correlation_id() {
        let mut runtime = WorkerRuntime::default();
        let reply = runtime.handle_json(r#"{"correlation_id":"abc","command":{"kind":"unknown"}}"#);
        assert_eq!(reply.correlation_id.as_deref(), Some("abc"));
        assert_eq!(reply.error_code, Some(WorkerErrorCode::InvalidRequest));
        assert!(reply.result.is_none());
    }

    #[test]
    fn invalid_evidence_is_rejected_before_dispatch() {
        let mut runtime = WorkerRuntime::default();
        let request = r#"{"version":1,"correlation_id":"abc","command":{"kind":"get_health"},"committed_evidence":[{}]}"#;

        assert_eq!(
            runtime.handle_json(request).error_code,
            Some(WorkerErrorCode::InvalidRequest)
        );
    }

    #[test]
    fn rejects_unverified_snapshot() {
        let mut runtime = WorkerRuntime::default();
        let reply = runtime.handle_json(
            r#"{"version":1,"correlation_id":"job","command":{"kind":"run_next","now_utc_ms":0}}"#,
        );
        assert_eq!(reply.error_code, Some(WorkerErrorCode::UnverifiedSnapshot));
    }

    #[test]
    fn endpoint_rejects_unsupported_version_and_remains_disabled() {
        let mut runtime = WorkerRuntime::default();
        let reply = runtime.handle_json(r#"{"version":2,"correlation_id":"abc","command":{"kind":"set_enabled","enabled":true}}"#);
        assert_eq!(reply.error_code, Some(WorkerErrorCode::ProtocolError));
        let health = runtime.handle_json(&health_request("next"));
        match health.result.unwrap() {
            ResultDto::Health { health } => assert_eq!(health.phase, PhaseDto::Disabled),
            _ => panic!("health result expected"),
        }
    }

    #[test]
    fn typed_errors_round_trip() {
        let mut runtime = WorkerRuntime::default();
        let response = runtime.handle_json("not JSON");
        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: ResponseDto = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.error_code, Some(WorkerErrorCode::InvalidRequest));
    }
}
