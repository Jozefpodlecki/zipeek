#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Difficult,
    Hardcore,
}

impl Difficulty {
    pub const ALL: [Self; 4] = [Self::Easy, Self::Medium, Self::Difficult, Self::Hardcore];

    pub const fn size(self) -> usize {
        match self {
            Self::Easy => 2,
            Self::Medium => 3,
            Self::Difficult => 4,
            Self::Hardcore => 5,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Difficult => "difficult",
            Self::Hardcore => "hardcore",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Difficult => "Difficult",
            Self::Hardcore => "Hardcore",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.key() == value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub hanzi: String,

    pub readings: Option<Vec<String>>,
    pub meaning: Option<String>,
}

impl Candidate {
    pub fn is_eligible(&self) -> bool {
        let hanzi = self.hanzi.trim();
        let characters = hanzi.chars().collect::<Vec<_>>();
        (2..=3).contains(&characters.len())
            && characters
                .iter()
                .all(|character| ('\u{3400}'..='\u{9fff}').contains(character))
            && characters
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == characters.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    pub character: char,
    pub pinyin: Option<String>,
    pub completed: bool,
    pub filler: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Answer {
    pub(super) word: String,
    pub(super) indices: Vec<usize>,
    pub(super) meaning: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundWord {
    pub hanzi: String,
    pub pinyin: Option<String>,
    pub meaning: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntError {
    InsufficientVocabulary { required: usize, available: usize },
    InvalidTile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapResult {
    Selected,
    Found,
    Incorrect,
    Ignored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterHunt {
    pub difficulty: Difficulty,
    pub tiles: Vec<Tile>,
    pub selected: Vec<usize>,
    pub found: Vec<String>,
    pub mistakes: u32,
    pub(super) answers: Vec<Answer>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HuntSessionStats {
    pub completed_rounds: u16,
    pub found_words: usize,
    pub mistakes: u32,
}

impl HuntSessionStats {
    pub fn with_completed_round(self, board: &CharacterHunt) -> Option<Self> {
        if !board.complete() {
            return None;
        }
        Some(Self {
            completed_rounds: self.completed_rounds.saturating_add(1),
            found_words: self.found_words.saturating_add(board.found.len()),
            mistakes: self.mistakes.saturating_add(board.mistakes),
        })
    }
}
