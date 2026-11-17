mod classifier;
mod decks;
pub mod dialogue;
pub mod generated_sentence;
pub mod market;
mod navigator;
pub mod radical;
mod sentence;

pub use classifier::{CLASSIFIER_QUESTIONS, ClassifierAction, ClassifierQuestion, ClassifierState};
pub use decks::{classifier_rounds, navigator_rounds, sentence_rounds};
pub use navigator::{Direction, NAVIGATION_ROUTE, NavigatorAction, NavigatorState};
pub use sentence::{SENTENCE_QUESTIONS, SentenceAction, SentenceQuestion, SentenceState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_states_are_distinct_and_start_independently() {
        let mut classifier = ClassifierState::default();
        let mut navigator = NavigatorState::default();
        let mut sentence = SentenceState::default();
        classifier.apply(ClassifierAction::Answer(2));
        navigator.apply(NavigatorAction::Move(Direction::Right));
        sentence.apply(SentenceAction::Select(0));
        assert_eq!(classifier.mistakes, 1);
        assert_eq!(navigator.mistakes, 1);
        assert_eq!(sentence.selected, [0]);
        assert_eq!(sentence.mistakes, 0);
        assert_eq!(navigator.position, (1, 1));
    }
}

fn validated_rounds(rounds: Vec<usize>, length: usize) -> Vec<usize> {
    let mut seen = vec![false; length];
    rounds
        .into_iter()
        .filter(|&index| {
            if index >= length || seen[index] {
                return false;
            }
            seen[index] = true;
            true
        })
        .collect()
}

#[cfg(test)]
mod deck_tests {
    use super::*;

    #[test]
    fn game_decks_filter_invalid_and_duplicate_indices() {
        let classifier = ClassifierState::from_rounds(vec![2, 2, usize::MAX, 0]);
        assert_eq!(classifier.rounds, [2, 0]);
        assert_eq!(classifier.current(), CLASSIFIER_QUESTIONS.get(2));
        let navigator = NavigatorState::from_rounds(vec![1, 1, 42]);
        assert_eq!(navigator.rounds, [1]);
        assert_eq!(navigator.current(), Some(Direction::Up));
        let sentence = SentenceState::from_rounds(vec![3, 3, 1]);
        assert_eq!(sentence.rounds, [3, 1]);
    }

    #[test]
    fn empty_decks_are_terminal_and_do_not_record_mistakes() {
        let mut classifier = ClassifierState::from_rounds(Vec::new());
        let mut navigator = NavigatorState::from_rounds(Vec::new());
        let mut sentence = SentenceState::from_rounds(Vec::new());
        assert!(classifier.complete());
        assert!(navigator.complete());
        assert!(sentence.complete());
        classifier.apply(ClassifierAction::Answer(0));
        navigator.apply(NavigatorAction::Move(Direction::Left));
        sentence.apply(SentenceAction::Check);
        assert_eq!(classifier.mistakes, 0);
        assert_eq!(navigator.mistakes, 0);
        assert_eq!(sentence.mistakes, 0);
    }

    #[test]
    fn restart_preserves_selected_decks() {
        let mut classifier = ClassifierState::from_rounds(vec![0]);
        classifier.apply(ClassifierAction::Answer(CLASSIFIER_QUESTIONS[0].answer));
        assert!(classifier.complete());
        classifier.apply(ClassifierAction::Restart);
        assert_eq!(classifier.rounds, [0]);
        let mut navigator = NavigatorState::from_rounds(vec![0]);
        navigator.apply(NavigatorAction::Move(Direction::Left));
        navigator.apply(NavigatorAction::Restart);
        assert_eq!(navigator.rounds, [0]);
        let mut sentence = SentenceState::from_rounds(vec![0]);
        sentence.apply(SentenceAction::Select(1));
        sentence.apply(SentenceAction::Restart);
        assert_eq!(sentence.rounds, [0]);
        assert!(sentence.selected.is_empty());
    }
}
