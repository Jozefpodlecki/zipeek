use super::assessment::AssessmentItem;
use std::collections::BTreeSet;

fn normalized_choice(choice: &str) -> String {
    choice
        .chars()
        .filter(|ch| {
            !ch.is_whitespace()
                && !matches!(
                    *ch,
                    '。' | '？'
                        | '?'
                        | '！'
                        | '!'
                        | '，'
                        | ','
                        | '、'
                        | '；'
                        | ';'
                        | '：'
                        | ':'
                        | '．'
                        | '.'
                )
        })
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn validate_item_quality(item: &AssessmentItem) -> Result<(), &'static str> {
    if item.correct >= item.choices.len() {
        return Err("correct choice index outside answer choices");
    }
    if item.prompt.trim().is_empty() || item.rationale.trim().is_empty() {
        return Err("empty assessment prompt or rationale");
    }
    let normalized: BTreeSet<String> = item
        .choices
        .iter()
        .map(|choice| normalized_choice(choice))
        .collect();
    if normalized.contains("") || normalized.len() != item.choices.len() {
        return Err("blank or equivalent assessment choices");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> AssessmentItem {
        AssessmentItem {
            id: "draft-test",
            point_id: "draft-point",
            prompt: "Choose the statement",
            choices: ["我有书", "我没有书", "我有一本书"],
            correct: 2,
            rationale: "本 is the classifier for books.",
        }
    }

    #[test]
    fn accepts_valid_draft_and_rejects_equivalent_distractors() {
        let mut item = example();
        assert_eq!(validate_item_quality(&item), Ok(()));
        item.choices[1] = " 我 有 书 ";
        assert_eq!(
            validate_item_quality(&item),
            Err("blank or equivalent assessment choices")
        );
    }

    #[test]
    fn rejects_punctuation_only_and_blank_distractors() {
        let mut item = example();
        item.choices[1] = "我有书！";
        assert_eq!(
            validate_item_quality(&item),
            Err("blank or equivalent assessment choices")
        );
        item.choices[1] = "？ 。";
        assert_eq!(
            validate_item_quality(&item),
            Err("blank or equivalent assessment choices")
        );
    }

    #[test]
    fn rejects_invalid_answer_key_and_missing_explanation() {
        let mut item = example();
        item.correct = 3;
        assert_eq!(
            validate_item_quality(&item),
            Err("correct choice index outside answer choices")
        );
        item.correct = 1;
        item.rationale = " ";
        assert_eq!(
            validate_item_quality(&item),
            Err("empty assessment prompt or rationale")
        );
    }
}
