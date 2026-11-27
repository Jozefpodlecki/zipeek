use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use web_sys::Storage;
use zipeek_ai_protocol::{Sentence, SentenceVocabulary};
use zipeek_core::Word;
use zipeek_exercises::ai_narrative::{
    ConversationPractice, NarrativeSettings, ReadingComprehension, StoryBuilder,
};
use zipeek_exercises::{
    image_questions::{ImageQuestionMode, VisualWord},
    local_quiz::QuizWord,
    translation::TranslationDirection,
};

use super::{ai, image_generation};
use crate::{contexts::database::Dictionary, pages::practice::vocabulary_exercises::load_words};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GeneratedImageChoice {
    pub id: i32,
    pub hanzi: String,
    pub english: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum GeneratedInstance {
    Matching {
        words: Vec<QuizWord>,
    },
    StoryBuilder {
        content: StoryBuilder,
    },
    ReadingComprehension {
        content: ReadingComprehension,
    },
    ConversationPractice {
        content: ConversationPractice,
    },
    SentenceTranslation {
        sentence: Sentence,
    },
    ImageVocabulary {
        choices: Vec<GeneratedImageChoice>,
        correct_ids: BTreeSet<i32>,
        image_url: String,
        scene: String,
    },
}

impl GeneratedInstance {
    pub fn decode(raw: &str, exercise_id: &str) -> Result<Self, String> {
        if raw.len() > 262_144 {
            return Err("Generated exercise exceeds the storage limit".to_owned());
        }
        let instance: Self = serde_json::from_str(raw)
            .map_err(|_| "Generated exercise has an invalid structure".to_owned())?;
        match (&instance, exercise_id) {
            (Self::StoryBuilder { content }, "story-builder") => {
                content
                    .validate(content.sentences.len() as u8)
                    .map_err(str::to_owned)?;
            }
            (Self::ReadingComprehension { content }, "reading-comprehension") => {
                content
                    .validate(content.sentences.len() as u8)
                    .map_err(str::to_owned)?;
            }
            (Self::ConversationPractice { content }, "conversation-practice") => {
                content
                    .validate(content.turns.len() as u8)
                    .map_err(str::to_owned)?;
            }
            (Self::Matching { words }, "matching") => {
                validate_matching_words(words)?;
            }
            (Self::SentenceTranslation { sentence }, "sentence-translation") => {
                let vocabulary = SentenceVocabulary::new(
                    "Saved exercise".to_owned(),
                    sentence.target_words.clone(),
                )?;
                let direction = TranslationDirection::from_code(&sentence.direction)
                    .ok_or("Generated sentence has an invalid direction")?;
                let pair = serde_json::json!({
                    "chinese": sentence.chinese.clone(),
                    "english": sentence.english.clone(),
                });
                let expected = zipeek_ai_protocol::parse_sentence_for_vocabulary(
                    &pair.to_string(),
                    &vocabulary,
                    direction,
                )?;
                if *sentence != expected {
                    return Err(
                        "Saved translation tiles differ from the validated sentence".to_owned()
                    );
                }
            }
            (
                Self::ImageVocabulary {
                    choices,
                    correct_ids,
                    image_url,
                    scene,
                },
                "image-vocabulary",
            ) => {
                let ids = choices
                    .iter()
                    .map(|choice| choice.id)
                    .collect::<BTreeSet<_>>();
                if !(3..=6).contains(&choices.len())
                    || ids.len() != choices.len()
                    || correct_ids.is_empty()
                    || correct_ids.len() > 2
                    || !correct_ids.is_subset(&ids)
                    || choices.iter().any(|choice| {
                        choice.id <= 0
                            || choice.hanzi.trim().is_empty()
                            || choice.hanzi.chars().count() > 36
                            || choice.english.trim().is_empty()
                            || choice.english.chars().count() > 160
                    })
                    || scene.trim().is_empty()
                    || scene.chars().count() > 400
                    || scene.contains('\0')
                    || choices
                        .iter()
                        .any(|choice| choice.hanzi.contains('\0') || choice.english.contains('\0'))
                    || image_url.len() > 2048
                    || !(image_url.starts_with("http://")
                        || image_url.starts_with("https://")
                        || (image_url.starts_with("blob:")
                            && (image_url.starts_with("blob:http://tauri.localhost/")
                                || image_url.starts_with("blob:http://localhost:1421/")
                                || image_url.starts_with("blob:tauri://localhost/"))))
                    || image_url.chars().any(char::is_whitespace)
                    || image_url.contains('@')
                    || image_url.contains('#')
                    || image_url.contains('\\')
                    || (!image_url.starts_with("blob:")
                        && image_url.split_once("://").is_some_and(|(_, rest)| {
                            let authority = rest.split('/').next().unwrap_or_default();
                            authority.is_empty() || authority.starts_with(':')
                        }))
                    || (!image_url.starts_with("blob:")
                        && image_url
                            .split_once("://")
                            .is_none_or(|(_, authority_and_path)| {
                                authority_and_path.is_empty()
                                    || authority_and_path.starts_with('/')
                                    || authority_and_path.starts_with('?')
                            }))
                {
                    return Err("Generated picture contains invalid question data".to_owned());
                }
            }
            _ => return Err("Generated content belongs to another exercise type".to_owned()),
        }
        Ok(instance)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedMatchingResponse {
    words: Vec<MatchingCandidate>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchingCandidate {
    hanzi: String,
    pinyin: String,
    meaning: String,
}

fn validate_matching_words(words: &[QuizWord]) -> Result<(), String> {
    if !(2..=7).contains(&words.len()) {
        return Err("Matching requires 2 to 7 distinct pairs".to_owned());
    }
    let mut hanzi = BTreeSet::new();
    let mut meaning = BTreeSet::new();
    for (index, word) in words.iter().enumerate() {
        if word.id != index as i32 + 1
            || word.hanzi.chars().count() == 0
            || word.hanzi.chars().count() > 12
            || !word
                .hanzi
                .chars()
                .all(|ch| matches!(ch as u32, 0x3400..=0x9fff | 0x20000..=0x2fa1f))
            || word.pinyin.trim().is_empty()
            || word.pinyin.chars().count() > 80
            || word.pinyin.chars().any(char::is_control)
            || word.meaning.trim().is_empty()
            || word.meaning.chars().count() > 160
            || word.meaning.chars().any(char::is_control)
            || !hanzi.insert(word.hanzi.clone())
            || !meaning.insert(word.meaning.trim().to_lowercase())
        {
            return Err("AI matching data has invalid, duplicate or incomplete pairs".to_owned());
        }
    }
    Ok(())
}

fn parse_generated_matching(raw: &str, count: usize) -> Result<Vec<QuizWord>, String> {
    if raw.len() > 16_384 {
        return Err("AI matching response is too large".to_owned());
    }
    let response: GeneratedMatchingResponse = serde_json::from_str(raw).map_err(|_| {
        "AI matching response must contain a JSON object with a words array".to_owned()
    })?;
    if response.words.len() != count {
        return Err("AI matching response contains the wrong number of pairs".to_owned());
    }
    let words = response
        .words
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| QuizWord {
            id: index as i32 + 1,
            hanzi: candidate.hanzi,
            pinyin: candidate.pinyin,
            meaning: candidate.meaning,
        })
        .collect::<Vec<_>>();
    validate_matching_words(&words)?;
    Ok(words)
}

fn sentence_vocabulary(words: &[Word], seed: u64) -> Result<SentenceVocabulary, String> {
    let mut usable = words
        .iter()
        .filter(|word| {
            word.english
                .as_ref()
                .is_some_and(|meaning| !meaning.trim().is_empty())
        })
        .map(|word| (word.id.as_i32(), word.simplified.clone()))
        .collect::<Vec<_>>();
    usable.sort_by_key(|(id, _)| *id);
    if usable.is_empty() {
        return Err("The selected vocabulary contains no sentence candidates".to_owned());
    }
    SentenceVocabulary::new(
        "Practice selection".to_owned(),
        usable.into_iter().map(|(_, word)| word),
    )
    .map(|set| set.sampled(seed))
}

fn configured_translation_direction(
    selected: &str,
    ordinal: usize,
) -> Result<TranslationDirection, String> {
    match selected {
        "zh-to-en" => Ok(TranslationDirection::ChineseToEnglish),
        "en-to-zh" => Ok(TranslationDirection::EnglishToChinese),
        "mixed" if ordinal.is_multiple_of(2) => Ok(TranslationDirection::ChineseToEnglish),
        "mixed" => Ok(TranslationDirection::EnglishToChinese),
        _ => Err("Invalid saved translation direction".to_owned()),
    }
}

pub(crate) async fn prepare(
    exercise_id: &str,
    vocabulary: &HashMap<String, String>,
    seed: u64,
    translation_direction: &str,
    translation_ordinal: usize,
    matching_pairs: Option<u8>,
    narrative: NarrativeSettings,
    dictionary: &Dictionary,
    storage: &Storage,
) -> Result<String, String> {
    let config = ai::configuration();
    if config.model.trim().is_empty() {
        return Err("Select an AI model in Settings before preparing this exercise".to_owned());
    }
    let generated = match exercise_id {
        "matching" => {
            let count = usize::from(matching_pairs.ok_or("Missing Matching pair count")?);
            if !(2..=7).contains(&count) {
                return Err("Matching requires 2 to 7 pairs".to_owned());
            }
            let raw = ai::complete(&config.url, &config.model, vec![
                serde_json::json!({"role":"system", "content":"Generate accurate educational Mandarin vocabulary. Return only a JSON object with a words array, no Markdown. Each item must have exactly hanzi, pinyin with tone marks, and a concise English meaning. All meanings and words must be unique. Never include IDs or additional fields."}),
                serde_json::json!({"role":"user", "content": format!("Create exactly {count} distinct beginner Mandarin vocabulary pairs, each 1 to 6 Chinese characters. The JSON format is an object with one words array and each word contains exactly hanzi, pinyin and meaning fields. No extra text.")}),
            ]).await?;
            GeneratedInstance::Matching {
                words: parse_generated_matching(&raw, count)?,
            }
        }
        "story-builder" | "reading-comprehension" | "conversation-practice" => {
            narrative.validate().map_err(str::to_owned)?;
            let words = load_words(dictionary, vocabulary, false).await?;
            let bank = sentence_vocabulary(&words, seed)?;
            let vocabulary_words = bank.vocabulary.iter().take(18).cloned().collect::<Vec<_>>();
            let count = if exercise_id == "conversation-practice" {
                narrative.dialogue_turns
            } else {
                narrative.passage_sentences
            };
            let grammar_focus = zipeek_exercises::sentence_transformation::item_for_seed(
                0,
                narrative.grammar_pattern_index,
            )
            .filter(|_| narrative.grammar_pattern_index != 0)
            .map(|item| item.target_structure)
            .unwrap_or("natural HSK-level grammar");
            let instructions = match exercise_id {
                "story-builder" => format!(
                    "Return exactly {{\"sentences\": [exactly {count} Chinese sentence strings], \"gaps\": [{{\"id\":0,\"sentence_index\":0,\"answer\":\"词\"}}, ...], \"word_bank\": [Chinese words]}}. Use 2 to 6 unique gaps, one in each selected sentence, marked exactly ____; answers must appear exactly once in word_bank. Word bank includes plausible distractors. Write a coherent story."
                ),
                "reading-comprehension" => format!(
                    "Return a JSON object with exactly {{\"sentences\": [exactly {count} complete Chinese sentences], \"question_type\":\"main_idea\" or \"character_action\" or \"circumstances\", \"question\":\"Chinese question\", \"choices\":[three distinct Chinese answers], \"correct_index\":0, \"explanation\":\"brief Chinese explanation\"}}. The question must be answerable only from the passage; exactly one defensible answer."
                ),
                _ => format!(
                    "Return a JSON object with exactly {{\"scenario\":\"short context\", \"turns\": [exactly {count} objects each with \"speaker_text\":\"AI speaker utterance\", \"choices\":[three distinct Chinese responses], \"correct_index\":0]}}. Produce a coherent 5 to 10 round dialogue with one contextually appropriate reply for each round."
                ),
            };
            let raw = ai::complete(&config.url, &config.model, vec![
                serde_json::json!({"role":"system","content":"Create accurate, cohesive Mandarin learning exercises. Return only valid JSON with the exact requested fields, no Markdown. Preserve the requested sentence/round count."}),
                serde_json::json!({"role":"user","content":format!("Exercise: {exercise_id}. Focus grammar: {grammar_focus}. Vocabulary to incorporate naturally: {}. {instructions}", vocabulary_words.join("、"))}),
            ]).await?;
            if raw.len() > 262_144 {
                return Err("AI passage exceeds the allowed response size".to_owned());
            }
            match exercise_id {
                "story-builder" => {
                    let content: StoryBuilder =
                        serde_json::from_str(&raw).map_err(|_| "Invalid AI story JSON")?;
                    content.validate(count).map_err(str::to_owned)?;
                    GeneratedInstance::StoryBuilder { content }
                }
                "reading-comprehension" => {
                    let content: ReadingComprehension =
                        serde_json::from_str(&raw).map_err(|_| "Invalid AI reading JSON")?;
                    content.validate(count).map_err(str::to_owned)?;
                    GeneratedInstance::ReadingComprehension { content }
                }
                _ => {
                    let content: ConversationPractice =
                        serde_json::from_str(&raw).map_err(|_| "Invalid AI dialogue JSON")?;
                    content.validate(count).map_err(str::to_owned)?;
                    GeneratedInstance::ConversationPractice { content }
                }
            }
        }
        "sentence-translation" => {
            let words = load_words(dictionary, vocabulary, false).await?;
            let set = sentence_vocabulary(&words, seed)?;
            let direction =
                configured_translation_direction(translation_direction, translation_ordinal)?;
            let raw = ai::complete(
                &config.url,
                &config.model,
                zipeek_ai_protocol::generation_messages_for_vocabulary(&set, direction),
            )
            .await?;
            let sentence =
                zipeek_ai_protocol::parse_sentence_for_vocabulary(&raw, &set, direction)?;
            GeneratedInstance::SentenceTranslation { sentence }
        }
        "image-vocabulary" => {
            let words = load_words(dictionary, vocabulary, false).await?;
            let candidates = words
                .iter()
                .filter_map(|word| {
                    Some(VisualWord {
                        id: word.id.as_i32(),
                        hanzi: word.simplified.clone(),
                        english: word.english.as_ref()?.clone(),
                    })
                })
                .collect::<Vec<_>>();
            let question = zipeek_exercises::image_questions::prepare(
                &candidates,
                ImageQuestionMode::OneOfThree,
                seed,
            )
            .map_err(|error| error.to_string())?;
            let scene_raw = ai::complete(
                &config.url,
                &config.model,
                zipeek_ai_protocol::image_scene::messages(&question),
            )
            .await?;
            let (scene, prompt) =
                zipeek_ai_protocol::image_scene::image_prompt(&question, &scene_raw)?;
            let image_server = image_generation::read_api_url(storage);
            let image_url = image_generation::generate(&image_server, &prompt).await?;
            GeneratedInstance::ImageVocabulary {
                choices: question
                    .options
                    .into_iter()
                    .map(|choice| GeneratedImageChoice {
                        id: choice.id,
                        hanzi: choice.hanzi,
                        english: choice.english,
                    })
                    .collect(),
                correct_ids: question.correct_ids,
                image_url,
                scene,
            }
        }
        _ => return Err("No generator is registered for this exercise".to_owned()),
    };
    let raw = serde_json::to_string(&generated).map_err(|error| error.to_string())?;
    GeneratedInstance::decode(&raw, exercise_id)?;
    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_translation_direction_controls_each_generated_question() {
        for ordinal in 0..10 {
            assert_eq!(
                configured_translation_direction("zh-to-en", ordinal),
                Ok(TranslationDirection::ChineseToEnglish),
            );
            assert_eq!(
                configured_translation_direction("en-to-zh", ordinal),
                Ok(TranslationDirection::EnglishToChinese),
            );
            assert_eq!(
                configured_translation_direction("mixed", ordinal),
                Ok(if ordinal % 2 == 0 {
                    TranslationDirection::ChineseToEnglish
                } else {
                    TranslationDirection::EnglishToChinese
                }),
            );
        }
        assert!(configured_translation_direction("invalid", 0).is_err());
    }

    #[test]
    fn generated_instances_reject_wrong_exercise_types_and_untrusted_urls() {
        let value = serde_json::json!({
            "kind": "image_vocabulary",
            "choices": [
                {"id":1,"hanzi":"猫","english":"cat"},
                {"id":2,"hanzi":"狗","english":"dog"},
                {"id":3,"hanzi":"花","english":"flower"}
            ],
            "correct_ids": [1],
            "image_url":"javascript:alert(1)",
            "scene":"A cat in a garden"
        });
        let raw = value.to_string();
        assert!(GeneratedInstance::decode(&raw, "image-vocabulary").is_err());
        let safe = raw.replace(
            "javascript:alert(1)",
            "http://localhost:1431/v1/images/test",
        );
        assert!(GeneratedInstance::decode(&safe, "image-vocabulary").is_ok());
        assert!(GeneratedInstance::decode(&safe, "sentence-translation").is_err());
        for unsafe_url in [
            "http://",
            "https://user:pass@host/image",
            "https:///image",
            "https://host/image#fragment",
            "https://:443/image",
            "https://host\\evil/image",
        ] {
            let invalid = safe.replace("http://localhost:1431/v1/images/test", unsafe_url);
            assert!(GeneratedInstance::decode(&invalid, "image-vocabulary").is_err());
        }
    }
    #[test]
    fn generated_instances_limit_size_before_parsing() {
        let oversized = " ".repeat(262_145);
        assert!(GeneratedInstance::decode(&oversized, "image-vocabulary").is_err());
        assert!(GeneratedInstance::decode("{}", "unknown").is_err());
    }

    #[test]
    fn generated_picture_rejects_nul_characters_in_presented_content() {
        let example = serde_json::json!({
            "kind": "image_vocabulary",
            "choices": [
                {"id": 1, "hanzi": "猫", "english": "cat"},
                {"id": 2, "hanzi": "狗", "english": "dog"},
                {"id": 3, "hanzi": "花", "english": "flower"}
            ],
            "correct_ids": [1],
            "image_url": "http://localhost:1431/v1/images/test",
            "scene": "A cat in a garden"
        });
        assert!(GeneratedInstance::decode(&example.to_string(), "image-vocabulary").is_ok());
        let mut bad = example.clone();
        bad["scene"] = serde_json::json!("A cat\0hidden");
        assert!(GeneratedInstance::decode(&bad.to_string(), "image-vocabulary").is_err());
        bad = example;
        bad["choices"][0]["english"] = serde_json::json!("cat\0hidden");
        assert!(GeneratedInstance::decode(&bad.to_string(), "image-vocabulary").is_err());
    }
}

#[cfg(test)]
mod generated_matching_tests {
    use super::*;

    #[test]
    fn ai_matching_pairs_are_strictly_validated_and_persistable() {
        let raw = r#"{"words":[{"hanzi":"你好","pinyin":"nǐ hǎo","meaning":"hello"},{"hanzi":"谢谢","pinyin":"xièxie","meaning":"thank you"}]}"#;
        let words = parse_generated_matching(raw, 2).unwrap();
        let persisted = serde_json::to_string(&GeneratedInstance::Matching {
            words: words.clone(),
        })
        .unwrap();
        assert_eq!(
            GeneratedInstance::decode(&persisted, "matching"),
            Ok(GeneratedInstance::Matching { words })
        );
        assert!(GeneratedInstance::decode(&persisted, "sentence-translation").is_err());
        assert!(parse_generated_matching(raw, 3).is_err());
    }

    #[test]
    fn ai_matching_rejects_untrusted_or_ambiguous_pairs() {
        let invalid = [
            r#"{"words":[{"hanzi":"你好","pinyin":"nǐ hǎo","meaning":"hello"},{"hanzi":"你好","pinyin":"nǐ hǎo","meaning":"greetings"}]}"#,
            r#"{"words":[{"hanzi":"你好","pinyin":"nǐ hǎo","meaning":"hello"},{"hanzi":"谢谢","pinyin":"xièxie","meaning":"HELLO"}]}"#,
            r#"{"words":[{"hanzi":"<script>","pinyin":"bad","meaning":"hello"},{"hanzi":"谢谢","pinyin":"xièxie","meaning":"thanks"}]}"#,
            r#"{"words":[{"hanzi":"你好","pinyin":"nǐ hǎo","meaning":"hello","extra":"no"},{"hanzi":"谢谢","pinyin":"xièxie","meaning":"thanks"}]}"#,
        ];
        for raw in invalid {
            assert!(parse_generated_matching(raw, 2).is_err(), "accepted {raw}");
        }
    }
}

#[cfg(test)]
mod narrative_generation_tests {
    use super::*;
    use zipeek_exercises::ai_narrative::{DialogueTurn, ReadingQuestionType, StoryGap};

    #[test]
    fn saved_story_respects_configured_sentence_length() {
        let mut sentences = (0..5)
            .map(|index| format!("第{index}句。"))
            .collect::<Vec<_>>();
        sentences[0] = "他喜欢____。".to_owned();
        sentences[3] = "我每天____。".to_owned();
        let raw = serde_json::to_string(&GeneratedInstance::StoryBuilder {
            content: StoryBuilder {
                sentences,
                gaps: vec![
                    StoryGap {
                        id: 0,
                        sentence_index: 0,
                        answer: "看书".to_owned(),
                    },
                    StoryGap {
                        id: 1,
                        sentence_index: 3,
                        answer: "学习".to_owned(),
                    },
                ],
                word_bank: vec!["看书".to_owned(), "学习".to_owned(), "工作".to_owned()],
            },
        })
        .unwrap();
        assert!(GeneratedInstance::decode(&raw, "story-builder").is_ok());
        assert!(GeneratedInstance::decode(&raw, "reading-comprehension").is_err());
    }

    #[test]
    fn generated_reading_and_dialogue_reject_invalid_choice_keys() {
        let reading = ReadingComprehension {
            sentences: (0..10).map(|index| format!("第{index}句话。")).collect(),
            question_type: ReadingQuestionType::MainIdea,
            question: "文章的主题是什么？".to_owned(),
            choices: ["学习".to_owned(), "旅行".to_owned(), "工作".to_owned()],
            correct_index: 2,
            explanation: "文章谈到工作。".to_owned(),
        };
        let raw = serde_json::to_string(&GeneratedInstance::ReadingComprehension {
            content: reading.clone(),
        })
        .unwrap();
        assert!(GeneratedInstance::decode(&raw, "reading-comprehension").is_ok());
        let mut invalid = reading;
        invalid.correct_index = 3;
        let raw =
            serde_json::to_string(&GeneratedInstance::ReadingComprehension { content: invalid })
                .unwrap();
        assert!(GeneratedInstance::decode(&raw, "reading-comprehension").is_err());
        let turn = DialogueTurn {
            speaker_text: "您好，今天想买什么？".to_owned(),
            choices: [
                "我要面包".to_owned(),
                "不用了".to_owned(),
                "几点了".to_owned(),
            ],
            correct_index: 0,
        };
        let raw = serde_json::to_string(&GeneratedInstance::ConversationPractice {
            content: ConversationPractice {
                scenario: "面包店".to_owned(),
                turns: vec![turn; 5],
            },
        })
        .unwrap();
        assert!(GeneratedInstance::decode(&raw, "conversation-practice").is_ok());
    }
}
