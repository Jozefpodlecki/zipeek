use crate::definition::SPEAKING;
use crate::session::{ExerciseInstance, SessionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeakingTarget {
    pub id: &'static str,
    pub sentence: &'static str,
    pub pinyin: &'static str,
    pub translation: &'static str,
    pub focus_word: &'static str,
    pub focus_reading: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakingTargetError {
    MissingContext,
    MissingFocusWord,
}

impl SpeakingTarget {
    pub fn validate(&self) -> Result<(), SpeakingTargetError> {
        if [
            self.id,
            self.sentence,
            self.pinyin,
            self.translation,
            self.focus_reading,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(SpeakingTargetError::MissingContext);
        }
        if self.focus_word.trim().is_empty() || !self.sentence.contains(self.focus_word) {
            return Err(SpeakingTargetError::MissingFocusWord);
        }
        Ok(())
    }

    pub fn visible_input(&self) -> Result<String, SpeakingTargetError> {
        self.validate()?;
        Ok(format!(
            "Speak this Mandarin sentence: {}\nPinyin: {}\nMeaning: {}\nFocus word: {}\nTarget reading: {}",
            self.sentence, self.pinyin, self.translation, self.focus_word, self.focus_reading
        ))
    }

    pub fn instance(&self) -> Result<ExerciseInstance, SpeakingInstanceError> {
        let input = self
            .visible_input()
            .map_err(SpeakingInstanceError::InvalidTarget)?;
        ExerciseInstance::new(self.id, SPEAKING.id, input)
            .map_err(SpeakingInstanceError::InvalidInstance)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakingInstanceError {
    InvalidTarget(SpeakingTargetError),
    InvalidInstance(SessionError),
}

pub const PREVIEW_TARGETS: &[SpeakingTarget] = &[
    SpeakingTarget {
        id: "speaking-preview-study",
        sentence: "我喜欢学习中文。",
        pinyin: "Wǒ xǐhuān xuéxí Zhōngwén.",
        translation: "I like learning Chinese.",
        focus_word: "学习",
        focus_reading: "xuéxí",
    },
    SpeakingTarget {
        id: "speaking-preview-bank",
        sentence: "我今天去银行。",
        pinyin: "Wǒ jīntiān qù yínháng.",
        translation: "I'm going to the bank today.",
        focus_word: "银行",
        focus_reading: "yínháng",
    },
    SpeakingTarget {
        id: "speaking-preview-music",
        sentence: "她喜欢听音乐。",
        pinyin: "Tā xǐhuān tīng yīnyuè.",
        translation: "She likes listening to music.",
        focus_word: "音乐",
        focus_reading: "yīnyuè",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_targets_have_unique_ids_and_explicit_readings() {
        let mut ids = std::collections::HashSet::new();
        for target in PREVIEW_TARGETS {
            assert!(ids.insert(target.id));
            assert!(target.validate().is_ok());
            let instance = target.instance().unwrap();
            assert_eq!(instance.definition_id, SPEAKING.id);
            assert!(instance.visible_input.contains(target.focus_reading));
        }
    }

    #[test]
    fn missing_spoken_context_or_focus_is_rejected() {
        let mut target = PREVIEW_TARGETS[0];
        target.pinyin = "";
        assert_eq!(target.validate(), Err(SpeakingTargetError::MissingContext));
        target = PREVIEW_TARGETS[0];
        target.focus_word = "不在句子里";
        assert_eq!(
            target.validate(),
            Err(SpeakingTargetError::MissingFocusWord)
        );
    }

    #[test]
    fn polyphonic_readings_are_specific_to_sentence_context() {
        assert_eq!(PREVIEW_TARGETS[1].focus_reading, "yínháng");
        assert_eq!(PREVIEW_TARGETS[2].focus_reading, "yīnyuè");
        assert!(
            !PREVIEW_TARGETS[1]
                .visible_input()
                .unwrap()
                .contains("correct_answer")
        );
    }
}
