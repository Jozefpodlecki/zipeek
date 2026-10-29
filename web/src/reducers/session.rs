use super::{
    matching::{CardChoice, MatchAction, MatchKey, MatchQuestion, MatchState},
    mcq::{McqAction, McqKey, McqQuestion, McqState},
};
use std::rc::Rc;
use yew::Reducible;
use zipeek_core::{
    practice::{ExerciseVariant, FixedProgress, NormalizedPracticeQuery},
    round::RoundContent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionKey {
    pub variant: ExerciseVariant,
    pub query: NormalizedPracticeQuery,
    pub types: Vec<ExerciseVariant>,
}
impl SessionKey {
    pub fn new(
        variant: ExerciseVariant,
        mut query: NormalizedPracticeQuery,
        types: Vec<ExerciseVariant>,
    ) -> Self {
        query.warnings.clear();
        Self {
            variant,
            query,
            types,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SessionStats {
    pub correct: u64,
    pub first_attempt: u64,
    pub mistakes: u64,
    pub streak: u64,
    pub highest_streak: u64,
    pub progress: Option<FixedProgress>,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub looped: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum ActiveRound {
    #[default]
    Empty,
    Mcq(Rc<McqState>),
    Matching(Rc<MatchState>),
}

impl ActiveRound {
    pub fn stats(&self) -> SessionStats {
        match self {
            Self::Empty => SessionStats::default(),
            Self::Mcq(s) => SessionStats {
                correct: s.correct,
                first_attempt: s.first_attempt,
                mistakes: s.mistakes,
                streak: s.streak,
                highest_streak: s.highest_streak,
                progress: s.progress,
                started_at: s.started_at,
                finished_at: s.finished_at,
                looped: s.looped,
            },
            Self::Matching(s) => SessionStats {
                correct: s.correct,
                first_attempt: s.first_attempt,
                mistakes: s.mistakes,
                streak: s.streak,
                highest_streak: s.highest_streak,
                progress: s.progress,
                started_at: s.started_at,
                finished_at: s.finished_at,
                looped: s.looped,
            },
        }
    }
    fn retain_stats(&mut self, stats: SessionStats) {
        match self {
            Self::Empty => {}
            Self::Mcq(state) => {
                let s = Rc::make_mut(state);
                s.correct = stats.correct;
                s.first_attempt = stats.first_attempt;
                s.mistakes = stats.mistakes;
                s.streak = stats.streak;
                s.highest_streak = stats.highest_streak;
                s.progress = stats.progress.or(s.progress);
                s.started_at = stats.started_at.or(s.started_at);
                s.finished_at = stats.finished_at;
                s.looped = stats.looped;
            }
            Self::Matching(state) => {
                let s = Rc::make_mut(state);
                s.correct = stats.correct;
                s.first_attempt = stats.first_attempt;
                s.mistakes = stats.mistakes;
                s.streak = stats.streak;
                s.highest_streak = stats.highest_streak;
                s.progress = stats.progress.or(s.progress);
                s.started_at = stats.started_at.or(s.started_at);
                s.finished_at = stats.finished_at;
                s.looped = stats.looped;
            }
        }
    }
    pub fn complete(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Mcq(s) => s.selected.is_some(),
            Self::Matching(s) => s.complete(),
        }
    }
    pub fn last(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Mcq(s) => s.is_last(),
            Self::Matching(s) => s.last(),
        }
    }
    fn fresh(key: &SessionKey, content: RoundContent, now: u64, epoch: u64) -> Self {
        match content {
            RoundContent::Mcq {
                options,
                correct_id,
                ..
            } => Self::Mcq(Rc::new(McqState::default()).reduce(McqAction::Ready(
                McqQuestion {
                    key: McqKey::new(key.variant, &key.query),
                    options: options.iter().map(|card| card.word_id).collect(),
                    correct: correct_id,
                    mode: key.query.mode,
                },
                now,
            ))),
            RoundContent::Matching { left, right } => {
                let mut state = Rc::new(MatchState::default()).reduce(MatchAction::Ready(
                    MatchQuestion {
                        key: MatchKey::new(key.variant, &key.query),
                        left: left.iter().map(|card| card.word_id).collect(),
                        right: right.iter().map(|card| card.word_id).collect(),
                        mode: key.query.mode,
                    },
                    now,
                ));
                Rc::make_mut(&mut state).epoch = epoch;
                Self::Matching(state)
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SessionState {
    pub key: Option<SessionKey>,
    pub expected: Option<SessionKey>,
    pub active: ActiveRound,
    pub required: u64,
    pub generation: u64,
}

pub(crate) enum SessionAction {
    Ready(SessionKey, RoundContent, u64),
    McqSelect(SessionKey, usize),
    MatchSelect(SessionKey, u64, CardChoice),
    Escape(SessionKey, u64),
    ClearWrong(SessionKey, u64, u64),
    Advance(SessionKey, SessionKey, bool),
    Finish(SessionKey, u64),
    Restart(SessionKey, SessionKey),
}

impl SessionState {
    fn current(&self, key: &SessionKey) -> bool {
        self.key.as_ref() == Some(key) && self.expected.is_none()
    }
}

impl Reducible for SessionState {
    type Action = SessionAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            SessionAction::Ready(key, content, now) => {
                if self.current(&key) {
                    return self;
                }
                let continuing = self.expected.as_ref() == Some(&key);
                next.generation = self.generation.wrapping_add(1);
                next.active = ActiveRound::fresh(&key, content, now, next.generation);
                if continuing {
                    next.active.retain_stats(self.active.stats());
                }
                next.required = if continuing { self.required } else { 0 };
                next.required =
                    next.required
                        .saturating_add(if key.variant.is_mcq() { 1 } else { 4 });
                next.key = Some(key);
                next.expected = None;
            }
            SessionAction::McqSelect(key, index) if self.current(&key) => {
                if let ActiveRound::Mcq(state) = &self.active {
                    next.active = ActiveRound::Mcq(state.clone().reduce(McqAction::Select(
                        McqKey::new(key.variant, &key.query),
                        index,
                    )));
                }
            }
            SessionAction::MatchSelect(key, epoch, choice) if self.current(&key) => {
                if let ActiveRound::Matching(state) = &self.active {
                    next.active = ActiveRound::Matching(
                        state.clone().reduce(MatchAction::Select(epoch, choice)),
                    );
                }
            }
            SessionAction::Escape(key, epoch) if self.current(&key) => {
                if let ActiveRound::Matching(state) = &self.active {
                    next.active =
                        ActiveRound::Matching(state.clone().reduce(MatchAction::Escape(epoch)));
                }
            }
            SessionAction::ClearWrong(key, epoch, serial) if self.current(&key) => {
                if let ActiveRound::Matching(state) = &self.active {
                    next.active = ActiveRound::Matching(
                        state.clone().reduce(MatchAction::ClearWrong(epoch, serial)),
                    );
                }
            }
            SessionAction::Advance(key, target, looped)
                if self.current(&key) && self.active.complete() && !self.active.last() =>
            {
                next.active = match &self.active {
                    ActiveRound::Mcq(state) => {
                        ActiveRound::Mcq(state.clone().reduce(McqAction::Advance(
                            McqKey::new(key.variant, &key.query),
                            McqKey::new(target.variant, &target.query),
                            looped,
                        )))
                    }
                    ActiveRound::Matching(state) => {
                        ActiveRound::Matching(state.clone().reduce(MatchAction::Advance(
                            state.epoch,
                            MatchKey::new(target.variant, &target.query),
                            looped,
                        )))
                    }
                    ActiveRound::Empty => return self,
                };
                next.expected = Some(target);
            }
            SessionAction::Finish(key, now)
                if self.current(&key) && self.active.complete() && self.active.last() =>
            {
                next.active = match &self.active {
                    ActiveRound::Mcq(state) => ActiveRound::Mcq(
                        state
                            .clone()
                            .reduce(McqAction::Finish(McqKey::new(key.variant, &key.query), now)),
                    ),
                    ActiveRound::Matching(state) => ActiveRound::Matching(
                        state.clone().reduce(MatchAction::Finish(state.epoch, now)),
                    ),
                    ActiveRound::Empty => return self,
                };
            }
            SessionAction::Restart(key, target)
                if self.current(&key) && self.active.stats().finished_at.is_some() =>
            {
                next.active = match &self.active {
                    ActiveRound::Mcq(state) => ActiveRound::Mcq(state.clone().reduce(
                        McqAction::Restart(McqKey::new(target.variant, &target.query)),
                    )),
                    ActiveRound::Matching(state) => ActiveRound::Matching(state.clone().reduce(
                        MatchAction::Restart(MatchKey::new(target.variant, &target.query)),
                    )),
                    ActiveRound::Empty => return self,
                };
                next.expected = Some(target);
                next.required = 0;
            }
            _ => return self,
        }
        if next == *self { self } else { next.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reducers::matching::Side;
    use zipeek_core::{WordId, practice::ParsedPracticeQuery, round::Card};

    fn card(id: i64) -> Card {
        Card {
            word_id: WordId::try_from(id).unwrap(),
            label: id.to_string(),
        }
    }
    fn key(variant: ExerciseVariant, offset: u64) -> SessionKey {
        let query = ParsedPracticeQuery::from_pairs([
            ("mode", "fixed"),
            ("rounds", "5"),
            ("offset", &offset.to_string()),
        ])
        .normalize(variant, |_| None);
        SessionKey::new(
            variant,
            query,
            vec![ExerciseVariant::HanziToMeaning, ExerciseVariant::HanziEng],
        )
    }
    fn content(variant: ExerciseVariant) -> RoundContent {
        if variant.is_mcq() {
            RoundContent::Mcq {
                prompt: card(1),
                options: (1..=3).map(card).collect(),
                correct_id: card(1).word_id,
            }
        } else {
            RoundContent::Matching {
                left: (1..=4).map(card).collect(),
                right: (1..=4).rev().map(card).collect(),
            }
        }
    }
    fn pick(state: Rc<SessionState>, side: Side, index: usize) -> Rc<SessionState> {
        let key = state.key.clone().unwrap();
        let epoch = state.generation;
        state.reduce(SessionAction::MatchSelect(
            key,
            epoch,
            CardChoice { side, index },
        ))
    }
    #[test]
    fn mixed_fixed_rounds_keep_shared_stats_and_weighted_required_items() {
        let mut variant = ExerciseVariant::HanziEng;
        let mut offset = 0;
        let mut state = Rc::new(SessionState::default()).reduce(SessionAction::Ready(
            key(variant, offset),
            content(variant),
            1000,
        ));
        for round in 0..5 {
            let current = state.key.clone().unwrap();
            if variant.is_mcq() {
                state = state.reduce(SessionAction::McqSelect(
                    current.clone(),
                    usize::from(round == 1),
                ));
            } else {
                if round == 0 {
                    state = pick(pick(state, Side::Left, 0), Side::Right, 2);
                    let ActiveRound::Matching(s) = &state.active else {
                        panic!("Expected matching")
                    };
                    let serial = s.wrong.as_ref().unwrap().serial;
                    let epoch = s.epoch;
                    state = state.reduce(SessionAction::ClearWrong(current.clone(), epoch, serial));
                }
                for index in 0..4 {
                    state = pick(pick(state, Side::Left, index), Side::Right, 3 - index);
                }
            }
            if round == 4 {
                state = state.reduce(SessionAction::Finish(current.clone(), 5000));
                let stats = state.active.stats();
                assert_eq!(
                    (
                        stats.correct,
                        stats.first_attempt,
                        stats.mistakes,
                        stats.highest_streak,
                        state.required
                    ),
                    (13, 11, 2, 9, 14)
                );
                assert_eq!(stats.finished_at.unwrap() - stats.started_at.unwrap(), 4000);
                assert!(stats.progress.unwrap().is_complete());
                let repeated = state
                    .clone()
                    .reduce(SessionAction::Finish(current.clone(), 9000));
                assert!(Rc::ptr_eq(&state, &repeated));
            } else {
                offset += if variant.is_mcq() { 1 } else { 4 };
                variant = if variant.is_mcq() {
                    ExerciseVariant::HanziEng
                } else {
                    ExerciseVariant::HanziToMeaning
                };
                let target = key(variant, offset);
                state = state.reduce(SessionAction::Advance(
                    current.clone(),
                    target.clone(),
                    false,
                ));
                state = state.reduce(SessionAction::Ready(target, content(variant), 2000));
                assert_eq!(
                    state.active.stats().progress.unwrap().completed(),
                    round + 1
                );
                let late = state
                    .clone()
                    .reduce(SessionAction::ClearWrong(current, 1, 1));
                assert!(Rc::ptr_eq(&state, &late));
            }
        }
        let current = state.key.clone().unwrap();
        let target = key(ExerciseVariant::HanziToMeaning, 30);
        state = state.reduce(SessionAction::Restart(current, target.clone()));
        state = state.reduce(SessionAction::Ready(
            target.clone(),
            content(target.variant),
            10000,
        ));
        assert_eq!(
            (
                state.required,
                state.active.stats().correct,
                state.active.stats().first_attempt
            ),
            (1, 0, 0)
        );
        assert_eq!(state.active.stats().started_at, Some(10000));
        assert_eq!(state.active.stats().progress.unwrap().completed(), 0);
    }
    #[test]
    fn duplicate_ready_and_answers_do_not_count_twice_and_external_query_resets() {
        let current = key(ExerciseVariant::HanziToMeaning, 0);
        let state = Rc::new(SessionState::default()).reduce(SessionAction::Ready(
            current.clone(),
            content(current.variant),
            1000,
        ));
        let state = state.reduce(SessionAction::McqSelect(current.clone(), 0));
        let duplicate = state
            .clone()
            .reduce(SessionAction::McqSelect(current.clone(), 1));
        assert!(Rc::ptr_eq(&state, &duplicate));
        let duplicate = state.clone().reduce(SessionAction::Ready(
            current.clone(),
            content(current.variant),
            2000,
        ));
        assert!(Rc::ptr_eq(&state, &duplicate));
        let changed = key(ExerciseVariant::HanziEng, 8);
        let reset = state.reduce(SessionAction::Ready(
            changed.clone(),
            content(changed.variant),
            3000,
        ));
        assert_eq!((reset.required, reset.active.stats().correct), (4, 0));
        assert_eq!(reset.active.stats().started_at, Some(3000));
        let stale = reset.clone().reduce(SessionAction::McqSelect(current, 0));
        assert!(Rc::ptr_eq(&reset, &stale));
    }
}
