use std::collections::HashSet;

use super::{CLASSIFIER_QUESTIONS, NAVIGATION_ROUTE, SENTENCE_QUESTIONS};

pub fn classifier_rounds<'a>(words: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let known: HashSet<&str> = words
        .into_iter()
        .map(str::trim)
        .filter(|word| !word.is_empty())
        .collect();
    CLASSIFIER_QUESTIONS
        .iter()
        .enumerate()
        .filter_map(|(index, question)| known.contains(question.noun).then_some(index))
        .collect()
}

pub fn navigator_rounds<'a>(words: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let known: HashSet<&str> = words
        .into_iter()
        .map(str::trim)
        .filter(|word| !word.is_empty())
        .collect();
    let mut position = (1_i8, 1_i8);
    NAVIGATION_ROUTE
        .iter()
        .enumerate()
        .filter_map(|(index, direction)| {
            if !direction
                .vocabulary_forms()
                .iter()
                .any(|word| known.contains(word))
            {
                return None;
            }
            let (dx, dy) = direction.offset();
            let next = (position.0 + dx, position.1 + dy);
            if !(0..=2).contains(&next.0) || !(0..=2).contains(&next.1) {
                return None;
            }
            position = next;
            Some(index)
        })
        .collect()
}

pub fn sentence_rounds<'a>(words: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let known: HashSet<&str> = words
        .into_iter()
        .map(str::trim)
        .filter(|word| !word.is_empty())
        .collect();
    SENTENCE_QUESTIONS
        .iter()
        .enumerate()
        .filter_map(|(index, question)| {
            question.tokens[..question.count]
                .iter()
                .all(|token| known.contains(token))
                .then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_selects_only_authored_eligible_questions() {
        assert_eq!(classifier_rounds(["书", "猫", "书", "unknown"]), [0, 1]);
        assert!(classifier_rounds(["杯", "本"]).is_empty());
    }

    #[test]
    fn navigator_filters_by_full_spoken_phrase() {
        assert_eq!(navigator_rounds(["向左走", "向右走"]), [0, 2, 3, 5]);
        assert_eq!(navigator_rounds(["向右走"]), [2]);
        assert_eq!(navigator_rounds(["左", "右"]), [0, 2, 3, 5]);
        assert_eq!(navigator_rounds(["左边", "右边"]), [0, 2, 3, 5]);
        assert!(navigator_rounds(["地图"]).is_empty());
    }

    #[test]
    fn navigator_selected_decks_remain_playable() {
        for vocabulary in [
            vec!["向右走"],
            vec!["左", "右"],
            vec!["前", "后"],
            vec!["向左走", "向前走", "向右走", "向后走"],
        ] {
            let rounds = navigator_rounds(vocabulary);
            let mut state = super::super::NavigatorState::from_rounds(rounds.clone());
            assert_eq!(state.rounds, rounds);
            for index in rounds {
                state.apply(super::super::NavigatorAction::Move(
                    super::NAVIGATION_ROUTE[index],
                ));
            }
            assert!(state.complete());
        }
    }

    #[test]
    fn sentence_requires_every_authored_tile() {
        assert_eq!(sentence_rounds(["我", "喜欢", "喝", "茶"]), [0]);
        assert!(sentence_rounds(["我", "喜欢", "喝"]).is_empty());
    }
}
