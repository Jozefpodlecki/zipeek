use super::assessment::ExerciseOutcome;
use crate::models::{
    CharacterBuilderOptions, ListeningChoiceOptions, SentenceBuilderOptions, TypePinyinOptions,
    VocabularyExerciseOptions, WriteHanziOptions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExerciseReadiness {
    Ready,
    Unavailable(String),
    NeedsGeneration,
    Queued,
    Generating,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "settings", rename_all = "snake_case")]
pub(crate) enum ExerciseConfiguration {
    CharacterBuilder(CharacterBuilderOptions),
    SentenceBuilder(SentenceBuilderOptions),
    TypePinyin(TypePinyinOptions),
    WriteHanzi(WriteHanziOptions),
    ListeningChoice(ListeningChoiceOptions),
    Quiz {
        formats: Vec<String>,
        reverse_matching: bool,
        pair_count: u8,
        #[serde(default)]
        ai_generated: bool,
        #[serde(default = "crate::models::default_matching_pinyin")]
        show_pinyin: bool,
    },
    SentenceTranslation {
        direction: String,
    },
    Narrative(zipeek_exercises::ai_narrative::NarrativeSettings),
    SentenceTransformation(crate::models::SentenceTransformationOptions),
    Standard,
}

impl ExerciseConfiguration {
    pub fn for_exercise(exercise_id: &str) -> Self {
        match exercise_id {
            "character-builder" => Self::CharacterBuilder(CharacterBuilderOptions::default()),
            "sentence-builder" => Self::SentenceBuilder(SentenceBuilderOptions::default()),
            "type-pinyin" => Self::TypePinyin(TypePinyinOptions::default()),
            "write-hanzi" => Self::WriteHanzi(WriteHanziOptions::default()),
            "listening-choice" => Self::ListeningChoice(ListeningChoiceOptions::default()),
            "matching" => Self::Quiz {
                formats: vec!["hanzi-eng".to_owned()],
                reverse_matching: false,
                pair_count: 4,
                ai_generated: false,
                show_pinyin: true,
            },
            "multiple-choice" => Self::Quiz {
                formats: vec!["hanzi-to-meaning".to_owned()],
                reverse_matching: false,
                pair_count: 4,
                ai_generated: false,
                show_pinyin: true,
            },
            "sentence-transformation" => {
                Self::SentenceTransformation(crate::models::SentenceTransformationOptions::default())
            }
            "story-builder" | "reading-comprehension" | "conversation-practice" => {
                Self::Narrative(zipeek_exercises::ai_narrative::NarrativeSettings::default())
            }
            "sentence-translation" => Self::SentenceTranslation {
                direction: default_translation_direction(),
            },
            _ => Self::Standard,
        }
    }

    pub fn vocabulary_options(&self) -> Option<VocabularyExerciseOptions> {
        match self {
            Self::CharacterBuilder(options) => {
                Some(VocabularyExerciseOptions::CharacterBuilder(*options))
            }
            Self::TypePinyin(options) => Some(VocabularyExerciseOptions::TypePinyin(*options)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PracticeStep {
    pub id: String,
    pub exercise_id: String,
    pub configuration_id: String,
    pub instance_seed: u64,
    pub readiness: ExerciseReadiness,
    pub outcome: Option<ExerciseOutcome>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub generated_instance: Option<String>,
    pub generation_attempt: u32,
    #[serde(default)]
    pub quiz_format: Option<String>,
    #[serde(default)]
    pub reverse_matching: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SessionMode {
    #[default]
    Fixed,
    Endless,
}

impl SessionMode {
    pub fn is_endless(self) -> bool {
        matches!(self, Self::Endless)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PracticeSession {
    pub id: String,
    pub revision: u64,
    pub current: usize,
    pub vocabulary: BTreeMap<String, String>,
    pub exercise_configurations: BTreeMap<String, ExerciseConfiguration>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub session_mode: SessionMode,
    #[serde(default)]
    pub scheduling_seed: u64,

    pub completed_at: Option<DateTime<Utc>>,
    pub steps: Vec<PracticeStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PracticeSessionError {
    InvalidId,
    InvalidSchedule(String),
    InvalidStoredSession,
    NotFound,
    WrongSession,
    OutOfDate,
    NotReady,
    AlreadyFinished,
    ExistingSession,
    InvalidResult,
    StaleGeneration,
    Storage(String),
}

pub(super) fn default_translation_direction() -> String {
    "mixed".to_owned()
}

pub(super) fn valid_translation_direction(direction: &str) -> bool {
    matches!(direction, "mixed" | "zh-to-en" | "en-to-zh")
}

pub(super) fn resumable_scores_supported(exercise_id: &str) -> bool {
    matches!(
        exercise_id,
        "sentence-translation"
            | "story-builder"
            | "reading-comprehension"
            | "conversation-practice"
            | "fill-blank"
            | "audio-dictation"
            | "spot-mistake"
            | "sentence-transformation"
            | "listening-choice"
            | "matching"
            | "multiple-choice"
            | "sentence-builder"
            | "type-pinyin"
            | "character-builder"
            | "word-mosaics"
    )
}
