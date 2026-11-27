use std::{collections::HashMap, fs, path::Path};

pub const ASSISTANT_START: [u32; 3] = [151644, 77091, 198];
pub const ASSISTANT_END: [u32; 2] = [151645, 198];

pub struct MandarinBpeTokenizer {
    vocabulary: HashMap<String, u32>,
    merge_order: HashMap<(String, String), usize>,
    byte_alphabet: [char; 256],
}

fn byte_alphabet() -> [char; 256] {
    let mut next = 256u32;
    std::array::from_fn(|index| {
        let byte = index as u8;
        let visible = (33..=126).contains(&byte)
            || (161..=172).contains(&byte)
            || (174..=255).contains(&byte);
        let value = if visible {
            byte as u32
        } else {
            let result = next;
            next += 1;
            result
        };
        char::from_u32(value).expect("Qwen byte alphabet is valid Unicode")
    })
}

fn pretokenize(text: &str) -> Vec<&str> {
    let chars = text.char_indices().collect::<Vec<_>>();
    let mut pieces = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let start = index;
        let current = chars[index].1;
        if current.is_whitespace() {
            while index < chars.len() && chars[index].1.is_whitespace() {
                index += 1;
            }
            if index - start == 1 && current == ' ' && index < chars.len() {
                let next = chars[index].1;
                if next.is_alphabetic() {
                    while index < chars.len() && chars[index].1.is_alphabetic() {
                        index += 1;
                    }
                } else if !next.is_numeric() && !next.is_whitespace() {
                    while index < chars.len()
                        && !chars[index].1.is_alphanumeric()
                        && !chars[index].1.is_whitespace()
                    {
                        index += 1;
                    }
                }
            }
        } else if current.is_alphabetic() {
            while index < chars.len() && chars[index].1.is_alphabetic() {
                index += 1;
            }
        } else if current.is_numeric() {
            while index < chars.len() && index - start < 3 && chars[index].1.is_numeric() {
                index += 1;
            }
        } else if index + 1 < chars.len() && chars[index + 1].1.is_alphabetic() {
            index += 1;
            while index < chars.len() && chars[index].1.is_alphabetic() {
                index += 1;
            }
        } else {
            while index < chars.len()
                && !chars[index].1.is_alphanumeric()
                && !chars[index].1.is_whitespace()
            {
                if index + 1 < chars.len() && chars[index + 1].1.is_alphabetic() {
                    break;
                }
                index += 1;
            }
            if index == start {
                index += 1;
            }
        }
        let end = chars.get(index).map_or(text.len(), |(offset, _)| *offset);
        pieces.push(&text[chars[start].0..end]);
    }
    pieces
}

impl MandarinBpeTokenizer {
    pub fn load(root: &Path) -> Result<Self, String> {
        let source = fs::read_to_string(root.join("vocab.json"))
            .map_err(|error| format!("Qwen3-TTS vocabulary: {error}"))?;
        let vocabulary = serde_json::from_str::<HashMap<String, u32>>(&source)
            .map_err(|error| format!("invalid Qwen3-TTS vocabulary JSON: {error}"))?;
        if vocabulary.len() < 150_000 {
            return Err("Qwen3-TTS vocabulary is incomplete".into());
        }
        let source = fs::read_to_string(root.join("merges.txt"))
            .map_err(|error| format!("Qwen3-TTS merge list: {error}"))?;
        let mut merge_order = HashMap::new();
        for line in source.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut tokens = line.split_whitespace();
            let (Some(left), Some(right), None) = (tokens.next(), tokens.next(), tokens.next())
            else {
                return Err("invalid Qwen3-TTS BPE merge rule".into());
            };
            let ordinal = merge_order.len();
            if merge_order
                .insert((left.to_owned(), right.to_owned()), ordinal)
                .is_some()
            {
                return Err("duplicate Qwen3-TTS BPE merge rule".into());
            }
        }
        if merge_order.len() < 150_000 {
            return Err("Qwen3-TTS BPE merge list is incomplete".into());
        }
        Ok(Self {
            vocabulary,
            merge_order,
            byte_alphabet: byte_alphabet(),
        })
    }

    fn encode_piece(&self, piece: &str) -> Result<Vec<u32>, String> {
        let mut fragments = piece
            .as_bytes()
            .iter()
            .map(|byte| self.byte_alphabet[*byte as usize].to_string())
            .collect::<Vec<_>>();
        while fragments.len() > 1 {
            let best = fragments
                .windows(2)
                .enumerate()
                .filter_map(|(index, pair)| {
                    self.merge_order
                        .get(&(pair[0].clone(), pair[1].clone()))
                        .map(|rank| (index, *rank))
                })
                .min_by_key(|(index, rank)| (*rank, *index));
            let Some((index, _)) = best else {
                break;
            };
            let right = fragments.remove(index + 1);
            fragments[index].push_str(&right);
        }
        fragments
            .iter()
            .map(|fragment| {
                self.vocabulary
                    .get(fragment)
                    .copied()
                    .ok_or_else(|| "Qwen3-TTS BPE token not in vocabulary".to_owned())
            })
            .collect()
    }

    pub fn encode_mandarin_run(&self, text: &str) -> Result<Vec<u32>, String> {
        if text.is_empty() || text.chars().any(char::is_control) {
            return Err("Qwen3-TTS text must be nonempty and contain no control characters".into());
        }
        let mut result = Vec::new();
        for piece in pretokenize(text) {
            result.extend(self.encode_piece(piece)?);
        }
        Ok(result)
    }

    pub fn encode_assistant_mandarin(&self, text: &str) -> Result<Vec<u32>, String> {
        let mut tokens = ASSISTANT_START.to_vec();
        tokens.extend(self.encode_mandarin_run(text)?);
        tokens.extend(ASSISTANT_END);
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_alphabet_preserves_each_byte_immutably() {
        let chars = byte_alphabet();
        let unique = chars
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), 256);
        assert_eq!(chars[b'A' as usize], 'A');
    }

    #[test]
    fn merges_a_utf8_hanzi_run_into_expected_token_and_chatml() {
        let alphabet = byte_alphabet();
        let raw = "你"
            .as_bytes()
            .iter()
            .map(|byte| alphabet[*byte as usize].to_string())
            .collect::<Vec<_>>();
        let combined = format!("{}{}", raw[0], raw[1]);
        let complete = format!("{}{}", combined, raw[2]);
        let vocabulary = HashMap::from([(complete, 42u32)]);
        let merge_order = HashMap::from([
            ((raw[0].clone(), raw[1].clone()), 0usize),
            ((combined, raw[2].clone()), 1usize),
        ]);
        let tokenizer = MandarinBpeTokenizer {
            vocabulary,
            merge_order,
            byte_alphabet: alphabet,
        };
        assert_eq!(tokenizer.encode_mandarin_run("你").unwrap(), vec![42]);
        let prompt = tokenizer.encode_assistant_mandarin("你").unwrap();
        assert_eq!(prompt, [151644, 77091, 198, 42, 151645, 198]);
    }

    #[test]
    fn punctuation_and_mixed_text_preserve_all_utf8_bytes() {
        let alphabet = byte_alphabet();
        let vocabulary = alphabet
            .iter()
            .enumerate()
            .map(|(index, ch)| (ch.to_string(), index as u32))
            .collect();
        let tokenizer = MandarinBpeTokenizer {
            vocabulary,
            merge_order: HashMap::new(),
            byte_alphabet: alphabet,
        };
        for input in [
            "你好，欢迎学习中文。",
            "你好 世界!",
            "今天是2026年11月24日。",
            "Hello, 中文 123!",
            "你好！你好吗？",
        ] {
            let tokens = tokenizer.encode_mandarin_run(input).unwrap();
            assert_eq!(
                tokens,
                input
                    .as_bytes()
                    .iter()
                    .map(|byte| *byte as u32)
                    .collect::<Vec<_>>()
            );
            assert_eq!(pretokenize(input).concat(), input);
            assert_eq!(
                tokenizer.encode_assistant_mandarin(input).unwrap().len(),
                tokens.len() + ASSISTANT_START.len() + ASSISTANT_END.len()
            );
        }
    }

    #[test]
    fn splits_digits_into_groups_of_three_and_keeps_punctuation() {
        assert_eq!(
            pretokenize("你好，欢迎学习中文。"),
            ["你好", "，欢迎学习中文", "。"]
        );
        assert_eq!(pretokenize("1234567"), ["123", "456", "7"]);
        assert_eq!(pretokenize("hi world!"), ["hi", " world", "!"]);
    }

    #[test]
    fn rejects_empty_or_control_character_text() {
        let tokenizer = MandarinBpeTokenizer {
            vocabulary: HashMap::new(),
            merge_order: HashMap::new(),
            byte_alphabet: byte_alphabet(),
        };
        assert!(tokenizer.encode_mandarin_run("").is_err());
        assert!(tokenizer.encode_mandarin_run("你好\n世界").is_err());
    }
}
