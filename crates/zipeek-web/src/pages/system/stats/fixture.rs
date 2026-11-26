#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EvidenceStatus {
    Observed,
    NeedsEvidence,
    NeedsReview,
}

impl EvidenceStatus {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Observed => "Observed",
            Self::NeedsEvidence => "Needs evidence",
            Self::NeedsReview => "Needs review",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SampleActivity {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub can_show_score: bool,
}

pub(super) const SAMPLE_ACTIVITIES: [SampleActivity; 24] = [
    SampleActivity {
        id: "matching",
        title: "Matching",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "multiple-choice",
        title: "Multiple choice",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "type-pinyin",
        title: "Type Pinyin",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "character-builder",
        title: "Character builder",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "word-mosaics",
        title: "Word mosaics",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "character-hunt",
        title: "Character hunt",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "listening-choice",
        title: "Listening choice",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "sentence-translation",
        title: "Translate sentence",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "speaking",
        title: "Speaking recorder",
        category: "Practice",
        can_show_score: false,
    },
    SampleActivity {
        id: "image-vocabulary",
        title: "Image vocabulary",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "fill-blank",
        title: "Fill in the blank",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "audio-dictation",
        title: "Audio dictation",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "sentence-builder",
        title: "Sentence builder",
        category: "Practice",
        can_show_score: true,
    },
    SampleActivity {
        id: "sentence-transformation",
        title: "Sentence transformation",
        category: "Practice",
        can_show_score: false,
    },
    SampleActivity {
        id: "spot-mistake",
        title: "Spot the mistake",
        category: "Practice",
        can_show_score: false,
    },
    SampleActivity {
        id: "write-hanzi",
        title: "Write Hanzi",
        category: "Practice",
        can_show_score: false,
    },
    SampleActivity {
        id: "province-quest",
        title: "China Map Challenge",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "time-weaver",
        title: "Time Weaver",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "cloud-match",
        title: "Cloud Match",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "crossword",
        title: "Crossword",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "tone-detective",
        title: "Tone Detective",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "icon-pairs",
        title: "Icon Pairs",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "mini-games",
        title: "Mini games",
        category: "Games",
        can_show_score: true,
    },
    SampleActivity {
        id: "character-hunt-game",
        title: "Character Hunt game",
        category: "Games",
        can_show_score: true,
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct SampleSession {
    pub id: &'static str,
    pub date: &'static str,
    pub activity_id: &'static str,
    pub observed: u8,
    pub correct: Option<u8>,
    pub focus: &'static str,
}

impl SampleSession {
    pub fn activity(&self) -> &'static SampleActivity {
        SAMPLE_ACTIVITIES
            .iter()
            .find(|entry| entry.id == self.activity_id)
            .expect("Every demo session references a declared sample activity")
    }
}

pub(super) const SAMPLE_SESSIONS: [SampleSession; 72] = [
    SampleSession {
        id: "session-001",
        date: "2026-09-10",
        activity_id: "matching",
        observed: 5,
        correct: Some(5),
        focus: "学习",
    },
    SampleSession {
        id: "session-002",
        date: "2026-09-11",
        activity_id: "multiple-choice",
        observed: 12,
        correct: Some(11),
        focus: "认识",
    },
    SampleSession {
        id: "session-003",
        date: "2026-09-12",
        activity_id: "type-pinyin",
        observed: 8,
        correct: Some(6),
        focus: "朋友",
    },
    SampleSession {
        id: "session-004",
        date: "2026-09-14",
        activity_id: "character-builder",
        observed: 15,
        correct: Some(12),
        focus: "天气",
    },
    SampleSession {
        id: "session-005",
        date: "2026-09-15",
        activity_id: "word-mosaics",
        observed: 11,
        correct: Some(11),
        focus: "练习",
    },
    SampleSession {
        id: "session-006",
        date: "2026-09-16",
        activity_id: "character-hunt",
        observed: 7,
        correct: Some(6),
        focus: "学校",
    },
    SampleSession {
        id: "session-007",
        date: "2026-09-18",
        activity_id: "listening-choice",
        observed: 14,
        correct: Some(12),
        focus: "老师",
    },
    SampleSession {
        id: "session-008",
        date: "2026-09-19",
        activity_id: "sentence-translation",
        observed: 10,
        correct: Some(7),
        focus: "学生",
    },
    SampleSession {
        id: "session-009",
        date: "2026-09-20",
        activity_id: "speaking",
        observed: 6,
        correct: None,
        focus: "中文",
    },
    SampleSession {
        id: "session-010",
        date: "2026-09-22",
        activity_id: "image-vocabulary",
        observed: 13,
        correct: Some(12),
        focus: "喜欢",
    },
    SampleSession {
        id: "session-011",
        date: "2026-09-23",
        activity_id: "fill-blank",
        observed: 9,
        correct: Some(7),
        focus: "今天",
    },
    SampleSession {
        id: "session-012",
        date: "2026-09-24",
        activity_id: "audio-dictation",
        observed: 5,
        correct: Some(2),
        focus: "明天",
    },
    SampleSession {
        id: "session-013",
        date: "2026-09-26",
        activity_id: "sentence-builder",
        observed: 12,
        correct: Some(12),
        focus: "昨天",
    },
    SampleSession {
        id: "session-014",
        date: "2026-09-27",
        activity_id: "sentence-transformation",
        observed: 8,
        correct: None,
        focus: "吃饭",
    },
    SampleSession {
        id: "session-015",
        date: "2026-09-28",
        activity_id: "spot-mistake",
        observed: 15,
        correct: None,
        focus: "喝水",
    },
    SampleSession {
        id: "session-016",
        date: "2026-09-30",
        activity_id: "write-hanzi",
        observed: 11,
        correct: None,
        focus: "阅读",
    },
    SampleSession {
        id: "session-017",
        date: "2026-10-01",
        activity_id: "province-quest",
        observed: 7,
        correct: Some(7),
        focus: "写字",
    },
    SampleSession {
        id: "session-018",
        date: "2026-10-02",
        activity_id: "time-weaver",
        observed: 14,
        correct: Some(13),
        focus: "工作",
    },
    SampleSession {
        id: "session-019",
        date: "2026-10-04",
        activity_id: "cloud-match",
        observed: 10,
        correct: Some(8),
        focus: "时间",
    },
    SampleSession {
        id: "session-020",
        date: "2026-10-05",
        activity_id: "crossword",
        observed: 6,
        correct: Some(3),
        focus: "地方",
    },
    SampleSession {
        id: "session-021",
        date: "2026-10-06",
        activity_id: "tone-detective",
        observed: 13,
        correct: Some(13),
        focus: "旅行",
    },
    SampleSession {
        id: "session-022",
        date: "2026-10-08",
        activity_id: "icon-pairs",
        observed: 9,
        correct: Some(8),
        focus: "音乐",
    },
    SampleSession {
        id: "session-023",
        date: "2026-10-09",
        activity_id: "mini-games",
        observed: 5,
        correct: Some(3),
        focus: "快乐",
    },
    SampleSession {
        id: "session-024",
        date: "2026-10-10",
        activity_id: "character-hunt-game",
        observed: 12,
        correct: Some(9),
        focus: "帮助",
    },
    SampleSession {
        id: "session-025",
        date: "2026-10-12",
        activity_id: "matching",
        observed: 8,
        correct: Some(8),
        focus: "问题",
    },
    SampleSession {
        id: "session-026",
        date: "2026-10-13",
        activity_id: "multiple-choice",
        observed: 15,
        correct: Some(14),
        focus: "回答",
    },
    SampleSession {
        id: "session-027",
        date: "2026-10-15",
        activity_id: "type-pinyin",
        observed: 11,
        correct: Some(9),
        focus: "知道",
    },
    SampleSession {
        id: "session-028",
        date: "2026-10-17",
        activity_id: "character-builder",
        observed: 7,
        correct: Some(4),
        focus: "可以",
    },
    SampleSession {
        id: "session-029",
        date: "2026-10-19",
        activity_id: "word-mosaics",
        observed: 14,
        correct: Some(14),
        focus: "因为",
    },
    SampleSession {
        id: "session-030",
        date: "2026-10-22",
        activity_id: "character-hunt",
        observed: 10,
        correct: Some(9),
        focus: "所以",
    },
    SampleSession {
        id: "session-031",
        date: "2026-10-23",
        activity_id: "listening-choice",
        observed: 6,
        correct: Some(4),
        focus: "学习",
    },
    SampleSession {
        id: "session-032",
        date: "2026-10-24",
        activity_id: "sentence-translation",
        observed: 13,
        correct: Some(10),
        focus: "认识",
    },
    SampleSession {
        id: "session-033",
        date: "2026-10-26",
        activity_id: "speaking",
        observed: 9,
        correct: None,
        focus: "朋友",
    },
    SampleSession {
        id: "session-034",
        date: "2026-10-27",
        activity_id: "image-vocabulary",
        observed: 5,
        correct: Some(4),
        focus: "天气",
    },
    SampleSession {
        id: "session-035",
        date: "2026-10-29",
        activity_id: "fill-blank",
        observed: 12,
        correct: Some(10),
        focus: "练习",
    },
    SampleSession {
        id: "session-036",
        date: "2026-10-29",
        activity_id: "audio-dictation",
        observed: 8,
        correct: Some(5),
        focus: "学校",
    },
    SampleSession {
        id: "session-037",
        date: "2026-10-30",
        activity_id: "sentence-builder",
        observed: 15,
        correct: Some(15),
        focus: "老师",
    },
    SampleSession {
        id: "session-038",
        date: "2026-10-31",
        activity_id: "sentence-transformation",
        observed: 11,
        correct: None,
        focus: "学生",
    },
    SampleSession {
        id: "session-039",
        date: "2026-11-01",
        activity_id: "spot-mistake",
        observed: 7,
        correct: None,
        focus: "中文",
    },
    SampleSession {
        id: "session-040",
        date: "2026-11-02",
        activity_id: "write-hanzi",
        observed: 14,
        correct: None,
        focus: "喜欢",
    },
    SampleSession {
        id: "session-041",
        date: "2026-11-03",
        activity_id: "province-quest",
        observed: 10,
        correct: Some(10),
        focus: "今天",
    },
    SampleSession {
        id: "session-042",
        date: "2026-11-04",
        activity_id: "time-weaver",
        observed: 6,
        correct: Some(5),
        focus: "明天",
    },
    SampleSession {
        id: "session-043",
        date: "2026-11-05",
        activity_id: "cloud-match",
        observed: 13,
        correct: Some(11),
        focus: "昨天",
    },
    SampleSession {
        id: "session-044",
        date: "2026-11-05",
        activity_id: "crossword",
        observed: 9,
        correct: Some(6),
        focus: "吃饭",
    },
    SampleSession {
        id: "session-045",
        date: "2026-11-06",
        activity_id: "tone-detective",
        observed: 5,
        correct: Some(5),
        focus: "喝水",
    },
    SampleSession {
        id: "session-046",
        date: "2026-11-06",
        activity_id: "icon-pairs",
        observed: 12,
        correct: Some(11),
        focus: "阅读",
    },
    SampleSession {
        id: "session-047",
        date: "2026-11-07",
        activity_id: "mini-games",
        observed: 8,
        correct: Some(6),
        focus: "写字",
    },
    SampleSession {
        id: "session-048",
        date: "2026-11-08",
        activity_id: "character-hunt-game",
        observed: 15,
        correct: Some(12),
        focus: "工作",
    },
    SampleSession {
        id: "session-049",
        date: "2026-11-08",
        activity_id: "matching",
        observed: 11,
        correct: Some(11),
        focus: "时间",
    },
    SampleSession {
        id: "session-050",
        date: "2026-11-09",
        activity_id: "multiple-choice",
        observed: 7,
        correct: Some(6),
        focus: "地方",
    },
    SampleSession {
        id: "session-051",
        date: "2026-11-10",
        activity_id: "type-pinyin",
        observed: 14,
        correct: Some(12),
        focus: "旅行",
    },
    SampleSession {
        id: "session-052",
        date: "2026-11-10",
        activity_id: "character-builder",
        observed: 10,
        correct: Some(7),
        focus: "音乐",
    },
    SampleSession {
        id: "session-053",
        date: "2026-11-11",
        activity_id: "word-mosaics",
        observed: 6,
        correct: Some(6),
        focus: "快乐",
    },
    SampleSession {
        id: "session-054",
        date: "2026-11-12",
        activity_id: "character-hunt",
        observed: 13,
        correct: Some(12),
        focus: "帮助",
    },
    SampleSession {
        id: "session-055",
        date: "2026-11-12",
        activity_id: "listening-choice",
        observed: 9,
        correct: Some(7),
        focus: "问题",
    },
    SampleSession {
        id: "session-056",
        date: "2026-11-12",
        activity_id: "sentence-translation",
        observed: 5,
        correct: Some(2),
        focus: "回答",
    },
    SampleSession {
        id: "session-057",
        date: "2026-11-13",
        activity_id: "speaking",
        observed: 12,
        correct: None,
        focus: "知道",
    },
    SampleSession {
        id: "session-058",
        date: "2026-11-13",
        activity_id: "image-vocabulary",
        observed: 8,
        correct: Some(7),
        focus: "可以",
    },
    SampleSession {
        id: "session-059",
        date: "2026-11-13",
        activity_id: "fill-blank",
        observed: 15,
        correct: Some(13),
        focus: "因为",
    },
    SampleSession {
        id: "session-060",
        date: "2026-11-14",
        activity_id: "audio-dictation",
        observed: 11,
        correct: Some(8),
        focus: "所以",
    },
    SampleSession {
        id: "session-061",
        date: "2026-11-14",
        activity_id: "sentence-builder",
        observed: 7,
        correct: Some(7),
        focus: "学习",
    },
    SampleSession {
        id: "session-062",
        date: "2026-11-14",
        activity_id: "sentence-transformation",
        observed: 14,
        correct: None,
        focus: "认识",
    },
    SampleSession {
        id: "session-063",
        date: "2026-11-15",
        activity_id: "spot-mistake",
        observed: 10,
        correct: None,
        focus: "朋友",
    },
    SampleSession {
        id: "session-064",
        date: "2026-11-15",
        activity_id: "write-hanzi",
        observed: 6,
        correct: None,
        focus: "天气",
    },
    SampleSession {
        id: "session-065",
        date: "2026-11-16",
        activity_id: "province-quest",
        observed: 13,
        correct: Some(13),
        focus: "练习",
    },
    SampleSession {
        id: "session-066",
        date: "2026-11-16",
        activity_id: "time-weaver",
        observed: 9,
        correct: Some(8),
        focus: "学校",
    },
    SampleSession {
        id: "session-067",
        date: "2026-11-16",
        activity_id: "cloud-match",
        observed: 5,
        correct: Some(3),
        focus: "老师",
    },
    SampleSession {
        id: "session-068",
        date: "2026-11-17",
        activity_id: "crossword",
        observed: 12,
        correct: Some(9),
        focus: "学生",
    },
    SampleSession {
        id: "session-069",
        date: "2026-11-17",
        activity_id: "tone-detective",
        observed: 8,
        correct: Some(8),
        focus: "中文",
    },
    SampleSession {
        id: "session-070",
        date: "2026-11-17",
        activity_id: "icon-pairs",
        observed: 15,
        correct: Some(14),
        focus: "喜欢",
    },
    SampleSession {
        id: "session-071",
        date: "2026-11-18",
        activity_id: "mini-games",
        observed: 11,
        correct: Some(9),
        focus: "今天",
    },
    SampleSession {
        id: "session-072",
        date: "2026-11-18",
        activity_id: "character-hunt-game",
        observed: 7,
        correct: Some(4),
        focus: "明天",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_registered_practice_types_have_demo_history() {
        let registered: HashSet<_> = zipeek_exercises::definition::DEFINITIONS
            .iter()
            .map(|definition| definition.id)
            .collect();
        let fixture: HashSet<_> = SAMPLE_ACTIVITIES
            .iter()
            .filter(|activity| activity.category == "Practice")
            .map(|activity| activity.id)
            .collect();
        assert_eq!(registered, fixture);
        for definition in zipeek_exercises::definition::DEFINITIONS {
            let sample = SAMPLE_ACTIVITIES
                .iter()
                .find(|activity| activity.id == definition.id)
                .unwrap();
            assert_eq!(sample.can_show_score, definition.accepts_scored_results());
        }
    }

    #[test]
    fn all_sessions_have_valid_categories_and_respect_scoring_contract() {
        let mut ids = HashSet::new();
        assert!(SAMPLE_SESSIONS.len() >= 64);
        assert!(
            SAMPLE_SESSIONS
                .windows(2)
                .all(|pair| pair[0].date <= pair[1].date)
        );
        for session in SAMPLE_SESSIONS {
            assert!(ids.insert(session.id));
            let activity = session.activity();
            assert!(activity.category == "Practice" || activity.category == "Games");
            assert_eq!(session.correct.is_some(), activity.can_show_score);
            assert!(
                session
                    .correct
                    .is_none_or(|correct| correct <= session.observed)
            );
        }
    }
}
