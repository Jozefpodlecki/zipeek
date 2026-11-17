#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PracticeStep {
    #[default]
    Path,
    Exercises,
    Vocabulary,
    Summary,
}

impl PracticeStep {
    pub(super) const ALL: [Self; 4] =
        [Self::Path, Self::Exercises, Self::Vocabulary, Self::Summary];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Exercises => "exercises",
            Self::Vocabulary => "vocabulary",
            Self::Summary => "summary",
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Path => "Path",
            Self::Exercises => "Exercises",
            Self::Vocabulary => "Vocabulary",
            Self::Summary => "Summary",
        }
    }

    pub(super) const fn guidance(self) -> &'static str {
        match self {
            Self::Path => "Choose Quick or Advanced Practice.",
            Self::Exercises => "Choose exercises, then continue. Configuration is optional.",
            Self::Vocabulary => "Choose Dictionary / HSK or My Word Sets.",
            Self::Summary => "Review your selections and start practice.",
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Path => 0,
            Self::Exercises => 1,
            Self::Vocabulary => 2,
            Self::Summary => 3,
        }
    }

    pub const fn is_before(self, current: Self) -> bool {
        self.index() < current.index()
    }

    pub fn previous(self) -> Option<Self> {
        self.index().checked_sub(1).map(|index| Self::ALL[index])
    }

    pub fn next(self) -> Option<Self> {
        Self::ALL.get(self.index() + 1).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::PracticeStep;

    #[test]
    fn only_earlier_steps_are_completed() {
        assert!(PracticeStep::Path.is_before(PracticeStep::Exercises));
        assert!(PracticeStep::Exercises.is_before(PracticeStep::Summary));
        assert!(!PracticeStep::Summary.is_before(PracticeStep::Exercises));
        assert!(!PracticeStep::Vocabulary.is_before(PracticeStep::Vocabulary));
    }
}
