use crate::{Word, normalize_for_comparison, practice::ExerciseVariant};

pub(crate) fn labels(word: &Word, variant: ExerciseVariant) -> (&str, &str) {
    let english = word
        .english
        .as_deref()
        .and_then(|value| value.split('|').next())
        .unwrap_or("");
    match variant {
        ExerciseVariant::MatchingEnglishPinyin => (english, &word.pinyin),
        ExerciseVariant::MatchingPinyinHanzi => (&word.pinyin, &word.simplified),
        ExerciseVariant::MatchingAudioHanzi => (&word.simplified, &word.simplified),
        ExerciseVariant::MatchingHanziMeaning | ExerciseVariant::MatchingAudioMeaning | ExerciseVariant::MultipleChoiceHanziMeaning => {
            (&word.simplified, english)
        }
        ExerciseVariant::MultipleChoiceMeaningHanzi => (english, &word.simplified),
        ExerciseVariant::MultipleChoiceMeaningPinyin => (english, &word.pinyin),
    }
}

pub(crate) fn eligible(word: &Word, variant: ExerciseVariant) -> bool {
    let (left, right) = labels(word, variant);
    (word.hsk2_level.is_some() || word.hsk3_level.is_some())
        && !left.trim().is_empty()
        && !right.trim().is_empty()
}

pub(crate) fn answer_key(word: &Word, variant: ExerciseVariant) -> String {
    let (_, right) = labels(word, variant);
    match variant {
        ExerciseVariant::MatchingEnglishPinyin | ExerciseVariant::MultipleChoiceMeaningPinyin => {
            normalize_for_comparison(right)
        }
        ExerciseVariant::MatchingPinyinHanzi | ExerciseVariant::MatchingAudioHanzi | ExerciseVariant::MultipleChoiceMeaningHanzi => {
            right.to_owned()
        }
        ExerciseVariant::MatchingHanziMeaning | ExerciseVariant::MatchingAudioMeaning | ExerciseVariant::MultipleChoiceHanziMeaning => {
            right
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        }
    }
}

pub fn distinct_playable_answers(words: &[Word], variant: ExerciseVariant) -> usize {
    use std::collections::BTreeSet;
    words
        .iter()
        .filter(|word| eligible(word, variant))
        .map(|word| answer_key(word, variant))
        .collect::<BTreeSet<_>>()
        .len()
}

#[cfg(test)]
mod playable_tests {
    use super::*;
    use crate::{Hsk3Level, WordId};

    fn word(id: i64, english: &str, pinyin: &str) -> Word {
        Word {
            id: WordId::try_from(id).unwrap(),
            simplified: format!("字{id}"),
            traditional: None,
            pinyin: pinyin.to_owned(),
            pinyin_numeric: None,
            english: Some(english.to_owned()),
            part_of_speech: Vec::new(),
            hsk2_level: None,
            hsk3_level: Some(Hsk3Level::One),
            frequency: None,
        }
    }

    #[test]
    fn distinct_answers_are_not_raw_row_counts() {
        let words = vec![
            word(1, "same", "tong2"),
            word(2, "same", "tong2"),
            word(3, "different", "yi1"),
        ];
        assert_eq!(
            distinct_playable_answers(&words, ExerciseVariant::MultipleChoiceHanziMeaning),
            2
        );
        assert_eq!(
            distinct_playable_answers(&words, ExerciseVariant::MatchingPinyinHanzi),
            3
        );
        assert_eq!(
            distinct_playable_answers(&words, ExerciseVariant::MatchingEnglishPinyin),
            2
        );
    }

    #[test]
    fn empty_labels_are_ineligible() {
        let words = vec![word(1, "", "a1"), word(2, "word", "")];
        assert_eq!(
            distinct_playable_answers(&words, ExerciseVariant::MultipleChoiceHanziMeaning),
            0
        );
    }
}
