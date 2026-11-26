use std::collections::{BTreeMap, BTreeSet};

use super::{
    AssessmentEvent, EvidenceTarget, KnowledgeEngine, KnowledgeError, KnowledgeState, Transition,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KnowledgeLedger {
    pub states: BTreeMap<EvidenceTarget, KnowledgeState>,
    pub completed_attempts: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerError {
    MissingTarget,
    MismatchedAttempt,
    DuplicateTarget,
    Scoring(KnowledgeError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedgerOutcome {
    DuplicateAttempt,
    Applied(Vec<(EvidenceTarget, Transition)>),
}

impl KnowledgeLedger {
    pub fn apply_validated_batch(
        &mut self,
        engine: &KnowledgeEngine,
        events: Vec<AssessmentEvent>,
    ) -> Result<LedgerOutcome, LedgerError> {
        let Some(first) = events.first() else {
            return Ok(LedgerOutcome::Applied(Vec::new()));
        };
        let attempt_id = first.attempt_id.clone();
        if self.completed_attempts.contains(&attempt_id) {
            return Ok(LedgerOutcome::DuplicateAttempt);
        }
        let mut next = self.clone();
        let mut targets = BTreeSet::new();
        let mut transitions = Vec::with_capacity(events.len());
        for event in events {
            if event.attempt_id != attempt_id {
                return Err(LedgerError::MismatchedAttempt);
            }
            let target = EvidenceTarget {
                entity: event.entity.clone(),
                dimension: event.dimension,
            };
            if !targets.insert(target.clone()) {
                return Err(LedgerError::DuplicateTarget);
            }
            let state = next
                .states
                .get_mut(&target)
                .ok_or(LedgerError::MissingTarget)?;
            let transition = engine.apply(state, event).map_err(LedgerError::Scoring)?;
            transitions.push((target, transition));
        }
        next.completed_attempts.insert(attempt_id);
        *self = next;
        Ok(LedgerOutcome::Applied(transitions))
    }
}
