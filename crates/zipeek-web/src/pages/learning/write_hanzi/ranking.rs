use zipeek_core::Word;

pub(super) fn fold_pinyin(value: &str) -> String {
    value
        .chars()
        .filter_map(|character| {
            let folded = match character.to_lowercase().next().unwrap_or(character) {
                'ā' | 'á' | 'ǎ' | 'à' => 'a',
                'ē' | 'é' | 'ě' | 'è' => 'e',
                'ī' | 'í' | 'ǐ' | 'ì' => 'i',
                'ō' | 'ó' | 'ǒ' | 'ò' => 'o',
                'ū' | 'ú' | 'ǔ' | 'ù' => 'u',
                'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => 'v',
                '0'..='5' | ' ' | '\t' | '\n' | '\'' | '-' => return None,
                other => other,
            };
            Some(folded)
        })
        .collect::<String>()
        .to_lowercase()
}

fn meaning_score(meaning: &str, query: &str) -> i32 {
    let meaning = meaning.to_lowercase();
    let query = query.to_lowercase();
    if meaning.split('|').any(|sense| sense.trim() == query) {
        900
    } else if meaning
        .split('|')
        .any(|sense| sense.trim().starts_with(&query))
    {
        650
    } else if meaning
        .split(|c: char| !c.is_alphanumeric())
        .any(|token| token == query)
    {
        480
    } else if meaning.contains(&query) {
        200
    } else {
        0
    }
}

pub(super) fn score(word: &Word, query: &str) -> i32 {
    let query = query.trim();
    if query.is_empty() {
        return 1;
    }
    let folded = fold_pinyin(query);
    let reading = fold_pinyin(&word.pinyin);
    let simplified = word.simplified.as_str();
    let traditional = word.traditional.as_deref().unwrap_or("");
    if query == simplified || query == traditional {
        1200
    } else if simplified.starts_with(query) || traditional.starts_with(query) {
        1000
    } else if simplified.contains(query) || traditional.contains(query) {
        750
    } else if !folded.is_empty() && folded == reading {
        950
    } else if !folded.is_empty() && reading.starts_with(&folded) {
        700
    } else if !folded.is_empty() && reading.contains(&folded) {
        350
    } else {
        word.english
            .as_deref()
            .map(|meaning| meaning_score(meaning, query))
            .unwrap_or(0)
    }
}

pub(super) fn rank(words: &mut [Word], query: &str) {
    words.sort_by(|left, right| {
        score(right, query)
            .cmp(&score(left, query))
            .then_with(|| {
                left.simplified
                    .chars()
                    .count()
                    .cmp(&right.simplified.chars().count())
            })
            .then_with(|| left.id.as_i32().cmp(&right.id.as_i32()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tones_spaces_and_numbers_fold_to_same_reading() {
        assert_eq!(fold_pinyin("jīng wěi xiàn"), "jingweixian");
        assert_eq!(fold_pinyin("jing1 wei3 xian4"), "jingweixian");
        assert_eq!(fold_pinyin("jing wei xian"), "jingweixian");
        assert_eq!(fold_pinyin("lǜ"), "lv");
    }

    #[test]
    fn meaning_ranking_prioritizes_complete_senses() {
        assert!(meaning_score("water | river", "water") > meaning_score("underwater", "water"));
        assert!(
            meaning_score("to make a sound", "make") > meaning_score("a handmade thing", "make")
        );
    }
}
