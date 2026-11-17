use std::rc::Rc;
use yew::Reducible;
use zipeek_exercises::cloud_match::{Difficulty, GameLength, LifeLimit, MatchMode, SpawnMode};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CloudSetupState {
    pub source: String,
    pub standard: u8,
    pub level: u8,
    pub selected_set: String,
    pub mode: MatchMode,
    pub difficulty: Difficulty,
    pub length: GameLength,
    pub spawn: SpawnMode,
    pub highlight_partner: bool,
    pub lives: LifeLimit,
}

impl Default for CloudSetupState {
    fn default() -> Self {
        Self {
            source: "hsk".to_owned(),
            standard: 3,
            level: 1,
            selected_set: String::new(),
            mode: MatchMode::HanziMeaning,
            difficulty: Difficulty::Normal,
            length: GameLength::Fixed(10),
            spawn: SpawnMode::Waves,
            highlight_partner: false,
            lives: LifeLimit::Fixed(3),
        }
    }
}

pub(crate) enum SetupAction {
    Source(String),
    Standard(u8),
    Level(u8),
    WordSet(String),
    Mode(MatchMode),
    Difficulty(Difficulty),
    Length(GameLength),
    Spawn(SpawnMode),
    HighlightPartner(bool),
    Lives(LifeLimit),
}

impl Reducible for CloudSetupState {
    type Action = SetupAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            SetupAction::Source(source) => next.source = source,
            SetupAction::Standard(standard) => {
                next.standard = standard;
                next.level = 1;
            }
            SetupAction::Level(level) => next.level = level,
            SetupAction::WordSet(id) => next.selected_set = id,
            SetupAction::Mode(mode) => next.mode = mode,
            SetupAction::Difficulty(difficulty) => next.difficulty = difficulty,
            SetupAction::Length(length) => next.length = length,
            SetupAction::Spawn(spawn) => next.spawn = spawn,
            SetupAction::HighlightPartner(enabled) => next.highlight_partner = enabled,
            SetupAction::Lives(lives) => next.lives = lives,
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_standard_resets_level_atomically() {
        let state = Rc::new(CloudSetupState::default()).reduce(SetupAction::Level(5));
        let state = state.reduce(SetupAction::Standard(2));
        assert_eq!((state.standard, state.level), (2, 1));
    }

    #[test]
    fn independent_settings_survive_source_changes() {
        let state = Rc::new(CloudSetupState::default())
            .reduce(SetupAction::HighlightPartner(true))
            .reduce(SetupAction::Length(GameLength::Endless))
            .reduce(SetupAction::Source("sets".to_owned()));
        assert!(state.highlight_partner);
        assert_eq!(state.length, GameLength::Endless);
    }
}
