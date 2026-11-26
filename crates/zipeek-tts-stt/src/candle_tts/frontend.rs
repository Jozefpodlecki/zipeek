#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextError {
    Empty,
    TooLong,
    InvalidCharacter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisSegment {
    pub text: String,
}

pub fn segment_mandarin(
    text: &str,
    max_characters: usize,
) -> Result<Vec<SynthesisSegment>, TextError> {
    if max_characters == 0 || text.trim().is_empty() {
        return Err(TextError::Empty);
    }
    if text.len() > 4096 {
        return Err(TextError::TooLong);
    }
    if text.chars().any(char::is_control) {
        return Err(TextError::InvalidCharacter);
    }
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut count = 0;
    for character in text.chars() {
        if character.is_whitespace() {
            if !current.ends_with(' ') && !current.is_empty() {
                current.push(' ');
            }
            continue;
        }
        if count == max_characters {
            let chunk = current.trim();
            if !chunk.is_empty() {
                segments.push(SynthesisSegment {
                    text: chunk.to_owned(),
                });
            }
            current.clear();
            count = 0;
        }
        current.push(character);
        count += 1;
        if matches!(character, '。' | '！' | '？' | '!' | '?' | '；' | ';') {
            segments.push(SynthesisSegment {
                text: current.trim().to_owned(),
            });
            current.clear();
            count = 0;
        }
    }
    if !current.trim().is_empty() {
        segments.push(SynthesisSegment {
            text: current.trim().to_owned(),
        });
    }
    if segments.is_empty() {
        return Err(TextError::Empty);
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_chinese_at_sentence_boundaries() {
        let parts = segment_mandarin("你好。今天天气很好！", 64).unwrap();
        assert_eq!(
            parts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            vec!["你好。", "今天天气很好！"]
        );
    }

    #[test]
    fn splits_long_unpunctuated_input() {
        let parts = segment_mandarin("你好你好你好", 2).unwrap();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.text.chars().count() <= 2));
    }

    #[test]
    fn rejects_bad_text() {
        assert_eq!(segment_mandarin("", 10), Err(TextError::Empty));
        assert_eq!(
            segment_mandarin("你好\n世界", 10),
            Err(TextError::InvalidCharacter)
        );
    }
}
