use super::*;

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
    pub(super) fn retain_stats(&mut self, stats: SessionStats) {
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
    pub(super) fn fresh(key: &SessionKey, content: RoundContent, now: u64, epoch: u64) -> Self {
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
