use serde::{Deserialize, Serialize};
use zipeek_pedagogy::{WORKER_PROTOCOL_VERSION, WorkerHealth, WorkerPhase};

use super::{
    error::{WorkerErrorCode, WorkerRuntimeError},
    evidence::EvidenceDto,
};

#[derive(Debug, Deserialize)]
pub struct Envelope {
    pub version: u16,
    pub correlation_id: String,
    pub command: CommandDto,
    #[serde(default)]
    pub committed_evidence: Vec<EvidenceDto>,
    #[serde(default)]
    pub snapshot_verified: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommandDto {
    SetEnabled {
        enabled: bool,
    },
    Enqueue {
        id: String,
        as_of_utc_ms: i64,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseDto {
    Disabled,
    Idle,
    Running,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerHealthDto {
    pub phase: PhaseDto,
    pub pending_jobs: usize,
    pub failed_jobs: usize,
    pub last_success_utc_ms: Option<i64>,
    pub last_error: Option<String>,
}

impl From<WorkerHealth> for WorkerHealthDto {
    fn from(health: WorkerHealth) -> Self {
        let phase = match health.phase {
            WorkerPhase::Disabled => PhaseDto::Disabled,
            WorkerPhase::Idle => PhaseDto::Idle,
            WorkerPhase::Running => PhaseDto::Running,
            WorkerPhase::Failed => PhaseDto::Failed,
        };
        Self {
            phase,
            pending_jobs: health.pending_jobs,
            failed_jobs: health.failed_jobs,
            last_success_utc_ms: health.last_success_utc_ms,
            last_error: health.last_error,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectionDto {
    pub accepted_attempts: usize,
    pub supporting_attempts: usize,
    pub acquired_characters: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResultDto {
    Ran {
        outcome: String,
        health: WorkerHealthDto,
        projection: Option<ProjectionDto>,
    },
    Enqueued {
        inserted: bool,
        health: WorkerHealthDto,
    },
    Health {
        health: WorkerHealthDto,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseDto {
    pub version: u16,
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ResultDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<WorkerErrorCode>,
}

impl ResponseDto {
    pub fn success(correlation_id: String, result: ResultDto) -> Self {
        Self {
            version: WORKER_PROTOCOL_VERSION,
            correlation_id: Some(correlation_id),
            result: Some(result),
            error: None,
            error_code: None,
        }
    }

    pub fn failure(correlation_id: Option<String>, error: WorkerRuntimeError) -> Self {
        Self {
            version: WORKER_PROTOCOL_VERSION,
            correlation_id,
            result: None,
            error: Some(error.to_string()),
            error_code: Some(error.code()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_preserves_distinct_result_variants() {
        let health = || WorkerHealthDto {
            phase: PhaseDto::Disabled,
            pending_jobs: 0,
            failed_jobs: 0,
            last_success_utc_ms: None,
            last_error: None,
        };
        let variants = [
            ResultDto::Health { health: health() },
            ResultDto::Enqueued {
                inserted: true,
                health: health(),
            },
            ResultDto::Ran {
                outcome: "Idle".into(),
                health: health(),
                projection: None,
            },
        ];
        for (index, variant) in variants.into_iter().enumerate() {
            let encoded = serde_json::to_string(&variant).unwrap();
            let decoded: ResultDto = serde_json::from_str(&encoded).unwrap();
            assert_eq!(
                std::mem::discriminant(&decoded),
                std::mem::discriminant(
                    &[
                        ResultDto::Health { health: health() },
                        ResultDto::Enqueued {
                            inserted: true,
                            health: health()
                        },
                        ResultDto::Ran {
                            outcome: "Idle".into(),
                            health: health(),
                            projection: None
                        },
                    ][index]
                )
            );
        }
    }
}
