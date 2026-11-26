use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    #[error("Cannot decode database response: {0}")]
    Decode(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PracticeOutcomeSummary {
    pub sessions: u64,
    pub scored_steps: u64,
    pub completed_steps: u64,
    pub skipped_steps: u64,
    pub earned_points: u64,
    pub possible_points: u64,
    pub utc_completion_dates: u64,
    completion_date_keys: BTreeSet<u64>,
    included_session_ids: BTreeSet<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceDocument {
    schema_version: u8,
    session_id: String,
    completed_at_ms: u64,
    steps: Vec<EvidenceStep>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceStep {
    step_id: String,
    exercise_id: String,
    outcome: EvidenceOutcome,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceOutcome {
    Scored { earned: u16, possible: u16 },
    Completed,
    Skipped,
}

fn valid_session_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

fn valid_exercise_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
}

impl PracticeOutcomeSummary {
    pub fn from_evidence_rows<'a>(
        rows: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, EvidenceError> {
        let mut summary = Self::default();
        for (session_id, evidence) in rows {
            summary.include(session_id, evidence)?;
        }
        Ok(summary)
    }

    pub(crate) fn from_transport_counts(
        sessions: u64,
        scored_steps: u64,
        completed_steps: u64,
        skipped_steps: u64,
        earned_points: u64,
        possible_points: u64,
        utc_completion_dates: u64,
    ) -> Self {
        Self {
            sessions,
            scored_steps,
            completed_steps,
            skipped_steps,
            earned_points,
            possible_points,
            utc_completion_dates,
            completion_date_keys: BTreeSet::new(),
            included_session_ids: BTreeSet::new(),
        }
    }

    pub(crate) fn include(&mut self, database_id: &str, json: &str) -> Result<(), EvidenceError> {
        if json.len() > 512 * 1024 {
            return Err(EvidenceError::Decode(
                "Learning-history evidence exceeds size limit".into(),
            ));
        }
        let document: EvidenceDocument = serde_json::from_str(json)
            .map_err(|_| EvidenceError::Decode("Invalid saved learning-history evidence".into()))?;
        if document.schema_version != 2
            || document.completed_at_ms == 0
            || !valid_session_id(database_id)
            || self.included_session_ids.contains(database_id)
            || document.session_id != database_id
            || document.steps.is_empty()
            || document.steps.len() > 1_000
        {
            return Err(EvidenceError::Decode(
                "Unsupported or inconsistent learning-history evidence".into(),
            ));
        }
        let mut scored = 0u64;
        let mut completed = 0u64;
        let mut skipped = 0u64;
        let mut earned = 0u64;
        let mut possible = 0u64;
        for (index, step) in document.steps.iter().enumerate() {
            if step.step_id != format!("{database_id}:{index}")
                || !valid_exercise_id(&step.exercise_id)
            {
                return Err(EvidenceError::Decode(
                    "Invalid saved Practice step identity".into(),
                ));
            }
            match step.outcome {
                EvidenceOutcome::Scored {
                    earned: points,
                    possible: max,
                } if step.exercise_id == "sentence-translation" && max == 1 && points <= 1 => {
                    scored += 1;
                    earned += u64::from(points);
                    possible += u64::from(max);
                }
                EvidenceOutcome::Completed => completed += 1,
                EvidenceOutcome::Skipped => skipped += 1,
                _ => {
                    return Err(EvidenceError::Decode(
                        "Invalid saved Practice outcome".into(),
                    ));
                }
            }
        }

        let sessions = self.sessions.checked_add(1).ok_or_else(|| {
            EvidenceError::Decode("Learning-history session counter overflow".into())
        })?;
        let scored_steps = self
            .scored_steps
            .checked_add(scored)
            .ok_or_else(|| EvidenceError::Decode("Scored step counter overflow".into()))?;
        let completed_steps = self
            .completed_steps
            .checked_add(completed)
            .ok_or_else(|| EvidenceError::Decode("Completed step counter overflow".into()))?;
        let skipped_steps = self
            .skipped_steps
            .checked_add(skipped)
            .ok_or_else(|| EvidenceError::Decode("Skipped step counter overflow".into()))?;
        let earned_points = self
            .earned_points
            .checked_add(earned)
            .ok_or_else(|| EvidenceError::Decode("Earned points overflow".into()))?;
        let possible_points = self
            .possible_points
            .checked_add(possible)
            .ok_or_else(|| EvidenceError::Decode("Possible points overflow".into()))?;
        let completion_date_key = document.completed_at_ms / 86_400_000;
        let mut completion_date_keys = self.completion_date_keys.clone();
        completion_date_keys.insert(completion_date_key);
        let utc_completion_dates = u64::try_from(completion_date_keys.len())
            .map_err(|_| EvidenceError::Decode("Too many completion dates".into()))?;
        self.sessions = sessions;
        self.scored_steps = scored_steps;
        self.completed_steps = completed_steps;
        self.skipped_steps = skipped_steps;
        self.earned_points = earned_points;
        self.possible_points = possible_points;
        self.utc_completion_dates = utc_completion_dates;
        self.completion_date_keys = completion_date_keys;
        self.included_session_ids.insert(database_id.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "123e4567-e89b-42d3-a456-426614174001";

    #[test]
    fn transported_summary_preserves_all_public_counts() {
        let result = PracticeOutcomeSummary::from_transport_counts(4, 3, 2, 1, 5, 8, 2);
        assert_eq!(result.sessions, 4);
        assert_eq!(result.scored_steps, 3);
        assert_eq!(result.completed_steps, 2);
        assert_eq!(result.skipped_steps, 1);
        assert_eq!(result.earned_points, 5);
        assert_eq!(result.possible_points, 8);
        assert_eq!(result.utc_completion_dates, 2);
    }

    #[test]
    fn counts_only_observed_outcomes() {
        let json = format!(
            r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:0","exercise_id":"sentence-translation","outcome":{{"scored":{{"earned":1,"possible":1}}}}}},{{"step_id":"{ID}:1","exercise_id":"matching","outcome":"completed"}},{{"step_id":"{ID}:2","exercise_id":"matching","outcome":"skipped"}}]}}"#
        );
        let mut summary = PracticeOutcomeSummary::default();
        summary.include(ID, &json).unwrap();
        assert_eq!(summary.sessions, 1);
        assert_eq!(summary.scored_steps, 1);
        assert_eq!(summary.completed_steps, 1);
        assert_eq!(summary.skipped_steps, 1);
        assert_eq!((summary.earned_points, summary.possible_points), (1, 1));
        let original = summary.clone();
        assert!(summary.include(ID, &json).is_err());
        assert_eq!(summary, original);
    }

    #[test]
    fn invalid_identifiers_cannot_enter_summary() {
        let bad_ids = [
            "invalid",
            "123e4567-e89b-42d3-a456-42661417400!",
            "123e4567-e89b-42d3-a456-426614174001 ",
        ];
        for id in bad_ids {
            let json = format!(
                r#"{{"schema_version":2,"session_id":"{id}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{id}:0","exercise_id":"type-pinyin","outcome":"completed"}}]}}"#
            );
            let mut summary = PracticeOutcomeSummary::default();
            assert!(summary.include(id, &json).is_err());
            assert_eq!(summary, PracticeOutcomeSummary::default());
        }
        let invalid_exercise = format!(
            r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:0","exercise_id":"invalid:exercise","outcome":"completed"}}]}}"#
        );
        assert!(
            PracticeOutcomeSummary::default()
                .include(ID, &invalid_exercise)
                .is_err()
        );
    }

    #[test]
    fn unscored_exercises_cannot_claim_scored_evidence() {
        let mut summary = PracticeOutcomeSummary::default();
        for exercise in [
            "matching",
            "multiple-choice",
            "type-pinyin",
            "speaking",
            "sentence-transformation",
        ] {
            let json = format!(
                r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:0","exercise_id":"{exercise}","outcome":{{"scored":{{"earned":1,"possible":1}}}}}}]}}"#
            );
            assert!(
                summary.include(ID, &json).is_err(),
                "unexpected score for {exercise}"
            );
            assert_eq!(summary, PracticeOutcomeSummary::default());
        }
    }

    #[test]
    fn saved_translation_scores_cannot_inflate_single_point_attempts() {
        for (earned, possible) in [(0, 0), (1, 2), (2, 2), (2, 1)] {
            let json = format!(
                r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:0","exercise_id":"sentence-translation","outcome":{{"scored":{{"earned":{earned},"possible":{possible}}}}}}}]}}"#
            );
            let mut summary = PracticeOutcomeSummary::default();
            assert!(summary.include(ID, &json).is_err());
            assert_eq!(summary, PracticeOutcomeSummary::default());
        }
    }

    #[test]
    fn requires_v2_timestamp_and_rejects_legacy_evidence() {
        let steps = format!(
            r#"[{{"step_id":"{ID}:0","exercise_id":"type-pinyin","outcome":"completed"}}]"#
        );
        let good = format!(
            r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":{steps}}}"#
        );
        let mut summary = PracticeOutcomeSummary::default();
        summary.include(ID, &good).unwrap();
        assert_eq!(summary.sessions, 1);
        for (version, timestamp) in [
            (1, ",\"completed_at_ms\":1700000000000"),
            (2, ""),
            (2, ",\"completed_at_ms\":0"),
        ] {
            let bad = format!(
                r#"{{"schema_version":{version},"session_id":"{ID}"{timestamp},"steps":{steps}}}"#
            );
            assert!(PracticeOutcomeSummary::default().include(ID, &bad).is_err());
        }
    }

    #[test]
    fn utc_completion_dates_are_distinct_and_not_learner_active_days() {
        let mut summary = PracticeOutcomeSummary::default();
        for (index, completed_at_ms) in [86_400_000, 86_400_001, 172_800_000].iter().enumerate() {
            let session_id = format!("123e4567-e89b-42d3-a456-42661417400{}", index + 1);
            let evidence = format!(
                r#"{{"schema_version":2,"session_id":"{session_id}","completed_at_ms":{completed_at_ms},"steps":[{{"step_id":"{session_id}:0","exercise_id":"matching","outcome":"completed"}}]}}"#
            );
            summary.include(&session_id, &evidence).unwrap();
        }
        assert_eq!(summary.sessions, 3);
        assert_eq!(summary.utc_completion_dates, 2);
        assert_eq!(summary.scored_steps, 0);
    }

    #[test]
    fn counter_overflow_does_not_partially_mutate_summary() {
        let json = format!(
            r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:0","exercise_id":"type-pinyin","outcome":"completed"}}]}}"#
        );
        let mut summary = PracticeOutcomeSummary {
            completed_steps: u64::MAX,
            ..Default::default()
        };
        let original = summary.clone();
        assert!(summary.include(ID, &json).is_err());
        assert_eq!(summary, original);
    }

    #[test]
    fn rejects_untrusted_or_mismatched_evidence() {
        let examples = [
            format!(
                r#"{{"schema_version":2,"session_id":"{ID}","steps":[{{"step_id":"{ID}:0","exercise_id":"matching","outcome":"completed"}}]}}"#
            ),
            format!(
                r#"{{"schema_version":1,"session_id":"other","steps":[{{"step_id":"{ID}:0","exercise_id":"matching","outcome":"completed"}}]}}"#
            ),
            format!(
                r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:1","exercise_id":"matching","outcome":"completed"}}]}}"#
            ),
            format!(
                r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[{{"step_id":"{ID}:0","exercise_id":"sentence-translation","outcome":{{"scored":{{"earned":8,"possible":5}}}}}}]}}"#
            ),
            format!(
                r#"{{"schema_version":2,"session_id":"{ID}","completed_at_ms":1700000000000,"steps":[]}}"#
            ),
        ];
        for json in examples {
            let mut summary = PracticeOutcomeSummary::default();
            assert!(summary.include(ID, &json).is_err());
            assert_eq!(summary, PracticeOutcomeSummary::default());
        }
    }
}
