use super::validated_rounds;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Left, Self::Up, Self::Right, Self::Down];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Up => "Up",
            Self::Down => "Down",
        }
    }

    pub const fn phrase(self) -> &'static str {
        match self {
            Self::Left => "向左走",
            Self::Right => "向右走",
            Self::Up => "向前走",
            Self::Down => "向后走",
        }
    }

    pub const fn vocabulary_forms(self) -> &'static [&'static str] {
        match self {
            Self::Left => &["左", "左边", "向左", "向左走"],
            Self::Right => &["右", "右边", "向右", "向右走"],
            Self::Up => &["前", "前面", "向前", "向前走"],
            Self::Down => &["后", "后面", "向后", "向后走"],
        }
    }

    pub const fn offset(self) -> (i8, i8) {
        match self {
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Up => (0, -1),
            Self::Down => (0, 1),
        }
    }
}

pub const NAVIGATION_ROUTE: [Direction; 6] = [
    Direction::Left,
    Direction::Up,
    Direction::Right,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigatorAction {
    Move(Direction),
    Restart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigatorState {
    pub step: usize,
    pub rounds: Vec<usize>,
    pub mistakes: u32,
    pub position: (i8, i8),
    pub notice: Option<&'static str>,
    pub last_wrong_direction: Option<Direction>,
}

impl Default for NavigatorState {
    fn default() -> Self {
        Self::from_rounds((0..NAVIGATION_ROUTE.len()).collect())
    }
}

impl NavigatorState {
    pub fn from_rounds(rounds: Vec<usize>) -> Self {
        let mut location = (1_i8, 1_i8);
        let rounds = validated_rounds(rounds, NAVIGATION_ROUTE.len())
            .into_iter()
            .filter(|&index| {
                let (dx, dy) = NAVIGATION_ROUTE[index].offset();
                let next = (location.0 + dx, location.1 + dy);
                if !(0..=2).contains(&next.0) || !(0..=2).contains(&next.1) {
                    return false;
                }
                location = next;
                true
            })
            .collect();
        Self {
            step: 0,
            rounds,
            mistakes: 0,
            position: (1, 1),
            notice: None,
            last_wrong_direction: None,
        }
    }

    pub fn total(&self) -> usize {
        self.rounds.len()
    }

    pub fn current(&self) -> Option<Direction> {
        self.rounds
            .get(self.step)
            .and_then(|&index| NAVIGATION_ROUTE.get(index))
            .copied()
    }

    pub fn complete(&self) -> bool {
        self.step >= self.total()
    }

    pub fn apply(&mut self, action: NavigatorAction) {
        match action {
            NavigatorAction::Restart => *self = Self::from_rounds(self.rounds.clone()),
            NavigatorAction::Move(direction) => {
                let Some(expected) = self.current() else {
                    return;
                };
                if direction != expected {
                    if self.last_wrong_direction != Some(direction) {
                        self.mistakes = self.mistakes.saturating_add(1);
                        self.last_wrong_direction = Some(direction);
                    }
                    self.notice = Some("Try listening again");
                    return;
                }
                let (dx, dy) = direction.offset();
                let (x, y) = self.position;
                let Some(next_x) = x.checked_add(dx) else {
                    return;
                };
                let Some(next_y) = y.checked_add(dy) else {
                    return;
                };
                if !(0..=2).contains(&next_x) || !(0..=2).contains(&next_y) {
                    return;
                }
                self.position = (next_x, next_y);
                self.step += 1;
                self.last_wrong_direction = None;
                self.notice = Some("Correct direction");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_is_bounded_and_restarts_at_center() {
        let mut state = NavigatorState::default();
        for direction in NAVIGATION_ROUTE {
            state.apply(NavigatorAction::Move(direction));
            assert!((0..=2).contains(&state.position.0));
            assert!((0..=2).contains(&state.position.1));
        }
        assert!(state.complete());
        let completed = state.clone();
        state.apply(NavigatorAction::Move(Direction::Left));
        assert_eq!(state, completed);
        state.apply(NavigatorAction::Restart);
        assert_eq!(state, NavigatorState::default());
    }

    #[test]
    fn wrong_direction_and_invalid_position_cannot_advance() {
        let mut state = NavigatorState::default();
        state.apply(NavigatorAction::Move(Direction::Right));
        assert_eq!((state.step, state.mistakes), (0, 1));
        state.position = (i8::MIN, i8::MAX);
        state.apply(NavigatorAction::Move(Direction::Left));
        assert_eq!((state.step, state.mistakes), (0, 1));
        assert_eq!(state.position, (i8::MIN, i8::MAX));
    }

    #[test]
    fn filtered_routes_cannot_cross_grid_boundaries() {
        let mut state = NavigatorState::from_rounds(vec![2, 3, 4]);
        assert_eq!(state.rounds, [2, 4]);
        state.apply(NavigatorAction::Move(Direction::Right));
        state.apply(NavigatorAction::Move(Direction::Down));
        assert!(state.complete());
        assert_eq!(state.position, (2, 2));
    }

    #[test]
    fn repeat_wrong_direction_does_not_inflate_mistakes() {
        let mut state = NavigatorState::default();
        state.apply(NavigatorAction::Move(Direction::Right));
        state.apply(NavigatorAction::Move(Direction::Right));
        assert_eq!(state.mistakes, 1);
        state.apply(NavigatorAction::Move(Direction::Up));
        assert_eq!(state.mistakes, 2);
        state.apply(NavigatorAction::Move(Direction::Left));
        assert_eq!(state.last_wrong_direction, None);
    }

    #[test]
    fn direction_prompts_are_present() {
        for direction in Direction::ALL {
            assert!(!direction.label().is_empty());
            assert!(!direction.phrase().is_empty());
            assert!(direction.vocabulary_forms().contains(&direction.phrase()));
        }
    }
}
