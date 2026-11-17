use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformationItem {
    pub id: &'static str,
    pub original: &'static str,
    pub target_structure: &'static str,
    pub instruction: &'static str,
    pub bank: &'static [&'static str],
    pub accepted_orders: &'static [&'static [usize]],
    pub explanation: &'static str,
    pub incorrect_feedback: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerMode {
    #[default]
    Tiles,
    Typing,
    Both,
}

pub fn item_for_seed(seed: u64, pattern_index: u8) -> Option<&'static TransformationItem> {
    if pattern_index == 0 {
        ITEMS.get(seed as usize % ITEMS.len())
    } else {
        ITEMS.get(usize::from(pattern_index - 1))
    }
}

pub fn selected_pattern_for_query(current: u8, query: &str) -> u8 {
    pattern_index_for_label(query).unwrap_or(current)
}

pub fn pattern_index_for_label(label: &str) -> Option<u8> {
    ITEMS.iter()
        .position(|item| item.target_structure == label.trim())
        .and_then(|index| u8::try_from(index + 1).ok())
}

pub fn grade_typed(item: &TransformationItem, typed: &str) -> bool {
    let normalized = normalize_answer(typed);
    !normalized.is_empty()
        && validate(item)
        && item.accepted_orders.iter().any(|order| {
            answer(item, order)
                .is_some_and(|expected| normalize_answer(&expected) == normalized)
        })
}

fn normalize_answer(sentence: &str) -> String {
    let compact: String = sentence.chars().filter(|c| !c.is_whitespace()).collect();
    compact
        .trim_end_matches(|c| matches!(c, '。' | '！' | '？' | '!' | '?'))
        .to_owned()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformationState {
    pub selected: Vec<usize>,
    pub typed: String,
    pub use_tiles: bool,
    pub checked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransformationAction {
    AddTile(usize),
    RemoveTile(usize),
    Undo,
    ResetTiles,
    SetTyped(String),
    UseTiles(bool),
    Check,
}

impl TransformationState {
    pub fn new(mode: AnswerMode) -> Self {
        Self {
            selected: Vec::new(),
            typed: String::new(),
            use_tiles: mode != AnswerMode::Typing,
            checked: false,
        }
    }

    pub fn can_check(&self, item: &TransformationItem) -> bool {
        if self.checked {
            return false;
        }
        if self.use_tiles {
            answer(item, &self.selected).is_some()
        } else {
            !self.typed.trim().is_empty()
        }
    }

    pub fn is_correct(&self, item: &TransformationItem) -> bool {
        if self.use_tiles {
            grade(item, &self.selected)
        } else {
            grade_typed(item, &self.typed)
        }
    }

    pub fn apply(
        &mut self,
        item: &TransformationItem,
        mode: AnswerMode,
        action: TransformationAction,
    ) -> bool {
        if self.checked {
            return false;
        }
        let before = self.clone();
        match action {
            TransformationAction::AddTile(id)
                if self.use_tiles && id < item.bank.len() && !self.selected.contains(&id) =>
            {
                self.selected.push(id);
            }
            TransformationAction::RemoveTile(position)
                if self.use_tiles && position < self.selected.len() =>
            {
                self.selected.remove(position);
            }
            TransformationAction::Undo if self.use_tiles => {
                self.selected.pop();
            }
            TransformationAction::ResetTiles if self.use_tiles => {
                self.selected.clear();
            }
            TransformationAction::SetTyped(value) if !self.use_tiles => {
                self.typed = value;
            }
            TransformationAction::UseTiles(value) if mode == AnswerMode::Both => {
                self.use_tiles = value;
            }
            TransformationAction::Check if self.can_check(item) => {
                self.checked = true;
            }
            _ => {}
        }
        *self != before
    }

}

pub const ITEMS: &[TransformationItem] = &[
    TransformationItem {
        id: "shi-de-time-01",
        original: "我昨天买了这本书。",
        target_structure: "是……的",
        instruction: "Emphasize when the book was bought without changing the event.",
        bank: &["这本书", "昨天", "我", "的", "是", "买", "了"],
        accepted_orders: &[&[2, 4, 1, 5, 0, 3]],
        explanation: "是……的 highlights the time of a completed event: 我是昨天买这本书的。",
        incorrect_feedback: "Keep 昨天 inside 是……的 and place 的 after the completed event. 了 is not needed in this authored transformation.",
    },
    TransformationItem {
        id: "ba-disposal-01",
        original: "我关上了门。",
        target_structure: "把",
        instruction: "Rewrite with 把 to emphasize what happened to the door.",
        bank: &["了", "门", "我", "把", "关上", "在"],
        accepted_orders: &[&[2, 3, 1, 4, 0]],
        explanation: "把 introduces the affected object before the verb and result: 我把门关上了。",
        incorrect_feedback: "Place 门 after 把, and keep the result complement 关上 after the object.",
    },
    TransformationItem {
        id: "bi-comparison-01",
        original: "哥哥很高，弟弟没有哥哥高。",
        target_structure: "比",
        instruction: "State that the older brother is taller than the younger brother.",
        bank: &["弟弟", "哥哥", "高", "比", "很", "更"],
        accepted_orders: &[&[1, 3, 0, 2], &[1, 3, 0, 5, 2]],
        explanation: "比 places the comparison target before the adjective: 哥哥比弟弟高。 更 is also natural here.",
        incorrect_feedback: "Use 哥哥 as the subject, 比弟弟 as the comparison, and 高 as the predicate; 很 is not used before 高 in this pattern.",
    },
];

pub fn validate(item: &TransformationItem) -> bool {
    let len = item.bank.len();
    !item.id.trim().is_empty()
        && !item.original.trim().is_empty()
        && !item.target_structure.trim().is_empty()
        && !item.instruction.trim().is_empty()
        && !item.explanation.trim().is_empty()
        && !item.incorrect_feedback.trim().is_empty()
        && len >= 2
        && item.bank.iter().all(|tile| !tile.trim().is_empty())
        && !item.accepted_orders.is_empty()
        && item
            .accepted_orders
            .iter()
            .enumerate()
            .all(|(position, order)| {
                !order.is_empty()
                    && !item.accepted_orders[..position].contains(order)
                    && order.iter().all(|&id| id < len)
                    && {
                        let mut seen = vec![false; len];
                        order.iter().all(|&id| {
                            if seen[id] {
                                return false;
                            }
                            seen[id] = true;
                            true
                        })
                    }
            })
}

pub fn answer(item: &TransformationItem, selected: &[usize]) -> Option<String> {
    if !validate(item) || selected.is_empty() {
        return None;
    }
    let mut seen = vec![false; item.bank.len()];
    let mut result = String::new();
    for &id in selected {
        if id >= item.bank.len() || seen[id] {
            return None;
        }
        seen[id] = true;
        result.push_str(item.bank[id]);
    }
    Some(result)
}

pub fn grade(item: &TransformationItem, selected: &[usize]) -> bool {
    answer(item, selected).is_some() && item.accepted_orders.iter().any(|order| *order == selected)
}

pub fn shuffled_ids(item: &TransformationItem, seed: u64) -> Vec<usize> {
    let mut ids: Vec<_> = (0..item.bank.len()).collect();
    let mut state = seed ^ 0x9e3779b97f4a7c15;
    for i in (1..ids.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let j = (state.wrapping_mul(0x2545f4914f6cdd1d) % (i as u64 + 1)) as usize;
        ids.swap(i, j);
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformation_state_enforces_mode_validation_and_submitted_lock() {
        let item = &ITEMS[1];
        let mut state = TransformationState::new(AnswerMode::Both);
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::Check));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::AddTile(2)));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::AddTile(2)));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::AddTile(99)));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::UseTiles(false)));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::SetTyped("我把门关上了。".into())));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::Check));
        assert!(state.is_correct(item));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::SetTyped("wrong".into())));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::UseTiles(true)));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::Undo));
    }

    #[test]
    fn inactive_answer_method_cannot_be_mutated() {
        let item = &ITEMS[1];
        let mut state = TransformationState::new(AnswerMode::Both);
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::SetTyped("wrong".into())));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::AddTile(2)));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::UseTiles(false)));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::AddTile(3)));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::Undo));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::ResetTiles));
        assert!(!state.apply(item, AnswerMode::Both, TransformationAction::RemoveTile(0)));
        assert_eq!(state.selected, vec![2]);
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::SetTyped("我把门关上了".into())));
        assert!(state.apply(item, AnswerMode::Both, TransformationAction::Check));
        assert!(!state.can_check(item));
    }

    #[test]
    fn single_answer_mode_cannot_be_switched() {
        let item = &ITEMS[0];
        let mut state = TransformationState::new(AnswerMode::Typing);
        assert!(!state.apply(item, AnswerMode::Typing, TransformationAction::UseTiles(true)));
        assert!(!state.use_tiles);
    }

    #[test]
    fn pattern_search_does_not_silently_change_selection() {
        assert_eq!(selected_pattern_for_query(2, "unsupported"), 2);
        assert_eq!(selected_pattern_for_query(2, ""), 2);
        assert_eq!(selected_pattern_for_query(2, "比"), 3);
    }

    #[test]
    fn pattern_selection_and_typed_answers_are_curated() {
        assert_eq!(pattern_index_for_label("把"), Some(2));
        assert_eq!(pattern_index_for_label("not supported"), None);
        assert_eq!(item_for_seed(99, 2).unwrap().target_structure, "把");
        assert!(item_for_seed(1, 4).is_none());
        assert!(grade_typed(&ITEMS[1], "我把门关上了。"));
        assert!(grade_typed(&ITEMS[1], "我 把 门 关上 了"));
        assert!(!grade_typed(&ITEMS[1], "我关上了门"));
        assert!(grade_typed(&ITEMS[2], "哥哥比弟弟更高"));
        assert!(!grade_typed(&ITEMS[2], "哥哥很高"));
    }

    #[test]
    fn authored_items_have_stable_unique_ids_and_valid_answers() {
        for (index, item) in ITEMS.iter().enumerate() {
            assert!(validate(item));
            assert!(!ITEMS[..index].iter().any(|other| other.id == item.id));
            for accepted in item.accepted_orders {
                assert!(grade(item, accepted));
                assert!(answer(item, accepted).is_some());
            }
        }
    }

    #[test]
    fn accepts_curated_alternatives_but_rejects_repeated_tiles_and_distractors() {
        let comparison = &ITEMS[2];
        assert!(grade(comparison, &[1, 3, 0, 2]));
        assert!(grade(comparison, &[1, 3, 0, 5, 2]));
        assert!(!grade(comparison, &[1, 3, 0, 4, 2]));
        assert!(!grade(comparison, &[1, 3, 0, 0, 2]));
        assert_eq!(answer(comparison, &[1, 3, 0, 0, 2]), None);
        assert_eq!(answer(comparison, &[99]), None);
    }

    #[test]
    fn invalid_authored_orders_are_never_graded() {
        let malformed = TransformationItem {
            accepted_orders: &[&[0, 0]],
            ..ITEMS[0]
        };
        assert!(!validate(&malformed));
        assert!(!grade(&malformed, &[0, 0]));
    }

    #[test]
    fn shuffle_is_repeatable_and_preserves_every_bank_id() {
        for item in ITEMS {
            let original = shuffled_ids(item, 371);
            assert_eq!(original, shuffled_ids(item, 371));
            let mut sorted = original;
            sorted.sort_unstable();
            assert_eq!(sorted, (0..item.bank.len()).collect::<Vec<_>>());
        }
    }
}
