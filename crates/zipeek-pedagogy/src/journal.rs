use std::collections::HashMap;

use crate::{EvidenceError, RecognitionEvent, RecognitionProjection, project_recognition};

pub trait EvidenceJournal {
    type Error;

    fn read_all(&self) -> Result<Vec<RecognitionEvent>, Self::Error>;
    fn append_atomic(&mut self, events: &[RecognitionEvent]) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum JournalError<E> {
    Storage(E),
    InvalidEvidence(EvidenceError),
    MissingReadback(String),
    ConflictingReadback(String),
}

pub fn project_committed<J: EvidenceJournal>(
    journal: &J,
    as_of_utc_ms: i64,
) -> Result<RecognitionProjection, JournalError<J::Error>> {
    let stored = journal.read_all().map_err(JournalError::Storage)?;
    project_recognition(&stored, as_of_utc_ms).map_err(JournalError::InvalidEvidence)
}

pub fn append_and_verify<J: EvidenceJournal>(
    journal: &mut J,
    incoming: &[RecognitionEvent],
    as_of_utc_ms: i64,
) -> Result<RecognitionProjection, JournalError<J::Error>> {
    let before = journal.read_all().map_err(JournalError::Storage)?;
    let mut candidate = before.clone();
    candidate.extend_from_slice(incoming);
    project_recognition(&candidate, as_of_utc_ms).map_err(JournalError::InvalidEvidence)?;

    if !incoming.is_empty() {
        journal
            .append_atomic(incoming)
            .map_err(JournalError::Storage)?;
    }

    let after = journal.read_all().map_err(JournalError::Storage)?;
    let by_id: HashMap<_, _> = after
        .iter()
        .map(|event| (event.attempt_id.as_str(), event))
        .collect();
    for event in &candidate {
        match by_id.get(event.attempt_id.as_str()) {
            None => return Err(JournalError::MissingReadback(event.attempt_id.clone())),
            Some(saved) if **saved != *event => {
                return Err(JournalError::ConflictingReadback(event.attempt_id.clone()));
            }
            Some(_) => {}
        }
    }
    project_recognition(&after, as_of_utc_ms).map_err(JournalError::InvalidEvidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssessmentFormat, MistakeAllowance, SessionEvidence};

    const DAY_MS: i64 = 86_400_000;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum StoreError {
        Unavailable,
    }

    #[derive(Default)]
    struct MemoryJournal {
        events: Vec<RecognitionEvent>,
        fail_write: bool,
        fail_read: bool,
        discard_write: bool,
        alter_write: bool,
    }

    impl EvidenceJournal for MemoryJournal {
        type Error = StoreError;

        fn read_all(&self) -> Result<Vec<RecognitionEvent>, Self::Error> {
            if self.fail_read {
                Err(StoreError::Unavailable)
            } else {
                Ok(self.events.clone())
            }
        }

        fn append_atomic(&mut self, events: &[RecognitionEvent]) -> Result<(), Self::Error> {
            if self.fail_write {
                return Err(StoreError::Unavailable);
            }
            if self.discard_write {
                return Ok(());
            }
            for event in events {
                if self
                    .events
                    .iter()
                    .any(|existing| existing.attempt_id == event.attempt_id)
                {
                    continue;
                }
                let mut saved = event.clone();
                if self.alter_write {
                    saved.correct = !saved.correct;
                }
                self.events.push(saved);
            }
            Ok(())
        }
    }

    fn event(index: usize) -> RecognitionEvent {
        RecognitionEvent {
            attempt_id: format!("attempt-{index}"),
            character: '学',
            session: SessionEvidence {
                session_id: format!("session-{}", index / 4),
                completed: true,
            },
            occurred_at_utc_ms: 20_000 * DAY_MS + 43_200_000 + index as i64 * 1_000,
            timezone_iana: "Etc/UTC".into(),
            timezone_offset_minutes: 0,
            local_date: "2024-10-04".into(),
            mistake_allowance: MistakeAllowance::Strict,
            evaluator_id: "reference".into(),
            assessment_version: "v1".into(),
            correct: true,
            assisted: false,
            format: AssessmentFormat::IndependentRecall,
        }
    }

    fn ten() -> Vec<RecognitionEvent> {
        (0..10).map(event).collect()
    }

    fn as_of() -> i64 {
        20_001 * DAY_MS
    }

    #[test]
    fn only_verified_committed_evidence_qualifies() {
        let mut journal = MemoryJournal::default();
        let result = append_and_verify(&mut journal, &ten(), as_of()).unwrap();
        assert_eq!(result.states[&'学'].streak_days, 1);
        assert_eq!(journal.events.len(), 10);
        let replay = append_and_verify(&mut journal, &ten(), as_of()).unwrap();
        assert_eq!(replay, result);
        assert_eq!(journal.events.len(), 10);
    }

    #[test]
    fn failed_write_does_not_award_progress() {
        let mut journal = MemoryJournal {
            fail_write: true,
            ..MemoryJournal::default()
        };
        assert_eq!(
            append_and_verify(&mut journal, &ten(), as_of()),
            Err(JournalError::Storage(StoreError::Unavailable))
        );
        assert!(journal.events.is_empty());
    }

    #[test]
    fn successful_write_without_readback_is_rejected() {
        let mut journal = MemoryJournal {
            discard_write: true,
            ..MemoryJournal::default()
        };
        assert_eq!(
            append_and_verify(&mut journal, &ten(), as_of()),
            Err(JournalError::MissingReadback("attempt-0".into()))
        );
    }

    #[test]
    fn modified_readback_is_rejected() {
        let mut journal = MemoryJournal {
            alter_write: true,
            ..MemoryJournal::default()
        };
        assert_eq!(
            append_and_verify(&mut journal, &ten(), as_of()),
            Err(JournalError::ConflictingReadback("attempt-0".into()))
        );
    }

    #[test]
    fn conflicting_attempt_is_rejected_before_write() {
        let mut journal = MemoryJournal::default();
        append_and_verify(&mut journal, &[event(0)], as_of()).unwrap();
        let mut conflict = event(0);
        conflict.correct = false;
        assert_eq!(
            append_and_verify(&mut journal, &[conflict], as_of()),
            Err(JournalError::InvalidEvidence(
                EvidenceError::ConflictingDuplicate("attempt-0".into())
            ))
        );
        assert_eq!(journal.events.len(), 1);
    }

    #[test]
    fn read_failure_is_explicit() {
        let mut journal = MemoryJournal {
            fail_read: true,
            ..MemoryJournal::default()
        };
        assert_eq!(
            append_and_verify(&mut journal, &ten(), as_of()),
            Err(JournalError::Storage(StoreError::Unavailable))
        );
    }

    #[test]
    fn projection_reads_only_committed_evidence() {
        let mut journal = MemoryJournal::default();
        journal.append_atomic(&ten()[..4]).unwrap();
        let snapshot = project_committed(&journal, as_of()).unwrap();
        assert_eq!(snapshot.states[&'学'].streak_days, 0);
        assert_eq!(snapshot.states[&'学'].current_day_correct, 4);
    }
}
