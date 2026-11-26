mod completed;
pub mod curriculum;
pub mod expansion;
pub mod pinyin;
pub mod plan;

pub fn course(region: usize) -> Option<&'static curriculum::GrammarCourse> {
    match region {
        1..=5 => curriculum::authored_grammar_region(region).map(|(course, _)| course),
        6..=100 => expansion::expanded_course(region),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entire_catalog_is_contiguous_and_valid() {
        let mut total = 0;
        for region in 1..=100 {
            let course = course(region).expect("each grammar region has a course");
            assert_eq!(course.id, format!("grammar-{region}"));
            assert!((4..=6).contains(&course.points.len()));
            total += course.points.len();
            assert!(course.points.iter().all(|p| !p.label.is_empty()
                && !p.explanation.is_empty()
                && !p.example.is_empty()
                && !p.translation.is_empty()));
        }
        assert_eq!(total, 550);
        assert!(course(0).is_none());
        assert!(course(101).is_none());
    }
}
