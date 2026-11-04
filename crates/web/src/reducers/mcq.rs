use std::rc::Rc;
use yew::Reducible;
use zipeek_core::{
    WordId,
    practice::{ExerciseVariant, FixedProgress, NormalizedPracticeQuery, PracticeMode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct McqKey {
    variant: ExerciseVariant,
    query: Vec<(String, String)>,
}

impl McqKey {
    pub fn new(variant: ExerciseVariant, query: &NormalizedPracticeQuery) -> Self {
        Self {
            variant,
            query: query.query_pairs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct McqQuestion {
    pub key: McqKey,
    pub options: Vec<WordId>,
    pub correct: WordId,
    pub mode: PracticeMode,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct McqState {
    pub question: Option<McqQuestion>,
    pub selected: Option<usize>,
    pub correct: u64,
    pub first_attempt: u64,
    pub streak: u64,
    pub highest_streak: u64,
    pub mistakes: u64,
    pub progress: Option<FixedProgress>,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub expected: Option<McqKey>,
    pub looped: bool,
}

pub(crate) enum McqAction {
    Ready(McqQuestion, u64),
    Select(McqKey, usize),
    Advance(McqKey, McqKey, bool),
    Finish(McqKey, u64),
    Restart(McqKey),
}

impl Reducible for McqState {
    type Action = McqAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            McqAction::Ready(question, now) => {
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
                    correct: if continuing { self.correct } else { 0 },
                    first_attempt: if continuing { self.first_attempt } else { 0 },
                    streak: if continuing { self.streak } else { 0 },
                    highest_streak: if continuing { self.highest_streak } else { 0 },
                    mistakes: if continuing { self.mistakes } else { 0 },
                    progress,
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
            McqAction::Select(key, index)
                if self.expected.is_none()
                    && self.selected.is_none()
                    && self.finished_at.is_none() =>
            {
                let Some(question) = self
                    .question
                    .as_ref()
                    .filter(|question| question.key == key)
                else {
                    return self;
                };
                let Some(id) = question.options.get(index) else {
                    return self;
                };
                let correct = *id == question.correct;
                let streak = if correct {
                    self.streak.saturating_add(1)
                } else {
                    0
                };
                Self {
                    selected: Some(index),
                    correct: self.correct.saturating_add(u64::from(correct)),
                    first_attempt: self.first_attempt.saturating_add(u64::from(correct)),
                    streak,
                    highest_streak: self.highest_streak.max(streak),
                    mistakes: self
                        .mistakes
                        .saturating_add(u64::from(!correct && self.progress.is_some())),
                    ..(*self).clone()
                }
                .into()
            }
            McqAction::Advance(key, next, looped)
                if self.expected.is_none()
                    && self.selected.is_some()
                    && self.finished_at.is_none()
                    && !self.is_last()
                    && self
                        .question
                        .as_ref()
                        .is_some_and(|question| question.key == key) =>
            {
                let mut progress = self.progress;
                if let Some(progress) = &mut progress {
                    progress.finish_round();
                }
                Self {
                    progress,
                    expected: Some(next),
                    looped,
                    ..(*self).clone()
                }
                .into()
            }
            McqAction::Finish(key, now)
                if self.selected.is_some()
                    && self.expected.is_none()
                    && self.finished_at.is_none()
                    && self.is_last()
                    && self
                        .question
                        .as_ref()
                        .is_some_and(|question| question.key == key) =>
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
            McqAction::Restart(next) if self.finished_at.is_some() => Self {
                expected: Some(next),
                ..Self::default()
            }
            .into(),
            _ => self,
        }
    }
}

impl McqState {
    pub fn is_last(&self) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_core::practice::ParsedPracticeQuery;

    fn question(offset: &str) -> McqQuestion {
        let query = ParsedPracticeQuery::from_pairs([("offset", offset)])
            .normalize(ExerciseVariant::HanziToMeaning, |_| None);
        McqQuestion {
            key: McqKey::new(ExerciseVariant::HanziToMeaning, &query),
            options: [1, 2, 3].map(|id| WordId::try_from(id).unwrap()).to_vec(),
            correct: WordId::try_from(2).unwrap(),
            mode: query.mode,
        }
    }

    #[test]
    fn first_answer_locks_and_stale_or_invalid_answers_do_not_count() {
        let q = question("0");
        let state = Rc::new(McqState::default()).reduce(McqAction::Ready(q.clone(), 1000));
        let stale = state
            .clone()
            .reduce(McqAction::Select(question("9").key, 1));
        assert!(Rc::ptr_eq(&state, &stale));
        let invalid = state.clone().reduce(McqAction::Select(q.key.clone(), 3));
        assert!(Rc::ptr_eq(&state, &invalid));
        let answered = state.reduce(McqAction::Select(q.key.clone(), 1));
        assert_eq!(
            (answered.correct, answered.streak, answered.selected),
            (1, 1, Some(1))
        );
        let repeat = answered.clone().reduce(McqAction::Select(q.key, 0));
        assert!(Rc::ptr_eq(&answered, &repeat));
    }

    #[test]
    fn only_expected_navigation_retains_counters_and_wrong_answer_resets_streak() {
        let q = question("0");
        let next = question("1");
        let state = Rc::new(McqState::default()).reduce(McqAction::Ready(q.clone(), 1000));
        let blocked =
            state
                .clone()
                .reduce(McqAction::Advance(q.key.clone(), next.key.clone(), false));
        assert!(Rc::ptr_eq(&state, &blocked));
        let answered = state.reduce(McqAction::Select(q.key.clone(), 1));
        let waiting = answered.reduce(McqAction::Advance(q.key, next.key.clone(), true));
        let late = waiting
            .clone()
            .reduce(McqAction::Select(next.key.clone(), 1));
        assert!(Rc::ptr_eq(&waiting, &late));
        let next = waiting.reduce(McqAction::Ready(next.clone(), 2000));
        assert_eq!(
            (next.correct, next.streak, next.selected, next.looped),
            (1, 1, None, true)
        );
        let wrong = next.reduce(McqAction::Select(question("1").key, 0));
        assert_eq!((wrong.correct, wrong.streak), (1, 0));
        let external = wrong.reduce(McqAction::Ready(question("8"), 3000));
        assert_eq!((external.correct, external.streak), (0, 0));
    }

    #[test]
    fn fixed_counts_completion_once_independently_of_wrapping_and_freezes_time() {
        let mut state = Rc::new(McqState::default());
        for round in 0..5 {
            let mut q = question(&(round % 2).to_string());
            q.mode = PracticeMode::Fixed(zipeek_core::practice::FixedRounds::Five);
            state = state.reduce(McqAction::Ready(q.clone(), 1000 + round * 100));
            assert_eq!(state.progress.unwrap().completed(), round as u8);
            let premature = state.clone().reduce(McqAction::Finish(q.key.clone(), 2000));
            assert!(Rc::ptr_eq(&state, &premature));
            state = state.reduce(McqAction::Select(
                q.key.clone(),
                if round == 1 { 0 } else { 1 },
            ));
            if round == 4 {
                state = state.reduce(McqAction::Finish(q.key.clone(), 2500));
                assert!(state.progress.unwrap().is_complete());
                assert_eq!(state.elapsed_ms(), 1500);
                let repeat = state.clone().reduce(McqAction::Finish(q.key.clone(), 9000));
                assert!(Rc::ptr_eq(&state, &repeat));
                let repeat = state.clone().reduce(McqAction::Select(q.key.clone(), 1));
                assert!(Rc::ptr_eq(&state, &repeat));
            } else {
                let mut next = question(&((round + 1) % 2).to_string());
                next.mode = q.mode;
                state = state.reduce(McqAction::Advance(q.key, next.key.clone(), round % 2 == 1));
            }
        }
        assert_eq!(
            (state.correct, state.mistakes, state.highest_streak),
            (4, 1, 3)
        );
        let restart = question("44");
        let state = state.reduce(McqAction::Restart(restart.key.clone()));
        let state = state.reduce(McqAction::Ready(restart, 10000));
        assert_eq!(
            (
                state.correct,
                state.mistakes,
                state.started_at,
                state.finished_at
            ),
            (0, 0, Some(10000), None)
        );
    }
}
