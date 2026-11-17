#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClozePrompt {
    pub sentence_before: &'static str,
    pub sentence_after: &'static str,
    pub meaning_cue: &'static str,
    pub accepted_hanzi: &'static [&'static str],
    pub explanation: &'static str,
}

pub const PROMPTS: &[ClozePrompt] = &[
    ClozePrompt {
        sentence_before: "我昨天",
        sentence_after: "了一本书。",
        meaning_cue: "I bought a book yesterday. Type the missing Chinese verb.",
        accepted_hanzi: &["买", "買"],
        explanation: "买 (mǎi) means to buy. 了 marks the completed action here.",
    },
    ClozePrompt {
        sentence_before: "我每天",
        sentence_after: "中文。",
        meaning_cue: "I study Chinese every day. Type the missing verb.",
        accepted_hanzi: &["学习", "學習"],
        explanation: "学习 (xuéxí) means to study or learn.",
    },
    ClozePrompt {
        sentence_before: "这杯茶太",
        sentence_after: "了。",
        meaning_cue: "This cup of tea is too hot. Type the missing adjective.",
        accepted_hanzi: &["热", "熱"],
        explanation: "热 (rè) means hot. 太…了 emphasizes the degree.",
    },
];

pub fn validate(prompt: &ClozePrompt) -> bool {
    !prompt.sentence_before.is_empty()
        && !prompt.sentence_after.is_empty()
        && !prompt.meaning_cue.is_empty()
        && !prompt.explanation.trim().is_empty()
        && !prompt.meaning_cue.trim().is_empty()
        && !prompt.accepted_hanzi.is_empty()
        && prompt
            .accepted_hanzi
            .iter()
            .enumerate()
            .all(|(index, answer)| !prompt.accepted_hanzi[..index].contains(answer))
        && prompt.accepted_hanzi.iter().all(|answer| {
            !answer.trim().is_empty()
                && answer.trim() == *answer
                && !prompt.sentence_before.contains(answer)
                && !prompt.sentence_after.contains(answer)
                && !prompt.meaning_cue.contains(answer)
        })
}

pub fn grade(prompt: &ClozePrompt, answer: &str) -> bool {
    let submitted = answer.trim();
    validate(prompt) && !submitted.is_empty() && prompt.accepted_hanzi.contains(&submitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloze_rejects_whitespace_only_accepted_answers() {
        assert!(!validate(&ClozePrompt {
            accepted_hanzi: &[" "],
            ..PROMPTS[0]
        }));
        assert!(!validate(&ClozePrompt {
            accepted_hanzi: &["买 "],
            ..PROMPTS[0]
        }));
    }

    #[test]
    fn prompts_hide_the_answer_and_have_approved_alternatives() {
        assert!(PROMPTS.iter().all(validate));
        assert!(grade(&PROMPTS[0], "買"));
        assert!(!grade(&PROMPTS[0], "卖"));
        assert!(!grade(&PROMPTS[0], "mai3"));
        assert!(!grade(&PROMPTS[0], ""));
    }

    #[test]
    fn invalid_authored_keys_cannot_grade_answers() {
        let duplicate = ClozePrompt {
            accepted_hanzi: &["买", "买"],
            ..PROMPTS[0]
        };
        assert!(!validate(&duplicate));
        assert!(!grade(&duplicate, "买"));
        let missing_explanation = ClozePrompt {
            explanation: " ",
            ..PROMPTS[0]
        };
        assert!(!validate(&missing_explanation));
        assert!(!grade(&missing_explanation, "买"));
    }
}
