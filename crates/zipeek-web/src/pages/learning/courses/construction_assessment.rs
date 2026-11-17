use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstructionItem {
    pub id: &'static str,
    pub point_id: &'static str,
    pub instruction: &'static str,
    pub bank: &'static [&'static str],
    pub display_order: &'static [usize],
    pub expected_order: &'static [usize],
    pub explanation: &'static str,
}

pub const ITEMS: [ConstructionItem; 5] = [
    ConstructionItem {
        id: "construct-g01-negation",
        point_id: "g01-bu-negation",
        instruction: "Construct: ‘I do not study Chinese.’",
        bank: &["中文", "我", "不", "学习", "吗"],
        display_order: &[4, 0, 3, 1, 2],
        expected_order: &[1, 2, 3, 0],
        explanation: "不 goes before 学习, followed by the object 中文.",
    },
    ConstructionItem {
        id: "construct-g02-time-place",
        point_id: "g02-time-location",
        instruction: "Construct: ‘I study at school today.’",
        bank: &["学习", "今天", "我", "学校", "在", "去"],
        display_order: &[5, 0, 3, 1, 4, 2],
        expected_order: &[2, 1, 4, 3, 0],
        explanation: "今天 precedes the location phrase 在学校, which precedes 学习.",
    },
    ConstructionItem {
        id: "construct-g03-classifier",
        point_id: "g03-classifiers",
        instruction: "Construct: ‘I have three apples.’",
        bank: &["苹果", "三", "我", "有", "个", "本"],
        display_order: &[5, 0, 3, 1, 4, 2],
        expected_order: &[2, 3, 1, 4, 0],
        explanation: "Use 我有 followed by 三个苹果; 个 is the classifier here.",
    },
    ConstructionItem {
        id: "construct-g04-bi-comparison",
        point_id: "g04-bi-comparison",
        instruction: "Construct: 'Today is colder than yesterday.'",
        bank: &["今天", "昨天", "冷", "比", "很"],
        display_order: &[4, 2, 1, 3, 0],
        expected_order: &[0, 3, 1, 2],
        explanation: "Place the compared target after 比 and before 冷; no 是 is needed.",
    },
    ConstructionItem {
        id: "construct-g05-keyi-permission",
        point_id: "g05-keyi-permission",
        instruction: "Construct: 'May I sit here?'",
        bank: &["坐", "我", "这里", "可以", "吗", "要"],
        display_order: &[5, 2, 0, 4, 1, 3],
        expected_order: &[1, 3, 0, 2, 4],
        explanation: "Place 可以 before 坐这里 and add 吗 at the end to ask permission.",
    },
];

pub fn item_for_region(region: usize) -> Option<&'static ConstructionItem> {
    ITEMS.get(region.checked_sub(1)?)
}

pub fn validate_item(item: &ConstructionItem) -> Result<(), &'static str> {
    if item.id.is_empty()
        || item.point_id.is_empty()
        || item.instruction.trim().is_empty()
        || item.explanation.trim().is_empty()
    {
        return Err("missing construction metadata");
    }
    if item.bank.len() < 3
        || item.expected_order.is_empty()
        || item.expected_order.len() >= item.bank.len()
        || item.display_order.len() != item.bank.len()
        || item.bank.iter().any(|tile| tile.trim().is_empty())
    {
        return Err("invalid construction tile bank");
    }
    let unique_words: BTreeSet<_> = item.bank.iter().map(|word| word.trim()).collect();
    if unique_words.len() != item.bank.len() {
        return Err("ambiguous duplicate construction tiles");
    }
    let unique_display: BTreeSet<_> = item.display_order.iter().copied().collect();
    let unique_expected: BTreeSet<_> = item.expected_order.iter().copied().collect();
    if unique_display.len() != item.bank.len()
        || unique_display.iter().copied().ne(0..item.bank.len())
        || unique_expected.len() != item.expected_order.len()
        || unique_expected
            .iter()
            .any(|&index| index >= item.bank.len())
    {
        return Err("invalid construction tile indices");
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionError {
    InvalidItem,
    InvalidTile,
    DuplicateTile,
    AlreadySubmitted,
    Incomplete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructionSession {
    item: ConstructionItem,
    selected: Vec<usize>,
    submitted: bool,
}

impl ConstructionSession {
    pub fn new(item: &ConstructionItem) -> Result<Self, ConstructionError> {
        validate_item(item).map_err(|_| ConstructionError::InvalidItem)?;
        Ok(Self {
            item: *item,
            selected: Vec::new(),
            submitted: false,
        })
    }

    pub fn selected(&self) -> &[usize] {
        &self.selected
    }
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }
    pub fn is_ready(&self) -> bool {
        self.selected.len() == self.item.expected_order.len()
    }

    pub fn add(&mut self, index: usize) -> Result<(), ConstructionError> {
        if self.submitted {
            return Err(ConstructionError::AlreadySubmitted);
        }
        if index >= self.item.bank.len() {
            return Err(ConstructionError::InvalidTile);
        }
        if self.selected.contains(&index) {
            return Err(ConstructionError::DuplicateTile);
        }
        if self.is_ready() {
            return Err(ConstructionError::InvalidTile);
        }
        self.selected.push(index);
        Ok(())
    }

    pub fn remove(&mut self, position: usize) -> Result<(), ConstructionError> {
        if self.submitted {
            return Err(ConstructionError::AlreadySubmitted);
        }
        if position >= self.selected.len() {
            return Err(ConstructionError::InvalidTile);
        }
        self.selected.remove(position);
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), ConstructionError> {
        if self.submitted {
            return Err(ConstructionError::AlreadySubmitted);
        }
        self.selected.pop();
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), ConstructionError> {
        if self.submitted {
            return Err(ConstructionError::AlreadySubmitted);
        }
        self.selected.clear();
        Ok(())
    }

    pub fn submit(&mut self) -> Result<(), ConstructionError> {
        if self.submitted {
            return Err(ConstructionError::AlreadySubmitted);
        }
        if !self.is_ready() {
            return Err(ConstructionError::Incomplete);
        }
        self.submitted = true;
        Ok(())
    }

    pub fn review(&self, item: &ConstructionItem) -> Option<bool> {
        (self.submitted && self.item == *item)
            .then_some(self.selected.as_slice() == item.expected_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_courses_have_valid_distinct_construction_items() {
        for (region, item) in ITEMS.iter().enumerate() {
            assert_eq!(item_for_region(region + 1), Some(item));
            assert_eq!(validate_item(item), Ok(()));
        }
        assert!(item_for_region(0).is_none());
        assert!(item_for_region(5).is_none());
        assert_eq!(
            ITEMS
                .iter()
                .map(|item| item.id)
                .collect::<BTreeSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn cannot_reveal_feedback_before_complete_locked_submission() {
        let item = item_for_region(1).unwrap();
        let mut session = ConstructionSession::new(item).unwrap();
        assert_eq!(session.review(item), None);
        assert_eq!(session.submit(), Err(ConstructionError::Incomplete));
        assert_eq!(
            session.add(item.bank.len()),
            Err(ConstructionError::InvalidTile)
        );
        session.add(item.expected_order[0]).unwrap();
        assert_eq!(
            session.add(item.expected_order[0]),
            Err(ConstructionError::DuplicateTile)
        );
        assert_eq!(session.review(item), None);
        for &tile in &item.expected_order[1..] {
            session.add(tile).unwrap();
        }
        assert!(session.is_ready());
        assert_eq!(session.add(4), Err(ConstructionError::InvalidTile));
        session.submit().unwrap();
        assert_eq!(session.review(item), Some(true));
        assert_eq!(session.undo(), Err(ConstructionError::AlreadySubmitted));
        assert_eq!(session.reset(), Err(ConstructionError::AlreadySubmitted));
        assert_eq!(session.submit(), Err(ConstructionError::AlreadySubmitted));
    }

    #[test]
    fn incorrect_answer_is_scored_only_after_submission() {
        let item = item_for_region(2).unwrap();
        let mut session = ConstructionSession::new(item).unwrap();
        for &tile in item.expected_order.iter().rev() {
            session.add(tile).unwrap();
        }
        assert_eq!(session.review(item), None);
        session.submit().unwrap();
        assert_eq!(session.review(item), Some(false));
    }

    #[test]
    fn editing_before_submission_and_changed_item_rejection() {
        let item = item_for_region(3).unwrap();
        let mut session = ConstructionSession::new(item).unwrap();
        session.add(item.display_order[0]).unwrap();
        session.undo().unwrap();
        session.add(item.display_order[1]).unwrap();
        session.remove(0).unwrap();
        for &tile in item.expected_order {
            session.add(tile).unwrap();
        }
        session.submit().unwrap();
        let mut altered = *item;
        altered.explanation = "Modified";
        assert_eq!(session.review(&altered), None);
        assert_eq!(session.review(item), Some(true));
    }

    #[test]
    fn malformed_tile_banks_and_orders_are_rejected() {
        let mut item = ITEMS[0];
        item.display_order = &[0, 0, 2, 3, 4];
        assert!(validate_item(&item).is_err());
        item = ITEMS[0];
        item.expected_order = &[0, 0, 3];
        assert!(validate_item(&item).is_err());
        item = ITEMS[0];
        item.expected_order = &[0, 1, 3, 99];
        assert!(validate_item(&item).is_err());
    }
}
