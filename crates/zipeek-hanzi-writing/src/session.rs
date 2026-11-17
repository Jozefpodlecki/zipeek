use crate::{
    geometry::Stroke,
    matching::{MatchConfig, StrokeRejection, StrokeScore, match_stroke},
};

#[derive(Debug, Clone, PartialEq)]
pub struct WritingSession {
    reference: Vec<Stroke>,
    scores: Vec<Option<StrokeScore>>,
    mistakes: usize,
    last_rejection: Option<StrokeRejection>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrokeOutcome {
    Accepted {
        completed: usize,
        total: usize,
        score: StrokeScore,
    },
    Rejected(StrokeRejection),
    Complete,
}

impl WritingSession {
    pub fn new(reference: Vec<Stroke>) -> Option<Self> {
        if reference.is_empty()
            || reference
                .iter()
                .any(|s| s.points.is_empty() || !s.length().is_finite())
        {
            return None;
        }
        Some(Self {
            reference,
            scores: Vec::new(),
            mistakes: 0,
            last_rejection: None,
        })
    }
    pub fn total(&self) -> usize {
        self.reference.len()
    }
    pub fn completed(&self) -> usize {
        self.scores.len()
    }
    pub fn is_complete(&self) -> bool {
        self.completed() == self.total()
    }
    pub fn assisted_strokes(&self) -> usize {
        self.scores.iter().filter(|score| score.is_none()).count()
    }
    pub fn mistakes(&self) -> usize {
        self.mistakes
    }
    pub fn last_rejection(&self) -> Option<StrokeRejection> {
        self.last_rejection
    }
    pub fn expected_stroke(&self) -> Option<&Stroke> {
        self.reference.get(self.completed())
    }
    pub fn reference_strokes(&self) -> &[Stroke] {
        &self.reference
    }
    pub fn progress_scores(&self) -> &[Option<StrokeScore>] {
        &self.scores
    }
    pub fn apply(&mut self, candidate: &Stroke, config: MatchConfig) -> StrokeOutcome {
        let Some(reference) = self.expected_stroke() else {
            return StrokeOutcome::Complete;
        };
        match match_stroke(reference, candidate, config) {
            Ok(score) => {
                let later_is_clearly_better = self.reference[self.completed() + 1..]
                    .iter()
                    .filter_map(|later| match_stroke(later, candidate, config).ok())
                    .any(|later| {
                        later.average_distance + 0.025 < score.average_distance
                            && later.average_distance < score.average_distance * 0.55
                            && later.start_distance <= score.start_distance
                            && later.end_distance <= score.end_distance
                    });
                if later_is_clearly_better {
                    self.mistakes += 1;
                    self.last_rejection = Some(StrokeRejection::WrongOrder);
                    return StrokeOutcome::Rejected(StrokeRejection::WrongOrder);
                }
                self.scores.push(Some(score));
                self.last_rejection = None;
                StrokeOutcome::Accepted {
                    completed: self.completed(),
                    total: self.total(),
                    score,
                }
            }
            Err(reason) => {
                let rejection = if reason != StrokeRejection::InvalidGeometry
                    && self.reference[self.completed() + 1..]
                        .iter()
                        .any(|later| match_stroke(later, candidate, config).is_ok())
                {
                    StrokeRejection::WrongOrder
                } else {
                    reason
                };
                self.mistakes += 1;
                self.last_rejection = Some(rejection);
                StrokeOutcome::Rejected(rejection)
            }
        }
    }
    pub fn assist_next(&mut self) -> bool {
        if self.is_complete() {
            return false;
        }
        self.scores.push(None);
        self.last_rejection = None;
        true
    }
    pub fn undo(&mut self) -> bool {
        if self.scores.pop().is_some() {
            self.last_rejection = None;
            true
        } else {
            false
        }
    }
    pub fn reset(&mut self) {
        self.scores.clear();
        self.last_rejection = None;
        self.mistakes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;
    fn stroke(a: (f64, f64), b: (f64, f64)) -> Stroke {
        Stroke::new(vec![
            Point::new(a.0, a.1).unwrap(),
            Point::new(b.0, b.1).unwrap(),
        ])
        .unwrap()
    }
    #[test]
    fn checks_order_and_only_advances_for_accepted_strokes() {
        let horizontal = stroke((0.2, 0.2), (0.8, 0.2));
        let vertical = stroke((0.5, 0.2), (0.5, 0.8));
        let mut session = WritingSession::new(vec![horizontal.clone(), vertical.clone()]).unwrap();
        assert_eq!(
            session.apply(&vertical, MatchConfig::default()),
            StrokeOutcome::Rejected(StrokeRejection::WrongOrder)
        );
        assert_eq!(session.completed(), 0);
        assert_eq!(session.mistakes(), 1);
        assert!(matches!(
            session.apply(&horizontal, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 1, .. }
        ));
        assert!(matches!(
            session.apply(&vertical, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 2, .. }
        ));
        assert!(session.is_complete());
        assert_eq!(
            session.apply(&vertical, MatchConfig::default()),
            StrokeOutcome::Complete
        );
        assert!(session.undo());
        assert!(!session.is_complete());
        session.reset();
        assert_eq!(session.completed(), 0);
        assert_eq!(session.mistakes(), 0);
    }
    #[test]
    fn wrong_order_does_not_advance_and_recovers_on_expected_stroke() {
        let first = stroke((0.1, 0.2), (0.9, 0.2));
        let second = stroke((0.5, 0.2), (0.5, 0.8));
        let third = stroke((0.1, 0.8), (0.9, 0.8));
        let mut session =
            WritingSession::new(vec![first.clone(), second.clone(), third.clone()]).unwrap();
        assert_eq!(
            session.apply(&third, MatchConfig::default()),
            StrokeOutcome::Rejected(StrokeRejection::WrongOrder)
        );
        assert_eq!(session.completed(), 0);
        assert_eq!(session.last_rejection(), Some(StrokeRejection::WrongOrder));
        assert!(matches!(
            session.apply(&first, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 1, .. }
        ));
        assert_eq!(session.last_rejection(), None);
        assert_eq!(
            session.apply(&third, MatchConfig::default()),
            StrokeOutcome::Rejected(StrokeRejection::WrongOrder)
        );
        assert!(matches!(
            session.apply(&second, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 2, .. }
        ));
        assert!(matches!(
            session.apply(&third, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 3, .. }
        ));
        assert!(session.is_complete());
    }

    #[test]
    fn malformed_stroke_does_not_get_wrong_order_feedback() {
        let first = stroke((0.1, 0.2), (0.9, 0.2));
        let second = stroke((0.1, 0.8), (0.9, 0.8));
        let mut session = WritingSession::new(vec![first, second]).unwrap();
        let invalid = Stroke {
            points: vec![crate::Point {
                x: f64::NAN,
                y: 0.0,
            }],
        };
        assert_eq!(
            session.apply(&invalid, MatchConfig::default()),
            StrokeOutcome::Rejected(StrokeRejection::InvalidGeometry)
        );
    }

    #[test]
    fn indistinguishable_overlapping_strokes_follow_reference_order() {
        let first = stroke((0.2, 0.4), (0.8, 0.4));
        let mut session = WritingSession::new(vec![first.clone(), first.clone()]).unwrap();
        assert!(matches!(
            session.apply(&first, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 1, .. }
        ));
        assert!(matches!(
            session.apply(&first, MatchConfig::default()),
            StrokeOutcome::Accepted { completed: 2, .. }
        ));
    }

    #[test]
    fn assisted_strokes_are_not_scored_as_handwriting_and_undo_restores_them() {
        let a = stroke((0.1, 0.2), (0.9, 0.2));
        let mut session = WritingSession::new(vec![a.clone(), a.clone()]).unwrap();
        assert!(session.assist_next());
        assert_eq!(session.completed(), 1);
        assert_eq!(session.assisted_strokes(), 1);
        assert_eq!(session.progress_scores(), &[None]);
        assert!(matches!(session.apply(&a, MatchConfig::default()), StrokeOutcome::Accepted { .. }));
        assert_eq!(session.assisted_strokes(), 1);
        assert!(!session.assist_next());
        assert!(session.undo());
        assert!(session.undo());
        assert_eq!(session.assisted_strokes(), 0);
        session.reset();
        assert_eq!(session.completed(), 0);
    }

    #[test]
    fn empty_reference_is_unavailable() {
        assert!(WritingSession::new(vec![]).is_none());
    }
}
