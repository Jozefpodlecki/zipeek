use std::collections::{BTreeMap, BTreeSet};

use super::{AssessmentEvent, AssessmentOutcome, KnowledgeDimension, KnowledgeEntityId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceModality {
    WrittenChinese,
    AudioChinese,
    SpokenChinese,
    FreeTextChinese,
    WordBank,
    HandDrawn,
    GrammarConstruction,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EvidenceTarget {
    pub entity: KnowledgeEntityId,
    pub dimension: KnowledgeDimension,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedTarget {
    pub modality: EvidenceModality,
    pub requires_independent_production: bool,
    pub required_readings: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExerciseManifest {
    pub attempt_id: String,
    pub policy_epoch: u64,
    pub policy_day: i64,
    pub context_family: String,
    pub targets: BTreeMap<EvidenceTarget, AuthorizedTarget>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticObservation {
    pub target: EvidenceTarget,
    pub outcome: AssessmentOutcome,
    pub modality: EvidenceModality,
    pub independently_demonstrated: bool,
    pub response_was_cued: bool,
    pub reading_key: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceValidationError {
    InvalidManifest,
    UnauthorizedTarget,
    DuplicateTarget,
    InvalidDimension,
    WrongModality,
    AssistedResponse,
    UnsubstantiatedOutcome,
    InvalidReading,
}

pub fn validate_evidence(
    manifest: &ExerciseManifest,
    observations: &[DiagnosticObservation],
) -> Result<Vec<AssessmentEvent>, EvidenceValidationError> {
    if manifest.attempt_id.trim().is_empty()
        || manifest.attempt_id.len() > 256
        || manifest.context_family.trim().is_empty()
        || manifest.context_family.len() > 256
        || manifest.targets.is_empty()
    {
        return Err(EvidenceValidationError::InvalidManifest);
    }
    for (target, authorization) in &manifest.targets {
        if !target
            .entity
            .category
            .dimensions()
            .contains(&target.dimension)
        {
            return Err(EvidenceValidationError::InvalidDimension);
        }
        if target.entity.category == super::KnowledgeCategory::Hanzi
            && target.dimension == KnowledgeDimension::Pronunciation
        {
            if authorization.required_readings.is_empty() {
                return Err(EvidenceValidationError::InvalidReading);
            }
        } else if !authorization.required_readings.is_empty() {
            return Err(EvidenceValidationError::InvalidReading);
        }
    }
    let mut seen = BTreeSet::new();
    let mut validated = Vec::with_capacity(observations.len());
    for observation in observations {
        let authorization = manifest
            .targets
            .get(&observation.target)
            .ok_or(EvidenceValidationError::UnauthorizedTarget)?;
        if !seen.insert(observation.target.clone()) {
            return Err(EvidenceValidationError::DuplicateTarget);
        }
        if authorization.modality != observation.modality
            || !modality_supports_target(&observation.target, observation.modality)
        {
            return Err(EvidenceValidationError::WrongModality);
        }
        if observation.response_was_cued
            && (authorization.requires_independent_production
                || matches!(
                    observation.outcome,
                    AssessmentOutcome::Correct | AssessmentOutcome::Incorrect
                ))
        {
            return Err(EvidenceValidationError::AssistedResponse);
        }
        if matches!(
            observation.outcome,
            AssessmentOutcome::Correct | AssessmentOutcome::Incorrect
        ) && !observation.independently_demonstrated
        {
            return Err(EvidenceValidationError::UnsubstantiatedOutcome);
        }
        if observation.outcome == AssessmentOutcome::Correct
            && observation.response_was_cued
            && observation.modality == EvidenceModality::HandDrawn
        {
            return Err(EvidenceValidationError::AssistedResponse);
        }
        if observation.outcome == AssessmentOutcome::Correct
            && observation
                .reading_key
                .as_ref()
                .is_some_and(|r| !authorization.required_readings.contains(r))
        {
            return Err(EvidenceValidationError::InvalidReading);
        }
        if observation.outcome == AssessmentOutcome::Correct
            && !authorization.required_readings.is_empty()
            && observation.reading_key.is_none()
        {
            return Err(EvidenceValidationError::InvalidReading);
        }
        if observation.reading_key.is_some() && authorization.required_readings.is_empty() {
            return Err(EvidenceValidationError::InvalidReading);
        }
        if matches!(
            observation.outcome,
            AssessmentOutcome::Uncertain
                | AssessmentOutcome::NotAssessed
                | AssessmentOutcome::Invalid
                | AssessmentOutcome::EvaluationFailure
        ) {
            continue;
        }
        validated.push(AssessmentEvent {
            attempt_id: manifest.attempt_id.clone(),
            policy_epoch: manifest.policy_epoch,
            entity: observation.target.entity.clone(),
            dimension: observation.target.dimension,
            policy_day: manifest.policy_day,
            context_key: Some(manifest.context_family.clone()),
            reading_key: observation.reading_key.clone(),
            outcome: observation.outcome,
        });
    }
    Ok(validated)
}

fn modality_supports_target(target: &EvidenceTarget, modality: EvidenceModality) -> bool {
    use super::KnowledgeCategory;
    use EvidenceModality::*;
    use KnowledgeDimension::*;
    matches!(
        (target.entity.category, target.dimension, modality),
        (KnowledgeCategory::Pinyin, Reading, WrittenChinese)
            | (KnowledgeCategory::Pinyin, Listening, AudioChinese)
            | (KnowledgeCategory::Pinyin, Speaking, SpokenChinese)
            | (KnowledgeCategory::Hanzi, Recognition, WrittenChinese)
            | (KnowledgeCategory::Hanzi, Pronunciation, SpokenChinese)
            | (KnowledgeCategory::Hanzi, Handwriting, HandDrawn)
            | (
                KnowledgeCategory::Vocabulary,
                Recognition,
                WrittenChinese | WordBank
            )
            | (
                KnowledgeCategory::Vocabulary,
                Listening,
                AudioChinese | WordBank
            )
            | (KnowledgeCategory::Vocabulary, Speaking, SpokenChinese)
            | (
                KnowledgeCategory::Vocabulary,
                Writing,
                FreeTextChinese | HandDrawn
            )
            | (
                KnowledgeCategory::Grammar,
                GrammarPattern,
                GrammarConstruction
            )
    )
}
