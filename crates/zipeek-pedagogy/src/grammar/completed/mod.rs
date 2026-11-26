mod regions_21_40;
mod regions_41_60;
mod regions_61_80;
mod regions_81_100;

use super::curriculum::GrammarCourse;

pub fn completed_course(region: usize) -> Option<&'static GrammarCourse> {
    match region {
        21..=40 => regions_21_40::COURSES.get(region - 21),
        41..=60 => regions_41_60::COURSES.get(region - 41),
        61..=80 => regions_61_80::COURSES.get(region - 61),
        81..=100 => regions_81_100::COURSES.get(region - 81),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn all_eighty_new_regions_have_distinct_teaching_content() {
        let mut labels = BTreeSet::new();
        let mut examples = BTreeSet::new();
        let mut total = 0;
        for region in 21..=100 {
            let course = completed_course(region).unwrap();
            assert_eq!(course.id, format!("grammar-{region}"));
            assert_eq!(
                course.title,
                super::super::plan::planned_title(region).unwrap()
            );
            assert!((5..=6).contains(&course.points.len()));
            for point in course.points {
                total += 1;
                assert!(labels.insert(point.label));
                assert!(examples.insert(point.example));
                assert!(!point.explanation.is_empty());
                assert!(!point.translation.is_empty());
                assert!(
                    point
                        .example
                        .chars()
                        .any(|ch| ('\u{3400}'..='\u{9fff}').contains(&ch))
                );
            }
        }
        assert_eq!(total, 451);
        assert!(completed_course(20).is_none());
        assert!(completed_course(101).is_none());
    }
}
