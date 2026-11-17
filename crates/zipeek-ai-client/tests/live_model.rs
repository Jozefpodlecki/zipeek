use std::io::{self, Write};

use zipeek_ai_client::{AiClient, StreamUpdate, TranslationDirection, translation_messages};
use zipeek_ai_protocol::sentence_analysis;

#[tokio::test]
#[ignore = "requires a user-configured OpenAI-compatible model"]
async fn complex_sentences_against_live_model() {
    let url = std::env::var("ZIPEEK_AI_ENDPOINT").expect("set ZIPEEK_AI_ENDPOINT");
    let model = std::env::var("ZIPEEK_AI_MODEL").expect("set ZIPEEK_AI_MODEL");
    let key = std::env::var("ZIPEEK_AI_KEY").unwrap_or_default();
    let client = AiClient::new(&url, &model).unwrap();
    for raw in [
        include_str!("fixtures/complex_zh_en.json"),
        include_str!("fixtures/complex_en_zh.json"),
    ] {
        let fixture: serde_json::Value = serde_json::from_str(raw).unwrap();
        let source = fixture["source"].as_str().unwrap();
        let direction = fixture["direction"].as_str().unwrap();
        let translation_direction = TranslationDirection::try_from(direction).unwrap();
        println!("Direction: {direction}; generating plain translation (streaming)...");
        let mut reasoning_report = 0_usize;
        let plain = client
            .complete_with_updates(
                translation_messages(source, translation_direction),
                &key,
                |update| match update {
                    StreamUpdate::Content(text) => {
                        print!("{text}");
                        let _ = io::stdout().flush();
                    }
                    StreamUpdate::Reasoning { received_chars } => {
                        if received_chars.saturating_sub(reasoning_report) >= 256 {
                            reasoning_report = received_chars;
                            println!("[model reasoning: {received_chars} characters received]");
                        }
                    }
                },
            )
            .await
            .unwrap();
        println!();
        assert!(!plain.trim().is_empty(), "no translation for {direction}");
        println!("Generating structured analysis (streaming):");
        let mut received = 0_usize;
        let mut reasoning_report = 0_usize;
        let analysis_raw = client
            .complete_with_updates(
                sentence_analysis::messages(source, direction),
                &key,
                |update| match update {
                    StreamUpdate::Content(delta) => {
                        received += delta.len();
                        print!("{delta}");
                        let _ = io::stdout().flush();
                    }
                    StreamUpdate::Reasoning { received_chars } => {
                        if received_chars.saturating_sub(reasoning_report) >= 256 {
                            reasoning_report = received_chars;
                            println!("[model reasoning: {received_chars} characters received]");
                        }
                    }
                },
            )
            .await
            .unwrap();
        println!("\nReceived {received} bytes of analysis.");
        let analysis =
            sentence_analysis::parse(&analysis_raw, source, direction).unwrap_or_else(|error| {
                panic!(
                    "invalid structured analysis for {direction}: {error}; reply: {}",
                    &analysis_raw.chars().take(800).collect::<String>()
                )
            });
        assert!(!analysis.rationale.is_empty());
        assert!(!analysis.translation.is_empty());
        assert!(
            !analysis.grammar.is_empty(),
            "model returned no grammar evidence for complex {direction} sentence"
        );
        assert!(
            !analysis.vocabulary.is_empty(),
            "model returned no vocabulary evidence for complex {direction} sentence"
        );
        println!("Direction: {direction}\nPlain translation: {plain}");
        println!(
            "Structured analysis: {}",
            serde_json::to_string_pretty(&analysis).unwrap()
        );
    }
}
