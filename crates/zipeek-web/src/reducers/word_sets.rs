use std::rc::Rc;
use yew::Reducible;

use crate::services::word_sets::{CourseWordSetSource, UserWordSet, WordSetId, find};

#[derive(Clone, PartialEq)]
pub(crate) enum WordSetsFeedback {
    Success(String),
    Error(String),
}

#[derive(Clone, PartialEq)]
pub(crate) struct WordSetsState {
    pub sets: Result<Vec<UserWordSet>, String>,
    pub target: Option<WordSetId>,
    pub feedback: Option<WordSetsFeedback>,
    pub course_feedback: Option<(CourseWordSetSource, WordSetsFeedback)>,
}

pub(crate) enum WordSetsAction {
    Loaded(Result<Vec<UserWordSet>, String>),
    Select(Option<WordSetId>),
    Feedback(WordSetsFeedback),
    CourseFeedback(CourseWordSetSource, WordSetsFeedback),
    Committed(Vec<UserWordSet>, String),
    Removed(Vec<UserWordSet>, String),
}

fn preferred_target(
    sets: &[UserWordSet],
    preferred: Option<&WordSetId>,
) -> Option<WordSetId> {
    preferred
        .filter(|id| find(sets, id).is_some())
        .cloned()
        .or_else(|| sets.first().map(|set| set.id.clone()))
}

impl WordSetsState {
    pub fn load(storage: &web_sys::Storage) -> Self {
        let sets = crate::services::word_sets::read(storage);
        let target = match &sets {
            Ok(items) => preferred_target(
                items,
                crate::services::word_sets::active_dictionary_set(storage).as_ref(),
            ),
            _ => None,
        };
        Self {
            sets,
            target,
            feedback: None,
            course_feedback: None,
        }
    }
}

impl Reducible for WordSetsState {
    type Action = WordSetsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            WordSetsAction::Loaded(sets) => {
                next.target = match &sets {
                    Ok(items)
                        if next
                            .target
                            .as_ref()
                            .is_some_and(|id| find(items, id).is_some()) =>
                    {
                        next.target.clone()
                    }
                    Ok(items) => items.first().map(|set| set.id.clone()),
                    Err(_) => None,
                };
                next.sets = sets;
                next.feedback = None;
                next.course_feedback = None;
            }
            WordSetsAction::Select(target) => {
                if target.as_ref().is_none_or(|id| {
                    next.sets.as_ref().is_ok_and(|sets| find(sets, id).is_some())
                }) {
                    next.target = target;
                }
            }
            WordSetsAction::Feedback(feedback) => {
                next.feedback = Some(feedback);
                next.course_feedback = None;
            }
            WordSetsAction::CourseFeedback(source, feedback) => {
                next.course_feedback = Some((source, feedback));
            }
            WordSetsAction::Committed(sets, message) => {
                if next
                    .target
                    .as_ref()
                    .is_none_or(|id| find(&sets, id).is_none())
                {
                    next.target = sets.last().map(|set| set.id.clone());
                }
                next.sets = Ok(sets);
                next.feedback = Some(WordSetsFeedback::Success(message));
                next.course_feedback = None;
            }
            WordSetsAction::Removed(sets, message) => {
                next.target = sets.first().map(|set| set.id.clone());
                next.sets = Ok(sets);
                next.feedback = Some(WordSetsFeedback::Success(message));
                next.course_feedback = None;
            }
        }
        next.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_dictionary_set_falls_back_if_deleted() {
        let first: WordSetId = "0123456789abcdef0123456789abcdef".parse().unwrap();
        let second: WordSetId = "fedcba9876543210fedcba9876543210".parse().unwrap();
        let saved = vec![UserWordSet {
            id: first.clone(),
            name: "Personal".to_owned(),
            word_ids: vec![],
            source: None,
        }];
        assert_eq!(preferred_target(&saved, Some(&first)), Some(first.clone()));
        assert_eq!(preferred_target(&saved, Some(&second)), Some(first.clone()));
        assert_eq!(preferred_target(&saved, None), Some(first));
        assert_eq!(preferred_target(&[], Some(&second)), None);
    }

    #[test]
    fn course_feedback_stays_attached_to_its_source() {
        let state = Rc::new(WordSetsState {
            sets: Ok(vec![]),
            target: None,
            feedback: None,
            course_feedback: None,
        });
        let source = CourseWordSetSource { region_id: 5, set_index: 2 };
        let acquired = state.reduce(WordSetsAction::CourseFeedback(
            source.clone(), WordSetsFeedback::Success("Acquired".to_owned()),
        ));
        assert!(matches!(&acquired.course_feedback,
            Some((actual, WordSetsFeedback::Success(_))) if actual == &source));
        let reloaded = acquired.reduce(WordSetsAction::Loaded(Ok(vec![])));
        assert!(reloaded.course_feedback.is_none());
    }

    #[test]
    fn only_existing_word_sets_can_be_selected() {
        let first: WordSetId = "0123456789abcdef0123456789abcdef".parse().unwrap();
        let absent: WordSetId = "fedcba9876543210fedcba9876543210".parse().unwrap();
        let state = Rc::new(WordSetsState {
            sets: Ok(vec![UserWordSet {
                id: first.clone(),
                name: "Saved".to_owned(),
                word_ids: vec![],
                source: None,
            }]),
            target: Some(first.clone()),
            feedback: None,
            course_feedback: None,
        });
        let after_invalid = state.reduce(WordSetsAction::Select(Some(absent)));
        assert_eq!(after_invalid.target.as_ref(), Some(&first));
        let cleared = after_invalid.reduce(WordSetsAction::Select(None));
        assert!(cleared.target.is_none());
    }

    #[test]
    fn feedback_is_mutually_exclusive_and_deletion_clears_target() {
        let initial = Rc::new(WordSetsState {
            sets: Ok(vec![]),
            target: None,
            feedback: None,
            course_feedback: None,
        });
        let error = initial.reduce(WordSetsAction::Feedback(WordSetsFeedback::Error(
            "invalid".into(),
        )));
        let deleted = error.reduce(WordSetsAction::Removed(vec![], "Deleted".into()));
        assert!(deleted.target.is_none());
        assert!(
            matches!(&deleted.feedback, Some(WordSetsFeedback::Success(message)) if message == "Deleted")
        );
    }
}
