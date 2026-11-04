use crate::theme::Theme;
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, PartialEq)]
pub(crate) struct ThemeState {
    pub current: Theme,
    pub modified: bool,
}

pub(crate) enum ThemeAction {
    Toggle,
}

impl ThemeState {
    pub fn new(current: Theme) -> Self {
        Self {
            current,
            modified: false,
        }
    }
}

impl Reducible for ThemeState {
    type Action = ThemeAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ThemeAction::Toggle => Self {
                current: self.current.toggled(),
                modified: true,
            }
            .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_preference_is_not_marked_for_persistence() {
        for theme in [Theme::Dark, Theme::Light] {
            let state = ThemeState::new(theme);
            assert_eq!(state.current, theme);
            assert!(!state.modified);
        }
    }

    #[test]
    fn rapid_toggles_use_latest_state() {
        let initial = Rc::new(ThemeState::new(Theme::Dark));
        let first = initial.reduce(ThemeAction::Toggle);
        assert_eq!(first.current, Theme::Light);
        let second = first.reduce(ThemeAction::Toggle);
        assert_eq!(second.current, Theme::Dark);
        assert!(second.modified);
    }
}
