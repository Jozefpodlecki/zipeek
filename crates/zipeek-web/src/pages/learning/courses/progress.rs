use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const STORAGE_KEY: &str = "zipeek.courses.progress.v1";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CourseProgress {
    #[serde(default)]
    pub pinyin_passed: BTreeSet<String>,
    #[serde(default)]
    pub grammar_steps: BTreeMap<String, usize>,
    #[serde(default)]
    pub grammar_assessments: BTreeMap<String, GrammarAssessmentRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarAssessmentRecord {
    pub correct: usize,
    pub total: usize,
}

impl CourseProgress {
    pub fn mark_pinyin_passed(&mut self, slug: &str) -> Result<(), String> {
        if !super::catalog::PINYIN_LESSONS.iter().any(|lesson| lesson.slug == slug) {
            return Err("Unknown Pinyin module".to_owned());
        }
        self.pinyin_passed.insert(slug.to_owned());
        Ok(())
    }

    pub fn mark_grammar_step(&mut self, course: &str, step: usize, total: usize) -> Result<(), String> {
        let authored_total = super::curriculum_registry::GRAMMAR_REGISTRY.iter()
            .find(|item| item.id == course)
            .and_then(|entry| super::curriculum::authored_grammar_region(entry.region))
            .map(|(course, checks)| course.points.len() + checks.len() + 2);
        let expected_total = authored_total.or_else(|| {
            super::grammar_expansion::expanded_course_by_id(course)
                .map(|item| super::grammar_learning_flow::step_count(item.points.len()))
        });
        if expected_total != Some(total) || step >= total {
            return Err("Invalid grammar checkpoint".to_owned());
        }
        self.grammar_steps.insert(course.to_owned(), step);
        Ok(())
    }

    pub fn record_grammar_assessment(
        &mut self,
        course: &str,
        correct: usize,
        total: usize,
    ) -> Result<(), String> {
        let Some(entry) = super::curriculum_registry::GRAMMAR_REGISTRY.iter().find(|entry| entry.id == course) else {
            return Err("Unknown grammar course".to_owned());
        };
        if total == 0 || correct > total || total != entry.point_ids.len() {
            return Err("Invalid grammar assessment score".to_owned());
        }
        let record = GrammarAssessmentRecord { correct, total };
        let best = self.grammar_assessments.entry(course.to_owned()).or_insert(record.clone());
        if correct * best.total > best.correct * total {
            *best = record;
        }
        Ok(())
    }

    pub fn validated(&self) -> bool {
        self.pinyin_passed.iter().all(|slug| super::catalog::PINYIN_LESSONS.iter().any(|lesson| lesson.slug == slug))
            && self.grammar_steps.iter().all(|(id, step)| {
                super::curriculum_registry::GRAMMAR_REGISTRY.iter().any(|entry| {
                    entry.id == id
                        && super::curriculum::authored_grammar_region(entry.region)
                            .is_some_and(|(course, checks)| *step < course.points.len() + checks.len() + 2)
                }) || super::grammar_expansion::expanded_course_by_id(id)
                    .is_some_and(|course| *step < super::grammar_learning_flow::step_count(course.points.len()))
            })
            && self.grammar_assessments.iter().all(|(id, score)| {
                super::curriculum_registry::GRAMMAR_REGISTRY.iter().any(|entry| entry.id == id && entry.point_ids.len() == score.total)
                    && score.correct <= score.total && score.total > 0
            })
    }
}

pub fn load() -> Result<CourseProgress, String> {
    let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten()) else {
        return Err("Browser storage is unavailable".to_owned());
    };
    let Some(value) = storage.get_item(STORAGE_KEY).map_err(|_| "Cannot read course progress")? else {
        return Ok(CourseProgress::default());
    };
    let progress: CourseProgress = serde_json::from_str(&value).map_err(|_| "Saved course progress is invalid")?;
    if !progress.validated() {
        return Err("Saved course progress contains invalid identifiers or scores".to_owned());
    }
    Ok(progress)
}

pub fn update(change: impl FnOnce(&mut CourseProgress) -> Result<(), String>) -> Result<(), String> {
    let mut progress = load()?;
    change(&mut progress)?;
    let serialized = serde_json::to_string(&progress).map_err(|_| "Cannot encode course progress")?;
    let storage = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .ok_or("Browser storage is unavailable")?;
    storage.set_item(STORAGE_KEY, &serialized).map_err(|_| "Cannot save course progress".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_forged_curriculum_identifiers_and_invalid_scores() {
        let mut progress = CourseProgress::default();
        assert!(progress.mark_pinyin_passed("invented").is_err());
        assert!(progress.mark_grammar_step("grammar-1", 999, 10).is_err());
        assert!(progress.record_grammar_assessment("grammar-1", 5, 4).is_err());
        assert!(progress.record_grammar_assessment("grammar-100", 4, 4).is_err());
        assert!(progress.validated());
    }

    #[test]
    fn expanded_grammar_checkpoints_are_bounded_and_validated() {
        let mut progress = CourseProgress::default();
        assert!(progress.mark_grammar_step("grammar-6", 9, 10).is_ok());
        assert_eq!(progress.grammar_steps["grammar-6"], 9);
        assert!(progress.validated());
        assert!(progress.mark_grammar_step("grammar-6", 10, 10).is_err());
        assert!(progress.mark_grammar_step("grammar-21", 1, 12).is_ok());
        assert!(progress.mark_grammar_step("grammar-100", 9, 10).is_ok());
        assert!(progress.mark_grammar_step("grammar-101", 1, 10).is_err());
    }

    #[test]
    fn records_only_best_valid_grammar_attempt() {
        let mut progress = CourseProgress::default();
        progress.record_grammar_assessment("grammar-1", 3, 4).unwrap();
        progress.record_grammar_assessment("grammar-1", 2, 4).unwrap();
        assert_eq!(progress.grammar_assessments["grammar-1"].correct, 3);
        progress.record_grammar_assessment("grammar-1", 4, 4).unwrap();
        assert_eq!(progress.grammar_assessments["grammar-1"].correct, 4);
    }
}
