use super::super::{data::WordRecord, fixture::EvidenceStatus, state::SkillGap};

pub(super) struct Recommendation {
    pub title: &'static str,
    pub count: usize,
    pub explanation: &'static str,
    pub skill: SkillGap,
}

pub(super) struct InsightSummary {
    pub priorities: Vec<Recommendation>,
    pub review_count: usize,
}

pub(super) fn summarize(records: &[WordRecord]) -> InsightSummary {
    let without_recognition = records
        .iter()
        .filter(|record| record.recognition == EvidenceStatus::NeedsEvidence)
        .count();
    let without_sentence = records
        .iter()
        .filter(|record| record.sentence == EvidenceStatus::NeedsEvidence)
        .count();
    let without_listening = records
        .iter()
        .filter(|record| record.listening == EvidenceStatus::NeedsEvidence)
        .count();
    let without_speaking = records
        .iter()
        .filter(|record| record.speaking == EvidenceStatus::NeedsEvidence)
        .count();
    let review_count = records
        .iter()
        .filter(|record| {
            [
                record.recognition,
                record.listening,
                record.speaking,
                record.writing,
                record.sentence,
            ]
            .contains(&EvidenceStatus::NeedsReview)
        })
        .count();
    let mut priorities = vec![
        Recommendation {
            title: "Recognition",
            count: without_recognition,
            explanation: "No activity yet",
            skill: SkillGap::Recognition,
        },
        Recommendation {
            title: "Listening",
            count: without_listening,
            explanation: "No activity yet",
            skill: SkillGap::Listening,
        },
        Recommendation {
            title: "Speaking",
            count: without_speaking,
            explanation: "No activity yet",
            skill: SkillGap::Speaking,
        },
        Recommendation {
            title: "Sentence use",
            count: without_sentence,
            explanation: "No activity yet",
            skill: SkillGap::Sentence,
        },
    ];
    priorities.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.title.cmp(b.title)));
    InsightSummary {
        priorities,
        review_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendations_are_ordered_by_missing_evidence() {
        let records = super::super::super::data::demo_words_for_tests();
        let result = summarize(&records);
        assert_eq!(result.priorities.len(), 4);
        assert!(
            result
                .priorities
                .windows(2)
                .all(|pair| pair[0].count >= pair[1].count)
        );
        assert!(result.review_count <= records.len());
    }
}
