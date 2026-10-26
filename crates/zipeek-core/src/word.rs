use crate::{Hsk2Level, Hsk3Level, PartOfSpeech};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WordId(i32);
impl WordId {
    pub const fn as_i32(self) -> i32 {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordIdError {
    pub value: i64,
}
impl fmt::Display for WordIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "word id {} is outside 1..={}", self.value, i32::MAX)
    }
}
impl Error for WordIdError {}
impl TryFrom<i64> for WordId {
    type Error = WordIdError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if !(1..=i64::from(i32::MAX)).contains(&value) {
            return Err(WordIdError { value });
        }
        Ok(Self(value as i32))
    }
}
impl fmt::Display for WordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub id: WordId,
    pub simplified: String,
    pub traditional: Option<String>,
    pub pinyin: String,
    pub pinyin_numeric: Option<String>,
    pub english: Option<String>,
    pub part_of_speech: Vec<PartOfSpeech>,
    pub hsk2_level: Option<Hsk2Level>,
    pub hsk3_level: Option<Hsk3Level>,
    pub frequency: Option<i32>,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_fit_positive_sql_integer_row_numbers() {
        for value in [i64::MIN, -1, 0, i64::from(i32::MAX) + 1, i64::MAX] {
            assert!(WordId::try_from(value).is_err());
        }
        for value in [1, i64::from(i32::MAX)] {
            assert_eq!(i64::from(WordId::try_from(value).unwrap().as_i32()), value);
        }
    }
}
