mod engine;
mod evidence;
mod ledger;
mod model;
mod policy;
mod projection;

pub use engine::{AssessmentEvent, AssessmentOutcome, KnowledgeEngine, KnowledgeError, Transition};
pub use evidence::{
    AuthorizedTarget, DiagnosticObservation, EvidenceModality, EvidenceTarget,
    EvidenceValidationError, ExerciseManifest, validate_evidence,
};
pub use ledger::{KnowledgeLedger, LedgerError, LedgerOutcome};
pub use model::{
    DailyProgress, KnowledgeCategory, KnowledgeDimension, KnowledgeEntityId, KnowledgeState,
    RelapsePhase,
};
pub use policy::{PolicyError, ScoringPolicy};
pub use projection::{DimensionScore, KnowledgeScore, score_catalogue, score_entity};

#[cfg(test)]
mod tests;
