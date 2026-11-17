use std::collections::BTreeMap;

use crate::WORKER_PROTOCOL_VERSION;

const MAX_SAFE_JS_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorError {
    InvalidTimeout,
    Closed,
    TimedOut,
    WorkerFailed,
    UnreadableMessage,
    ProtocolMismatch,
    InvalidHealth,
    SendFailed,
    Remote(String),
}

impl MonitorError {
    pub fn description(&self) -> &str {
        match self {
            Self::InvalidTimeout => "timeoutMs must be a positive safe integer",
            Self::Closed => "worker monitor is closed",
            Self::TimedOut => "worker health request timed out",
            Self::WorkerFailed => "worker failed",
            Self::UnreadableMessage => "worker sent an unreadable message",
            Self::ProtocolMismatch => "worker protocol version mismatch",
            Self::InvalidHealth => "invalid worker health response",
            Self::SendFailed => "failed to send worker health request",
            Self::Remote(message) => message,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthPhase {
    Disabled,
    Idle,
    Running,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub phase: HealthPhase,
    pub pending_jobs: u64,
    pub failed_jobs: u64,
    pub last_success_utc_ms: Option<u64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WireHealth {
    pub phase: String,
    pub pending_jobs: f64,
    pub failed_jobs: f64,
    pub last_success_utc_ms: Option<f64>,
    pub last_error: Option<String>,
}

fn safe_nonnegative_integer(number: f64) -> Option<u64> {
    (number.is_finite() && number >= 0.0 && number <= MAX_SAFE_JS_INTEGER && number.fract() == 0.0)
        .then_some(number as u64)
}

impl TryFrom<WireHealth> for HealthSnapshot {
    type Error = MonitorError;

    fn try_from(wire: WireHealth) -> Result<Self, Self::Error> {
        let phase = match wire.phase.as_str() {
            "disabled" => HealthPhase::Disabled,
            "idle" => HealthPhase::Idle,
            "running" => HealthPhase::Running,
            "failed" => HealthPhase::Failed,
            _ => return Err(MonitorError::InvalidHealth),
        };
        Ok(Self {
            phase,
            pending_jobs: safe_nonnegative_integer(wire.pending_jobs)
                .ok_or(MonitorError::InvalidHealth)?,
            failed_jobs: safe_nonnegative_integer(wire.failed_jobs)
                .ok_or(MonitorError::InvalidHealth)?,
            last_success_utc_ms: wire
                .last_success_utc_ms
                .map(|number| safe_nonnegative_integer(number).ok_or(MonitorError::InvalidHealth))
                .transpose()?,
            last_error: wire.last_error,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthRequest {
    pub version: u16,
    pub correlation_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthReply {
    pub version: u16,
    pub correlation_id: String,
    pub health: Result<WireHealth, MonitorError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub correlation_id: String,
    pub result: Result<HealthSnapshot, MonitorError>,
}

#[derive(Debug)]
pub struct HealthMonitor {
    next_id: u64,
    timeout_ms: u64,
    pending: BTreeMap<String, u64>,
    closed: bool,
}

impl HealthMonitor {
    pub fn new(timeout_ms: u64) -> Result<Self, MonitorError> {
        if timeout_ms == 0 || timeout_ms > MAX_SAFE_JS_INTEGER as u64 {
            return Err(MonitorError::InvalidTimeout);
        }
        Ok(Self {
            next_id: 0,
            timeout_ms,
            pending: BTreeMap::new(),
            closed: false,
        })
    }

    pub fn get_health(&mut self, now_ms: u64) -> Result<HealthRequest, MonitorError> {
        if self.closed {
            return Err(MonitorError::Closed);
        }
        self.next_id = self.next_id.checked_add(1).ok_or(MonitorError::Closed)?;
        let correlation_id = format!("health-{}", self.next_id);
        self.pending.insert(
            correlation_id.clone(),
            now_ms.saturating_add(self.timeout_ms),
        );
        Ok(HealthRequest {
            version: WORKER_PROTOCOL_VERSION,
            correlation_id,
        })
    }

    pub fn receive(&mut self, reply: HealthReply) -> Option<Resolution> {
        if self.closed || self.pending.remove(&reply.correlation_id).is_none() {
            return None;
        }
        let result = if reply.version != WORKER_PROTOCOL_VERSION {
            Err(MonitorError::ProtocolMismatch)
        } else {
            reply.health.and_then(TryInto::try_into)
        };
        Some(Resolution {
            correlation_id: reply.correlation_id,
            result,
        })
    }

    pub fn expire(&mut self, now_ms: u64) -> Vec<Resolution> {
        let expired: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, deadline)| **deadline <= now_ms)
            .map(|(id, _)| id.clone())
            .collect();
        expired
            .into_iter()
            .map(|correlation_id| {
                self.pending.remove(&correlation_id);
                Resolution {
                    correlation_id,
                    result: Err(MonitorError::TimedOut),
                }
            })
            .collect()
    }

    pub fn shutdown(&mut self, reason: MonitorError) -> Vec<Resolution> {
        if self.closed {
            return Vec::new();
        }
        self.closed = true;
        std::mem::take(&mut self.pending)
            .into_keys()
            .map(|correlation_id| Resolution {
                correlation_id,
                result: Err(reason.clone()),
            })
            .collect()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy(phase: &str) -> WireHealth {
        WireHealth {
            phase: phase.into(),
            pending_jobs: 0.0,
            failed_jobs: 0.0,
            last_success_utc_ms: None,
            last_error: None,
        }
    }
    fn reply(id: &str, phase: &str) -> HealthReply {
        HealthReply {
            version: WORKER_PROTOCOL_VERSION,
            correlation_id: id.into(),
            health: Ok(healthy(phase)),
        }
    }

    #[test]
    fn health_only_request_with_version_and_checked_timeout() {
        assert_eq!(
            HealthMonitor::new(0).unwrap_err(),
            MonitorError::InvalidTimeout
        );
        assert_eq!(
            HealthMonitor::new(u64::MAX).unwrap_err(),
            MonitorError::InvalidTimeout
        );
        let mut monitor = HealthMonitor::new(10_000).unwrap();
        let request = monitor.get_health(100).unwrap();
        assert_eq!(request.version, WORKER_PROTOCOL_VERSION);
        assert_eq!(request.correlation_id, "health-1");
        assert_eq!(
            monitor
                .receive(reply(&request.correlation_id, "disabled"))
                .unwrap()
                .result
                .unwrap()
                .phase,
            HealthPhase::Disabled
        );
        assert_eq!(monitor.pending_len(), 0);
    }

    #[test]
    fn concurrent_replies_resolve_by_correlation_and_ignore_unknown() {
        let mut monitor = HealthMonitor::new(100).unwrap();
        let a = monitor.get_health(0).unwrap();
        let b = monitor.get_health(0).unwrap();
        assert_ne!(a.correlation_id, b.correlation_id);
        assert!(monitor.receive(reply("unknown", "failed")).is_none());
        assert_eq!(
            monitor
                .receive(reply(&b.correlation_id, "idle"))
                .unwrap()
                .result
                .unwrap()
                .phase,
            HealthPhase::Idle
        );
        assert_eq!(
            monitor
                .receive(reply(&a.correlation_id, "disabled"))
                .unwrap()
                .result
                .unwrap()
                .phase,
            HealthPhase::Disabled
        );
        assert_eq!(monitor.pending_len(), 0);
    }

    #[test]
    fn protocol_mismatch_and_malformed_response_reject_only_matching_request() {
        let mut monitor = HealthMonitor::new(100).unwrap();
        let a = monitor.get_health(0).unwrap();
        let mut bad_version = reply(&a.correlation_id, "idle");
        bad_version.version = 2;
        assert_eq!(
            monitor.receive(bad_version).unwrap().result,
            Err(MonitorError::ProtocolMismatch)
        );
        let b = monitor.get_health(0).unwrap();
        assert_eq!(
            monitor
                .receive(HealthReply {
                    version: 1,
                    correlation_id: b.correlation_id,
                    health: Err(MonitorError::InvalidHealth)
                })
                .unwrap()
                .result,
            Err(MonitorError::InvalidHealth)
        );
    }

    #[test]
    fn remote_worker_initialization_errors_are_preserved() {
        let mut monitor = HealthMonitor::new(100).unwrap();
        let request = monitor.get_health(0).unwrap();
        let error = MonitorError::Remote("worker initialization failed".into());
        assert_eq!(
            monitor
                .receive(HealthReply {
                    version: 1,
                    correlation_id: request.correlation_id,
                    health: Err(error.clone())
                })
                .unwrap()
                .result,
            Err(error)
        );
    }

    #[test]
    fn expiration_releases_only_expired_requests_and_ignores_late_replies() {
        let mut monitor = HealthMonitor::new(10).unwrap();
        let a = monitor.get_health(0).unwrap();
        let b = monitor.get_health(5).unwrap();
        assert!(monitor.expire(9).is_empty());
        assert_eq!(monitor.expire(10).len(), 1);
        assert!(monitor.receive(reply(&a.correlation_id, "idle")).is_none());
        assert_eq!(monitor.pending_len(), 1);
        assert_eq!(
            monitor
                .receive(reply(&b.correlation_id, "idle"))
                .unwrap()
                .result
                .unwrap()
                .phase,
            HealthPhase::Idle
        );
    }

    #[test]
    fn worker_failure_and_shutdown_cancel_all_pending_and_close_idempotently() {
        let mut monitor = HealthMonitor::new(100).unwrap();
        monitor.get_health(0).unwrap();
        monitor.get_health(0).unwrap();
        assert_eq!(monitor.shutdown(MonitorError::WorkerFailed).len(), 2);
        assert_eq!(monitor.pending_len(), 0);
        assert!(monitor.is_closed());
        assert_eq!(monitor.get_health(0), Err(MonitorError::Closed));
        assert!(monitor.shutdown(MonitorError::Closed).is_empty());
    }

    #[test]
    fn invalid_health_fields_rejected_without_unsafe_integer_coercion() {
        let mut variants = Vec::new();
        let mut unknown = healthy("enabled");
        variants.push(unknown.clone());
        unknown.phase = "idle".into();
        unknown.pending_jobs = -1.0;
        variants.push(unknown.clone());
        unknown.pending_jobs = 0.0;
        unknown.failed_jobs = 1.5;
        variants.push(unknown.clone());
        unknown.failed_jobs = 0.0;
        unknown.last_success_utc_ms = Some(f64::NAN);
        variants.push(unknown.clone());
        unknown.last_success_utc_ms = Some(MAX_SAFE_JS_INTEGER + 1.0);
        variants.push(unknown.clone());
        for raw in variants {
            assert_eq!(
                HealthSnapshot::try_from(raw),
                Err(MonitorError::InvalidHealth)
            );
        }
        assert!(HealthSnapshot::try_from(healthy("failed")).is_ok());
        assert!(HealthSnapshot::try_from(healthy("running")).is_ok());
    }

    #[test]
    fn failed_send_can_be_recorded_and_cannot_resolve_other_requests() {
        let mut monitor = HealthMonitor::new(100).unwrap();
        let a = monitor.get_health(0).unwrap();
        let b = monitor.get_health(0).unwrap();
        let failure = HealthReply {
            version: 1,
            correlation_id: a.correlation_id,
            health: Err(MonitorError::SendFailed),
        };
        assert_eq!(
            monitor.receive(failure).unwrap().result,
            Err(MonitorError::SendFailed)
        );
        assert_eq!(monitor.pending_len(), 1);
        assert!(monitor.receive(reply(&b.correlation_id, "idle")).is_some());
    }
}
