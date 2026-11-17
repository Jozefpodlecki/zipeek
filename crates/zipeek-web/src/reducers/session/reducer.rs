use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SessionState {
    pub key: Option<SessionKey>,
    pub expected: Option<SessionKey>,
    pub active: ActiveRound,
    pub required: u64,
    pub generation: u64,
    pub domain_session: Option<QuizSession>,
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
    fn round_definition(key: &SessionKey) -> &'static str {
        if key.variant.is_mcq() {
            "multiple-choice"
        } else {
            "matching"
        }
    }

    pub(super) fn visible_round(content: &RoundContent) -> String {
        match content {
            RoundContent::Mcq {
                prompt, options, ..
            } => {
                format!(
                    "Prompt: {}\nVisible options: {}",
                    prompt.label,
                    options
                        .iter()
                        .enumerate()
                        .map(|(index, card)| format!("{index}: {}", card.label))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            RoundContent::Matching { left, right } => {
                format!(
                    "Left tiles: {}\nRight tiles: {}",
                    left.iter()
                        .enumerate()
                        .map(|(index, card)| format!("L{index}: {}", card.label))
                        .collect::<Vec<_>>()
                        .join(", "),
                    right
                        .iter()
                        .enumerate()
                        .map(|(index, card)| format!("R{index}: {}", card.label))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }

    fn record_current_round(&mut self) {
        let Some(session) = self.domain_session.as_mut() else {
            return;
        };
        let stats = self.active.stats();
        let previous = session.totals();
        let possible = if matches!(&self.active, ActiveRound::Mcq(_)) {
            1
        } else {
            4
        };
        let result = ExerciseResult {
            earned: stats.correct.saturating_sub(previous.earned).min(possible),
            possible,
            mistakes: stats.mistakes.saturating_sub(previous.mistakes),
        };
        session
            .record(result)
            .expect("a complete round must have one pending exercise");
    }

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
                let visible_input = Self::visible_round(&content);
                next.active = ActiveRound::fresh(&key, content, now, next.generation);
                if !continuing {
                    next.domain_session =
                        Some(QuizSession::new(format!("practice-{}", next.generation)));
                }
                if let Some(session) = next.domain_session.as_mut() {
                    let instance = ExerciseInstance::new(
                        format!("round-{}", next.generation),
                        Self::round_definition(&key),
                        visible_input,
                    )
                    .expect("built-in exercise definition");
                    session.push(instance).expect("practice session transition");
                }
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
                next.record_current_round();
                next.expected = Some(target);
            }
            SessionAction::Finish(key, now)
                if self.current(&key) && self.active.complete() && self.active.last() =>
            {
                next.record_current_round();
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
                if let Some(session) = next.domain_session.as_mut() {
                    session.complete().expect("completed practice session");
                }
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
                next.domain_session = None;
            }
            _ => return self,
        }
        if next == *self { self } else { next.into() }
    }
}
