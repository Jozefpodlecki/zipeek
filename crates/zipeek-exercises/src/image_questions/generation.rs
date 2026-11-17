use std::collections::BTreeSet;

use super::eligibility::eligible_words;
use super::model::{ImageQuestion, ImageQuestionError, ImageQuestionMode, VisualWord};

pub fn prepare(
    words: &[VisualWord],
    mode: ImageQuestionMode,
    seed: u64,
) -> Result<ImageQuestion, ImageQuestionError> {
    let mut eligible = eligible_words(words)?;
    let required = match mode {
        ImageQuestionMode::OneOfThree => 3,
        ImageQuestionMode::SelectAll => 4,
    };
    if eligible.len() < required {
        return Err(ImageQuestionError::NotEnoughVisualWords);
    }

    let mut rng = seed;
    for index in (1..eligible.len()).rev() {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        eligible.swap(index, (rng as usize) % (index + 1));
    }
    let count = match mode {
        ImageQuestionMode::OneOfThree => 3,
        ImageQuestionMode::SelectAll => eligible.len().min(6),
    };
    let mut options = eligible.into_iter().take(count).collect::<Vec<_>>();
    let target_count = match mode {
        ImageQuestionMode::OneOfThree => 1,
        ImageQuestionMode::SelectAll => 2,
    };
    let correct_ids = options[..target_count]
        .iter()
        .map(|word| word.id)
        .collect::<BTreeSet<_>>();
    let subjects = options[..target_count]
        .iter()
        .map(|word| word.english.as_str())
        .collect::<Vec<_>>()
        .join(" and ");
    let prompt = format!(
        "Realistic photo of {subjects}, clear isolated subjects, simple scene, natural light, no text"
    );

    options.rotate_left(((seed >> 12) as usize) % count);
    let question = ImageQuestion {
        mode,
        options,
        correct_ids,
        prompt,
    };
    question.validate()?;
    Ok(question)
}
