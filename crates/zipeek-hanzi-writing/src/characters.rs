pub fn hanzi_characters(text: &str) -> Vec<char> {
    text.chars()
        .filter(|character| {
            matches!(*character as u32,
                0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF |
                0x20000..=0x2A6DF | 0x2A700..=0x2B73F | 0x2B740..=0x2B81F |
                0x2B820..=0x2CEAF | 0x2CEB0..=0x2EBEF | 0x30000..=0x323AF
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_order_and_repeated_hanzi() {
        assert_eq!(hanzi_characters("学习学"), vec!['学', '习', '学']);
    }

    #[test]
    fn excludes_ascii_digits_and_punctuation() {
        assert_eq!(hanzi_characters("A中2-国!"), vec!['中', '国']);
        assert!(hanzi_characters("English only").is_empty());
    }

    #[test]
    fn supports_extension_a_and_supplementary_characters() {
        assert_eq!(hanzi_characters("㐀𠀀"), vec!['㐀', '𠀀']);
    }
}
