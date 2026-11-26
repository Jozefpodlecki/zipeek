use std::{collections::HashMap, fs, path::Path};

pub const ASR_PREFIX: [u32; 9] = [151644, 8948, 198, 151645, 198, 151644, 872, 198, 151669];
pub const ASR_SUFFIX: [u32; 6] = [151670, 151645, 198, 151644, 77091, 198];
pub const AUDIO_PAD: u32 = 151676;
pub const ASR_TEXT: u32 = 151704;
pub const END_OF_TEXT: u32 = 151643;
pub const END_OF_MESSAGE: u32 = 151645;

pub fn asr_prompt(audio_tokens: usize) -> Result<Vec<u32>, String> {
    if audio_tokens == 0 || audio_tokens > 500 {
        return Err("invalid number of Qwen3-ASR audio tokens".into());
    }
    let mut prompt = Vec::with_capacity(ASR_PREFIX.len() + audio_tokens + ASR_SUFFIX.len());
    prompt.extend(ASR_PREFIX);
    prompt.extend(std::iter::repeat_n(AUDIO_PAD, audio_tokens));
    prompt.extend(ASR_SUFFIX);
    Ok(prompt)
}

pub struct QwenVocabulary {
    entries: HashMap<u32, String>,
}

impl QwenVocabulary {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Qwen vocabulary: {e}"))?;
        let mut parser = JsonVocabularyParser::new(&content);
        let entries = parser.parse()?;
        if entries.len() < 150_000 {
            return Err("Qwen vocabulary is incomplete".into());
        }
        Ok(Self { entries })
    }

    pub fn decode_asr(&self, tokens: &[u32]) -> Result<String, String> {
        let start = tokens
            .iter()
            .position(|id| *id == ASR_TEXT)
            .ok_or_else(|| "Qwen ASR response omitted <asr_text>".to_owned())?;
        let mut bytes = Vec::new();
        let reverse = byte_decoder();
        for id in tokens.iter().skip(start + 1) {
            if *id == END_OF_MESSAGE || *id == END_OF_TEXT {
                break;
            }
            if *id >= 151_643 {
                return Err(format!("unexpected special token in ASR transcript: {id}"));
            }
            let token = self
                .entries
                .get(id)
                .ok_or_else(|| format!("unknown Qwen vocabulary token {id}"))?;
            for codepoint in token.chars() {
                bytes.push(
                    *reverse
                        .get(&codepoint)
                        .ok_or("invalid Qwen byte vocabulary character")?,
                );
            }
        }
        let text = String::from_utf8(bytes).map_err(|_| "invalid decoded ASR UTF-8")?;
        if text.trim().is_empty() {
            return Err("ASR decoder produced empty transcription".into());
        }
        Ok(text.trim().to_owned())
    }
}

fn byte_decoder() -> HashMap<char, u8> {
    let mut result = HashMap::with_capacity(256);
    let mut next = 256u32;
    for byte in 0..=255u8 {
        let visible = (33..=126).contains(&byte)
            || (161..=172).contains(&byte)
            || (174..=255).contains(&byte);
        let value = if visible {
            byte as u32
        } else {
            let value = next;
            next += 1;
            value
        };
        result.insert(char::from_u32(value).expect("valid byte alphabet"), byte);
    }
    result
}

struct JsonVocabularyParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> JsonVocabularyParser<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            data: content.as_bytes(),
            position: 0,
        }
    }
    fn whitespace(&mut self) {
        while self.position < self.data.len() && self.data[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
    }
    fn consume(&mut self, expected: u8) -> Result<(), String> {
        self.whitespace();
        if self.data.get(self.position) != Some(&expected) {
            return Err(format!(
                "invalid Qwen vocabulary JSON at byte {}",
                self.position
            ));
        }
        self.position += 1;
        Ok(())
    }
    fn unicode_escape(&mut self) -> Result<u32, String> {
        let end = self
            .position
            .checked_add(4)
            .ok_or("invalid Unicode escape")?;
        let digits = self
            .data
            .get(self.position..end)
            .ok_or("short Unicode escape")?;
        let digits = std::str::from_utf8(digits).map_err(|_| "invalid Unicode escape")?;
        let code = u32::from_str_radix(digits, 16).map_err(|_| "invalid Unicode escape")?;
        self.position = end;
        Ok(code)
    }
    fn json_string(&mut self) -> Result<String, String> {
        self.consume(b'"')?;
        let mut output = String::new();
        let mut raw_start = self.position;
        while self.position < self.data.len() {
            match self.data[self.position] {
                b'"' => {
                    output.push_str(
                        std::str::from_utf8(&self.data[raw_start..self.position])
                            .map_err(|_| "invalid vocabulary UTF-8")?,
                    );
                    self.position += 1;
                    return Ok(output);
                }
                b'\\' => {
                    output.push_str(
                        std::str::from_utf8(&self.data[raw_start..self.position])
                            .map_err(|_| "invalid vocabulary UTF-8")?,
                    );
                    self.position += 1;
                    let escaped = *self.data.get(self.position).ok_or("invalid JSON escape")?;
                    self.position += 1;
                    let character = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{0008}',
                        b'f' => '\u{000c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let first = self.unicode_escape()?;
                            let code = if (0xD800..=0xDBFF).contains(&first) {
                                self.consume(b'\\')?;
                                self.consume(b'u')?;
                                let second = self.unicode_escape()?;
                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err("invalid Unicode surrogate pair".into());
                                }
                                0x10000 + (first - 0xD800) * 1024 + second - 0xDC00
                            } else {
                                first
                            };
                            char::from_u32(code).ok_or("invalid Unicode scalar")?
                        }
                        _ => return Err("invalid JSON escape".into()),
                    };
                    output.push(character);
                    raw_start = self.position;
                }
                byte if byte < 0x20 => return Err("invalid control byte in vocabulary".into()),
                _ => self.position += 1,
            }
        }
        Err("unterminated JSON vocabulary string".into())
    }
    fn parse(&mut self) -> Result<HashMap<u32, String>, String> {
        self.consume(b'{')?;
        let mut entries = HashMap::new();
        loop {
            self.whitespace();
            if self.data.get(self.position) == Some(&b'}') {
                self.position += 1;
                break;
            }
            let token = self.json_string()?;
            self.consume(b':')?;
            self.whitespace();
            let start = self.position;
            while self.data.get(self.position).is_some_and(u8::is_ascii_digit) {
                self.position += 1;
            }
            let id = std::str::from_utf8(&self.data[start..self.position])
                .map_err(|_| "invalid token ID")?
                .parse::<u32>()
                .map_err(|_| "invalid token ID")?;
            if entries.insert(id, token).is_some() {
                return Err("duplicate vocabulary token ID".into());
            }
            self.whitespace();
            match self.data.get(self.position) {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    break;
                }
                _ => return Err("invalid vocabulary separator".into()),
            }
        }
        self.whitespace();
        if self.position != self.data.len() {
            return Err("trailing vocabulary data".into());
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prompt_audio_slots_are_contiguous() {
        let ids = asr_prompt(3).unwrap();
        assert_eq!(&ids[9..12], &[AUDIO_PAD; 3]);
        assert_eq!(&ids[12..], &ASR_SUFFIX);
        assert!(asr_prompt(0).is_err());
    }
    #[test]
    fn json_vocabulary_parser_handles_escapes_and_unicode() {
        let mut parser = JsonVocabularyParser::new("{\"Hello\":1,\"\\u4f60\\u597d\":2,\"\\\\\":3}");
        let entries = parser.parse().unwrap();
        assert_eq!(entries.get(&2).unwrap(), "你好");
        assert_eq!(entries.get(&3).unwrap(), "\\");
    }
    #[test]
    fn byte_alphabet_is_bijective() {
        let reverse = byte_decoder();
        assert_eq!(reverse.len(), 256);
        assert_eq!(reverse[&'!'], b'!');
    }
}
