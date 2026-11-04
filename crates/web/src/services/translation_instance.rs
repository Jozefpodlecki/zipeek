use zipeek_ai::Sentence;
use zipeek_exercises::session::{ExerciseInstance, SessionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TranslationInstanceError {
    InvalidTileOrder,
    Session(SessionError),
}

use zipeek_exercises::translation::TranslationDirection;

pub(crate) fn shuffled_bank_order(
    sentence: &Sentence,
    mut sample_index: impl FnMut(usize) -> usize,
) -> Vec<usize> {
    let mut order = (0..sentence.word_bank.len()).collect::<Vec<_>>();
    for index in (1..order.len()).rev() {
        let random = sample_index(index + 1).min(index);
        order.swap(index, random);
    }
    if order
        .iter()
        .map(|index| &sentence.word_bank[*index])
        .eq(sentence.solution.iter())
    {
        if let Some(different) = (1..order.len())
            .find(|index| sentence.word_bank[order[*index]] != sentence.word_bank[order[0]])
        {
            order.swap(0, different);
        }
    }
    order
}

pub(crate) fn visible_instance(
    sentence: &Sentence,
    direction: TranslationDirection,
    order: &[usize],
    instance_id: String,
) -> Result<ExerciseInstance, TranslationInstanceError> {
    if order.len() != sentence.word_bank.len() {
        return Err(TranslationInstanceError::InvalidTileOrder);
    }
    let mut seen = vec![false; sentence.word_bank.len()];
    for &index in order {
        let Some(slot) = seen.get_mut(index) else {
            return Err(TranslationInstanceError::InvalidTileOrder);
        };
        if *slot {
            return Err(TranslationInstanceError::InvalidTileOrder);
        }
        *slot = true;
    }
    let source = match direction {
        TranslationDirection::ChineseToEnglish => &sentence.chinese,
        TranslationDirection::EnglishToChinese => &sentence.english,
    };
    let visible_tiles = order
        .iter()
        .enumerate()
        .map(|(id, index)| {
            serde_json::json!({"id": format!("T{id}"), "text": sentence.word_bank[*index]})
        })
        .collect::<Vec<_>>();
    let visible_input = serde_json::json!({
        "direction": direction.code(),
        "source": source,
        "tiles": visible_tiles,
    })
    .to_string();
    ExerciseInstance::new(instance_id, "sentence-translation", visible_input)
        .map_err(TranslationInstanceError::Session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_instance_does_not_disclose_reference_translation() {
        let sentence = Sentence {
            direction: "zh-to-en".to_owned(),
            chinese: "我坐地铁。".to_owned(),
            english: "I take the subway.".to_owned(),
            target_words: vec!["地铁".to_owned()],
            word_bank: vec!["I".to_owned(), "take".to_owned(), "the subway.".to_owned()],
            solution: vec!["I".to_owned(), "take".to_owned(), "the subway.".to_owned()],
        };
        let instance = visible_instance(
            &sentence,
            TranslationDirection::ChineseToEnglish,
            &[2, 0, 1],
            "translation-0".to_owned(),
        )
        .unwrap();
        assert!(instance.visible_input.contains("我坐地铁。"));
        assert!(!instance.visible_input.contains("I take the subway."));
        assert!(!instance.visible_input.contains("solution"));
        assert!(instance.visible_input.contains("T0"));
        assert_eq!(shuffled_bank_order(&sentence, |_| 0).len(), 3);
        assert_eq!(
            visible_instance(
                &sentence,
                TranslationDirection::ChineseToEnglish,
                &[0, 0, 2],
                "bad".to_owned(),
            ),
            Err(TranslationInstanceError::InvalidTileOrder),
        );
        assert_eq!(
            visible_instance(
                &sentence,
                TranslationDirection::ChineseToEnglish,
                &[0, 1, 9],
                "bad".to_owned(),
            ),
            Err(TranslationInstanceError::InvalidTileOrder),
        );
    }

    #[test]
    fn reverse_direction_hides_chinese_reference_sentence() {
        let sentence = Sentence {
            direction: "en-to-zh".to_owned(),
            chinese: "我坐地铁。".to_owned(),
            english: "I take the subway.".to_owned(),
            target_words: vec!["地铁".to_owned()],
            word_bank: vec!["我".to_owned(), "坐".to_owned(), "地铁".to_owned(), "。".to_owned()],
            solution: vec!["我".to_owned(), "坐".to_owned(), "地铁".to_owned(), "。".to_owned()],
        };
        let instance = visible_instance(
            &sentence,
            TranslationDirection::EnglishToChinese,
            &[2, 0, 3, 1],
            "translation-1".to_owned(),
        ).unwrap();
        assert!(instance.visible_input.contains("I take the subway."));
        assert!(!instance.visible_input.contains("我坐地铁。"));
    }
}
