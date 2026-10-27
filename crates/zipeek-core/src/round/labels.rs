use crate::{Word, normalize_for_comparison, practice::ExerciseVariant};

pub(crate) fn labels(word: &Word, variant: ExerciseVariant) -> (&str, &str) {
    let english = word
        .english
        .as_deref()
        .and_then(|value| value.split('|').next())
        .unwrap_or("");
    match variant {
        ExerciseVariant::EngPinyin => (english, &word.pinyin),
        ExerciseVariant::PinyinHanzi => (&word.pinyin, &word.simplified),
        ExerciseVariant::HanziEng | ExerciseVariant::HanziToMeaning => (&word.simplified, english),
        ExerciseVariant::MeaningToHanzi => (english, &word.simplified),
        ExerciseVariant::MeaningToPinyin => (english, &word.pinyin),
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
        ExerciseVariant::EngPinyin | ExerciseVariant::MeaningToPinyin => {
            normalize_for_comparison(right)
        }
        ExerciseVariant::PinyinHanzi | ExerciseVariant::MeaningToHanzi => right.to_owned(),
        ExerciseVariant::HanziEng | ExerciseVariant::HanziToMeaning => right
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase(),
    }
}
