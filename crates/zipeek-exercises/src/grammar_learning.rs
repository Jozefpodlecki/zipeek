#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearningPhase {
    Explain,
    Practise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LearningStep {
    pub point_index: usize,
    pub phase: LearningPhase,
}

pub fn step_count(point_count: usize) -> usize {
    point_count.saturating_mul(2)
}

pub fn step_at(position: usize, point_count: usize) -> Option<LearningStep> {
    if position >= step_count(point_count) {
        return None;
    }
    if position < point_count {
        Some(LearningStep {
            point_index: position,
            phase: LearningPhase::Explain,
        })
    } else {
        Some(LearningStep {
            point_index: position - point_count,
            phase: LearningPhase::Practise,
        })
    }
}

pub fn can_advance(step: LearningStep, recalled: bool, answer: Option<usize>) -> bool {
    match step.phase {
        LearningPhase::Explain => true,
        LearningPhase::Practise => recalled && answer == Some(step.point_index),
    }
}

pub fn choice_indices(point_index: usize, point_count: usize) -> Vec<usize> {
    if point_count == 0 || point_index >= point_count {
        return Vec::new();
    }
    let start = (point_index * 3 + 1) % point_count;
    (0..point_count)
        .map(|offset| (start + offset) % point_count)
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PracticeState {
    pub revealed: Vec<bool>,
    pub answers: Vec<Option<usize>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PracticeAction {
    Reveal(usize),
    Choose { point: usize, choice: usize },
}

impl PracticeState {
    pub fn new(point_count: usize) -> Self {
        Self {
            revealed: vec![false; point_count],
            answers: vec![None; point_count],
        }
    }

    pub fn apply(&mut self, action: PracticeAction) -> bool {
        match action {
            PracticeAction::Reveal(point) if point < self.revealed.len() => {
                if self.revealed[point] {
                    return false;
                }
                self.revealed[point] = true;
            }
            PracticeAction::Choose { point, choice }
                if point < self.answers.len()
                    && choice < self.answers.len()
                    && self.revealed[point] =>
            {
                if self.answers[point] == Some(point) || self.answers[point] == Some(choice) {
                    return false;
                }
                self.answers[point] = Some(choice);
            }
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_invariants_hold() {
        for count in [4, 5, 6] {
            assert_eq!(step_count(count), count * 2);
            for index in 0..count {
                assert_eq!(
                    step_at(index, count),
                    Some(LearningStep {
                        point_index: index,
                        phase: LearningPhase::Explain
                    })
                );
                assert_eq!(
                    step_at(index + count, count),
                    Some(LearningStep {
                        point_index: index,
                        phase: LearningPhase::Practise
                    })
                );
            }
            assert_eq!(step_at(count * 2, count), None);
        }
    }

    #[test]
    fn practice_must_be_revealed_and_correct_to_advance() {
        let practise = step_at(5, 5).unwrap();
        let mut state = PracticeState::new(5);
        assert!(!state.apply(PracticeAction::Choose {
            point: 0,
            choice: 0
        }));
        assert!(state.apply(PracticeAction::Reveal(0)));
        assert!(!state.apply(PracticeAction::Choose {
            point: 0,
            choice: 99
        }));
        assert!(state.apply(PracticeAction::Choose {
            point: 0,
            choice: 1
        }));
        assert!(!can_advance(practise, state.revealed[0], state.answers[0]));
        assert!(state.apply(PracticeAction::Choose {
            point: 0,
            choice: 0
        }));
        assert!(can_advance(practise, state.revealed[0], state.answers[0]));
        assert!(!state.apply(PracticeAction::Choose {
            point: 0,
            choice: 1
        }));
        assert_eq!(state.answers[0], Some(0));
        assert!(!state.apply(PracticeAction::Reveal(99)));
    }

    #[test]
    fn option_order_covers_each_point_exactly_once() {
        for count in [4, 5, 6] {
            for point in 0..count {
                let mut choices = choice_indices(point, count);
                assert_eq!(choices.iter().filter(|&&choice| choice == point).count(), 1);
                choices.sort_unstable();
                assert_eq!(choices, (0..count).collect::<Vec<_>>());
            }
        }
    }
}
