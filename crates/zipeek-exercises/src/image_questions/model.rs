use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualWord {
    pub id: i32,
    pub hanzi: String,
    pub english: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageQuestionMode {
    OneOfThree,
    SelectAll,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageQuestion {
    pub mode: ImageQuestionMode,
    pub options: Vec<VisualWord>,
    pub correct_ids: BTreeSet<i32>,
    pub prompt: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageQuestionError {
    NotEnoughVisualWords,
    DuplicateWordId,
    InvalidSelection,
    InvalidQuestion,
}

impl std::fmt::Display for ImageQuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::NotEnoughVisualWords => {
                "Not enough distinct, visually depictable words in this Word Set"
            }
            Self::DuplicateWordId => "The Word Set contains duplicate word identifiers",
            Self::InvalidSelection => "Select valid picture answers before checking",
            Self::InvalidQuestion => "Generated image question is not internally consistent",
        };
        f.write_str(text)
    }
}

impl std::error::Error for ImageQuestionError {}

impl ImageQuestion {
    pub fn validate(&self) -> Result<(), ImageQuestionError> {
        let ids = self
            .options
            .iter()
            .map(|word| word.id)
            .collect::<BTreeSet<_>>();
        if !(3..=6).contains(&self.options.len())
            || ids.len() != self.options.len()
            || self.correct_ids.is_empty()
            || !self.correct_ids.is_subset(&ids)
            || (self.mode == ImageQuestionMode::OneOfThree && self.correct_ids.len() != 1)
            || (self.mode == ImageQuestionMode::SelectAll && self.correct_ids.len() != 2)
            || self.options.iter().any(|word| {
                word.id <= 0
                    || word.hanzi.trim().is_empty()
                    || word.hanzi.chars().count() > 36
                    || word.english.trim().is_empty()
                    || word.english.chars().count() > 160
            })
            || self.prompt.trim().is_empty()
        {
            return Err(ImageQuestionError::InvalidQuestion);
        }
        Ok(())
    }

    pub fn grade(&self, selected: &BTreeSet<i32>) -> Result<bool, ImageQuestionError> {
        self.validate()?;
        if selected.is_empty()
            || (self.mode == ImageQuestionMode::OneOfThree && selected.len() != 1)
            || selected
                .iter()
                .any(|id| !self.options.iter().any(|option| option.id == *id))
        {
            return Err(ImageQuestionError::InvalidSelection);
        }
        Ok(*selected == self.correct_ids)
    }
}
