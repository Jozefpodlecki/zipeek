use serde::Deserialize;

#[derive(Deserialize)]
struct RecognitionResponse {
    candidates: Vec<String>,
}

fn recognition_response_kind(content: &str) -> &'static str {
    match content.trim_start().chars().next() {
        Some('{') => "json-object",
        Some('[') => "json-array",
        Some('`') => "markdown-fence",
        Some(_) => "plain-text",
        None => "empty",
    }
}

pub(super) fn parse_candidates(raw: &str) -> Result<Vec<String>, String> {
    if raw.len() > 8_192 {
        log::warn!("Hanzi recognition response exceeded size limit: bytes={}", raw.len());
        return Err("The model response exceeded the recognition limit (8 KiB).".to_owned());
    }
    let trimmed = raw.trim();
    let content = if let Some(fenced) = trimmed.strip_prefix("```") {
        let after_language = fenced.strip_prefix("json").unwrap_or(fenced).trim_start();
        after_language.strip_suffix("```").map(str::trim).unwrap_or(trimmed)
    } else { trimmed };
    let response: RecognitionResponse = match serde_json::from_str(content) {
        Ok(response) => response,
        Err(primary_error) => {
            let embedded = content
                .find('{')
                .zip(content.rfind('}'))
                .filter(|(start, end)| end > start)
                .and_then(|(start, end)| serde_json::from_str(&content[start..=end]).ok());
            if let Some(response) = embedded {
                response
            } else if !content.is_empty()
                && content.chars().count() <= 12
                && content.chars().all(is_hanzi)
            {
                RecognitionResponse {
                    candidates: vec![content.to_owned()],
                }
            } else {
                let kind = recognition_response_kind(content);
                log::warn!(
                    "Hanzi recognition response rejected: kind={kind} bytes={} json_line={} json_column={} json_category={:?}",
                    raw.len(),
                    primary_error.line(),
                    primary_error.column(),
                    primary_error.classify(),
                );
                return Err(format!(
                    "The AI model returned an invalid candidate format ({kind}, line {}, column {}). The model must return JSON such as {{\"candidates\":[\"汉\"]}}. Check that it supports image input.",
                    primary_error.line(),
                    primary_error.column(),
                ));
            }
        }
    };
    let mut candidates = Vec::new();
    for candidate in response.candidates {
        let value = candidate.trim();
        if value.is_empty() || value.chars().count() > 12 || !value.chars().all(is_hanzi) {
            log::warn!(
                "Hanzi recognition candidate rejected: character_count={} is_hanzi_only={}",
                value.chars().count(),
                value.chars().all(is_hanzi),
            );
            return Err("The model suggested a candidate that was not exclusively Chinese characters (maximum 12).".to_owned());
        }
        if !candidates.iter().any(|item: &String| item == value) {
            candidates.push(value.to_owned());
        }
        if candidates.len() == 5 {
            break;
        }
    }
    if candidates.is_empty() {
        log::warn!("Hanzi recognition response included no candidates");
        return Err("No Chinese characters were recognized".to_owned());
    }
    Ok(candidates)
}

fn is_hanzi(character: char) -> bool {
    matches!(character as u32, 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0x20000..=0x2fa1f)
}

#[cfg(test)]
mod tests {
    use super::parse_candidates;

    #[test]
    fn recognition_candidates_are_bounded_and_deduplicated() {
        assert_eq!(parse_candidates(r#"{"candidates":["汉", "汉", "漢字"]}"#).unwrap(), vec!["汉", "漢字"]);
        assert!(parse_candidates(r#"{"candidates":["Hello"]}"#).is_err());
        assert!(parse_candidates(r#"{"candidates":["汉 English"]}"#).is_err());
        assert!(parse_candidates(r#"{"candidates":[]}"#).is_err());
        assert!(parse_candidates("```json\n{}\n```").is_err());
        assert_eq!(parse_candidates("```json\n{\"candidates\":[\"十\"]}\n```").unwrap(), vec!["十"]);
        assert_eq!(parse_candidates("十").unwrap(), vec!["十"]);
        assert_eq!(parse_candidates("Result: {\"candidates\":[\"汉\"]}").unwrap(), vec!["汉"]);
        assert!(parse_candidates("Guess: ten 十").is_err());
        let error = parse_candidates("{broken json}").unwrap_err();
        assert!(error.contains("line 1"));
        assert!(error.contains("json-object"));
        assert!(parse_candidates("[]").unwrap_err().contains("json-array"));
        assert!(parse_candidates(&"x".repeat(9_000)).is_err());
    }
}
