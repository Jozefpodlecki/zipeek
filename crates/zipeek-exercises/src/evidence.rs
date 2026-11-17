use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillDefinition {
    pub id: &'static str,
    pub description: &'static str,
}

pub const RECOGNITION: SkillDefinition = SkillDefinition {
    id: "recognition",
    description: "Identify a written Mandarin word and its meaning.",
};
pub const LISTENING: SkillDefinition = SkillDefinition {
    id: "listening",
    description: "Identify a spoken Mandarin word without a written answer cue.",
};
pub const PRONUNCIATION: SkillDefinition = SkillDefinition {
    id: "pronunciation",
    description: "Produce the intended Mandarin reading in a specified spoken context.",
};
pub const RECALL: SkillDefinition = SkillDefinition {
    id: "recall",
    description: "Retrieve the Mandarin word from a meaning without recognising a supplied answer.",
};
pub const WRITING: SkillDefinition = SkillDefinition {
    id: "writing",
    description: "Produce the written Mandarin characters without a supplied word bank.",
};
pub const USAGE: SkillDefinition = SkillDefinition {
    id: "usage",
    description: "Apply the word appropriately in a sentence or translation.",
};

pub const SKILLS: &[SkillDefinition] = &[
    RECOGNITION,
    LISTENING,
    PRONUNCIATION,
    RECALL,
    WRITING,
    USAGE,
];

pub fn skill(id: &str) -> Option<&'static SkillDefinition> {
    SKILLS.iter().find(|candidate| candidate.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeEvidence {
    pub id: String,
    pub word_id: i32,
    pub skill_id: &'static str,
    pub exercise_definition_id: &'static str,
    pub exercise_instance_id: String,
    pub day: String,
    pub earned: u32,
    pub possible: u32,
    pub assisted: bool,
    pub target_reading: Option<String>,
    pub spoken_context: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidIdentity,
    UnknownSkill,
    UnknownExercise,
    PracticeOnlyExercise,
    InvalidDay,
    InvalidScore,
    DuplicateEvidence,
    MissingPronunciationContext,
}

fn valid_day(day: &str) -> bool {
    let bytes = day.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, byte)| i == 4 || i == 7 || byte.is_ascii_digit())
    {
        return false;
    }
    let Ok(year) = day[..4].parse::<u16>() else {
        return false;
    };
    let Ok(month) = day[5..7].parse::<u8>() else {
        return false;
    };
    let Ok(day_of_month) = day[8..10].parse::<u8>() else {
        return false;
    };
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day_of_month)
}

impl PracticeEvidence {
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.id.trim().is_empty()
            || self.word_id <= 0
            || self.exercise_instance_id.trim().is_empty()
        {
            return Err(EvidenceError::InvalidIdentity);
        }
        if skill(self.skill_id).is_none() {
            return Err(EvidenceError::UnknownSkill);
        }
        let Some(definition) = crate::definition::definition(self.exercise_definition_id) else {
            return Err(EvidenceError::UnknownExercise);
        };
        if !definition.accepts_scored_results() {
            return Err(EvidenceError::PracticeOnlyExercise);
        }
        if !valid_day(&self.day) {
            return Err(EvidenceError::InvalidDay);
        }
        if self.skill_id == PRONUNCIATION.id
            && (self
                .target_reading
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
                || self
                    .spoken_context
                    .as_deref()
                    .map_or(true, |value| value.trim().is_empty()))
        {
            return Err(EvidenceError::MissingPronunciationContext);
        }
        if self.possible == 0 || self.earned > self.possible {
            return Err(EvidenceError::InvalidScore);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvidenceLedger {
    entries: Vec<PracticeEvidence>,
}

impl EvidenceLedger {
    pub fn record(&mut self, evidence: PracticeEvidence) -> Result<(), EvidenceError> {
        evidence.validate()?;
        if self
            .entries
            .iter()
            .any(|existing| existing.id == evidence.id)
        {
            return Err(EvidenceError::DuplicateEvidence);
        }
        self.entries.push(evidence);
        Ok(())
    }

    pub fn entries(&self) -> &[PracticeEvidence] {
        &self.entries
    }

    pub fn observed_days(&self, word_id: i32, skill_id: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.word_id == word_id && entry.skill_id == skill_id && !entry.assisted
            })
            .map(|entry| entry.day.as_str())
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn daily_results(&self, word_id: i32, skill_id: &str) -> BTreeMap<&str, (u64, u64)> {
        let mut by_day = BTreeMap::new();
        for entry in self.entries.iter().filter(|entry| {
            entry.word_id == word_id && entry.skill_id == skill_id && !entry.assisted
        }) {
            let totals = by_day.entry(entry.day.as_str()).or_insert((0u64, 0u64));
            totals.0 += u64::from(entry.earned);
            totals.1 += u64::from(entry.possible);
        }
        by_day
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(id: &str, day: &str) -> PracticeEvidence {
        PracticeEvidence {
            id: id.into(),
            word_id: 42,
            skill_id: RECOGNITION.id,
            exercise_definition_id: crate::definition::MULTIPLE_CHOICE.id,
            exercise_instance_id: "q-1".into(),
            day: day.into(),
            earned: 1,
            possible: 1,
            assisted: false,
            target_reading: None,
            spoken_context: None,
        }
    }

    #[test]
    fn every_unscored_exercise_is_refused_by_legacy_mastery_evidence() {
        for exercise_id in [
            crate::definition::SPEAKING.id,
            crate::definition::SPOT_MISTAKE.id,
            crate::definition::WRITE_HANZI.id,
        ] {
            let mut entry = sample(exercise_id, "2026-11-08");
            entry.exercise_definition_id = exercise_id;
            assert_eq!(entry.validate(), Err(EvidenceError::PracticeOnlyExercise));
            assert_eq!(
                EvidenceLedger::default().record(entry),
                Err(EvidenceError::PracticeOnlyExercise)
            );
        }
    }

    #[test]
    fn write_hanzi_cannot_enter_scored_legacy_ledger() {
        let mut evidence = sample("recall-1", "2026-11-08");
        evidence.exercise_definition_id = crate::definition::WRITE_HANZI.id;
        assert_eq!(
            evidence.validate(),
            Err(EvidenceError::PracticeOnlyExercise)
        );
        let mut ledger = EvidenceLedger::default();
        assert_eq!(
            ledger.record(evidence),
            Err(EvidenceError::PracticeOnlyExercise)
        );
        assert!(ledger.entries().is_empty());
    }

    #[test]
    fn evidence_records_days_without_inflating_distinct_day_count() {
        let mut ledger = EvidenceLedger::default();
        ledger.record(sample("a", "2026-11-04")).unwrap();
        ledger.record(sample("b", "2026-11-04")).unwrap();
        ledger.record(sample("c", "2026-11-05")).unwrap();
        assert_eq!(ledger.observed_days(42, "recognition"), 2);
        assert_eq!(
            ledger.daily_results(42, "recognition")["2026-11-04"],
            (2, 2)
        );
        assert_eq!(ledger.observed_days(42, "listening"), 0);
    }

    #[test]
    fn duplicate_evidence_is_not_recorded_twice() {
        let mut ledger = EvidenceLedger::default();
        ledger.record(sample("a", "2026-11-04")).unwrap();
        assert_eq!(
            ledger.record(sample("a", "2026-11-05")),
            Err(EvidenceError::DuplicateEvidence)
        );
        assert_eq!(ledger.entries().len(), 1);
    }

    #[test]
    fn invalid_identity_day_score_and_skill_are_rejected() {
        let mut record = sample("a", "2026-11-04");
        record.word_id = 0;
        assert_eq!(record.validate(), Err(EvidenceError::InvalidIdentity));
        record.word_id = 42;
        record.day = "04-11-2026".into();
        assert_eq!(record.validate(), Err(EvidenceError::InvalidDay));
        record.day = "2026-02-30".into();
        assert_eq!(record.validate(), Err(EvidenceError::InvalidDay));
        record.day = "2026-11-04".into();
        record.earned = 2;
        assert_eq!(record.validate(), Err(EvidenceError::InvalidScore));
        record.earned = 1;
        record.skill_id = "unsupported";
        assert_eq!(record.validate(), Err(EvidenceError::UnknownSkill));
    }

    #[test]
    fn pronunciation_requires_a_specific_reading_and_spoken_context() {
        let mut record = sample("pronunciation", "2026-11-04");
        record.skill_id = PRONUNCIATION.id;
        assert_eq!(
            record.validate(),
            Err(EvidenceError::MissingPronunciationContext)
        );
        record.target_reading = Some("xing2".into());
        assert_eq!(
            record.validate(),
            Err(EvidenceError::MissingPronunciationContext)
        );
        record.spoken_context = Some("银行".into());
        assert_eq!(record.validate(), Ok(()));
    }

    #[test]
    fn skill_definitions_have_unique_identifiers() {
        let mut ids = HashSet::new();
        for definition in SKILLS {
            assert!(ids.insert(definition.id));
            assert!(!definition.description.is_empty());
        }
    }
    #[test]
    fn assisted_practice_remains_auditable_but_does_not_inflate_mastery_days() {
        let mut ledger = EvidenceLedger::default();
        let mut assisted = sample("hinted", "2026-11-04");
        assisted.assisted = true;
        ledger.record(assisted).unwrap();
        assert_eq!(ledger.entries().len(), 1);
        assert_eq!(ledger.observed_days(42, RECOGNITION.id), 0);
        assert!(ledger.daily_results(42, RECOGNITION.id).is_empty());
        ledger.record(sample("independent", "2026-11-04")).unwrap();
        assert_eq!(ledger.observed_days(42, RECOGNITION.id), 1);
    }
}
