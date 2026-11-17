use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use web_sys::Storage;
use zipeek_ai_protocol::{Sentence, SentenceVocabulary};
use zipeek_core::Word;
use zipeek_exercises::{
    image_questions::{ImageQuestionMode, VisualWord},
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
                    || !(image_url.starts_with("http://") || image_url.starts_with("https://"))
                    || image_url.chars().any(char::is_whitespace)
                    || image_url.contains('@')
                    || image_url.contains('#')
                    || image_url
                        .split_once("://")
                        .is_none_or(|(_, authority_and_path)| {
                            authority_and_path.is_empty()
                                || authority_and_path.starts_with('/')
                                || authority_and_path.starts_with('?')
                        })
                {
                    return Err("Generated picture contains invalid question data".to_owned());
                }
            }
            _ => return Err("Generated content belongs to another exercise type".to_owned()),
        }
        Ok(instance)
    }
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
    dictionary: &Dictionary,
    storage: &Storage,
) -> Result<String, String> {
    let config = ai::configuration();
    if config.model.trim().is_empty() {
        return Err("Select an AI model in Settings before preparing this exercise".to_owned());
    }
    let words = load_words(dictionary, vocabulary, false).await?;
    let generated = match exercise_id {
        "sentence-translation" => {
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
