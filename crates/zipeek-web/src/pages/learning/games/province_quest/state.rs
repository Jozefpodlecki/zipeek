use std::rc::Rc;

use yew::Reducible;
use zipeek_exercises::province_quest::{
    Action as SessionAction, Difficulty, GeoDataset, LearningSupport, Session,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    Map(String),
    Seed(String),
    Deck(String),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Map(message) | Self::Seed(message) | Self::Deck(message) => message,
        };
        formatter.write_str(message)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GamePhase {
    Setup,
    Loading {
        request: u64,
    },
    Playing {
        dataset: Rc<GeoDataset>,
        session: Session,
    },
    Failed(GameError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProvinceQuestState {
    pub difficulty: Difficulty,
    pub support: LearningSupport,
    pub phase: GamePhase,
    pub active_province: Option<String>,
    next_request: u64,
}

impl Default for ProvinceQuestState {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Easy,
            support: LearningSupport::Guided,
            phase: GamePhase::Setup,
            active_province: None,
            next_request: 0,
        }
    }
}

pub enum ProvinceQuestAction {
    SetDifficulty(Difficulty),
    SetSupport(LearningSupport),
    Start,
    Loaded {
        request: u64,
        dataset: Rc<GeoDataset>,
        session: Session,
    },
    LoadFailed {
        request: u64,
        error: GameError,
    },
    BackToSetup,
    SetActiveProvince(Option<String>),
    Session(SessionAction),
}

impl ProvinceQuestState {
    pub fn request_generation(&self) -> u64 {
        self.next_request
    }
}

impl Reducible for ProvinceQuestState {
    type Action = ProvinceQuestAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            ProvinceQuestAction::SetDifficulty(value)
                if matches!(&state.phase, GamePhase::Setup | GamePhase::Failed(_)) =>
            {
                state.difficulty = value;
            }
            ProvinceQuestAction::SetSupport(value)
                if matches!(&state.phase, GamePhase::Setup | GamePhase::Failed(_)) =>
            {
                state.support = value;
            }
            ProvinceQuestAction::Start if !matches!(&state.phase, GamePhase::Loading { .. }) => {
                state.next_request = state.next_request.wrapping_add(1);
                state.active_province = None;
                state.phase = GamePhase::Loading {
                    request: state.next_request,
                };
            }
            ProvinceQuestAction::Loaded {
                request,
                dataset,
                session,
            } if matches!(&state.phase, GamePhase::Loading { request: active } if *active == request) =>
            {
                state.phase = GamePhase::Playing { dataset, session };
                state.active_province = None;
            }
            ProvinceQuestAction::LoadFailed { request, error } if matches!(&state.phase, GamePhase::Loading { request: active } if *active == request) =>
            {
                state.phase = GamePhase::Failed(error);
            }
            ProvinceQuestAction::BackToSetup => {
                state.next_request = state.next_request.wrapping_add(1);
                state.phase = GamePhase::Setup;
                state.active_province = None;
            }
            ProvinceQuestAction::SetActiveProvince(value)
                if matches!(&state.phase, GamePhase::Playing { .. }) =>
            {
                state.active_province = value;
            }
            ProvinceQuestAction::Session(action) => {
                if let GamePhase::Playing { session, .. } = &mut state.phase {
                    let continuing = matches!(action, SessionAction::Continue);
                    session.apply(action);
                    if continuing {
                        state.active_province = None;
                    }
                }
            }
            _ => {}
        }
        Rc::new(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_request_cannot_replace_newer_setup() {
        let state = Rc::new(ProvinceQuestState::default()).reduce(ProvinceQuestAction::Start);
        let request = match &state.phase {
            GamePhase::Loading { request } => *request,
            _ => unreachable!(),
        };
        let state = state.reduce(ProvinceQuestAction::BackToSetup);
        let state = state.reduce(ProvinceQuestAction::LoadFailed {
            request,
            error: GameError::Map("outdated".into()),
        });
        assert!(matches!(&state.phase, GamePhase::Setup));
    }

    #[test]
    fn duplicate_start_does_not_advance_request_generation() {
        let state = Rc::new(ProvinceQuestState::default()).reduce(ProvinceQuestAction::Start);
        let request = match &state.phase {
            GamePhase::Loading { request } => *request,
            _ => unreachable!(),
        };
        let state = state.reduce(ProvinceQuestAction::Start);
        assert!(
            matches!(&state.phase, GamePhase::Loading { request: active } if *active == request)
        );
    }
}
