use super::super::simulator::{demo_grammar, demo_words};

#[derive(Clone, PartialEq)]
pub(super) struct SelectedItem {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub grammar: bool,
}

pub(super) fn resolve_selection(
    word_ids: &std::collections::BTreeSet<String>,
    pattern_ids: &std::collections::BTreeSet<String>,
) -> (Vec<SelectedItem>, usize) {
    let words = demo_words();
    let grammar = demo_grammar();
    let items = words
        .iter()
        .filter(|word| word_ids.contains(&word.id))
        .map(|word| SelectedItem {
            id: word.id.clone(),
            title: word.hanzi.clone(),
            detail: format!("{} · {}", word.pinyin, word.meaning),
            grammar: false,
        })
        .chain(
            grammar
                .iter()
                .filter(|pattern| pattern_ids.contains(&pattern.id))
                .map(|pattern| SelectedItem {
                    id: pattern.id.clone(),
                    title: pattern.pattern.clone(),
                    detail: format!("{} · {}", pattern.course, pattern.example),
                    grammar: true,
                }),
        )
        .collect::<Vec<_>>();
    let unresolved = word_ids.len() + pattern_ids.len() - items.len();
    (items, unresolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn selected_preview_items_are_resolved_to_readable_catalogue_content() {
        let words = demo_words();
        let grammar = demo_grammar();
        let word_ids = BTreeSet::from([words[0].id.clone()]);
        let pattern_ids = BTreeSet::from([grammar[0].id.clone()]);
        let (items, unresolved) = resolve_selection(&word_ids, &pattern_ids);
        assert_eq!(unresolved, 0);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, words[0].hanzi);
        assert_eq!(items[1].title, grammar[0].pattern);
        assert!(!items[0].detail.is_empty());
        assert!(!items[1].detail.is_empty());
    }

    #[test]
    fn unknown_ids_are_not_displayed_as_learner_facing_material() {
        let word_ids = BTreeSet::from(["preview-word-unavailable".to_owned()]);
        let (items, unresolved) = resolve_selection(&word_ids, &BTreeSet::new());
        assert!(items.is_empty());
        assert_eq!(unresolved, 1);
    }
}
