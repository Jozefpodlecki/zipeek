pub(super) fn demo_grammar_catalog() -> Vec<super::data::GrammarRecord> {
    let source =
        include_str!("../../../../../../docs/learning-practice/GRAMMAR_TEACHING_INDEX.tsv");
    source
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let teaching_id = fields.next()?;
            let region = fields.next()?;
            let ordinal = fields.next()?;
            let pattern = fields.next()?;
            let example = fields.next()?;
            let kind = fields.next()?;
            let _parent = fields.next()?;
            let review_status = fields.next()?;
            if teaching_id.is_empty() || pattern.is_empty() {
                return None;
            }
            Some(super::data::GrammarRecord {
                id: teaching_id.to_owned(),
                pattern: pattern.to_owned(),
                description: if kind == "unclassified" {
                    "Grammar pattern".to_owned()
                } else {
                    kind.to_owned()
                },
                example: example.to_owned(),
                attempts: ((region.parse::<u32>().unwrap_or(0) * 7
                    + ordinal.parse::<u32>().unwrap_or(0) * 3)
                    % 14),
                course: format!("Region {region}"),
                canonical_id: None,
                course_id: None,
                assessed_attempts: None,
                teaching_review_status: review_status.to_owned(),
            })
        })
        .collect()
}

pub(super) fn demo_grammar() -> GrammarRecords {
    thread_local! { static GRAMMAR: GrammarRecords = GrammarRecords::new(demo_grammar_catalog()); }
    GRAMMAR.with(Clone::clone)
}

use super::{
    data::{GrammarRecords, SessionRecord, SessionRecords, WordRecord, WordRecords},
    fixture::{EvidenceStatus, SAMPLE_SESSIONS},
};

const PREVIEW_WORDS: usize = 240;
const PREVIEW_SESSIONS: usize = 220;
const PREVIEW_SEED: u64 = 0x5A17_2026;

fn sample_status(value: u64, threshold: u64) -> EvidenceStatus {
    match value % 100 {
        score if score < threshold => EvidenceStatus::Observed,
        score if score < threshold + 9 => EvidenceStatus::NeedsReview,
        _ => EvidenceStatus::NeedsEvidence,
    }
}

fn mix_seed(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

pub(super) fn generate_vocabulary_with_seed(count: usize, seed: u64) -> WordRecords {
    let source = include_str!("vocabulary_seed.tsv")
        .lines()
        .collect::<Vec<_>>();
    let maximum = count.min(source.len());
    let step = 7_919_usize;
    let offset = seed as usize % source.len();
    WordRecords::new(
        (0..maximum)
            .filter_map(|index| {
                let source_index = (offset + index * step) % source.len();
                let mut fields = source[source_index].splitn(3, '\t');
                let hanzi = fields.next()?.to_owned();
                let pinyin = fields.next()?.to_owned();
                let meaning = fields.next()?.to_owned();
                let value = mix_seed(seed ^ source_index as u64);
                let sessions = 1 + ((value >> 8) % 8) as u32;
                let encounters = sessions + (value % 28) as u32;
                let days = 1 + ((value >> 16) % u64::from(sessions)) as u32;
                Some(WordRecord {
                    id: format!("preview-word-{source_index:05}"),
                    hanzi,
                    pinyin,
                    meaning,
                    encounters,
                    sessions,
                    days,
                    recognition: sample_status(value >> 3, 72),
                    listening: sample_status(value >> 11, 49),
                    speaking: sample_status(value >> 19, 27),
                    writing: sample_status(value >> 27, 34),
                    sentence: sample_status(value >> 35, 31),
                })
            })
            .collect(),
    )
}

pub(super) fn demo_words() -> WordRecords {
    thread_local! {
        static WORDS: WordRecords = {
            let mut words = generate_vocabulary_with_seed(PREVIEW_WORDS, PREVIEW_SEED).as_slice().to_vec();
            let sessions = generate_sessions(PREVIEW_SESSIONS);
            let mut dates = vec![std::collections::BTreeSet::<String>::new(); words.len()];
            let identities = words.iter().enumerate().map(|(index, word)| (word.hanzi.clone(), index))
                .collect::<std::collections::HashMap<_, _>>();
            for word in &mut words {
                word.encounters = 0;
                word.sessions = 0;
                word.days = 0;
            }
            for session in sessions.iter() {
                if let Some(&index) = identities.get(&session.focus) {
                    words[index].encounters += session.observed;
                    words[index].sessions += 1;
                    dates[index].insert(session.date.clone());
                }
            }
            for (word, dates) in words.iter_mut().zip(dates) {
                word.days = dates.len() as u32;
            }
            WordRecords::new(words)
        };
    }
    WORDS.with(Clone::clone)
}

pub(super) fn demo_sessions() -> SessionRecords {
    thread_local! { static SESSIONS: SessionRecords = generate_sessions(PREVIEW_SESSIONS); }
    SESSIONS.with(Clone::clone)
}

pub(super) fn generate_sessions(recent_count: usize) -> SessionRecords {
    let vocabulary = generate_vocabulary_with_seed(PREVIEW_WORDS, PREVIEW_SEED);
    let mut records: Vec<SessionRecord> = SAMPLE_SESSIONS
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let activity = row.activity();
            let focus_index = (PREVIEW_SESSIONS + index) % vocabulary.len();
            let hour = 8 + (index * 7 % 14);
            let minute = index * 13 % 60;
            SessionRecord {
                id: row.id.to_owned(),
                date: row.date.to_owned(),
                started_at: format!("{}T{hour:02}:{minute:02}:00Z", row.date),
                duration_minutes: 3 + (index * 11 % 18) as u32,
                activity_id: row.activity_id.to_owned(),
                activity_title: activity.title.to_owned(),
                category: activity.category.to_owned(),
                observed: u32::from(row.observed),
                correct: row.correct.map(u32::from),
                focus: vocabulary[focus_index].hanzi.clone(),
            }
        })
        .collect();
    const RECENT_DATES: [&str; 14] = [
        "2026-11-05",
        "2026-11-06",
        "2026-11-07",
        "2026-11-08",
        "2026-11-09",
        "2026-11-10",
        "2026-11-11",
        "2026-11-12",
        "2026-11-13",
        "2026-11-14",
        "2026-11-15",
        "2026-11-16",
        "2026-11-17",
        "2026-11-18",
    ];
    for index in 0..recent_count {
        let source = SAMPLE_SESSIONS[(index * 7 + index / 24) % SAMPLE_SESSIONS.len()];
        let activity = source.activity();
        let focus_index = index % vocabulary.len();
        let date = RECENT_DATES[(index / 6) % RECENT_DATES.len()];
        let hour = 7 + (index * 5 % 15);
        let minute = index * 17 % 60;
        records.push(SessionRecord {
            id: format!("recent-session-{index:03}"),
            date: date.to_owned(),
            started_at: format!("{date}T{hour:02}:{minute:02}:00Z"),
            duration_minutes: 4 + (index * 7 % 20) as u32,
            activity_id: source.activity_id.to_owned(),
            activity_title: activity.title.to_owned(),
            category: activity.category.to_owned(),
            observed: u32::from(source.observed),
            correct: source.correct.map(u32::from),
            focus: vocabulary[focus_index].hanzi.clone(),
        });
    }
    SessionRecords::new(records)
}

pub(super) fn generate_vocabulary(count: usize) -> WordRecords {
    generate_vocabulary_with_seed(count, PREVIEW_SEED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn immutable_demo_snapshots_are_shared_between_queries() {
        assert!(demo_words().ptr_eq(&demo_words()));
        assert!(demo_sessions().ptr_eq(&demo_sessions()));
        assert!(demo_grammar().ptr_eq(&demo_grammar()));
        let grammar = demo_grammar();
        let ids = grammar
            .iter()
            .map(|row| row.id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(grammar.len(), 550);
        assert_eq!(ids.len(), 550);
        assert!(
            grammar
                .iter()
                .all(|row| row.canonical_id.is_none() && row.assessed_attempts.is_none())
        );
        assert!(grammar.iter().any(|row| row.attempts > 0));
    }

    #[test]
    fn generated_large_vocabulary_has_stable_unique_identities() {
        for size in [5_000, 10_000] {
            let records = generate_vocabulary(size);
            assert_eq!(records.len(), size);
            let ids: HashSet<_> = records.iter().map(|record| &record.id).collect();
            assert_eq!(ids.len(), size);
            let hanzi = records
                .iter()
                .map(|record| record.hanzi.as_str())
                .collect::<HashSet<_>>();
            assert_eq!(hanzi.len(), size);
            assert_ne!(records[0].hanzi, records[30].hanzi);
            let query = super::super::query::WordQuery {
                search: String::new(),
                sort: super::super::state::WordSort::Alphabetical,
                gap: None,
                page: super::super::query::PageRequest::new(50, 100).unwrap(),
            };
            let page = super::super::query::query_words(&records, &query).unwrap();
            assert_eq!(page.total, size);
            assert_eq!(page.records.len(), 100);
            assert!(page.validate(query.page).is_ok());
        }
    }

    #[test]
    fn preview_seed_is_repeatable_and_changes_projection() {
        let first = generate_vocabulary_with_seed(240, 17);
        let repeat = generate_vocabulary_with_seed(240, 17);
        let different = generate_vocabulary_with_seed(240, 18);
        assert!(first.as_ref() == repeat.as_ref());
        assert_ne!(first[0].hanzi, different[0].hanzi);
        assert_eq!(first.len(), 240);
        assert!(
            first.iter().all(
                |record| record.encounters >= record.sessions && record.sessions >= record.days
            )
        );
        assert!(
            first
                .iter()
                .any(|record| record.speaking == EvidenceStatus::NeedsEvidence)
        );
        assert!(
            first
                .iter()
                .any(|record| record.recognition == EvidenceStatus::Observed)
        );
    }

    #[test]
    fn preview_history_matches_vocabulary_rollups() {
        let words = demo_words();
        let sessions = demo_sessions();
        let total_sessions = words.iter().map(|word| word.sessions).sum::<u32>();
        let total_encounters = words.iter().map(|word| word.encounters).sum::<u32>();
        assert_eq!(total_sessions as usize, sessions.len());
        assert_eq!(
            total_encounters,
            sessions.iter().map(|session| session.observed).sum()
        );
        assert!(words.iter().all(|word| word.sessions > 0 && word.days > 0));
        let identities = words
            .iter()
            .map(|word| word.hanzi.as_str())
            .collect::<HashSet<_>>();
        assert!(
            sessions
                .iter()
                .all(|session| identities.contains(session.focus.as_str()))
        );
    }

    #[test]
    fn session_generator_is_deterministic_and_preserves_scoring_policy() {
        let first = generate_sessions(300);
        let second = generate_sessions(300);
        assert!(first == second);
        assert_eq!(first.len(), SAMPLE_SESSIONS.len() + 300);
        assert!(first.iter().any(|record| record.correct.is_none()));
        assert!(first.iter().any(|record| record.category == "Games"));
        assert!(first.iter().any(|record| record.category == "Practice"));
    }
}
