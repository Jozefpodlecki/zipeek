use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use zipeek_core::practice::ExerciseVariant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuizWord {
    pub id: i32,
    pub hanzi: String,
    pub meaning: String,
    pub pinyin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuizError {
    InsufficientDistinctWords,
    DuplicateIds,
    InvalidSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairAttempt {
    Correct,
    Incorrect,
    AlreadyMatched,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuizRound {
    pub words: Vec<QuizWord>,
    pub answer_id: i32,
    pub format: ExerciseVariant,
}

pub fn prepare(words: &[QuizWord], seed: u64) -> Result<QuizRound, QuizError> {
    prepare_for_format(words, seed, ExerciseVariant::MatchingHanziMeaning)
}

pub fn prepare_for_format(
    words: &[QuizWord],
    seed: u64,
    format: ExerciseVariant,
) -> Result<QuizRound, QuizError> {
    prepare_for_format_with_count(words, seed, format, 4)
}

pub fn prepare_for_format_with_count(
    words: &[QuizWord],
    seed: u64,
    format: ExerciseVariant,
    pair_count: usize,
) -> Result<QuizRound, QuizError> {
    if !(2..=7).contains(&pair_count) || (format.is_mcq() && pair_count != 4) {
        return Err(QuizError::InvalidSelection);
    }
    let mut eligible = words
        .iter()
        .filter(|word| {
            word.id > 0
                && !word.hanzi.trim().is_empty()
                && !word.meaning.trim().is_empty()
                && !word.pinyin.trim().is_empty()
                && word.hanzi.chars().count() <= 36
                && word.meaning.chars().count() <= 160
                && word.pinyin.chars().count() <= 120
        })
        .cloned()
        .collect::<Vec<_>>();
    for word in &mut eligible {
        word.hanzi = word.hanzi.trim().to_owned();
        word.meaning = word.meaning.trim().to_owned();
        word.pinyin = word.pinyin.trim().to_owned();
    }
    eligible.sort_by_key(|word| word.id);
    if eligible.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(QuizError::DuplicateIds);
    }
    let mut prompts = BTreeSet::new();
    let mut choices = BTreeSet::new();
    eligible.retain(|word| {
        prompts.insert(prompt_text(word, format).to_lowercase())
            && choices.insert(option_text(word, format).to_lowercase())
    });
    if eligible.len() < pair_count {
        return Err(QuizError::InsufficientDistinctWords);
    }
    let mut rng = seed.max(1);
    for index in (1..eligible.len()).rev() {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        eligible.swap(index, (rng as usize) % (index + 1));
    }
    eligible.truncate(pair_count);
    let answer_id = eligible[0].id;
    eligible.rotate_left(((seed >> 16) as usize) % pair_count);
    Ok(QuizRound {
        words: eligible,
        answer_id,
        format,
    })
}

fn prompt_text(word: &QuizWord, format: ExerciseVariant) -> &str {
    match format {
        ExerciseVariant::MatchingEnglishPinyin
        | ExerciseVariant::MultipleChoiceMeaningHanzi
        | ExerciseVariant::MultipleChoiceMeaningPinyin => &word.meaning,
        ExerciseVariant::MatchingPinyinHanzi => &word.pinyin,
        ExerciseVariant::MatchingHanziMeaning
        | ExerciseVariant::MatchingAudioHanzi
        | ExerciseVariant::MatchingAudioMeaning
        | ExerciseVariant::MultipleChoiceHanziMeaning => &word.hanzi,
    }
}

fn option_text(word: &QuizWord, format: ExerciseVariant) -> &str {
    match format {
        ExerciseVariant::MatchingEnglishPinyin | ExerciseVariant::MultipleChoiceMeaningPinyin => {
            &word.pinyin
        }
        ExerciseVariant::MatchingPinyinHanzi
        | ExerciseVariant::MatchingAudioHanzi
        | ExerciseVariant::MultipleChoiceMeaningHanzi => &word.hanzi,
        ExerciseVariant::MatchingHanziMeaning
        | ExerciseVariant::MatchingAudioMeaning
        | ExerciseVariant::MultipleChoiceHanziMeaning => &word.meaning,
    }
}

impl QuizRound {
    pub fn prompt_for<'a>(&self, word: &'a QuizWord) -> &'a str {
        prompt_text(word, self.format)
    }

    pub fn option_for<'a>(&self, word: &'a QuizWord) -> &'a str {
        option_text(word, self.format)
    }

    pub fn answer(&self) -> &QuizWord {
        self.words
            .iter()
            .find(|word| word.id == self.answer_id)
            .expect("prepared round contains its answer")
    }

    pub fn grade(&self, id: i32) -> Result<bool, QuizError> {
        if !self.words.iter().any(|word| word.id == id) {
            return Err(QuizError::InvalidSelection);
        }
        Ok(id == self.answer_id)
    }

    pub fn evaluate_pair(
        &self,
        matched: &BTreeSet<i32>,
        left_id: i32,
        right_id: i32,
    ) -> Result<PairAttempt, QuizError> {
        let correct = self.pair_is_correct(left_id, right_id)?;
        if matched.contains(&left_id) || matched.contains(&right_id) {
            return Ok(PairAttempt::AlreadyMatched);
        }
        Ok(if correct {
            PairAttempt::Correct
        } else {
            PairAttempt::Incorrect
        })
    }

    pub fn pair_is_correct(&self, id: i32, expected_id: i32) -> Result<bool, QuizError> {
        if !self.words.iter().any(|word| word.id == id)
            || !self.words.iter().any(|word| word.id == expected_id)
        {
            return Err(QuizError::InvalidSelection);
        }
        Ok(id == expected_id)
    }
}

#[cfg(test)]
#[path = "local_quiz_tests.rs"]
mod tests;
