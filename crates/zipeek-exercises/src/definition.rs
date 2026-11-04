#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictAuthority {
    Deterministic,
    TranslationEvaluator,
    PronunciationEvaluator,
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
    description: "Speak the intended Mandarin reading of a word in an unambiguous sentence context.",
    visible_input_contract: "The sentence, its pinyin, English meaning, focus word, target reading, and audio-capture affordance. The learner must not see an assessment verdict.",
    learner_output_contract: "An audio recording submitted through the exercise's recording interface, not a textual self-assessment.",
    verdict_authority: VerdictAuthority::PronunciationEvaluator,
};

pub const DEFINITIONS: &[ExerciseDefinition] = &[
    MATCHING,
    MULTIPLE_CHOICE,
    SENTENCE_TRANSLATION,
    LISTENING_CHOICE,
    SPEAKING,
];

pub fn definition(id: &str) -> Option<&'static ExerciseDefinition> {
    DEFINITIONS.iter().find(|candidate| candidate.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
