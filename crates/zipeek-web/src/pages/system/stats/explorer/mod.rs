mod page;
mod row;
pub(super) use page::VocabularyExplorer;

#[cfg(test)]
mod tests {
    use super::super::{data::demo_words_for_tests, query::query_words};
    use super::*;

    #[test]
    fn explorer_queries_preserve_stable_sorting_and_search() {
        let words = demo_words_for_tests();
        let query = WordQuery {
            search: "study".to_owned(),
            sort: WordSort::Alphabetical,
            gap: None,
            page: PageRequest::new(0, 10).unwrap(),
        };
        let found = query_words(&words, &query).unwrap();
        assert!(found.records.iter().any(|word| word.hanzi == "学习"));
        let review = query_words(
            &words,
            &WordQuery {
                search: String::new(),
                sort: WordSort::Review,
                ..query
            },
        )
        .unwrap();
        assert_eq!(review.total, words.len());
    }
}
