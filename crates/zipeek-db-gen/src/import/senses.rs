use zipeek_core::WordId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSense {
    pub word_id: WordId,
    pub sense_order: i32,
    pub gloss: String,
    pub source: &'static str,
    pub source_ref: i32,
    pub source_gloss_order: i32,
}

pub(super) fn source_glosses(values: &[String]) -> Vec<(i32, String)> {
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let gloss = value.trim();
            (!gloss.is_empty()).then(|| ((index + 1) as i32, gloss.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_source_glosses_preserve_original_source_ordinals() {
        let values = vec![
            "first".to_owned(),
            " ".to_owned(),
            "(Tw) (loanword) code".to_owned(),
        ];
        assert_eq!(
            source_glosses(&values),
            vec![(1, "first".into()), (3, "(Tw) (loanword) code".into())]
        );
    }
}
