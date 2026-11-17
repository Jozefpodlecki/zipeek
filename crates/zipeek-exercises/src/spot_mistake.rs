#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MistakePrompt {
    pub meaning: &'static str,
    pub spans: &'static [&'static str],
    pub mistaken_span: usize,
    pub accepted_replacements: &'static [&'static str],
    pub explanation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MistakeVerdict {
    pub identified: bool,
    pub repaired: bool,
}

pub const PROMPTS: &[MistakePrompt] = &[
    MistakePrompt {
        meaning: "I will go to school tomorrow.",
        spans: &["我", "明天", "去了", "学校", "。"],
        mistaken_span: 2,
        accepted_replacements: &["去", "要去", "会去"],
        explanation: "明天 describes a future action. Replace the past-event form 去了 with 去, 要去, or 会去.",
    },
    MistakePrompt {
        meaning: "I have three books.",
        spans: &["我", "有", "三本", "书们", "。"],
        mistaken_span: 3,
        accepted_replacements: &["书"],
        explanation: "The suffix 们 is not used to pluralize 书. The number and classifier already give the quantity.",
    },
    MistakePrompt {
        meaning: "She drinks tea every day.",
        spans: &["她", "每天", "喝", "茶了", "。"],
        mistaken_span: 3,
        accepted_replacements: &["茶"],
        explanation: "The habitual statement with 每天 does not need the completed-event particle 了.",
    },
];

pub fn validate(prompt: &MistakePrompt) -> bool {
    prompt.spans.len() >= 3
        && prompt.mistaken_span < prompt.spans.len()
        && prompt.spans.iter().all(|span| !span.trim().is_empty())
        && !prompt.meaning.trim().is_empty()
        && !prompt.explanation.trim().is_empty()
        && !prompt.accepted_replacements.is_empty()
        && prompt
            .accepted_replacements
            .iter()
            .enumerate()
            .all(|(index, replacement)| {
                !replacement.trim().is_empty()
                    && replacement.trim() == *replacement
                    && !prompt.accepted_replacements[..index].contains(replacement)
            })
        && !prompt
            .accepted_replacements
            .contains(&prompt.spans[prompt.mistaken_span])
}

pub fn grade(
    prompt: &MistakePrompt,
    selected_span: Option<usize>,
    replacement: &str,
) -> MistakeVerdict {
    if !validate(prompt) {
        return MistakeVerdict {
            identified: false,
            repaired: false,
        };
    }
    let identified = selected_span == Some(prompt.mistaken_span);
    MistakeVerdict {
        identified,
        repaired: identified && prompt.accepted_replacements.contains(&replacement.trim()),
    }
}

pub fn corrected_sentence(prompt: &MistakePrompt, replacement: &str) -> Option<String> {
    if !validate(prompt) || !prompt.accepted_replacements.contains(&replacement.trim()) {
        return None;
    }
    let replacement = replacement.trim();
    Some(
        prompt
            .spans
            .iter()
            .enumerate()
            .map(|(index, span)| {
                if index == prompt.mistaken_span {
                    replacement
                } else {
                    span
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_have_valid_corrections() {
        for prompt in PROMPTS {
            assert!(validate(prompt));
            for replacement in prompt.accepted_replacements {
                assert!(grade(prompt, Some(prompt.mistaken_span), replacement).repaired);
                assert!(corrected_sentence(prompt, replacement).is_some());
            }
        }
    }

    #[test]
    fn detection_and_repair_are_distinct() {
        let prompt = &PROMPTS[0];
        assert_eq!(
            grade(prompt, Some(0), "去"),
            MistakeVerdict {
                identified: false,
                repaired: false
            }
        );
        assert_eq!(
            grade(prompt, Some(2), "去了"),
            MistakeVerdict {
                identified: true,
                repaired: false
            }
        );
        assert_eq!(
            grade(prompt, Some(2), "会去"),
            MistakeVerdict {
                identified: true,
                repaired: true
            }
        );
        assert_eq!(
            grade(prompt, None, "去"),
            MistakeVerdict {
                identified: false,
                repaired: false
            }
        );
        assert_eq!(
            grade(prompt, Some(100), "去"),
            MistakeVerdict {
                identified: false,
                repaired: false
            }
        );
    }

    #[test]
    fn corrections_are_unique_and_have_no_hidden_whitespace() {
        let invalid = MistakePrompt {
            accepted_replacements: &["去", "去"],
            ..PROMPTS[0]
        };
        assert!(!validate(&invalid));
        let untrimmed = MistakePrompt {
            accepted_replacements: &["去 "],
            ..PROMPTS[0]
        };
        assert!(!validate(&untrimmed));
    }

    #[test]
    fn malformed_content_cannot_receive_credit() {
        let broken = MistakePrompt {
            mistaken_span: 99,
            ..PROMPTS[0]
        };
        assert!(!validate(&broken));
        assert!(!grade(&broken, Some(99), "去").repaired);
        assert_eq!(corrected_sentence(&broken, "去"), None);
    }

    #[test]
    fn correction_display_and_grading_normalize_input_identically() {
        let prompt = &PROMPTS[0];
        assert!(grade(prompt, Some(prompt.mistaken_span), "  去  ").repaired);
        assert_eq!(
            corrected_sentence(prompt, "  去  "),
            corrected_sentence(prompt, "去")
        );
        assert_eq!(corrected_sentence(prompt, "其它"), None);
    }
}
