use std::collections::BTreeSet;

use super::assessment::{items_for_region, validate_assessments};
use super::construction_assessment::{item_for_region, validate_item};
use super::curriculum::{GrammarCourse, GuidedQuestion, authored_grammar_region};
use super::curriculum_registry::{GRAMMAR_REGISTRY, validate_registry};
use super::guided_bank::{GuidedActivity, GuidedCourse, assembled};

#[derive(Debug, PartialEq, Eq)]
pub enum AuthoringError {
    Registry,
    AssessmentCatalogue,
    EmptyCourseContent(usize),
    GuidedQuestionCount(usize),
    InvalidGuidedQuestion(usize, usize),
    DuplicateGuidedQuestion(usize, usize),
    EmptyGuidedActivities(usize),
    DuplicateActivityId(&'static str),
    InvalidGuidedActivity(&'static str),
    IncorrectAssessmentCount(usize),
    MissingConstruction(usize),
    InvalidConstruction(usize),
    ConstructionPointMismatch(usize),
    DuplicateConstructionId(&'static str),
}

pub fn validate_course_content(
    region: usize,
    course: &GrammarCourse,
    questions: &[GuidedQuestion],
    activities: &[GuidedActivity],
) -> Result<(), AuthoringError> {
    if course.id.trim().is_empty()
        || course.title.trim().is_empty()
        || course.description.trim().is_empty()
        || course.points.is_empty()
        || course.points.iter().any(|point| {
            point.label.trim().is_empty()
                || point.explanation.trim().is_empty()
                || point.example.trim().is_empty()
                || point.translation.trim().is_empty()
        })
    {
        return Err(AuthoringError::EmptyCourseContent(region));
    }
    if questions.len() != course.points.len() {
        return Err(AuthoringError::GuidedQuestionCount(region));
    }
    let mut prompts = BTreeSet::new();
    for (index, question) in questions.iter().enumerate() {
        if question.prompt.trim().is_empty()
            || question.rationale.trim().is_empty()
            || question.correct >= question.choices.len()
            || question
                .choices
                .iter()
                .any(|choice| choice.trim().is_empty())
        {
            return Err(AuthoringError::InvalidGuidedQuestion(region, index));
        }
        let normalized: BTreeSet<String> = question
            .choices
            .iter()
            .map(|choice| {
                choice
                    .chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
            })
            .collect();
        if normalized.len() != question.choices.len()
            || !prompts.insert(question.prompt.trim().to_lowercase())
        {
            return Err(AuthoringError::DuplicateGuidedQuestion(region, index));
        }
    }
    if activities.is_empty() {
        return Err(AuthoringError::EmptyGuidedActivities(region));
    }
    let mut ids = BTreeSet::new();
    for activity in activities {
        if !ids.insert(activity.id) {
            return Err(AuthoringError::DuplicateActivityId(activity.id));
        }
        if activity.id.trim().is_empty()
            || activity.title.trim().is_empty()
            || activity.instruction.trim().is_empty()
            || activity.source.trim().is_empty()
            || activity.target_pattern.trim().is_empty()
            || activity.explanation.trim().is_empty()
            || activity.incorrect_feedback.trim().is_empty()
            || activity.bank.len() < 2
            || activity.bank.iter().any(|tile| tile.trim().is_empty())
            || activity.presented_order.len() != activity.bank.len()
            || activity.accepted_orders.is_empty()
        {
            return Err(AuthoringError::InvalidGuidedActivity(activity.id));
        }
        let bank: BTreeSet<&str> = activity.bank.iter().copied().collect();
        let order: BTreeSet<usize> = activity.presented_order.iter().copied().collect();
        if bank.len() != activity.bank.len()
            || order.iter().copied().ne(0..activity.bank.len())
            || activity.accepted_orders.iter().any(|accepted| {
                accepted.is_empty()
                    || accepted.len() >= activity.bank.len()
                    || assembled(activity, accepted).is_none()
            })
        {
            return Err(AuthoringError::InvalidGuidedActivity(activity.id));
        }
    }
    Ok(())
}

pub fn validate_published_courses() -> Result<(), AuthoringError> {
    validate_registry(&GRAMMAR_REGISTRY).map_err(|_| AuthoringError::Registry)?;
    validate_assessments().map_err(|_| AuthoringError::AssessmentCatalogue)?;
    let mut global_activity_ids = BTreeSet::new();
    let mut construction_ids = BTreeSet::new();
    for entry in &GRAMMAR_REGISTRY {
        let region = entry.region;
        let (course, questions) =
            authored_grammar_region(region).ok_or(AuthoringError::EmptyCourseContent(region))?;
        let activities = GuidedCourse::from_region(region)
            .ok_or(AuthoringError::EmptyGuidedActivities(region))?
            .activities();
        validate_course_content(region, course, questions, activities)?;
        for activity in activities {
            if !global_activity_ids.insert(activity.id) {
                return Err(AuthoringError::DuplicateActivityId(activity.id));
            }
        }
        if items_for_region(region).len() != entry.point_ids.len() {
            return Err(AuthoringError::IncorrectAssessmentCount(region));
        }
        let construction =
            item_for_region(region).ok_or(AuthoringError::MissingConstruction(region))?;
        validate_item(construction).map_err(|_| AuthoringError::InvalidConstruction(region))?;
        if !entry.point_ids.contains(&construction.point_id) {
            return Err(AuthoringError::ConstructionPointMismatch(region));
        }
        if !construction_ids.insert(construction.id) {
            return Err(AuthoringError::DuplicateConstructionId(construction.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::curriculum::{INTRO, INTRO_CHECKS};
    use super::super::guided_bank::ACTIVITIES;
    use super::*;

    #[test]
    fn all_five_published_courses_have_consistent_authored_formats() {
        assert_eq!(validate_published_courses(), Ok(()));
    }

    #[test]
    fn region_mapping_is_complete_and_excludes_unauthored_regions() {
        for region in 1..=5 {
            assert_eq!(GuidedCourse::from_region(region).unwrap().region(), region);
        }
        assert!(GuidedCourse::from_region(0).is_none());
        assert!(GuidedCourse::from_region(6).is_none());
    }

    #[test]
    fn rejects_missing_teaching_content_and_question_coverage() {
        let mut course = GrammarCourse {
            id: INTRO.id,
            title: "",
            description: INTRO.description,
            points: INTRO.points,
        };
        assert_eq!(
            validate_course_content(1, &course, &INTRO_CHECKS, &ACTIVITIES),
            Err(AuthoringError::EmptyCourseContent(1))
        );
        course.title = INTRO.title;
        assert_eq!(
            validate_course_content(1, &course, &INTRO_CHECKS[..3], &ACTIVITIES),
            Err(AuthoringError::GuidedQuestionCount(1))
        );
    }

    #[test]
    fn rejects_bad_guided_answer_keys_and_duplicates() {
        let mut questions = INTRO_CHECKS;
        questions[0].correct = 3;
        assert_eq!(
            validate_course_content(1, &INTRO, &questions, &ACTIVITIES),
            Err(AuthoringError::InvalidGuidedQuestion(1, 0))
        );
        questions = INTRO_CHECKS;
        questions[0].choices[1] = questions[0].choices[0];
        assert_eq!(
            validate_course_content(1, &INTRO, &questions, &ACTIVITIES),
            Err(AuthoringError::DuplicateGuidedQuestion(1, 0))
        );
    }

    #[test]
    fn rejects_duplicate_activity_ids_and_invalid_permutations() {
        let mut activities: [_; ACTIVITIES.len()] = std::array::from_fn(|index| ACTIVITIES[index]);
        activities[1].id = activities[0].id;
        assert_eq!(
            validate_course_content(1, &INTRO, &INTRO_CHECKS, &activities),
            Err(AuthoringError::DuplicateActivityId(activities[0].id))
        );
        activities = std::array::from_fn(|index| ACTIVITIES[index]);
        activities[0].presented_order = &[0, 0, 2, 3];
        assert_eq!(
            validate_course_content(1, &INTRO, &INTRO_CHECKS, &activities),
            Err(AuthoringError::InvalidGuidedActivity(activities[0].id))
        );
        activities = std::array::from_fn(|index| ACTIVITIES[index]);
        activities[0].accepted_orders = &[&[1, 1, 0]];
        assert_eq!(
            validate_course_content(1, &INTRO, &INTRO_CHECKS, &activities),
            Err(AuthoringError::InvalidGuidedActivity(activities[0].id))
        );
    }
}
