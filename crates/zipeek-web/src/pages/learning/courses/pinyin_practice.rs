#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinyinQuestion {
    pub prompt: &'static str,
    pub options: [&'static str; 3],
    pub answer: usize,
    pub explanation: &'static str,
}

pub const PINYIN_CHECKS: [[PinyinQuestion; 2]; 12] = [
    [
        PinyinQuestion { prompt: "Which syllable uses the rising second tone?", options: ["mā", "má", "mà"], answer: 1, explanation: "An acute accent marks the rising second tone." },
        PinyinQuestion { prompt: "Which syllable uses the low third tone?", options: ["mǎ", "má", "mā"], answer: 0, explanation: "The caron indicates the third tone; its pitch is often low in connected speech." },
    ],
    [
        PinyinQuestion { prompt: "In standard Mandarin, how is 你好 normally pronounced?", options: ["nǐ hǎo", "ní hǎo", "nì hào"], answer: 1, explanation: "The first of two adjacent third tones changes to a rising tone." },
        PinyinQuestion { prompt: "Which is the usual reading of 不是?", options: ["bù shì", "bú shì", "bū shì"], answer: 1, explanation: "不 changes to the second tone before a fourth-tone syllable." },
    ],
    [
        PinyinQuestion { prompt: "Which Pinyin initial is written for the sound at the start of 妈 mā?", options: ["n", "m", "b"], answer: 1, explanation: "The initial in mā is m." },
        PinyinQuestion { prompt: "Which pair contrasts unaspirated and aspirated stops?", options: ["b / p", "m / n", "f / h"], answer: 0, explanation: "The main difference between b and p in standard Mandarin is aspiration." },
    ],
    [
        PinyinQuestion { prompt: "Which group contains retroflex initials?", options: ["j q x", "z c s", "zh ch sh"], answer: 2, explanation: "zh, ch and sh are retroflex initials in standard Mandarin." },
        PinyinQuestion { prompt: "Which initial in qī is aspirated?", options: ["q", "j", "x"], answer: 0, explanation: "q is an aspirated palatal affricate." },
    ],
    [
        PinyinQuestion { prompt: "Which Pinyin symbol denotes the rounded front vowel?", options: ["u", "ü", "e"], answer: 1, explanation: "ü represents a front rounded vowel, distinct from u." },
        PinyinQuestion { prompt: "Which spelling has no consonant initial?", options: ["ài", "mā", "bā"], answer: 0, explanation: "ài begins directly with a vowel final." },
    ],
    [
        PinyinQuestion { prompt: "In hǎo, which final follows h?", options: ["ao", "an", "ou"], answer: 0, explanation: "hǎo combines the initial h with the final ao." },
        PinyinQuestion { prompt: "Which syllable has a compound final?", options: ["mā", "hǎo", "dē"], answer: 1, explanation: "ao is a compound final." },
    ],
    [
        PinyinQuestion { prompt: "Which ending is written with a final -ng?", options: ["ān", "āng", "āi"], answer: 1, explanation: "-ng marks a velar nasal ending." },
        PinyinQuestion { prompt: "Which pair contrasts -n and -ng?", options: ["ān / āng", "mā / má", "bā / pā"], answer: 0, explanation: "ān ends with -n, while āng ends with -ng." },
    ],
    [
        PinyinQuestion { prompt: "After j, q and x, how is ü normally written?", options: ["ü", "u", "v"], answer: 1, explanation: "The umlaut is omitted after j, q and x even though the vowel is ü." },
        PinyinQuestion { prompt: "Which spelling separates Xi and an?", options: ["Xian", "Xi'an", "Xian'"], answer: 1, explanation: "The apostrophe prevents an ambiguous syllable boundary." },
    ],
    [
        PinyinQuestion { prompt: "How many syllables does Zhōngguó have?", options: ["One", "Two", "Three"], answer: 1, explanation: "Zhōng and guó are two syllables." },
        PinyinQuestion { prompt: "Which spelling begins with no consonant initial?", options: ["ér", "rén", "lái"], answer: 0, explanation: "ér begins with a vowel final." },
    ],
    [
        PinyinQuestion { prompt: "In māma, what happens to the second syllable?", options: ["It is usually neutral", "It is always fourth tone", "It becomes first tone"], answer: 0, explanation: "The second syllable of 妈妈 normally has a neutral tone." },
        PinyinQuestion { prompt: "In connected speech, two third tones usually become which sequence?", options: ["3 + 3", "2 + 3", "4 + 3"], answer: 1, explanation: "The first third tone is normally realized as rising before another third tone." },
    ],
    [
        PinyinQuestion { prompt: "Which two initials differ mainly in aspiration?", options: ["z / c", "sh / x", "n / l"], answer: 0, explanation: "z is unaspirated; c is aspirated." },
        PinyinQuestion { prompt: "Which pair differs in its nasal ending?", options: ["lín / líng", "mā / má", "bā / pā"], answer: 0, explanation: "lín ends in -n and líng in -ng." },
    ],
    [
        PinyinQuestion { prompt: "Which reading shows the usual 不 tone change?", options: ["bú shì", "bù shì", "bū shì"], answer: 0, explanation: "Before fourth tone shì, 不 is pronounced bú." },
        PinyinQuestion { prompt: "Which spelling represents the ü vowel after l?", options: ["lù", "lǜ", "lǔ"], answer: 1, explanation: "ü retains its dots after l, as in lǜ." },
    ],
];

pub fn grade(module: usize, answers: &[Option<usize>; 2]) -> Option<usize> {
    let questions = PINYIN_CHECKS.get(module)?;
    if answers.iter().any(Option::is_none) {
        return None;
    }
    Some(answers.iter().enumerate().filter(|(i, answer)| **answer == Some(questions[*i].answer)).count())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_lesson_has_two_valid_checks() {
        assert_eq!(PINYIN_CHECKS.len(), super::super::catalog::PINYIN_LESSONS.len());
        for questions in PINYIN_CHECKS {
            for question in questions {
                assert!(!question.prompt.is_empty());
                assert!(question.answer < question.options.len());
                assert!(!question.explanation.is_empty());
                assert_eq!(question.options.iter().collect::<std::collections::BTreeSet<_>>().len(), 3);
            }
        }
    }
    #[test]
    fn grades_only_complete_answers() {
        assert_eq!(grade(0, &[None, Some(0)]), None);
        assert_eq!(grade(0, &[Some(1), Some(0)]), Some(2));
        assert_eq!(grade(0, &[Some(0), Some(0)]), Some(1));
    }
}
