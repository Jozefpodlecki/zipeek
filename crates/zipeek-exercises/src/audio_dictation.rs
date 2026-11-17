#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictationPrompt {
    pub transcript: &'static str,
    pub accepted_hanzi: &'static [&'static str],
    pub meaning: &'static str,
}

pub const PROMPTS: &[DictationPrompt] = &[
    DictationPrompt {
        transcript: "你好。",
        accepted_hanzi: &["你好", "你好。"],
        meaning: "Hello.",
    },
    DictationPrompt {
        transcript: "谢谢你。",
        accepted_hanzi: &["谢谢你", "谢谢你。", "謝謝你", "謝謝你。"],
        meaning: "Thank you.",
    },
    DictationPrompt {
        transcript: "明天见。",
        accepted_hanzi: &["明天见", "明天见。", "明天見", "明天見。"],
        meaning: "See you tomorrow.",
    },
];

pub fn validate(prompt: &DictationPrompt) -> bool {
    !prompt.transcript.trim().is_empty()
        && prompt.transcript.trim() == prompt.transcript
        && !prompt.accepted_hanzi.is_empty()
        && prompt
            .accepted_hanzi
            .iter()
            .enumerate()
            .all(|(index, answer)| !prompt.accepted_hanzi[..index].contains(answer))
        && prompt
            .accepted_hanzi
            .iter()
            .all(|answer| !answer.trim().is_empty() && answer.trim() == *answer)
        && !prompt.meaning.trim().is_empty()
}

pub fn grade(prompt: &DictationPrompt, answer: &str) -> bool {
    let submitted = answer.trim();
    validate(prompt) && !submitted.is_empty() && prompt.accepted_hanzi.contains(&submitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_untrimmed_transcripts_are_not_accepted_as_content() {
        assert!(!validate(&DictationPrompt {
            transcript: " ",
            ..PROMPTS[0]
        }));
        assert!(!validate(&DictationPrompt {
            accepted_hanzi: &["你好 "],
            ..PROMPTS[0]
        }));
    }

    #[test]
    fn dictation_rejects_homophones_and_pinyin() {
        assert!(PROMPTS.iter().all(validate));
        assert!(grade(&PROMPTS[0], "你好"));
        assert!(!grade(&PROMPTS[0], "你号"));
        assert!(!grade(&PROMPTS[0], "ni3hao3"));
    }

    #[test]
    fn duplicate_or_unusable_transcripts_never_grade() {
        let duplicate = DictationPrompt {
            accepted_hanzi: &["你好", "你好"],
            ..PROMPTS[0]
        };
        assert!(!validate(&duplicate));
        assert!(!grade(&duplicate, "你好"));
        let hidden = DictationPrompt {
            transcript: " 你好。 ",
            ..PROMPTS[0]
        };
        assert!(!validate(&hidden));
        assert!(!grade(&hidden, "你好"));
    }
}
