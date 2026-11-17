use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityResolution {
    Resolved { current_word_id: i32 },
    Missing,
    Ambiguous { candidates: Vec<i32> },
}

pub fn reconcile<'a>(
    previous_keys: impl IntoIterator<Item = &'a str>,
    current: impl IntoIterator<Item = (&'a str, i32)>,
) -> BTreeMap<String, IdentityResolution> {
    let mut lookup: BTreeMap<&str, Vec<i32>> = BTreeMap::new();
    for (key, word_id) in current {
        if word_id > 0
            && !key.trim().is_empty()
            && key.trim() == key
            && !key.chars().any(char::is_control)
        {
            lookup.entry(key).or_default().push(word_id);
        }
    }
    for candidates in lookup.values_mut() {
        candidates.sort_unstable();
        candidates.dedup();
    }
    previous_keys
        .into_iter()
        .map(|key| {
            let resolution = match lookup.get(key).filter(|_| {
                !key.trim().is_empty() && key.trim() == key && !key.chars().any(char::is_control)
            }) {
                None => IdentityResolution::Missing,
                Some(candidates) if candidates.len() == 1 => IdentityResolution::Resolved {
                    current_word_id: candidates[0],
                },
                Some(candidates) => IdentityResolution::Ambiguous {
                    candidates: candidates.clone(),
                },
            };
            (key.to_owned(), resolution)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_invalid_dictionary_ids_never_resolve() {
        let results = reconcile(
            ["missing", "negative", "zero", "positive"],
            [
                ("negative", -7),
                ("zero", 0),
                ("positive", 19),
                ("positive", 19),
            ],
        );
        assert_eq!(results["missing"], IdentityResolution::Missing);
        assert_eq!(results["negative"], IdentityResolution::Missing);
        assert_eq!(results["zero"], IdentityResolution::Missing);
        assert_eq!(
            results["positive"],
            IdentityResolution::Resolved {
                current_word_id: 19
            }
        );
    }

    #[test]
    fn reorders_are_safe_and_ambiguity_is_not_silently_resolved() {
        let answers = reconcile(["a", "b", "c"], [("a", 99), ("b", 10), ("b", 11)]);
        assert_eq!(
            answers["a"],
            IdentityResolution::Resolved {
                current_word_id: 99
            }
        );
        assert_eq!(
            answers["b"],
            IdentityResolution::Ambiguous {
                candidates: vec![10, 11]
            }
        );
        assert_eq!(answers["c"], IdentityResolution::Missing);
    }

    #[test]
    fn duplicate_dictionary_rows_are_not_false_ambiguity() {
        let results = reconcile(
            ["lexeme", " "],
            [("lexeme", 7), ("lexeme", 7), ("lexeme", -1), (" ", 4)],
        );
        assert_eq!(
            results["lexeme"],
            IdentityResolution::Resolved { current_word_id: 7 }
        );
        assert_eq!(results[" "], IdentityResolution::Missing);
    }

    #[test]
    fn invisible_and_padded_dictionary_keys_are_never_reconciled() {
        let result = reconcile(
            ["word", " word", "word\n"],
            [("word", 1), (" word", 2), ("word\n", 3)],
        );
        assert_eq!(
            result["word"],
            IdentityResolution::Resolved { current_word_id: 1 }
        );
        assert_eq!(result[" word"], IdentityResolution::Missing);
        assert_eq!(result["word\n"], IdentityResolution::Missing);
    }
}
