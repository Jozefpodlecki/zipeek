use zipeek_core::Word;
use zipeek_exercises::image_questions::{ImageQuestion, ImageQuestionMode, VisualWord, prepare};

use crate::services::{ai, image_generation};
use zipeek_ai_protocol::image_scene;

pub(super) struct GeneratedQuestion {
    pub question: ImageQuestion,
    pub sentence: String,
    pub image_url: String,
}

pub(super) async fn generate(
    words: &[Word],
    mode: ImageQuestionMode,
    api_url: &str,
    mut progress: impl FnMut(&'static str, usize),
) -> Result<GeneratedQuestion, String> {
    progress("Preparing eligible HSK vocabulary", 0);
    let candidates = words
        .iter()
        .filter_map(|word| {
            word.english.as_ref().map(|english| VisualWord {
                id: word.id.as_i32(),
                hanzi: word.simplified.clone(),
                english: english.clone(),
            })
        })
        .collect::<Vec<_>>();
    let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
    let question = prepare(&candidates, mode, seed)
        .map_err(|error| format!("Cannot prepare visual vocabulary: {error}"))?;
    question.validate().map_err(|error| error.to_string())?;

    let config = ai::configuration();
    if config.model.trim().is_empty() {
        return Err(
            "Configure an AI model in Settings before starting Picture Vocabulary".to_owned(),
        );
    }
    progress("Composing a scene with AI", 0);
    let mut received = 0usize;
    let mut last_update = 0usize;
    let raw = ai::complete_with_updates(
        &config.url,
        &config.model,
        image_scene::messages(&question),
        |update| match update {
            zipeek_ai_client::StreamUpdate::Content(chunk) => {
                received += chunk.chars().count();
                if received.saturating_sub(last_update) >= 32 {
                    last_update = received;
                    progress("Composing a scene with AI", received);
                }
            }
            zipeek_ai_client::StreamUpdate::Reasoning { received_chars } => {
                if received == 0 && received_chars.saturating_sub(last_update) >= 128 {
                    last_update = received_chars;
                    progress("AI preparing scene", received_chars);
                }
            }
        },
    )
    .await?;
    let (sentence, prompt) = image_scene::image_prompt(&question, &raw)?;
    progress("Generating image from the scene", 0);
    let image_url = image_generation::generate(api_url, &prompt).await?;
    Ok(GeneratedQuestion {
        question,
        sentence,
        image_url,
    })
}
