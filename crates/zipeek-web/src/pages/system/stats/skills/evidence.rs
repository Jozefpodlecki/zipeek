use super::super::{data::WordRecord, fixture::EvidenceStatus, state::SkillGap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EvidenceCounts {
    pub recorded: usize,
    pub review: usize,
    pub total: usize,
}

pub(super) fn status_for(record: &WordRecord, skill: SkillGap) -> EvidenceStatus {
    match skill {
        SkillGap::Recognition => record.recognition,
        SkillGap::Listening => record.listening,
        SkillGap::Speaking => record.speaking,
        SkillGap::Writing => record.writing,
        SkillGap::Sentence => record.sentence,
    }
}

pub(super) fn counts_for(records: &[WordRecord], skill: SkillGap) -> EvidenceCounts {
    EvidenceCounts {
        recorded: records
            .iter()
            .filter(|record| status_for(record, skill) == EvidenceStatus::Observed)
            .count(),
        review: records
            .iter()
            .filter(|record| status_for(record, skill) == EvidenceStatus::NeedsReview)
            .count(),
        total: records.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_skill_partitions_evidence() {
        let records = super::super::super::data::demo_words_for_tests();
        for skill in SkillGap::ALL {
            let counts = counts_for(&records, skill);
            let missing = records
                .iter()
                .filter(|record| status_for(record, skill) == EvidenceStatus::NeedsEvidence)
                .count();
            assert_eq!(counts.recorded + counts.review + missing, counts.total);
        }
    }
}
