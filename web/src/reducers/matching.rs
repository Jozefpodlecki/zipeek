use std::{collections::BTreeSet, rc::Rc};
use yew::Reducible;
use zipeek_core::{
    WordId,
    practice::{ExerciseVariant, FixedProgress, NormalizedPracticeQuery, PracticeMode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchKey {
    variant: ExerciseVariant,
    query: Vec<(String, String)>,
}

impl MatchKey {
    pub fn new(variant: ExerciseVariant, query: &NormalizedPracticeQuery) -> Self {
        Self {
            variant,
            query: query.query_pairs(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CardChoice {
    pub side: Side,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchQuestion {
    pub key: MatchKey,
    pub left: Vec<WordId>,
    pub right: Vec<WordId>,
    pub mode: PracticeMode,
}

impl MatchQuestion {
    pub fn word(&self, choice: CardChoice) -> Option<WordId> {
        match choice.side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
        .get(choice.index)
        .copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WrongPair {
    pub first: CardChoice,
    pub second: CardChoice,
    pub serial: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MatchState {
    pub question: Option<MatchQuestion>,
    pub epoch: u64,
    pub selected: Option<CardChoice>,
    pub wrong: Option<WrongPair>,
    pub matched: BTreeSet<WordId>,
    pub retried: BTreeSet<WordId>,
    pub correct: u64,
    pub first_attempt: u64,
    pub mistakes: u64,
    pub streak: u64,
    pub highest_streak: u64,
    pub progress: Option<FixedProgress>,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub expected: Option<MatchKey>,
    pub looped: bool,
    serial: u64,
}

pub(crate) enum MatchAction {
    Ready(MatchQuestion, u64),
    Select(u64, CardChoice),
    Escape(u64),
    ClearWrong(u64, u64),
    Advance(u64, MatchKey, bool),
    Finish(u64, u64),
    Restart(MatchKey),
}

impl MatchState {
    pub fn complete(&self) -> bool {
        self.matched.len() == 4
    }
    pub fn last(&self) -> bool {
        self.progress.is_some_and(|mut progress| {
            progress.finish_round();
            progress.is_complete()
        })
    }
    pub fn elapsed_ms(&self) -> u64 {
        self.finished_at
            .zip(self.started_at)
            .map_or(0, |(end, start)| end.saturating_sub(start))
    }
    fn interactive(&self, epoch: u64) -> bool {
        self.epoch == epoch
            && self.expected.is_none()
            && self.finished_at.is_none()
            && self.wrong.is_none()
    }
}

impl Reducible for MatchState {
    type Action = MatchAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MatchAction::Ready(question, now) => {
                if self.question.as_ref() == Some(&question) && self.expected.is_none() {
                    return self;
                }
                let continuing = self.expected.as_ref() == Some(&question.key);
                let progress = match question.mode {
                    PracticeMode::Fixed(rounds) => Some(if continuing {
                        self.progress.unwrap_or_else(|| FixedProgress::new(rounds))
                    } else {
                        FixedProgress::new(rounds)
                    }),
                    PracticeMode::Endless => None,
                };
                Self {
                    question: Some(question),
                    epoch: self.epoch.wrapping_add(1),
                    progress,
                    correct: if continuing { self.correct } else { 0 },
                    first_attempt: if continuing { self.first_attempt } else { 0 },
                    mistakes: if continuing { self.mistakes } else { 0 },
                    streak: if continuing { self.streak } else { 0 },
                    highest_streak: if continuing { self.highest_streak } else { 0 },
                    started_at: Some(if continuing {
                        self.started_at.unwrap_or(now)
                    } else {
                        now
                    }),
                    looped: continuing && self.looped,
                    ..Self::default()
                }
                .into()
            }
            MatchAction::Select(epoch, choice) if self.interactive(epoch) => {
                let Some(question) = &self.question else {
                    return self;
                };
                let Some(word) = question
                    .word(choice)
                    .filter(|word| !self.matched.contains(word))
                else {
                    return self;
                };
                let Some(selected) = self.selected else {
                    return Self {
                        selected: Some(choice),
                        ..(*self).clone()
                    }
                    .into();
                };
                if choice == selected {
                    return Self {
                        selected: None,
                        ..(*self).clone()
                    }
                    .into();
                }
                if choice.side == selected.side {
                    return self;
                }
                let Some(first) = question.word(selected) else {
                    return self;
                };
                if word == first {
                    let mut matched = self.matched.clone();
                    matched.insert(word);
                    let streak = self.streak.saturating_add(1);
                    Self {
                        selected: None,
                        matched,
                        correct: self.correct.saturating_add(1),
                        first_attempt: self
                            .first_attempt
                            .saturating_add(u64::from(!self.retried.contains(&word))),
                        streak,
                        highest_streak: self.highest_streak.max(streak),
                        ..(*self).clone()
                    }
                    .into()
                } else {
                    let mut retried = self.retried.clone();
                    retried.extend([first, word]);
                    let serial = self.serial.wrapping_add(1);
                    Self {
                        wrong: Some(WrongPair {
                            first: selected,
                            second: choice,
                            serial,
                        }),
                        retried,
                        serial,
                        mistakes: self
                            .mistakes
                            .saturating_add(u64::from(self.progress.is_some())),
                        streak: 0,
                        ..(*self).clone()
                    }
                    .into()
                }
            }
            MatchAction::Escape(epoch) if self.interactive(epoch) => Self {
                selected: None,
                ..(*self).clone()
            }
            .into(),
            MatchAction::ClearWrong(epoch, serial)
                if self.epoch == epoch
                    && self
                        .wrong
                        .as_ref()
                        .is_some_and(|wrong| wrong.serial == serial) =>
            {
                Self {
                    wrong: None,
                    selected: None,
                    ..(*self).clone()
                }
                .into()
            }
            MatchAction::Advance(epoch, next, looped)
                if self.interactive(epoch) && self.complete() && !self.last() =>
            {
                let mut progress = self.progress;
                if let Some(progress) = &mut progress {
                    progress.finish_round();
                }
                Self {
                    expected: Some(next),
                    progress,
                    looped,
                    ..(*self).clone()
                }
                .into()
            }
            MatchAction::Finish(epoch, now)
                if self.interactive(epoch) && self.complete() && self.last() =>
            {
                let mut progress = self.progress;
                if let Some(progress) = &mut progress {
                    progress.finish_round();
                }
                Self {
                    progress,
                    finished_at: Some(now),
                    ..(*self).clone()
                }
                .into()
            }
            MatchAction::Restart(next) if self.finished_at.is_some() => Self {
                expected: Some(next),
                epoch: self.epoch,
                ..Self::default()
            }
            .into(),
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_core::practice::ParsedPracticeQuery;

    fn question(offset: &str) -> MatchQuestion {
        let query = ParsedPracticeQuery::from_pairs([
            ("offset", offset),
            ("mode", "fixed"),
            ("rounds", "5"),
        ])
        .normalize(ExerciseVariant::HanziEng, |_| None);
        MatchQuestion {
            key: MatchKey::new(ExerciseVariant::HanziEng, &query),
            left: [1, 2, 3, 4]
                .map(|id| WordId::try_from(id).unwrap())
                .to_vec(),
            right: [3, 1, 4, 2]
                .map(|id| WordId::try_from(id).unwrap())
                .to_vec(),
            mode: query.mode,
        }
    }
    fn choice(side: Side, index: usize) -> CardChoice {
        CardChoice { side, index }
    }
    fn select(state: Rc<MatchState>, side: Side, index: usize) -> Rc<MatchState> {
        let epoch = state.epoch;
        state.reduce(MatchAction::Select(epoch, choice(side, index)))
    }
    fn ready() -> Rc<MatchState> {
        Rc::new(MatchState::default()).reduce(MatchAction::Ready(question("0"), 1000))
    }

    #[test]
    fn same_side_noop_toggle_escape_and_stale_selection_follow_the_contract() {
        let state = select(ready(), Side::Right, 1);
        let unchanged = select(state.clone(), Side::Right, 2);
        assert!(Rc::ptr_eq(&state, &unchanged));
        let cleared = select(state.clone(), Side::Right, 1);
        assert_eq!(cleared.selected, None);
        let cleared = state.clone().reduce(MatchAction::Escape(state.epoch));
        assert_eq!(cleared.selected, None);
        let stale = state
            .clone()
            .reduce(MatchAction::Select(state.epoch + 1, choice(Side::Left, 0)));
        assert!(Rc::ptr_eq(&state, &stale));
        let invalid = select(state.clone(), Side::Left, 99);
        assert!(Rc::ptr_eq(&state, &invalid));
    }

    #[test]
    fn wrong_attempt_locks_and_disqualifies_both_pairs_without_double_counting() {
        let state = select(select(ready(), Side::Left, 0), Side::Right, 3);
        assert_eq!(
            (state.mistakes, state.streak, state.retried.len()),
            (1, 0, 2)
        );
        let blocked = select(state.clone(), Side::Right, 1);
        assert!(Rc::ptr_eq(&state, &blocked));
        let wrong = state.wrong.as_ref().unwrap();
        let late = state
            .clone()
            .reduce(MatchAction::ClearWrong(state.epoch, wrong.serial + 1));
        assert!(Rc::ptr_eq(&state, &late));
        let state = state
            .clone()
            .reduce(MatchAction::ClearWrong(state.epoch, wrong.serial));
        let state = select(select(state, Side::Left, 0), Side::Right, 1);
        let state = select(select(state, Side::Right, 3), Side::Left, 1);
        assert_eq!(
            (state.correct, state.first_attempt, state.streak),
            (2, 0, 2)
        );
        let duplicate = select(state.clone(), Side::Left, 0);
        assert!(Rc::ptr_eq(&state, &duplicate));
        let state = select(select(state, Side::Left, 2), Side::Right, 0);
        let state = select(select(state, Side::Right, 2), Side::Left, 3);
        assert_eq!(
            (state.correct, state.first_attempt, state.highest_streak),
            (4, 2, 4)
        );
    }

    #[test]
    fn only_complete_rounds_advance_and_wrapping_preserves_fixed_stats_and_time() {
        let mut state = ready();
        for round in 0..5 {
            let early =
                state
                    .clone()
                    .reduce(MatchAction::Advance(state.epoch, question("0").key, false));
            assert!(Rc::ptr_eq(&state, &early));
            for (left, right) in [(0, 1), (1, 3), (2, 0), (3, 2)] {
                state = select(select(state, Side::Left, left), Side::Right, right);
            }
            let epoch = state.epoch;
            if round == 4 {
                state = state.reduce(MatchAction::Finish(epoch, 5000));
                assert_eq!(
                    (state.correct, state.first_attempt, state.elapsed_ms()),
                    (20, 20, 4000)
                );
                assert!(state.progress.unwrap().is_complete());
                let duplicate = state.clone().reduce(MatchAction::Finish(epoch, 9000));
                assert!(Rc::ptr_eq(&state, &duplicate));
            } else {
                state = state.reduce(MatchAction::Advance(epoch, question("0").key, true));
                state = state.reduce(MatchAction::Ready(question("0"), 2000));
                assert_eq!(state.progress.unwrap().completed(), round + 1);
                assert!(state.looped);
                let stale = state.clone().reduce(MatchAction::ClearWrong(epoch, 1));
                assert!(Rc::ptr_eq(&state, &stale));
            }
        }
        let state = state.reduce(MatchAction::Restart(question("8").key));
        let state = state.reduce(MatchAction::Ready(question("8"), 10000));
        assert_eq!(
            (
                state.correct,
                state.first_attempt,
                state.mistakes,
                state.started_at
            ),
            (0, 0, 0, Some(10000))
        );
        let reset = state.reduce(MatchAction::Ready(question("12"), 20000));
        assert_eq!(reset.progress.unwrap().completed(), 0);
    }
}
