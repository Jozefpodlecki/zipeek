use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const MIN_PASSAGE_SENTENCES: u8 = 5;
pub const MAX_PASSAGE_SENTENCES: u8 = 10;
pub const MIN_DIALOGUE_TURNS: u8 = 5;
pub const MAX_DIALOGUE_TURNS: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NarrativeSettings {
    pub passage_sentences: u8,
    pub dialogue_turns: u8,
    pub grammar_pattern_index: u8,
}

impl Default for NarrativeSettings {
    fn default() -> Self {
        Self {
            passage_sentences: 7,
            dialogue_turns: 7,
            grammar_pattern_index: 0,
        }
    }
}

impl NarrativeSettings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(MIN_PASSAGE_SENTENCES..=MAX_PASSAGE_SENTENCES).contains(&self.passage_sentences) {
            return Err("Passage length must be 5 to 10 sentences");
        }
        if !(MIN_DIALOGUE_TURNS..=MAX_DIALOGUE_TURNS).contains(&self.dialogue_turns) {
            return Err("Dialogue length must be 5 to 10 turns");
        }
        if self.grammar_pattern_index > 0
            && crate::sentence_transformation::item_for_seed(0, self.grammar_pattern_index)
                .is_none()
        {
            return Err("Select an available grammar pattern");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryGap {
    pub id: u8,
    pub sentence_index: u8,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryBuilder {
    pub sentences: Vec<String>,
    pub gaps: Vec<StoryGap>,
    pub word_bank: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingQuestionType {
    MainIdea,
    CharacterAction,
    Circumstances,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingComprehension {
    pub sentences: Vec<String>,
    pub question_type: ReadingQuestionType,
    pub question: String,
    pub choices: [String; 3],
    pub correct_index: u8,
    pub explanation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueTurn {
    pub speaker_text: String,
    pub choices: [String; 3],
    pub correct_index: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationPractice {
    pub scenario: String,
    pub turns: Vec<DialogueTurn>,
}

fn valid_text(text: &str, max_chars: usize) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= max_chars
        && !text.chars().any(char::is_control)
}

fn validate_sentences(sentences: &[String], expected: u8) -> Result<(), &'static str> {
    if !(MIN_PASSAGE_SENTENCES..=MAX_PASSAGE_SENTENCES).contains(&expected)
        || sentences.len() != usize::from(expected)
        || sentences.iter().any(|sentence| {
            !valid_text(sentence, 240)
                || sentence
                    .trim_end()
                    .chars()
                    .last()
                    .is_none_or(|ch| !matches!(ch, '。' | '！' | '？' | '!' | '?'))
        })
    {
        return Err("The AI passage must contain exactly the configured number of sentences");
    }
    Ok(())
}

fn valid_choices(choices: &[String; 3], correct_index: u8) -> bool {
    correct_index < 3
        && choices.iter().all(|choice| valid_text(choice, 200))
        && choices
            .iter()
            .map(|choice| choice.trim())
            .collect::<BTreeSet<_>>()
            .len()
            == 3
}

impl StoryBuilder {
    pub fn validate(&self, expected_sentences: u8) -> Result<(), &'static str> {
        validate_sentences(&self.sentences, expected_sentences)?;
        if !(2..=6).contains(&self.gaps.len())
            || self.word_bank.len() < self.gaps.len()
            || self.word_bank.len() > 10
            || self.word_bank.iter().any(|word| !valid_text(word, 16))
            || self.word_bank.iter().collect::<BTreeSet<_>>().len() != self.word_bank.len()
            || self
                .gaps
                .iter()
                .map(|gap| &gap.answer)
                .collect::<BTreeSet<_>>()
                .len()
                != self.gaps.len()
        {
            return Err("Story Builder needs unique word-bank choices and 2 to 6 gaps");
        }
        let mut sentences_with_gaps = BTreeSet::new();
        for (position, gap) in self.gaps.iter().enumerate() {
            if usize::from(gap.id) != position
                || usize::from(gap.sentence_index) >= self.sentences.len()
                || !sentences_with_gaps.insert(gap.sentence_index)
                || self.sentences[usize::from(gap.sentence_index)]
                    .matches("____")
                    .count()
                    != 1
                || !self.word_bank.contains(&gap.answer)
            {
                return Err("Story Builder has an invalid gap or answer mapping");
            }
        }
        if self.sentences.iter().enumerate().any(|(index, sentence)| {
            sentence.contains("____") != sentences_with_gaps.contains(&(index as u8))
        }) {
            return Err("Story Builder has an unassigned gap");
        }
        Ok(())
    }
}

impl ReadingComprehension {
    pub fn validate(&self, expected_sentences: u8) -> Result<(), &'static str> {
        validate_sentences(&self.sentences, expected_sentences)?;
        if !valid_text(&self.question, 200)
            || !valid_text(&self.explanation, 400)
            || !valid_choices(&self.choices, self.correct_index)
        {
            return Err("Reading Comprehension needs one valid answer and three unique choices");
        }
        Ok(())
    }
}

impl ConversationPractice {
    pub fn validate(&self, expected_turns: u8) -> Result<(), &'static str> {
        if !(MIN_DIALOGUE_TURNS..=MAX_DIALOGUE_TURNS).contains(&expected_turns)
            || !valid_text(&self.scenario, 300)
            || self.turns.len() != usize::from(expected_turns)
            || self.turns.iter().any(|turn| {
                !valid_text(&turn.speaker_text, 200)
                    || !valid_choices(&turn.choices, turn.correct_index)
            })
        {
            return Err("Conversation Practice needs 5 to 10 valid dialogue rounds");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sentences(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("第{i}句话。")).collect()
    }

    #[test]
    fn passage_size_is_exact_and_configured_independently_from_dialogue() {
        for count in 5..=10 {
            assert!(validate_sentences(&sentences(count), count as u8).is_ok());
            assert!(validate_sentences(&sentences(count - 1), count as u8).is_err());
        }
        assert!(
            NarrativeSettings {
                passage_sentences: 4,
                dialogue_turns: 7,
                grammar_pattern_index: 0
            }
            .validate()
            .is_err()
        );
        assert!(
            NarrativeSettings {
                passage_sentences: 10,
                dialogue_turns: 11,
                grammar_pattern_index: 0
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn story_requires_exact_gaps_and_distinct_wordbank() {
        let mut story = StoryBuilder {
            sentences: sentences(5),
            gaps: vec![
                StoryGap {
                    id: 0,
                    sentence_index: 0,
                    answer: "工作".into(),
                },
                StoryGap {
                    id: 1,
                    sentence_index: 2,
                    answer: "学习".into(),
                },
            ],
            word_bank: vec!["工作".into(), "学习".into(), "天气".into()],
        };
        story.sentences[0] = "他每天____。".into();
        story.sentences[2] = "她喜欢____。".into();
        assert!(story.validate(5).is_ok());
        story.word_bank[2] = "工作".into();
        assert!(story.validate(5).is_err());
    }

    #[test]
    fn reading_has_three_unique_answers_and_valid_key() {
        let mut reading = ReadingComprehension {
            sentences: sentences(6),
            question_type: ReadingQuestionType::Circumstances,
            question: "为什么他迟到了？".into(),
            choices: ["下雨了".into(), "起床晚了".into(), "他不想去".into()],
            correct_index: 0,
            explanation: "故事提到下雨。".into(),
        };
        assert!(reading.validate(6).is_ok());
        reading.correct_index = 3;
        assert!(reading.validate(6).is_err());
    }

    #[test]
    fn dialogue_has_configured_number_of_turns() {
        let turn = DialogueTurn {
            speaker_text: "你好，你想喝什么？".into(),
            choices: ["我要茶".into(), "我不认识".into(), "昨天晚上".into()],
            correct_index: 0,
        };
        let conversation = ConversationPractice {
            scenario: "在茶馆".into(),
            turns: vec![turn; 7],
        };
        assert!(conversation.validate(7).is_ok());
        assert!(conversation.validate(10).is_err());
    }
}
