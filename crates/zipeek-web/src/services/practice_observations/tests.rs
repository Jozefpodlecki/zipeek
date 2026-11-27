use super::*;
use crate::services::practice_session_store::{
    AssessmentCapability, AssessmentEvidence, ExerciseAssessment, ExerciseOutcome, ExerciseVerdict,
};

fn observation() -> PracticeObservation {
    let mut assessment = ExerciseAssessment::binary(true);
    assessment.item_evidence.push(AssessmentEvidence {
        capability: AssessmentCapability::CharacterSequenceConstruction,
        target: "朋友".into(),
        verdict: ExerciseVerdict::Correct,
        attempts: 1,
        mistakes: 0,
        hints: 0,
        assisted: false,
    });
    PracticeObservation {
        session_id: "123e4567-e89b-72d3-a456-426614174001".into(),
        step_id: "123e4567-e89b-72d3-a456-426614174001:3".into(),
        exercise_id: "character-builder".into(),
        finished_at_ms: 1764250000000,
        activated_at_ms: Some(1764249999000),
        outcome: ExerciseOutcome::Assessed(assessment),
    }
}

#[test]
fn observational_construction_round_trips_without_mastery_dimension() {
    let original = observation();
    let stored = original.encoded().unwrap();
    assert_eq!(PracticeObservation::decoded(&stored).unwrap(), original);
    assert!(original.is_construction_only());
    assert!(original.is_observational_only());
    assert!(!stored.contains("knowledge_dimension"));
}

#[test]
fn refuses_inverted_time_and_unrelated_step_identity() {
    let mut original = observation();
    original.activated_at_ms = Some(original.finished_at_ms + 1);
    assert!(original.validate().is_err());
    original = observation();
    original.step_id = "123e4567-e89b-72d3-a456-426614174001:bad".into();
    assert!(original.validate().is_err());
}

#[test]
fn rejects_invalid_assessment_before_inbox_insert() {
    let mut original = observation();
    if let ExerciseOutcome::Assessed(assessment) = &mut original.outcome {
        assessment.item_evidence[0].mistakes = 1;
    }
    assert!(original.encoded().is_err());
}

#[test]
fn generic_assessed_result_has_no_authorized_construction_target() {
    let mut original = observation();
    original.exercise_id = "matching".into();
    original.outcome = ExerciseOutcome::Assessed(ExerciseAssessment::binary(true));
    assert!(original.validate().is_ok());
    assert!(!original.is_construction_only());
    assert!(!original.is_observational_only());
}
