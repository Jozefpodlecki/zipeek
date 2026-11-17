use super::assessment::AssessmentItem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssessmentError {
    InvalidQuestion,
    InvalidChoice,
    AlreadySubmitted,
    Incomplete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssessmentSession {
    authored_items: Vec<AssessmentItem>,
    answers: Vec<Option<usize>>,
    submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointReview {
    pub item_id: &'static str,
    pub point_id: &'static str,
    pub chosen: usize,
    pub expected: usize,
    pub correct: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssessmentReview {
    pub points: Vec<PointReview>,
    pub correct: usize,
    pub total: usize,
}

impl AssessmentSession {
    pub fn for_items(items: &[&AssessmentItem]) -> Self {
        Self {
            authored_items: items.iter().map(|item| **item).collect(),
            answers: vec![None; items.len()],
            submitted: false,
        }
    }

    pub fn answer(&mut self, question: usize, choice: usize) -> Result<(), AssessmentError> {
        if self.submitted {
            return Err(AssessmentError::AlreadySubmitted);
        }
        let choice_count = self
            .authored_items
            .get(question)
            .ok_or(AssessmentError::InvalidQuestion)?
            .choices
            .len();
        if choice >= choice_count {
            return Err(AssessmentError::InvalidChoice);
        }
        let response = self
            .answers
            .get_mut(question)
            .ok_or(AssessmentError::InvalidQuestion)?;
        *response = Some(choice);
        Ok(())
    }

    pub fn submit(&mut self) -> Result<(), AssessmentError> {
        if self.submitted {
            return Err(AssessmentError::AlreadySubmitted);
        }
        if self.answers.is_empty() || self.answers.iter().any(Option::is_none) {
            return Err(AssessmentError::Incomplete);
        }
        self.submitted = true;
        Ok(())
    }

    pub fn response(&self, question: usize) -> Option<usize> {
        self.answers.get(question).copied().flatten()
    }

    pub fn answered_count(&self) -> usize {
        self.answers.iter().flatten().count()
    }

    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    pub fn review(&self, items: &[&AssessmentItem]) -> Option<AssessmentReview> {
        if !self.submitted || items.len() != self.authored_items.len() || items.is_empty() {
            return None;
        }
        if !items
            .iter()
            .zip(&self.authored_items)
            .all(|(item, original)| **item == *original)
        {
            return None;
        }
        let points = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let chosen = self.answers[index]?;
                Some(PointReview {
                    item_id: item.id,
                    point_id: item.point_id,
                    chosen,
                    expected: item.correct,
                    correct: chosen == item.correct,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let correct = points.iter().filter(|point| point.correct).count();
        Some(AssessmentReview {
            total: points.len(),
            correct,
            points,
        })
    }

    pub fn score(&self, items: &[&AssessmentItem]) -> Option<usize> {
        self.review(items).map(|review| review.correct)
    }
}

#[cfg(test)]
mod tests {
    use super::super::assessment::items_for_region;
    use super::*;

    #[test]
    fn incomplete_or_invalid_answers_cannot_be_submitted() {
        let items = items_for_region(1);
        let mut session = AssessmentSession::for_items(&items);
        assert_eq!(session.submit(), Err(AssessmentError::Incomplete));
        assert_eq!(
            session.answer(items.len(), 0),
            Err(AssessmentError::InvalidQuestion)
        );
        assert_eq!(session.answer(0, 3), Err(AssessmentError::InvalidChoice));
        assert_eq!(session.answer(0, 1), Ok(()));
        assert_eq!(session.submit(), Err(AssessmentError::Incomplete));
        assert_eq!(session.score(&items), None);
    }

    #[test]
    fn rejects_invalid_choice_without_modifying_previous_response() {
        let items = items_for_region(1);
        let mut session = AssessmentSession::for_items(&items);
        assert_eq!(session.answer(0, 1), Ok(()));
        assert_eq!(
            session.answer(0, items[0].choices.len()),
            Err(AssessmentError::InvalidChoice)
        );
        assert_eq!(session.response(0), Some(1));
        assert_eq!(
            session.answer(usize::MAX, 0),
            Err(AssessmentError::InvalidQuestion)
        );
    }

    #[test]
    fn submit_locks_answers_and_only_then_exposes_score() {
        let items = items_for_region(1);
        let mut session = AssessmentSession::for_items(&items);
        for (i, item) in items.iter().enumerate() {
            session.answer(i, item.correct).unwrap();
        }
        assert_eq!(session.score(&items), None);
        assert_eq!(session.submit(), Ok(()));
        assert_eq!(session.score(&items), Some(4));
        assert_eq!(session.answer(0, 0), Err(AssessmentError::AlreadySubmitted));
        assert_eq!(session.submit(), Err(AssessmentError::AlreadySubmitted));
    }

    #[test]
    fn rejects_scoring_with_different_question_order_or_course() {
        let items = items_for_region(1);
        let mut session = AssessmentSession::for_items(&items);
        for (index, item) in items.iter().enumerate() {
            session.answer(index, item.correct).unwrap();
        }
        session.submit().unwrap();
        let mut reversed = items.clone();
        reversed.reverse();
        assert_eq!(session.score(&reversed), None);
        assert_eq!(session.score(&items_for_region(2)), None);
        assert_eq!(session.score(&items), Some(items.len()));
    }

    #[test]
    fn rejects_changed_question_content_with_matching_ids() {
        let items = items_for_region(1);
        let mut session = AssessmentSession::for_items(&items);
        for (index, item) in items.iter().enumerate() {
            session.answer(index, item.correct).unwrap();
        }
        session.submit().unwrap();
        assert_eq!(session.score(&items), Some(items.len()));
        let mut changed = items.iter().map(|item| **item).collect::<Vec<_>>();
        changed[0].correct = (changed[0].correct + 1) % changed[0].choices.len();
        let changed_refs = changed.iter().collect::<Vec<_>>();
        assert_eq!(session.score(&changed_refs), None);
        changed[0] = *items[0];
        changed[0].rationale = "Different explanation";
        let changed_refs = changed.iter().collect::<Vec<_>>();
        assert_eq!(session.score(&changed_refs), None);
        changed[0] = *items[0];
        changed[0].choices.swap(0, 1);
        let changed_refs = changed.iter().collect::<Vec<_>>();
        assert_eq!(session.score(&changed_refs), None);
    }

    #[test]
    fn review_is_point_specific_locked_and_rejects_content_substitution() {
        let items = items_for_region(3);
        let mut session = AssessmentSession::for_items(&items);
        for (index, item) in items.iter().enumerate() {
            let chosen = if index == 1 {
                (item.correct + 1) % 3
            } else {
                item.correct
            };
            session.answer(index, chosen).unwrap();
        }
        assert_eq!(session.review(&items), None);
        session.submit().unwrap();
        let review = session.review(&items).expect("submitted review");
        assert_eq!(review.total, 5);
        assert_eq!(review.correct, 4);
        assert_eq!(review.points[1].point_id, items[1].point_id);
        assert!(!review.points[1].correct);
        assert_eq!(review.points[1].expected, items[1].correct);
        assert_eq!(
            review.points.iter().filter(|point| !point.correct).count(),
            1
        );
        let mut edited = items.iter().map(|item| **item).collect::<Vec<_>>();
        edited[1].prompt = "Modified prompt";
        assert_eq!(session.review(&edited.iter().collect::<Vec<_>>()), None);
        let mut reordered = items.clone();
        reordered.reverse();
        assert_eq!(session.review(&reordered), None);
    }

    #[test]
    fn draft_result_has_no_persistent_award_or_reveal_state() {
        let session = AssessmentSession::for_items(&items_for_region(2));
        assert_eq!(session.answered_count(), 0);
        assert!(!session.is_submitted());
        assert_eq!(session.score(&items_for_region(2)), None);
    }
}
