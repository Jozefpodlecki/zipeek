use std::rc::Rc;
use yew::Reducible;
use crate::models::StrokeAssistance;
use zipeek_hanzi_writing::{InkCapture, MatchConfig, Point, StrokeOutcome, StrokeRejection};

use super::stroke_loading::StrokeStudy;
use super::targets::TARGETS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AttemptFeedback {
    Accepted,
    Rejected(StrokeRejection),
    Complete,
    Assisting,
    Assisted,
}

impl AttemptFeedback {
    pub fn label(self) -> &'static str {
        match self {
            Self::Accepted => "Stroke accepted",
            Self::Rejected(reason) => reason.message(),
            Self::Complete => "Character complete",
            Self::Assisting => "Showing the correct stroke, then continuing automatically",
            Self::Assisted => "Stroke demonstrated and completed with assistance",
        }
    }

    pub fn kind(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected(_) => "rejected",
            Self::Complete => "complete",
            Self::Assisting => "assisting",
            Self::Assisted => "assisted",
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct WritingState {
    pub character: String,
    pub target: usize,
    pub guide: bool,
    pub feedback: Option<AttemptFeedback>,
    pub replay_epoch: Option<u64>,
    pub replay_counter: u64,
    pub rejection_count: usize,
    pub assistance: StrokeAssistance,
    pub assist_epoch: Option<u64>,
    pub assist_index: Option<usize>,
    pub ink: InkCapture,
    pub study: Option<Result<StrokeStudy, String>>,
}

impl Default for WritingState {
    fn default() -> Self {
        Self {
            character: String::new(),
            target: 0,
            guide: true,
            feedback: None,
            replay_epoch: None,
            replay_counter: 0,
            rejection_count: 0,
            assistance: StrokeAssistance::AfterThree,
            assist_epoch: None,
            assist_index: None,
            ink: InkCapture::new(),
            study: None,
        }
    }
}

pub(super) enum WritingAction {
    Select {
        target: usize,
        character: String,
    },
    Loading {
        character: String,
    },
    Load {
        character: String,
        result: Result<StrokeStudy, String>,
    },
    Begin {
        pointer: i64,
        point: Point,
    },
    Move {
        pointer: i64,
        point: Point,
    },
    Finish {
        pointer: i64,
        point: Option<Point>,
    },
    Cancel {
        pointer: i64,
    },
    Undo,
    Clear,
    ToggleGuide,
    ConfigureAssistance(StrokeAssistance),
    AssistFinished { epoch: u64 },
    Replay,
    ReplayFinished {
        epoch: u64,
    },
}

impl Reducible for WritingState {
    type Action = WritingAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            WritingAction::Select { target, character } => {
                if target >= TARGETS.len()
                    || (state.target == target && state.character == character)
                {
                    return self;
                }
                state.target = target;
                state.character = character;
                state.study = None;
                state.feedback = None;
                state.rejection_count = 0;
                state.assist_epoch = None;
                state.assist_index = None;
                state.ink.clear();
                state.replay_epoch = None;
            }
            WritingAction::Loading { character } => {
                if state.character != character {
                    return self;
                }
                state.ink.clear();
                state.feedback = None;
                state.rejection_count = 0;
                state.assist_epoch = None;
                state.assist_index = None;
                state.study = None;
                state.replay_epoch = None;
            }
            WritingAction::Load { character, result } => {
                if state.character != character {
                    return self;
                }
                state.ink.clear();
                state.feedback = None;
                state.rejection_count = 0;
                state.assist_epoch = None;
                state.assist_index = None;
                state.study = Some(result);
                state.replay_epoch = None;
            }
            WritingAction::Begin { pointer, point } => {
                if !state.ready() || state.ink.begin(pointer, point).is_err() {
                    return self;
                }
                state.feedback = None;
                state.replay_epoch = None;
            }
            WritingAction::Move { pointer, point } => {
                if !matches!(state.ink.append(pointer, point), Ok(true)) {
                    return self;
                }
            }
            WritingAction::Finish { pointer, point } => {
                if let Some(point) = point {
                    let _ = state.ink.append(pointer, point);
                }
                if state.ink.finish(pointer).is_err() {
                    return self;
                }
                if let (Some(Ok(study)), Some(stroke)) =
                    (&mut state.study, state.ink.strokes().last())
                {
                    state.feedback =
                        Some(match study.session.apply(stroke, MatchConfig::default()) {
                            StrokeOutcome::Accepted { .. } if study.session.is_complete() => {
                                state.rejection_count = 0;
                                AttemptFeedback::Complete
                            }
                            StrokeOutcome::Accepted { .. } => {
                                state.rejection_count = 0;
                                AttemptFeedback::Accepted
                            },
                            StrokeOutcome::Rejected(reason) => {
                                state.rejection_count = state.rejection_count.saturating_add(1);
                                if state.assistance.threshold().is_some_and(|threshold| state.rejection_count >= threshold) {
                                    state.replay_counter = state.replay_counter.wrapping_add(1);
                                    state.assist_epoch = Some(state.replay_counter);
                                    state.assist_index = Some(study.session.completed());
                                    state.replay_epoch = None;
                                    AttemptFeedback::Assisting
                                } else {
                                    AttemptFeedback::Rejected(reason)
                                }
                            },
                            StrokeOutcome::Complete => AttemptFeedback::Complete,
                        });
                }
                state.ink.clear();
            }
            WritingAction::Cancel { pointer } => {
                if state.ink.cancel(pointer).is_err() {
                    return self;
                }
            }
            WritingAction::Undo => {
                state.assist_epoch = None;
                state.assist_index = None;
                state.rejection_count = 0;
                state.feedback = None;
                state.ink.clear();
                state.replay_epoch = None;
                if let Some(Ok(study)) = &mut state.study {
                    study.session.undo();
                }
            }
            WritingAction::Clear => {
                state.assist_epoch = None;
                state.assist_index = None;
                state.rejection_count = 0;
                state.feedback = None;
                state.ink.clear();
                state.replay_epoch = None;
                if let Some(Ok(study)) = &mut state.study {
                    study.session.reset();
                }
            }
            WritingAction::ToggleGuide => state.guide = !state.guide,
            WritingAction::ConfigureAssistance(assistance) => state.assistance = assistance,
            WritingAction::AssistFinished { epoch } => {
                if state.assist_epoch != Some(epoch) {
                    return self;
                }
                state.assist_epoch = None;
                state.assist_index = None;
                state.rejection_count = 0;
                if let Some(Ok(study)) = &mut state.study {
                    if study.session.assist_next() {
                        state.feedback = Some(if study.session.is_complete() {
                            AttemptFeedback::Complete
                        } else {
                            AttemptFeedback::Assisted
                        });
                    }
                }
            },
            WritingAction::Replay => {
                if !matches!(&state.study, Some(Ok(_))) || state.ink.is_drawing() || state.assist_epoch.is_some() {
                    return self;
                }
                state.replay_counter = state.replay_counter.wrapping_add(1);
                state.replay_epoch = Some(state.replay_counter);
            }
            WritingAction::ReplayFinished { epoch } => {
                if state.replay_epoch != Some(epoch) {
                    return self;
                }
                state.replay_epoch = None;
            }
        }
        Rc::new(state)
    }
}

impl WritingState {
    pub fn ready(&self) -> bool {
        self.assist_epoch.is_none() && matches!(&self.study, Some(Ok(study)) if !study.session.is_complete())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn select(state: Rc<WritingState>, name: &str) -> Rc<WritingState> {
        state.reduce(WritingAction::Select {
            target: 0,
            character: name.into(),
        })
    }
    #[test]
    fn replay_is_reference_gated_and_resets_on_clear() {
        let state = Rc::new(WritingState::default());
        let state = state.reduce(WritingAction::Replay);
        assert_eq!(state.replay_epoch, None);
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let state = select(state, "人").reduce(WritingAction::Load {
            character: "人".into(),
            result: Ok(study),
        });
        let state = state.reduce(WritingAction::Replay);
        assert_eq!(state.replay_epoch, Some(1));
        let state = state.reduce(WritingAction::Replay);
        assert_eq!(state.replay_epoch, Some(2));
        let state = state.reduce(WritingAction::ReplayFinished { epoch: 1 });
        assert_eq!(state.replay_epoch, Some(2));
        let state = state.reduce(WritingAction::ReplayFinished { epoch: 2 });
        assert_eq!(state.replay_epoch, None);
        let state = state.reduce(WritingAction::Replay);
        let state = state.reduce(WritingAction::Clear);
        assert_eq!(state.replay_epoch, None);
    }

    #[test]
    fn replay_cannot_interrupt_active_pointer_capture() {
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(),
            result: Ok(study),
        });
        let state = state.reduce(WritingAction::Begin {
            pointer: 7,
            point: Point::new(0.2, 0.3).unwrap(),
        });
        assert!(state.ink.is_drawing());
        let state = state.reduce(WritingAction::Replay);
        assert_eq!(state.replay_epoch, None);
        assert!(state.ink.is_drawing());
        let state = state.reduce(WritingAction::Cancel { pointer: 7 });
        let state = state.reduce(WritingAction::Replay);
        assert_eq!(state.replay_epoch, Some(1));
    }

    #[test]
    fn switching_character_clears_previous_reference() {
        let state = select(Rc::new(WritingState::default()), "人");
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let state = state.reduce(WritingAction::Load {
            character: "人".into(),
            result: Ok(study),
        });
        let state = select(state, "我");
        assert!(state.study.is_none());
        assert!(!state.ink.is_drawing());
    }
    #[test]
    fn stale_async_results_cannot_replace_active_character() {
        let state = select(Rc::new(WritingState::default()), "人");
        let state = select(state, "我");
        let state = state.reduce(WritingAction::Load {
            character: "人".into(),
            result: Err("stale".into()),
        });
        assert!(state.study.is_none());
        assert_eq!(state.character, "我");
    }
    #[test]
    fn invalid_target_is_ignored() {
        let state = Rc::new(WritingState::default());
        let next = state.clone().reduce(WritingAction::Select {
            target: usize::MAX,
            character: "人".into(),
        });
        assert!(Rc::ptr_eq(&state, &next));
    }
    #[test]
    fn cancel_discards_ink_without_affecting_session() {
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(),
            result: Ok(study),
        });
        let point = Point::new(0.2, 0.3).unwrap();
        let state = state.reduce(WritingAction::Begin { pointer: 7, point });
        assert!(state.ink.is_drawing());
        let state = state.reduce(WritingAction::Cancel { pointer: 7 });
        assert!(!state.ink.is_drawing());
        assert_eq!(
            state
                .study
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .session
                .completed(),
            0
        );
        assert_eq!(
            state
                .study
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .session
                .mistakes(),
            0
        );
    }

    #[test]
    fn clear_resets_incomplete_ink_and_undo_is_safe() {
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(),
            result: Ok(study),
        });
        let state = state.reduce(WritingAction::Begin {
            pointer: 1,
            point: Point::new(0.1, 0.2).unwrap(),
        });
        let state = state
            .reduce(WritingAction::Clear)
            .reduce(WritingAction::Undo);
        assert!(!state.ink.is_drawing());
        assert_eq!(
            state
                .study
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .session
                .completed(),
            0
        );
    }

    #[test]
    fn loading_clears_previous_error_for_retry() {
        let state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(),
            result: Err("offline".into()),
        });
        assert!(matches!(&state.study, Some(Err(_))));
        let state = state.reduce(WritingAction::Loading {
            character: "人".into(),
        });
        assert!(state.study.is_none());
    }

    #[test]
    fn rejected_trace_sets_feedback_without_progress_and_clear_resets_it() {
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(),
            result: Ok(study),
        });
        let state = state
            .reduce(WritingAction::Begin {
                pointer: 3,
                point: Point::new(0.1, 0.1).unwrap(),
            })
            .reduce(WritingAction::Move {
                pointer: 3,
                point: Point::new(0.1, 0.9).unwrap(),
            })
            .reduce(WritingAction::Finish {
                pointer: 3,
                point: None,
            });
        assert!(matches!(state.feedback, Some(AttemptFeedback::Rejected(_))));
        assert_eq!(state.rejection_count, 1);
        let session = &state.study.as_ref().unwrap().as_ref().unwrap().session;
        assert_eq!(session.completed(), 0);
        assert_eq!(session.mistakes(), 1);
        let state = state.reduce(WritingAction::Clear);
        assert_eq!(state.feedback, None);
        assert_eq!(state.rejection_count, 0);
        assert_eq!(
            state
                .study
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .session
                .mistakes(),
            0
        );
    }

    #[test]
    fn three_rejections_trigger_single_stroke_assistance_then_advance() {
        let study = StrokeStudy::parse(r#"{"strokes":["M 0 0","M 0 0"],"medians":[[[100,700],[800,700]],[[100,600],[800,600]]]}"#).unwrap();
        let mut state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(), result: Ok(study),
        });
        for attempt in 0..3 {
            state = state.reduce(WritingAction::Begin { pointer: 7, point: Point::new(0.1, 0.1).unwrap() })
                .reduce(WritingAction::Move { pointer: 7, point: Point::new(0.1, 0.9).unwrap() })
                .reduce(WritingAction::Finish { pointer: 7, point: None });
            if attempt < 2 {
                assert_eq!(state.assist_epoch, None);
                assert_eq!(state.study.as_ref().unwrap().as_ref().unwrap().session.completed(), 0);
            }
        }
        assert_eq!(state.feedback, Some(AttemptFeedback::Assisting));
        assert!(!state.ready());
        assert_eq!(state.assist_index, Some(0));
        let epoch = state.assist_epoch.unwrap();
        let state = state.reduce(WritingAction::AssistFinished { epoch });
        let session = &state.study.as_ref().unwrap().as_ref().unwrap().session;
        assert_eq!(session.completed(), 1);
        assert_eq!(session.assisted_strokes(), 1);
        assert_eq!(session.mistakes(), 3);
        assert_eq!(state.rejection_count, 0);
        assert!(state.ready());
        let state = state.reduce(WritingAction::AssistFinished { epoch });
        assert_eq!(state.study.as_ref().unwrap().as_ref().unwrap().session.completed(), 1);
    }

    #[test]
    fn five_attempt_policy_waits_until_fifth_rejection() {
        let study = StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#).unwrap();
        let mut state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(), result: Ok(study),
        }).reduce(WritingAction::ConfigureAssistance(StrokeAssistance::AfterFive));
        for index in 0..5 {
            state = state.reduce(WritingAction::Begin { pointer: 2, point: Point::new(0.1, 0.1).unwrap() })
                .reduce(WritingAction::Move { pointer: 2, point: Point::new(0.1, 0.9).unwrap() })
                .reduce(WritingAction::Finish { pointer: 2, point: None });
            assert_eq!(state.assist_epoch.is_some(), index == 4);
        }
    }

    #[test]
    fn assist_can_be_cancelled_and_off_never_auto_advances() {
        let study = StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#).unwrap();
        let mut state = select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
            character: "人".into(), result: Ok(study),
        }).reduce(WritingAction::ConfigureAssistance(StrokeAssistance::Off));
        for _ in 0..5 {
            state = state.reduce(WritingAction::Begin { pointer: 1, point: Point::new(0.1, 0.1).unwrap() })
                .reduce(WritingAction::Move { pointer: 1, point: Point::new(0.1, 0.9).unwrap() })
                .reduce(WritingAction::Finish { pointer: 1, point: None });
        }
        assert_eq!(state.assist_epoch, None);
        assert_eq!(state.study.as_ref().unwrap().as_ref().unwrap().session.completed(), 0);
        state = state.reduce(WritingAction::ConfigureAssistance(StrokeAssistance::AfterThree));
        state = state.reduce(WritingAction::Begin { pointer: 1, point: Point::new(0.1, 0.1).unwrap() })
            .reduce(WritingAction::Move { pointer: 1, point: Point::new(0.1, 0.9).unwrap() })
            .reduce(WritingAction::Finish { pointer: 1, point: None });
        let epoch = state.assist_epoch.unwrap();
        state = state.reduce(WritingAction::Clear);
        state = state.reduce(WritingAction::AssistFinished { epoch });
        assert_eq!(state.study.as_ref().unwrap().as_ref().unwrap().session.completed(), 0);
        assert_eq!(state.study.as_ref().unwrap().as_ref().unwrap().session.assisted_strokes(), 0);
    }

    #[test]
    fn a_new_attempt_clears_previous_feedback_and_wrong_pointer_cannot_finish() {
        let study =
            StrokeStudy::parse(r#"{"strokes":["M 0 0"],"medians":[[[100,700],[800,700]]]}"#)
                .unwrap();
        let mut state =
            select(Rc::new(WritingState::default()), "人").reduce(WritingAction::Load {
                character: "人".into(),
                result: Ok(study),
            });
        state = state
            .reduce(WritingAction::Begin {
                pointer: 3,
                point: Point::new(0.1, 0.1).unwrap(),
            })
            .reduce(WritingAction::Finish {
                pointer: 3,
                point: None,
            });
        assert!(matches!(state.feedback, Some(AttemptFeedback::Rejected(_))));
        state = state.reduce(WritingAction::Begin {
            pointer: 5,
            point: Point::new(0.1, 0.7).unwrap(),
        });
        assert_eq!(state.feedback, None);
        let ignored = state.clone().reduce(WritingAction::Finish {
            pointer: 6,
            point: None,
        });
        assert!(Rc::ptr_eq(&state, &ignored));
        assert!(state.ink.is_drawing());
    }
}
