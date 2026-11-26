use std::collections::BTreeMap;
use zipeek_core::Word;
use zipeek_pedagogy::curriculum::VocabularyRegion;

fn validate_course_words(words: Vec<Word>, expected: usize) -> Result<Vec<Word>, String> {
    if words.len() != expected {
        return Err("Course vocabulary could not be resolved completely".to_owned());
    }
    let mut ids: Vec<_> = words.iter().map(|word| word.id.as_i32()).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() != expected || ids.iter().any(|id| *id <= 0) {
        return Err("Course contains duplicate or invalid dictionary entries".to_owned());
    }
    Ok(words)
}

pub fn resolve_course_words(
    course: &VocabularyRegion,
    matches: Vec<Word>,
) -> Result<Vec<Word>, String> {
    let mut candidates = BTreeMap::<(String, String), Word>::new();
    for word in matches {
        if let Some(numeric) = word.pinyin_numeric.clone() {
            candidates
                .entry((word.simplified.clone(), numeric))
                .or_insert(word);
        }
    }
    let mut resolved = Vec::with_capacity(course.readings.len());
    for &(spelling, numeric) in course.readings {
        let word = candidates
            .remove(&(spelling.to_owned(), numeric.to_owned()))
            .ok_or_else(|| format!("HSK reading unavailable: {spelling} ({numeric})"))?;
        resolved.push(word);
    }
    validate_course_words(resolved, course.readings.len())
}
