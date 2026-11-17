use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    #[default]
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningSupport {
    #[default]
    Guided,
    Reading,
    Challenge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestionKind {
    FindName,
    FindCapital,
    FindOutline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerMode {
    SelectMap,
    SelectHanzi,
}

impl Difficulty {
    pub fn kinds(self) -> &'static [QuestionKind] {
        match self {
            Self::Easy => &[QuestionKind::FindName],
            Self::Medium => &[
                QuestionKind::FindName,
                QuestionKind::FindCapital,
                QuestionKind::FindOutline,
            ],
            Self::Hard => &[QuestionKind::FindName],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easy_never_asks_for_capital() {
        assert_eq!(Difficulty::Easy.kinds(), &[QuestionKind::FindName]);
    }

    #[test]
    fn medium_includes_capital_and_outline() {
        assert!(
            Difficulty::Medium
                .kinds()
                .contains(&QuestionKind::FindCapital)
        );
        assert!(
            Difficulty::Medium
                .kinds()
                .contains(&QuestionKind::FindOutline)
        );
    }
}
