use serde::{Deserialize, Serialize};
use zipeek_pedagogy::{
    AssessmentFormat, EvidenceJournal, MistakeAllowance, RecognitionEvent, SessionEvidence,
};

use super::error::{WorkerErrorCode, WorkerRuntimeError};

#[derive(Debug, Deserialize, Serialize)]
pub struct EvidenceDto {
    pub attempt_id: String,
    pub character: String,
    pub session_id: String,
    pub session_completed: bool,
    pub occurred_at_utc_ms: i64,
    pub timezone_iana: String,
    pub timezone_offset_minutes: i32,
    pub local_date: String,
    pub mistake_allowance: u8,
    pub evaluator_id: String,
    pub assessment_version: String,
    pub correct: bool,
    pub assisted: bool,
    pub format: FormatDto,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FormatDto {
    IndependentRecall,
    ValidatedChoice { validation_id: String },
    SupportingOnly,
}

impl TryFrom<EvidenceDto> for RecognitionEvent {
    type Error = WorkerRuntimeError;

    fn try_from(value: EvidenceDto) -> Result<Self, Self::Error> {
        let invalid = |message| WorkerRuntimeError::new(WorkerErrorCode::InvalidEvidence, message);
        let mut characters = value.character.chars();
        let character = characters
            .next()
            .ok_or_else(|| invalid("character must be one Hanzi"))?;
        if characters.next().is_some() {
            return Err(invalid("character must contain exactly one scalar value"));
        }
        let mistake_allowance = match value.mistake_allowance {
            0 => MistakeAllowance::Strict,
            1 => MistakeAllowance::One,
            2 => MistakeAllowance::Two,
            _ => return Err(invalid("mistake allowance must be 0, 1 or 2")),
        };
        let format = match value.format {
            FormatDto::IndependentRecall => AssessmentFormat::IndependentRecall,
            FormatDto::ValidatedChoice { validation_id } => {
                AssessmentFormat::ValidatedChoice { validation_id }
            }
            FormatDto::SupportingOnly => AssessmentFormat::SupportingOnly,
        };
        Ok(Self {
            attempt_id: value.attempt_id,
            character,
            session: SessionEvidence {
                session_id: value.session_id,
                completed: value.session_completed,
            },
            occurred_at_utc_ms: value.occurred_at_utc_ms,
            timezone_iana: value.timezone_iana,
            timezone_offset_minutes: value.timezone_offset_minutes,
            local_date: value.local_date,
            mistake_allowance,
            evaluator_id: value.evaluator_id,
            assessment_version: value.assessment_version,
            correct: value.correct,
            assisted: value.assisted,
            format,
        })
    }
}

pub struct CommittedSnapshot(pub Vec<RecognitionEvent>);

impl EvidenceJournal for CommittedSnapshot {
    type Error = &'static str;

    fn read_all(&self) -> Result<Vec<RecognitionEvent>, Self::Error> {
        Ok(self.0.clone())
    }

    fn append_atomic(&mut self, _: &[RecognitionEvent]) -> Result<(), Self::Error> {
        Err("the worker is read-only")
    }
}
