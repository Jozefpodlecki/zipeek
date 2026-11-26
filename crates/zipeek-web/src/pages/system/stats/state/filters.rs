#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GrammarView {
    #[default]
    Table,
    Cards,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HistoryView {
    #[default]
    Timeline,
    Table,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HistoryCategory {
    #[default]
    All,
    Practice,
    Games,
}

impl HistoryCategory {
    pub const ALL: [Self; 3] = [Self::All, Self::Practice, Self::Games];

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All activity",
            Self::Practice => "Practice",
            Self::Games => "Games",
        }
    }

    pub fn matches(self, category: &str) -> bool {
        match self {
            Self::All => true,
            Self::Practice => category == "Practice",
            Self::Games => category == "Games",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkillGap {
    Recognition,
    Listening,
    Speaking,
    Writing,
    Sentence,
}

impl SkillGap {
    pub const ALL: [Self; 5] = [
        Self::Recognition,
        Self::Listening,
        Self::Speaking,
        Self::Writing,
        Self::Sentence,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Recognition => "Recognition",
            Self::Listening => "Listening",
            Self::Speaking => "Speaking",
            Self::Writing => "Writing",
            Self::Sentence => "Sentence use",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GrammarSort {
    #[default]
    TeachingOrder,
    PatternAscending,
    PatternDescending,
    MostAttempts,
    FewestAttempts,
}

impl GrammarSort {
    pub fn toggle_pattern(self) -> Self {
        if self == Self::PatternAscending {
            Self::PatternDescending
        } else {
            Self::PatternAscending
        }
    }

    pub fn toggle_attempts(self) -> Self {
        if self == Self::MostAttempts {
            Self::FewestAttempts
        } else {
            Self::MostAttempts
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GrammarRegion {
    #[default]
    All,
    Regions1To20,
    Regions21To40,
    Regions41To60,
    Regions61To80,
    Regions81To100,
}

impl GrammarRegion {
    pub const ALL: [Self; 6] = [
        Self::All,
        Self::Regions1To20,
        Self::Regions21To40,
        Self::Regions41To60,
        Self::Regions61To80,
        Self::Regions81To100,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All collections",
            Self::Regions1To20 => "Collection A · 01-20",
            Self::Regions21To40 => "Collection B · 21-40",
            Self::Regions41To60 => "Collection C · 41-60",
            Self::Regions61To80 => "Collection D · 61-80",
            Self::Regions81To100 => "Collection E · 81-100",
        }
    }

    pub fn contains(self, teaching_id: &str) -> bool {
        if self == Self::All {
            return true;
        }
        let Some(region) = teaching_id
            .strip_prefix('g')
            .and_then(|part| part.split('-').next())
            .and_then(|part| part.parse::<usize>().ok())
        else {
            return false;
        };
        match self {
            Self::All => true,
            Self::Regions1To20 => (1..=20).contains(&region),
            Self::Regions21To40 => (21..=40).contains(&region),
            Self::Regions41To60 => (41..=60).contains(&region),
            Self::Regions61To80 => (61..=80).contains(&region),
            Self::Regions81To100 => (81..=100).contains(&region),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WordSort {
    #[default]
    Alphabetical,
    AlphabeticalReverse,
    Encounters,
    FewestEncounters,
    Review,
}
