use rand::{rngs::ChaCha12Rng, seq::IndexedRandom};

use super::{BuildError, labels::eligible};
use crate::{
    HskLevel, Word, WordId,
    practice::{ExerciseVariant, PracticeFilter},
};

pub(crate) fn recover<'a>(
    words: &'a [Word],
    variant: ExerciseVariant,
    rng: &mut ChaCha12Rng,
    attempts: &mut Vec<WordId>,
) -> Result<(&'a Word, bool), BuildError> {
    if !words.is_empty() {
        for _ in 0..10 {
            if let Some(word) = words.sample(rng, 1).next() {
                attempts.push(word.id);
                if eligible(word, variant) {
                    return Ok((word, false));
                }
            }
        }
    }
    words
        .iter()
        .find(|word| eligible(word, variant))
        .map(|word| (word, true))
        .ok_or_else(|| BuildError::RecoveryUnavailable {
            attempts: attempts.clone(),
        })
}

pub(crate) fn seed_filter(word: &Word) -> Option<PracticeFilter> {
    let level = word
        .hsk3_level
        .map(HskLevel::Hsk3)
        .or_else(|| word.hsk2_level.map(HskLevel::Hsk2))?;
    let mut part_of_speech = word.part_of_speech.clone();
    part_of_speech.sort_by_key(|pos| pos.as_str());
    part_of_speech.dedup();
    Some(PracticeFilter {
        level,
        part_of_speech,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};

    fn candidate(eligible: bool) -> Word {
        Word {
            id: WordId::try_from(1).unwrap(),
            simplified: "一".to_owned(),
            pinyin: "yī".to_owned(),
            english: Some("one".to_owned()),
            hsk3_level: eligible.then_some(crate::Hsk3Level::One),
            hsk2_level: None,
            part_of_speech: vec![],
            traditional: None,
            pinyin_numeric: None,
            frequency: None,
        }
    }

    #[test]
    fn early_success_and_exhaustion_consume_exactly_the_required_requests() {
        for (is_eligible, count) in [(true, 1), (false, 10)] {
            let words = vec![candidate(is_eligible)];
            let mut actual = ChaCha12Rng::seed_from_u64(42);
            let mut control = ChaCha12Rng::seed_from_u64(42);
            let mut attempts = Vec::new();
            let result = recover(
                &words,
                ExerciseVariant::EngPinyin,
                &mut actual,
                &mut attempts,
            );
            assert_eq!(result.is_ok(), is_eligible);
            for _ in 0..count {
                words.as_slice().sample(&mut control, 1).next();
            }
            assert_eq!(attempts.len(), count);
            assert_eq!(actual.next_u64(), control.next_u64());
        }
    }

    #[test]
    fn empty_recovery_consumes_no_rng() {
        let mut actual = ChaCha12Rng::seed_from_u64(42);
        let mut control = ChaCha12Rng::seed_from_u64(42);
        assert!(
            recover(
                &[],
                ExerciseVariant::EngPinyin,
                &mut actual,
                &mut Vec::new()
            )
            .is_err()
        );
        assert_eq!(actual.next_u64(), control.next_u64());
    }

    #[test]
    fn by_id_fallback_consumes_no_rng_after_ten_rejections() {
        let words: Vec<_> = (1..=128)
            .map(|value| {
                let mut word = candidate(value == 1);
                word.id = WordId::try_from(value).unwrap();
                word
            })
            .collect();
        let mut actual = ChaCha12Rng::seed_from_u64(0xfc682b615de651a8);
        let mut control = ChaCha12Rng::seed_from_u64(0xfc682b615de651a8);
        let mut attempts = Vec::new();
        let (word, fallback) = recover(
            &words,
            ExerciseVariant::EngPinyin,
            &mut actual,
            &mut attempts,
        )
        .unwrap();
        assert_eq!(word.id.as_i32(), 1);
        assert!(fallback);
        for _ in 0..10 {
            words.as_slice().sample(&mut control, 1).next();
        }
        assert_eq!(attempts.len(), 10);
        assert_eq!(actual.next_u64(), control.next_u64());
    }
}
