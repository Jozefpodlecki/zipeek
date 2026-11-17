use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod prompt;
mod schema;

pub use schema::json_schema;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VocabularyObservation {
    pub hanzi: String,
    pub meaning: String,
    pub pinyin: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GrammarObservation {
    pub pattern: String,
    pub explanation: String,
    pub example_fragment: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SentenceAnalysis {
    pub translation: String,
    pub alternatives: Vec<String>,
    pub features: Vec<String>,
    pub nuance: Vec<String>,
    pub difficulty: String,
    pub estimated_hsk: Option<u8>,
    pub rationale: String,
    pub vocabulary: Vec<VocabularyObservation>,
    pub grammar: Vec<GrammarObservation>,
    pub notes: Vec<String>,
}

fn meaningful(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max && !value.chars().any(char::is_control)
}

fn chinese_text(value: &str) -> bool {
    value
        .chars()
        .any(|character| ('\u{3400}'..='\u{9fff}').contains(&character))
}

pub fn parse(raw: &str, source: &str, direction: &str) -> Result<SentenceAnalysis, String> {
    if !matches!(direction, "zh-en" | "en-zh") || source.trim().is_empty() {
        return Err("Invalid source or translation direction".to_owned());
    }
    if raw.len() > 32_000 {
        return Err("AI analysis exceeds the accepted response size".to_owned());
    }
    let cleaned = raw.trim();

    let unwrapped = cleaned
        .strip_prefix("```json")
        .or_else(|| cleaned.strip_prefix("```"))
        .unwrap_or(cleaned);
    let unwrapped = unwrapped.strip_suffix("```").unwrap_or(unwrapped).trim();
    let candidate = if unwrapped.starts_with('{') && unwrapped.ends_with('}') {
        unwrapped
    } else {
        let start = unwrapped
            .find('{')
            .ok_or("AI response contained no JSON object")?;
        let end = unwrapped
            .rfind('}')
            .ok_or("AI response contained no complete JSON object")?;
        &unwrapped[start..=end]
    };
    let analysis: SentenceAnalysis = serde_json::from_str(candidate)
        .map_err(|error| format!("AI structured analysis JSON was invalid: {error}"))?;
    if !meaningful(&analysis.translation, 4000)
        || !meaningful(&analysis.rationale, 600)
        || !matches!(
            analysis.difficulty.as_str(),
            "beginner" | "elementary" | "intermediate" | "advanced"
        )
        || analysis
            .estimated_hsk
            .is_some_and(|level| !(1..=9).contains(&level))
        || analysis.vocabulary.len() > 24
        || analysis.grammar.len() > 8
        || analysis.notes.len() > 8
    {
        return Err("AI analysis contained invalid or oversized fields".to_owned());
    }
    if analysis.alternatives.len() > 3
        || analysis.features.len() > 8
        || analysis.nuance.len() > 6
        || analysis.alternatives.iter().any(|item| {
            !meaningful(item, 4000)
                || item.trim() == analysis.translation.trim()
                || (direction == "en-zh" && !chinese_text(item))
        })
        || analysis.features.iter().any(|item| !meaningful(item, 80))
        || analysis.nuance.iter().any(|item| !meaningful(item, 400))
    {
        return Err("AI returned invalid alternatives or sentence insights".to_owned());
    }
    if direction == "en-zh" && !chinese_text(&analysis.translation) {
        return Err("AI translation did not contain Chinese text".to_owned());
    }
    let chinese_sentence = if direction == "zh-en" {
        source
    } else {
        &analysis.translation
    };
    let mut words = BTreeSet::new();
    for word in &analysis.vocabulary {
        if !meaningful(&word.hanzi, 20)
            || !chinese_text(&word.hanzi)
            || !chinese_sentence.contains(word.hanzi.as_str())
            || !meaningful(&word.meaning, 200)
            || !meaningful(&word.pinyin, 120)
            || !words.insert(&word.hanzi)
        {
            return Err("AI vocabulary did not match the Chinese sentence".to_owned());
        }
    }
    for pattern in &analysis.grammar {
        if !meaningful(&pattern.pattern, 120)
            || !meaningful(&pattern.explanation, 500)
            || !meaningful(&pattern.example_fragment, 160)
            || !chinese_sentence.contains(pattern.example_fragment.as_str())
        {
            return Err("AI grammar evidence did not match the Chinese sentence".to_owned());
        }
    }
    if analysis.notes.iter().any(|note| !meaningful(note, 400)) {
        return Err("AI returned an invalid learning note".to_owned());
    }
    Ok(analysis)
}

pub fn repair_messages(
    mut original: Vec<serde_json::Value>,
    invalid_response: &str,
    validation_error: &str,
) -> Vec<serde_json::Value> {
    let excerpt: String = invalid_response.chars().take(6000).collect();
    original.push(serde_json::json!({
        "role": "assistant",
        "content": excerpt,
    }));
    original.push(serde_json::json!({
        "role": "user",
        "content": format!(
            "Your previous response failed the required structured-analysis contract: {validation_error}. \
             Return one complete corrected JSON object matching the required schema, with EVERY required key. \
             Do not invent vocabulary, grammatical evidence, pinyin, or linguistic features. \
             All Hanzi and example_fragment values must occur verbatim in the Chinese sentence. \
             Use empty arrays if you cannot reliably provide an observation. No markdown or explanation."
        ),
    }));
    original
}

pub fn messages(source: &str, direction: &str) -> Vec<serde_json::Value> {
    let target = match direction {
        "zh-en" => "idiomatic English",
        "en-zh" => "idiomatic Simplified Chinese",
        _ => "the target language",
    };
    vec![
        serde_json::json!({"role": "system", "content": prompt::system_prompt(target, direction)}),
        serde_json::json!({"role": "user", "content": source.trim()}),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> String {
        serde_json::json!({"translation":"I am going to the library tomorrow.","alternatives":[],"features":[],"nuance":[],"difficulty":"elementary","estimated_hsk":2,"rationale":"Common time expression and a purpose construction.","vocabulary":[{"hanzi":"明天","meaning":"tomorrow","pinyin":"ming2 tian1"}],"grammar":[{"pattern":"要 + verb","explanation":"Plans or intention","example_fragment":"要去"}],"notes":["The time word precedes the verb."]}).to_string()
    }

    #[test]
    fn accepts_bounded_analysis() {
        let value = parse(&fixture(), "我明天要去图书馆。", "zh-en").unwrap();
        assert_eq!(value.estimated_hsk, Some(2));
        assert_eq!(value.grammar.len(), 1);
    }

    #[test]
    fn rejects_unrelated_vocabulary() {
        let invalid = fixture().replace("明天", "苹果");
        assert!(parse(&invalid, "我明天要去图书馆。", "zh-en").is_err());
    }

    #[test]
    fn rejects_non_matching_grammar_evidence() {
        let invalid = fixture().replace("要去", "把字句");
        assert!(parse(&invalid, "我明天要去图书馆。", "zh-en").is_err());
    }

    #[test]
    fn rejects_out_of_range_hsk() {
        let invalid = fixture().replace("\"estimated_hsk\":2", "\"estimated_hsk\":10");
        assert!(parse(&invalid, "我明天要去图书馆。", "zh-en").is_err());
    }

    #[test]
    fn accepts_english_to_chinese_with_chinese_output() {
        let sample = serde_json::json!({"translation":"我明天去图书馆。","alternatives":[],"features":[],"nuance":[],"difficulty":"beginner","estimated_hsk":null,"rationale":"Short common sentence.","vocabulary":[],"grammar":[],"notes":[]}).to_string();
        assert!(parse(&sample, "I will go to the library tomorrow.", "en-zh").is_ok());
    }

    #[test]
    fn accepts_optional_source_insights_and_alternatives() {
        let mut value: serde_json::Value = serde_json::from_str(&fixture()).unwrap();
        value["alternatives"] = serde_json::json!(["Tomorrow I plan to visit the library."]);
        value["features"] = serde_json::json!(["Intention", "Time expression"]);
        value["nuance"] = serde_json::json!(["The original indicates a planned action."]);
        let parsed = parse(&value.to_string(), "我明天要去图书馆。", "zh-en").unwrap();
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.features.len(), 2);
    }

    #[test]
    fn rejects_non_chinese_alternatives_for_english_source() {
        let raw = serde_json::json!({
            "translation": "明天去图书馆。", "alternatives": ["Tomorrow at the library"], "features":[], "nuance":[],
            "difficulty": "beginner", "estimated_hsk": null,
            "rationale": "Simple", "vocabulary": [], "grammar": [], "notes": []
        }).to_string();
        assert!(parse(&raw, "I will visit the library tomorrow.", "en-zh").is_err());
    }

    #[test]
    fn extracts_json_after_local_model_preamble() {
        let text = format!(
            "<think>Reasoning withheld</think>\n```json\n{}\n```",
            fixture()
        );
        assert!(parse(&text, "我明天要去图书馆。", "zh-en").is_ok());
    }

    #[test]
    fn repair_messages_include_validation_error_and_bound_reply() {
        let result = repair_messages(
            messages("你好", "zh-en"),
            &"x".repeat(10000),
            "missing grammar evidence",
        );
        assert_eq!(result.len(), 4);
        assert!(
            result[3]["content"]
                .as_str()
                .unwrap()
                .contains("missing grammar evidence")
        );
        assert_eq!(result[2]["content"].as_str().unwrap().len(), 6000);
    }

    #[test]
    fn rejects_unknown_fields() {
        let invalid = fixture().replace("\"notes\":", "\"secret\":true,\"notes\":");
        assert!(parse(&invalid, "我明天要去图书馆。", "zh-en").is_err());
    }
}
