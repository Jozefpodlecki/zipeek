#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Neutral,
    Travel,
    Food,
    Shopping,
    Nature,
}

impl ThemeId {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Travel => "travel",
            Self::Food => "food",
            Self::Shopping => "shopping",
            Self::Nature => "nature",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeRequest {
    pub categories: Vec<String>,
    pub seed: u64,
}
