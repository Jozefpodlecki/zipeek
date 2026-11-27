mod card;
mod evidence;
mod page;
mod writing;
use evidence::status_for;
pub(super) use page::SkillsExplorer;

#[cfg(test)]
mod tests {
    use super::super::{fixture::EvidenceStatus, state::SkillGap};
    use super::*;
    #[test]
    fn each_skill_has_mutually_exclusive_counts() {
        let records = super::super::data::demo_words_for_tests();
        for skill in SkillGap::ALL {
            let observed = records
                .iter()
                .filter(|item| status_for(item, skill) == EvidenceStatus::Observed)
                .count();
            let review = records
                .iter()
                .filter(|item| status_for(item, skill) == EvidenceStatus::NeedsReview)
                .count();
            let missing = records
                .iter()
                .filter(|item| status_for(item, skill) == EvidenceStatus::NeedsEvidence)
                .count();
            assert_eq!(observed + review + missing, records.len());
        }
    }
}
