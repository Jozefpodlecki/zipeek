use std::rc::Rc;
use yew::Reducible;

use crate::services::word_sets::UserWordSet;

#[derive(Clone, PartialEq)]
pub(crate) enum WordSetsFeedback {
    Success(String),
    Error(String),
}

#[derive(Clone, PartialEq)]
pub(crate) struct WordSetsState {
    pub sets: Result<Vec<UserWordSet>, String>,
    pub target: String,
    pub feedback: Option<WordSetsFeedback>,
}

pub(crate) enum WordSetsAction {
    Loaded(Result<Vec<UserWordSet>, String>),
    Select(String),
    Feedback(WordSetsFeedback),
    Committed(Vec<UserWordSet>, String),
    Removed(Vec<UserWordSet>, String),
}

impl WordSetsState {
    pub fn load() -> Self {
        let sets = crate::services::word_sets::read();
        let target = match &sets {
            Ok(items) if !items.is_empty() => "0".to_owned(),
            _ => String::new(),
        };
        Self { sets, target, feedback: None }
    }
}

impl Reducible for WordSetsState {
    type Action = WordSetsAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            WordSetsAction::Loaded(sets) => {
                next.sets = sets;
                next.feedback = None;
            }
            WordSetsAction::Select(target) => next.target = target,
            WordSetsAction::Feedback(feedback) => next.feedback = Some(feedback),
            WordSetsAction::Committed(sets, message) => {
                if next.target.parse::<usize>().ok().is_none_or(|index| index >= sets.len()) {
                    next.target = if sets.is_empty() { String::new() } else { (sets.len() - 1).to_string() };
                }
                next.sets = Ok(sets);
                next.feedback = Some(WordSetsFeedback::Success(message));
            }
            WordSetsAction::Removed(sets, message) => {
                next.sets = Ok(sets);
                next.target.clear();
                next.feedback = Some(WordSetsFeedback::Success(message));
            }
        }
        next.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_is_mutually_exclusive_and_deletion_clears_target() {
        let initial = Rc::new(WordSetsState {
            sets: Ok(vec![]), target: "0".into(), feedback: None,
        });
        let error = initial.reduce(WordSetsAction::Feedback(WordSetsFeedback::Error("invalid".into())));
        let deleted = error.reduce(WordSetsAction::Removed(vec![], "Deleted".into()));
        assert!(deleted.target.is_empty());
        assert!(matches!(&deleted.feedback, Some(WordSetsFeedback::Success(message)) if message == "Deleted"));
    }
}
