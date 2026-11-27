#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalService {
    AiText,
    ImageGeneration,
}

pub fn required_external_services(exercise_id: &str) -> &'static [ExternalService] {
    match exercise_id {
        "sentence-translation"
        | "story-builder"
        | "reading-comprehension"
        | "conversation-practice"
        | "fill-blank"
        | "sentence-builder"
        | "sentence-transformation"
        | "spot-mistake" => &[ExternalService::AiText],
        "image-vocabulary" => &[ExternalService::AiText, ExternalService::ImageGeneration],
        _ => &[],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictAuthority {
    Deterministic,
    TranslationEvaluator,
    PracticeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExerciseDefinition {
    pub id: &'static str,
    pub version: u32,
    pub description: &'static str,
    pub visible_input_contract: &'static str,
    pub learner_output_contract: &'static str,
    pub verdict_authority: VerdictAuthority,
}

impl ExerciseDefinition {
    pub fn accepts_scored_results(&self) -> bool {
        self.verdict_authority != VerdictAuthority::PracticeOnly
    }

    pub fn learner_instructions(&self) -> String {
        format!(
            "Exercise: {} (v{})\nDescription: {}\nVisible input: {}\nExpected answer: {}\nReturn only the expected answer; do not grade your own response.",
            self.id,
            self.version,
            self.description,
            self.visible_input_contract,
            self.learner_output_contract,
        )
    }
}

pub const MATCHING: ExerciseDefinition = ExerciseDefinition {
    id: "matching",
    version: 1,
    description: "Match each visible Chinese vocabulary tile to its corresponding meaning. Each tile may be used once.",
    visible_input_contract: "Visible left and right tiles have independent side-local IDs (L0, R0, etc.) and display text. Hidden matches and shared word IDs are never provided.",
    learner_output_contract: "JSON object with pairs: an array of {left_id, right_id} objects. IDs must refer to visible side-local tiles, without duplicates.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const MULTIPLE_CHOICE: ExerciseDefinition = ExerciseDefinition {
    id: "multiple-choice",
    version: 1,
    description: "Choose the single correct vocabulary option for the displayed prompt.",
    visible_input_contract: "A prompt and candidate options, each with a stable visible ID; no correct option is marked.",
    learner_output_contract: "JSON object with selected_id: the ID of one visible option.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const SENTENCE_TRANSLATION: ExerciseDefinition = ExerciseDefinition {
    id: "sentence-translation",
    version: 2,
    description: "Translate the displayed sentence between Mandarin and English by ordering target-language word-bank tiles. The direction is set by the exercise instance.",
    visible_input_contract: "An explicit source and target language, a source sentence, and shuffled target-language tiles with distinct IDs, including repeated words. The hidden solution is not shown.",
    learner_output_contract: "JSON object with selected_ids: an ordered array of visible target-language tile IDs. Use each required tile exactly once.",
    verdict_authority: VerdictAuthority::TranslationEvaluator,
};

pub const LISTENING_CHOICE: ExerciseDefinition = ExerciseDefinition {
    id: "listening-choice",
    version: 1,
    description: "Choose the vocabulary response corresponding to the heard audio.",
    visible_input_contract: "Audio content and labeled candidate IDs; text transcript and hidden answer must not be disclosed to the learner.",
    learner_output_contract: "JSON object with selected_id: the ID of one visible candidate.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const SPEAKING: ExerciseDefinition = ExerciseDefinition {
    id: "speaking",
    version: 2,
    description: "Record, replay, and improve Mandarin pronunciation without automated scoring.",
    visible_input_contract: "Pronunciation practice context and a local microphone recorder; no simulated scoring verdict.",
    learner_output_contract: "Optional local speech recording for replay and self-comparison, never treated as verified acoustic assessment.",
    verdict_authority: VerdictAuthority::PracticeOnly,
};

pub const IMAGE_VOCABULARY: ExerciseDefinition = ExerciseDefinition {
    id: "image-vocabulary",
    version: 1,
    description: "Identify vocabulary from a reviewed generated picture.",
    visible_input_contract: "An approved image and visible Chinese candidate IDs; hidden correct IDs are not supplied.",
    learner_output_contract: "JSON object with selected_ids: chosen visible candidate IDs.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const CHARACTER_HUNT: ExerciseDefinition = ExerciseDefinition {
    id: "character-hunt",
    version: 1,
    description: "Find dictionary words by selecting two or three Hanzi from a grid. Matched tiles are removed from play.",
    visible_input_contract: "A shuffled grid of 2–3-character word fragments, completed cells and a difficulty-dependent board size. Hidden target groupings are not shown.",
    learner_output_contract: "An ordered sequence of two or three visible tile indices forming a dictionary-backed word.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const TYPE_PINYIN: ExerciseDefinition = ExerciseDefinition {
    id: "type-pinyin",
    version: 1,
    description: "Type the pinyin reading for the displayed Chinese word.",
    visible_input_contract: "A Chinese word and its English meaning; no reading is exposed before checking.",
    learner_output_contract: "The typed pinyin reading, using tone marks or tone numbers.",
    verdict_authority: VerdictAuthority::Deterministic,
};
pub const CHARACTER_BUILDER: ExerciseDefinition = ExerciseDefinition {
    id: "character-builder",
    version: 1,
    description: "Build the Chinese word corresponding to the displayed English meaning from character tiles.",
    visible_input_contract: "An English meaning, empty character slots and a shuffled bank containing all target characters and distractors.",
    learner_output_contract: "An ordered selection of character tile indices.",
    verdict_authority: VerdictAuthority::Deterministic,
};
pub const WORD_MOSAICS: ExerciseDefinition = ExerciseDefinition {
    id: "word-mosaics",
    version: 1,
    description: "Match Chinese vocabulary tiles to their English meanings until the mosaic is cleared.",
    visible_input_contract: "Shuffled Chinese and English tiles for several dictionary words, without visible correspondence.",
    learner_output_contract: "Selected pairs of tile indices.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const FILL_BLANK: ExerciseDefinition = ExerciseDefinition {
    id: "fill-blank",
    version: 1,
    description: "Complete an authored Chinese sentence by supplying the missing Hanzi.",
    visible_input_contract: "A sentence with one missing span and an English context cue; the accepted answer is hidden.",
    learner_output_contract: "A Chinese character or word typed into the missing span.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const AUDIO_DICTATION: ExerciseDefinition = ExerciseDefinition {
    id: "audio-dictation",
    version: 1,
    description: "Listen to Mandarin speech and transcribe the hidden Chinese sentence.",
    visible_input_contract: "Mandarin audio playback controls without a visible transcript.",
    learner_output_contract: "Hanzi transcript typed after listening; no microphone required.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const SENTENCE_BUILDER: ExerciseDefinition = ExerciseDefinition {
    id: "sentence-builder",
    version: 1,
    description: "Reorder authored Chinese word or phrase tiles into a natural sentence.",
    visible_input_contract: "An English meaning and a shuffled bank of Chinese phrases with distinct tile IDs; approved solutions remain hidden.",
    learner_output_contract: "An ordered list of unique visible tile IDs using every tile exactly once.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const SENTENCE_TRANSFORMATION: ExerciseDefinition = ExerciseDefinition {
    id: "sentence-transformation",
    version: 1,
    description: "Rewrite an authored Mandarin sentence using a specified grammar construction and a word bank with possible distractors.",
    visible_input_contract: "The original sentence, target grammar construction, task instruction, and shuffled Chinese tile IDs. Approved alternatives are hidden until checking.",
    learner_output_contract: "An ordered subset of unique visible tile IDs; curated semantically valid variants are accepted.",
    verdict_authority: VerdictAuthority::PracticeOnly,
};

pub const SPOT_MISTAKE: ExerciseDefinition = ExerciseDefinition {
    id: "spot-mistake",
    version: 1,
    description: "Find an incorrect phrase in an authored Mandarin sentence, then type a valid replacement.",
    visible_input_contract: "An English meaning and a segmented Chinese sentence. Neither the mistaken span nor the correction is revealed before checking.",
    learner_output_contract: "A selected span index and a typed Hanzi replacement; detection and repair are evaluated separately.",
    verdict_authority: VerdictAuthority::PracticeOnly,
};

pub const WRITE_HANZI: ExerciseDefinition = ExerciseDefinition {
    id: "write-hanzi",
    version: 1,
    description: "Trace each character of a vocabulary word in canonical stroke order, with immediate geometric feedback.",
    visible_input_contract: "A vocabulary word, its reading, meaning, and canonical stroke reference for each character.",
    learner_output_contract: "Ordered pointer traces matched locally against the canonical character strokes; unverified geometry is never accepted as a completed exercise.",
    verdict_authority: VerdictAuthority::PracticeOnly,
};

pub const STORY_BUILDER: ExerciseDefinition = ExerciseDefinition {
    id: "story-builder",
    version: 1,
    description: "Complete a 5 to 10 sentence AI-authored Mandarin story by selecting missing words from a word bank.",
    visible_input_contract: "A coherent ordered passage with numbered gaps and a shuffled bank of Chinese words; hidden solutions are excluded.",
    learner_output_contract: "One word-bank choice per gap; each choice may be used at most once.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const READING_COMPREHENSION: ExerciseDefinition = ExerciseDefinition {
    id: "reading-comprehension",
    version: 1,
    description: "Read an AI-authored passage of 5 to 10 Mandarin sentences and answer a contextual question.",
    visible_input_contract: "Ordered story sentences, one question about topic, action or circumstances, and three unmarked options.",
    learner_output_contract: "The index of one of three visible answers.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const CONVERSATION_PRACTICE: ExerciseDefinition = ExerciseDefinition {
    id: "conversation-practice",
    version: 1,
    description: "Choose replies in an AI-authored dialogue spanning 5 to 10 turns.",
    visible_input_contract: "A dialogue scenario, the current utterance, and three candidate replies.",
    learner_output_contract: "One reply index per conversational turn.",
    verdict_authority: VerdictAuthority::Deterministic,
};

pub const DEFINITIONS: &[ExerciseDefinition] = &[
    TYPE_PINYIN,
    CHARACTER_BUILDER,
    WORD_MOSAICS,
    MATCHING,
    MULTIPLE_CHOICE,
    SENTENCE_TRANSLATION,
    STORY_BUILDER,
    READING_COMPREHENSION,
    CONVERSATION_PRACTICE,
    LISTENING_CHOICE,
    IMAGE_VOCABULARY,
    SPEAKING,
    CHARACTER_HUNT,
    FILL_BLANK,
    AUDIO_DICTATION,
    SENTENCE_BUILDER,
    SENTENCE_TRANSFORMATION,
    SPOT_MISTAKE,
    WRITE_HANZI,
];

pub const PRACTICE_PLAYLIST_ORDER: [&str; 19] = [
    "matching",
    "multiple-choice",
    "type-pinyin",
    "character-builder",
    "word-mosaics",
    "character-hunt",
    "listening-choice",
    "sentence-translation",
    "story-builder",
    "reading-comprehension",
    "conversation-practice",
    "speaking",
    "image-vocabulary",
    "fill-blank",
    "audio-dictation",
    "sentence-builder",
    "sentence-transformation",
    "spot-mistake",
    "write-hanzi",
];

pub fn needs_generated_content(id: &str) -> bool {
    matches!(
        id,
        "sentence-translation"
            | "image-vocabulary"
            | "story-builder"
            | "reading-comprehension"
            | "conversation-practice"
    )
}

pub fn supports_endless_session_host(id: &str) -> bool {
    matches!(
        id,
        "matching"
            | "multiple-choice"
            | "type-pinyin"
            | "character-builder"
            | "word-mosaics"
            | "character-hunt"
    )
}

pub fn supports_resumable_session_host(id: &str) -> bool {
    matches!(
        id,
        "type-pinyin"
            | "character-builder"
            | "word-mosaics"
            | "fill-blank"
            | "audio-dictation"
            | "sentence-builder"
            | "sentence-transformation"
            | "spot-mistake"
            | "write-hanzi"
            | "speaking"
            | "listening-choice"
            | "matching"
            | "multiple-choice"
            | "character-hunt"
            | "sentence-translation"
            | "image-vocabulary"
            | "story-builder"
            | "reading-comprehension"
            | "conversation-practice"
    )
}

pub fn definition(id: &str) -> Option<&'static ExerciseDefinition> {
    DEFINITIONS.iter().find(|candidate| candidate.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generation_capabilities_are_explicit_and_known() {
        assert!(needs_generated_content("sentence-translation"));
        assert!(needs_generated_content("image-vocabulary"));
        assert!(DEFINITIONS.iter().all(|item| {
            needs_generated_content(item.id)
                == matches!(
                    item.id,
                    "sentence-translation"
                        | "image-vocabulary"
                        | "story-builder"
                        | "reading-comprehension"
                        | "conversation-practice"
                )
        }));
        assert!(!needs_generated_content("matching"));
        assert!(!needs_generated_content("fill-blank"));
        assert!(!needs_generated_content("audio-dictation"));
        assert!(!needs_generated_content("spot-mistake"));
        assert_eq!(
            SPOT_MISTAKE.verdict_authority,
            VerdictAuthority::PracticeOnly
        );
        assert_eq!(SPEAKING.verdict_authority, VerdictAuthority::PracticeOnly);
    }

    #[test]
    fn endless_compatibility_covers_only_implemented_repeatable_hosts() {
        for id in PRACTICE_PLAYLIST_ORDER {
            assert_eq!(
                supports_endless_session_host(id),
                matches!(
                    id,
                    "matching"
                        | "multiple-choice"
                        | "type-pinyin"
                        | "character-builder"
                        | "word-mosaics"
                        | "character-hunt"
                )
            );
        }
        assert!(!supports_endless_session_host("fill-blank"));
        assert!(!supports_endless_session_host("not-an-exercise"));
    }

    #[test]
    fn definitions_are_unique_versioned_and_have_ai_visible_contracts() {
        let mut ids = HashSet::new();
        for item in DEFINITIONS {
            assert!(ids.insert(item.id));
            assert!(item.version > 0);
            assert!(!item.description.is_empty());
            assert!(!item.visible_input_contract.is_empty());
            assert!(!item.learner_output_contract.is_empty());
            assert!(item.learner_instructions().contains(item.id));
            assert!(!item.learner_instructions().contains("correct_id"));
            assert!(definition(item.id).is_some());
        }
    }
}

#[cfg(test)]
mod playlist_tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_exercise_has_exactly_one_playlist_position() {
        let unique = PRACTICE_PLAYLIST_ORDER
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), DEFINITIONS.len());
        assert!(
            DEFINITIONS
                .iter()
                .all(|definition| unique.contains(definition.id))
        );
    }
    #[test]
    fn all_resumable_exercises_have_a_real_definition_and_preparation_class() {
        for id in PRACTICE_PLAYLIST_ORDER {
            assert!(definition(id).is_some());
            assert_eq!(
                needs_generated_content(id),
                matches!(
                    id,
                    "sentence-translation"
                        | "image-vocabulary"
                        | "story-builder"
                        | "reading-comprehension"
                        | "conversation-practice"
                )
            );
        }
        assert!(!supports_resumable_session_host("not-a-real-exercise"));
    }

    #[test]
    fn resumable_host_capability_covers_generated_and_local_exercises() {
        assert!(
            DEFINITIONS
                .iter()
                .all(|definition| { supports_resumable_session_host(definition.id) })
        );
        assert!(supports_resumable_session_host("speaking"));
        assert!(supports_resumable_session_host("listening-choice"));
        assert!(supports_resumable_session_host("matching"));
        assert!(supports_resumable_session_host("multiple-choice"));
        assert!(supports_resumable_session_host("character-hunt"));

        assert!(supports_resumable_session_host("image-vocabulary"));
        assert!(supports_resumable_session_host("sentence-translation"));
    }

    #[test]
    fn every_playlisted_exercise_has_a_unique_resumable_host_contract() {
        let mut visited = HashSet::new();
        for id in PRACTICE_PLAYLIST_ORDER {
            assert!(visited.insert(id), "duplicate playlist exercise {id}");
            assert!(definition(id).is_some());
            assert!(
                supports_resumable_session_host(id),
                "missing fixed session host for {id}"
            );
        }
        assert_eq!(visited.len(), DEFINITIONS.len());
    }

    #[test]
    fn exercise_catalog_keeps_unscored_modalities_unscored() {
        for id in ["speaking", "write-hanzi", "spot-mistake"] {
            assert_eq!(
                definition(id).unwrap().verdict_authority,
                VerdictAuthority::PracticeOnly
            );
            assert!(!definition(id).unwrap().accepts_scored_results());
        }
        for id in ["matching", "multiple-choice", "listening-choice"] {
            assert_eq!(
                definition(id).unwrap().verdict_authority,
                VerdictAuthority::Deterministic
            );
            assert!(definition(id).unwrap().accepts_scored_results());
        }
    }
}

#[cfg(test)]
mod service_requirement_tests {
    use super::{ExternalService, required_external_services};

    #[test]
    fn generated_exercises_expose_their_actual_service_dependencies() {
        assert_eq!(
            required_external_services("sentence-translation"),
            &[ExternalService::AiText]
        );
        assert_eq!(
            required_external_services("image-vocabulary"),
            &[ExternalService::AiText, ExternalService::ImageGeneration]
        );
        assert!(required_external_services("matching").is_empty());
        for id in [
            "fill-blank",
            "sentence-builder",
            "sentence-transformation",
            "spot-mistake",
        ] {
            assert_eq!(required_external_services(id), &[ExternalService::AiText]);
        }
    }
}
