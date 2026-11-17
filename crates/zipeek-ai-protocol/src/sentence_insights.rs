use crate::sentence_analysis::GrammarObservation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrammarFamily {
    Comparison,
    Condition,
    CauseAndEffect,
    Contrast,
    Aspect,
    Disposal,
    Passive,
    Complement,
    Intention,
    Quantity,
    Other,
}

impl GrammarFamily {
    pub fn label(self) -> &'static str {
        match self {
            Self::Comparison => "Comparison",
            Self::Condition => "Condition",
            Self::CauseAndEffect => "Cause and effect",
            Self::Contrast => "Contrast",
            Self::Aspect => "Aspect / time",
            Self::Disposal => "把 construction",
            Self::Passive => "Passive",
            Self::Complement => "Complement",
            Self::Intention => "Intention",
            Self::Quantity => "Quantity / measure words",
            Self::Other => "Other structure (AI identified)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrammarEvidence {
    pub family: GrammarFamily,
    pub anchored: bool,
}

pub fn grammar_evidence(sentence: &str, item: &GrammarObservation) -> GrammarEvidence {
    let fragment = item.example_fragment.as_str();
    let anchored = !fragment.is_empty() && sentence.contains(fragment);
    if !anchored {
        return GrammarEvidence {
            family: GrammarFamily::Other,
            anchored: false,
        };
    }
    let pattern = item.pattern.to_lowercase();
    let contains = |token: &str| fragment.contains(token) || pattern.contains(token);
    let family = if contains("虽然") || contains("但是") || contains("however") {
        GrammarFamily::Contrast
    } else if contains("如果") || contains("要是") || contains("条件") || contains("condition")
    {
        GrammarFamily::Condition
    } else if contains("因为") || contains("所以") || contains("原因") || contains("cause") {
        GrammarFamily::CauseAndEffect
    } else if contains("比") || contains("更") || contains("comparison") || contains("compare") {
        GrammarFamily::Comparison
    } else if contains("把") {
        GrammarFamily::Disposal
    } else if contains("被") || contains("passive") {
        GrammarFamily::Passive
    } else if contains("结果补语")
        || contains("趋向补语")
        || contains("complement")
        || contains("得 +")
    {
        GrammarFamily::Complement
    } else if contains("过") || contains("着") || contains("了") || contains("aspect") {
        GrammarFamily::Aspect
    } else if contains("要")
        || contains("打算")
        || contains("想")
        || contains("intend")
        || contains("plan")
    {
        GrammarFamily::Intention
    } else if contains("量词") || contains("measure word") || contains("classifier") {
        GrammarFamily::Quantity
    } else {
        GrammarFamily::Other
    };
    GrammarEvidence { family, anchored }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexicalCoverage {
    pub observed: usize,
    pub matched: usize,
    pub hsk_tagged: usize,
    pub ambiguous: usize,
    pub highest_verified_hsk3: Option<i32>,
}

impl LexicalCoverage {
    pub fn complete(&self) -> bool {
        self.observed > 0 && self.observed == self.matched && self.ambiguous == 0
    }

    pub fn scope_note(&self) -> &'static str {
        if self.observed == 0 {
            "No AI vocabulary candidates were supplied; lexical coverage cannot be estimated."
        } else if self.matched == 0 {
            "No suggested words could be verified against the dictionary."
        } else if !self.complete() {
            "Some suggested words are missing or ambiguous; this is not full sentence coverage."
        } else {
            "All suggested words matched dictionary entries; the AI word list may still omit tokens."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grammar(pattern: &str, fragment: &str) -> GrammarObservation {
        GrammarObservation {
            pattern: pattern.into(),
            explanation: "Example".into(),
            example_fragment: fragment.into(),
        }
    }

    #[test]
    fn categorizes_only_when_evidence_occurs_in_sentence() {
        let sentence = "我已经把作业做完了。";
        assert_eq!(
            grammar_evidence(sentence, &grammar("把 + object", "把作业")),
            GrammarEvidence {
                family: GrammarFamily::Disposal,
                anchored: true
            }
        );
        assert_eq!(
            grammar_evidence(sentence, &grammar("被 + verb", "被老师")),
            GrammarEvidence {
                family: GrammarFamily::Other,
                anchored: false
            }
        );
    }

    #[test]
    fn explicit_patterns_can_describe_intention_and_condition() {
        assert_eq!(
            grammar_evidence("我明天要去学校", &grammar("要 + Verb", "要去")).family,
            GrammarFamily::Intention
        );
        assert_eq!(
            grammar_evidence("如果下雨就回家", &grammar("如果…就…", "如果下雨")).family,
            GrammarFamily::Condition
        );
    }

    #[test]
    fn unknown_grammar_is_not_forced_into_a_named_category() {
        assert_eq!(
            grammar_evidence("我喜欢中文", &grammar("Topic comment", "中文")).family,
            GrammarFamily::Other
        );
    }

    #[test]
    fn coverage_notes_do_not_claim_complete_segmentation() {
        let partial = LexicalCoverage {
            observed: 3,
            matched: 2,
            hsk_tagged: 1,
            ambiguous: 1,
            highest_verified_hsk3: Some(4),
        };
        assert!(!partial.complete());
        assert!(partial.scope_note().contains("missing or ambiguous"));
        let complete = LexicalCoverage {
            observed: 2,
            matched: 2,
            hsk_tagged: 2,
            ambiguous: 0,
            highest_verified_hsk3: Some(3),
        };
        assert!(complete.scope_note().contains("may still omit"));
    }
}
