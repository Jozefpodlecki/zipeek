#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

pub mod curriculum;
pub mod grammar;
pub mod knowledge;

mod calendar;
mod clock;
mod journal;
mod monitor;
mod recognition;
mod worker;
mod worker_protocol;

pub use calendar::{CalendarError, local_day_number};
pub use recognition::{
    AssessmentFormat, DailyStatus, DaySummary, EvidenceError, MistakeAllowance, RecognitionEvent,
    RecognitionProjection, RecognitionState, SessionEvidence, project_recognition,
};

pub use journal::{EvidenceJournal, JournalError, append_and_verify, project_committed};

pub use worker::{
    AnalysisRequest, RunOutcome, WorkerCoordinator, WorkerError, WorkerHealth, WorkerPhase,
};

pub use worker_protocol::{
    ProtocolError, WORKER_PROTOCOL_VERSION, WorkerCommand, WorkerEndpoint, WorkerMessage,
    WorkerReply, WorkerResponse,
};

pub use monitor::{
    HealthMonitor, HealthPhase, HealthReply, HealthRequest, HealthSnapshot, MonitorError,
    Resolution, WireHealth,
};

pub use clock::{RecognitionClock, SystemRecognitionClock, project_at_clock};
