use std::rc::Rc;
use yew::Reducible;
use zipeek_exercises::cloud_match::{CloudGame, CloudSide, TapResult};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CloudSession {
    pub game: Option<CloudGame>,
    pub feedback: String,
    pub feedback_sequence: u64,
    pub incorrect_pairs: Vec<(CloudSide, i32)>,
    pub setup_error: String,
}

pub(crate) enum CloudAction {
    Start(CloudGame),
    StartFailed(String),
    ReturnToSetup,
    Pause,
    PauseWhenHidden,
    Tick,
    Finish,
    Choose(CloudSide, i32),
    ClearFeedback(u64),
}

impl Reducible for CloudSession {
    type Action = CloudAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            CloudAction::Start(game) => {
                next.game = Some(game);
                next.feedback.clear();
                next.incorrect_pairs.clear();
                next.setup_error.clear();
            }
            CloudAction::StartFailed(error) => {
                next.game = None;
                next.feedback.clear();
                next.incorrect_pairs.clear();
                next.setup_error = error;
            }
            CloudAction::ReturnToSetup => {
                next.setup_error.clear();
                next.game = None;
                next.feedback.clear();
                next.incorrect_pairs.clear();
            }
            CloudAction::Pause => {
                if let Some(game) = &mut next.game {
                    if !game.finished {
                        game.paused = !game.paused;
                    }
                }
            }
            CloudAction::PauseWhenHidden => {
                if let Some(game) = &mut next.game {
                    if !game.finished {
                        game.paused = true;
                    }
                }
            }
            CloudAction::Tick => {
                if let Some(game) = &mut next.game {
                    game.tick();
                }
            }
            CloudAction::Finish => {
                if let Some(game) = &mut next.game {
                    game.finish_endless();
                }
            }
            CloudAction::ClearFeedback(sequence) => {
                if next.feedback_sequence == sequence {
                    next.feedback.clear();
                    next.incorrect_pairs.clear();
                }
            }
            CloudAction::Choose(side, id) => {
                if let Some(game) = &mut next.game {
                    let previous = game.selected;
                    let outcome = game.tap(side, id);
                    if matches!(outcome, TapResult::Ignored) {
                        return self;
                    }
                    next.feedback_sequence = next.feedback_sequence.wrapping_add(1);
                    next.incorrect_pairs = if matches!(outcome, TapResult::Incorrect) {
                        previous
                            .into_iter()
                            .chain(std::iter::once((side, id)))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    next.feedback = match outcome {
                        TapResult::Correct => "Match!".to_owned(),
                        TapResult::Incorrect if game.infinite_lives => "Incorrect pair".to_owned(),
                        TapResult::Incorrect => "−1 life".to_owned(),
                        TapResult::Selected => String::new(),
                        TapResult::Ignored => unreachable!(),
                    };
                }
            }
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_exercises::cloud_match::{CloudPair, Difficulty, MatchMode};

    fn session() -> Rc<CloudSession> {
        let words = (1..=12)
            .map(|id| CloudPair {
                id,
                hanzi: format!("字{id}"),
                pinyin: format!("zi{id}"),
                meaning: format!("word {id}"),
            })
            .collect();
        Rc::new(CloudSession::default()).reduce(CloudAction::Start(
            CloudGame::new(words, MatchMode::HanziMeaning, Difficulty::Normal, 7).unwrap(),
        ))
    }

    #[test]
    fn failed_restart_discards_previous_game_state() {
        let running = session();
        let failed = running.reduce(CloudAction::StartFailed("no candidates".into()));
        assert!(failed.game.is_none());
        assert_eq!(failed.setup_error, "no candidates");
        let reset = failed.reduce(CloudAction::ReturnToSetup);
        assert!(reset.setup_error.is_empty());
    }

    #[test]
    fn tick_after_quick_selection_preserves_selection() {
        let original = session();
        let id = original.game.as_ref().unwrap().wave_pairs()[0].id;
        let selected = original.reduce(CloudAction::Choose(CloudSide::Left, id));
        let ticked = selected.reduce(CloudAction::Tick);
        assert_eq!(
            ticked.game.as_ref().unwrap().selected,
            Some((CloudSide::Left, id)),
        );
    }

    #[test]
    fn hiding_and_resuming_preserves_game() {
        let started = session();
        let hidden = started.reduce(CloudAction::PauseWhenHidden);
        assert!(hidden.game.as_ref().unwrap().paused);
        let resumed = hidden.reduce(CloudAction::Pause);
        assert_eq!(resumed.game.as_ref().unwrap().score, 0);
        assert!(!resumed.game.as_ref().unwrap().paused);
    }

    #[test]
    fn incorrect_attempt_marks_only_the_two_tapped_tiles() {
        let started = session();
        let pairs = started.game.as_ref().unwrap().wave_pairs();
        let first = pairs[0].id;
        let second = pairs[1].id;
        let selected = started.reduce(CloudAction::Choose(CloudSide::Left, first));
        let wrong = selected.reduce(CloudAction::Choose(CloudSide::Right, second));
        assert_eq!(
            wrong.incorrect_pairs,
            vec![(CloudSide::Left, first), (CloudSide::Right, second)]
        );
        assert_eq!(wrong.game.as_ref().unwrap().mistakes, 1);
        let cleared = wrong
            .clone()
            .reduce(CloudAction::ClearFeedback(wrong.feedback_sequence));
        assert!(cleared.incorrect_pairs.is_empty());
        assert!(cleared.feedback.is_empty());
        assert_eq!(cleared.game.as_ref().unwrap().mistakes, 1);
    }

    #[test]
    fn stale_feedback_timeout_does_not_erase_new_selection_feedback() {
        let started = session();
        let pairs = started.game.as_ref().unwrap().wave_pairs();
        let first = pairs[0].id;
        let second = pairs[1].id;
        let selected = started.reduce(CloudAction::Choose(CloudSide::Left, first));
        let wrong = selected.reduce(CloudAction::Choose(CloudSide::Right, second));
        let old_sequence = wrong.feedback_sequence;
        let latest = wrong.reduce(CloudAction::Choose(CloudSide::Left, first));
        let after_old_timeout = latest.reduce(CloudAction::ClearFeedback(old_sequence));
        assert_eq!(
            after_old_timeout.game.as_ref().unwrap().selected,
            Some((CloudSide::Left, first))
        );
        assert_eq!(after_old_timeout.feedback_sequence, old_sequence + 1);
    }

    #[test]
    fn returning_to_setup_clears_feedback_without_erasing_configuration() {
        let started = session();
        let ended = started.reduce(CloudAction::ReturnToSetup);
        assert!(ended.game.is_none());
        assert!(ended.feedback.is_empty());
    }
}
