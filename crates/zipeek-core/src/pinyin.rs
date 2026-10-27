mod syllables;

#[derive(Clone, Copy)]
struct Letter {
    base: char,
    tone: Option<char>,
}

pub fn normalize_for_comparison(value: &str) -> String {
    let value = value.to_lowercase().replace("u:", "v");
    let mut result = String::new();
    let mut letters: Vec<Letter> = Vec::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if let Some(tone) = combining_tone(character) {
            if let Some(letter) = letters.last_mut() {
                letter.tone = Some(tone);
            } else {
                result.push(character);
            }
        } else if character == '\u{308}' && letters.last().is_some_and(|letter| letter.base == 'u')
        {
            if let Some(letter) = letters.last_mut() {
                letter.base = 'v';
            }
        } else if character.is_ascii_digit() {
            append_letters(&mut result, &letters, Some(character));
            letters.clear();
        } else if character.is_whitespace()
            && characters
                .clone()
                .find(|next| !next.is_whitespace())
                .is_some_and(|next| next.is_ascii_digit())
        {
            continue;
        } else if character.is_whitespace() || matches!(character, '\'' | '’' | '-') {
            append_letters(&mut result, &letters, None);
            letters.clear();
        } else if character.is_alphabetic() {
            letters.push(decode_letter(character));
        } else {
            append_letters(&mut result, &letters, None);
            letters.clear();
            result.push(character);
        }
    }
    append_letters(&mut result, &letters, None);
    result
}

fn append_letters(result: &mut String, letters: &[Letter], explicit_tone: Option<char>) {
    if letters.is_empty() {
        if let Some(tone) = explicit_tone {
            result.push(tone);
        }
        return;
    }
    if let Some(tone) = explicit_tone {
        result.extend(letters.iter().map(|letter| letter.base));
        if tone != '5'
            || !letters
                .iter()
                .any(|letter| matches!(letter.base, 'a' | 'e' | 'i' | 'o' | 'u' | 'v'))
        {
            result.push(tone);
        }
    } else if letters.iter().any(|letter| letter.tone.is_some()) {
        let mut suffixes: Vec<Option<String>> = vec![None; letters.len() + 1];
        suffixes[letters.len()] = Some(String::new());
        for start in (0..letters.len()).rev() {
            for end in ((start + 1)..=(start + 7).min(letters.len())).rev() {
                let segment = &letters[start..end];
                let base: String = segment.iter().map(|letter| letter.base).collect();
                if !syllables::is_syllable(&base)
                    || segment
                        .iter()
                        .filter(|letter| letter.tone.is_some())
                        .count()
                        > 1
                {
                    continue;
                }
                if let Some(suffix) = &suffixes[end] {
                    let mut normalized = base;
                    normalized.extend(segment.iter().filter_map(|letter| letter.tone));
                    normalized.push_str(suffix);
                    suffixes[start] = Some(normalized);
                    break;
                }
            }
        }
        if let Some(normalized) = &suffixes[0] {
            result.push_str(normalized);
        } else {
            for letter in letters {
                result.push(letter.base);
                result.extend(letter.tone);
            }
        }
    } else {
        result.extend(letters.iter().map(|letter| letter.base));
    }
}

fn combining_tone(character: char) -> Option<char> {
    match character {
        '\u{304}' => Some('1'),
        '\u{301}' => Some('2'),
        '\u{30c}' => Some('3'),
        '\u{300}' => Some('4'),
        _ => None,
    }
}

fn decode_letter(character: char) -> Letter {
    let (base, tone) = match character {
        'ā' => ('a', Some('1')),
        'á' => ('a', Some('2')),
        'ǎ' => ('a', Some('3')),
        'à' => ('a', Some('4')),
        'ē' => ('e', Some('1')),
        'é' => ('e', Some('2')),
        'ě' => ('e', Some('3')),
        'è' => ('e', Some('4')),
        'ī' => ('i', Some('1')),
        'í' => ('i', Some('2')),
        'ǐ' => ('i', Some('3')),
        'ì' => ('i', Some('4')),
        'ō' => ('o', Some('1')),
        'ó' => ('o', Some('2')),
        'ǒ' => ('o', Some('3')),
        'ò' => ('o', Some('4')),
        'ū' => ('u', Some('1')),
        'ú' => ('u', Some('2')),
        'ǔ' => ('u', Some('3')),
        'ù' => ('u', Some('4')),
        'ǖ' => ('v', Some('1')),
        'ǘ' => ('v', Some('2')),
        'ǚ' => ('v', Some('3')),
        'ǜ' => ('v', Some('4')),
        'ń' => ('n', Some('2')),
        'ň' => ('n', Some('3')),
        'ǹ' => ('n', Some('4')),
        'ḿ' => ('m', Some('2')),
        'ü' => ('v', None),
        other => (other, None),
    };
    Letter { base, tone }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_fives_are_removed_per_vowel_syllable() {
        for (input, expected) in [
            ("ma5ma5", "mama"),
            ("mai5", "mai"),
            ("r5", "r5"),
            ("ng5", "ng5"),
            ("chi1fan4", "chi1fan4"),
            ("ma5ma", "mama"),
            ("ma5r5", "mar5"),
            ("mai 5", "mai"),
        ] {
            assert_eq!(normalize_for_comparison(input), expected, "{input}");
        }
    }

    #[test]
    fn tone_marks_umlauts_and_case_match_numeric_readings() {
        for (input, expected) in [
            ("NǙ ÉR", "nv3er2"),
            ("nu:3 er2", "nv3er2"),
            ("nü3er2", "nv3er2"),
            ("chī fàn", "chi1fan4"),
            ("chīfàn", "chi1fan4"),
            ("māma", "ma1ma"),
            ("xiān", "xian1"),
            ("xī'ān", "xi1an1"),
            ("shāngǔ", "shan1gu3"),
            ("bàn lu:3", "ban4lv3"),
            ("nu\u{308}\u{30c} er2", "nv3er2"),
            ("lǜ4", "lv4"),
        ] {
            assert_eq!(normalize_for_comparison(input), expected, "{input}");
        }
    }

    #[test]
    fn empty_and_non_pinyin_labels_remain_distinct() {
        assert_eq!(normalize_for_comparison(" \t\n"), "");
        assert_eq!(normalize_for_comparison("食"), "食");
        assert_eq!(normalize_for_comparison("x!"), "x!");
        assert_eq!(normalize_for_comparison("5"), "5");
    }
}
