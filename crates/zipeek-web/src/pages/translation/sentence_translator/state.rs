use crate::services::database::VerifiedVocabulary;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum TranslationDirection {
    ChineseToEnglish,
    EnglishToChinese,
}

impl TranslationDirection {
    pub(super) fn code(&self) -> &'static str {
        match self {
            Self::ChineseToEnglish => "zh-en",
            Self::EnglishToChinese => "en-zh",
        }
    }
}

#[derive(Clone)]
pub(super) struct TranslatorState {
    pub(super) source: String,
    pub(super) direction: TranslationDirection,
    pub(super) result: String,
    pub(super) analysis: Option<zipeek_ai_protocol::sentence_analysis::SentenceAnalysis>,
    pub(super) verified: Option<Result<Vec<VerifiedVocabulary>, String>>,
    pub(super) error: String,
    pub(super) analysis_warning: Option<String>,
    pub(super) busy: bool,
    pub(super) progress_phase: String,
    pub(super) progress_chars: usize,
    pub(super) copy_status: String,
}

impl Default for TranslatorState {
    fn default() -> Self {
        Self {
            source: String::new(),
            direction: TranslationDirection::EnglishToChinese,
            result: String::new(),
            analysis: None,
            verified: None,
            error: String::new(),
            analysis_warning: None,
            busy: false,
            progress_phase: String::new(),
            progress_chars: 0,
            copy_status: String::new(),
        }
    }
}

pub(super) enum TranslatorAction {
    EditSource(String),
    SetDirection(TranslationDirection),
    Clear,
    BeginRequest,
    Progress {
        phase: String,
        received_chars: usize,
    },
    Resolve {
        text: String,
        analysis: Option<zipeek_ai_protocol::sentence_analysis::SentenceAnalysis>,
        verified: Option<Result<Vec<VerifiedVocabulary>, String>>,
        warning: Option<String>,
    },
    Fail(String),
    CopyStatus(String),
}

impl TranslatorState {
    fn clear_result(&mut self) {
        self.result.clear();
        self.analysis = None;
        self.verified = None;
        self.error.clear();
        self.analysis_warning = None;
        self.copy_status.clear();
        self.busy = false;
        self.progress_phase.clear();
        self.progress_chars = 0;
    }
}

impl Reducible for TranslatorState {
    type Action = TranslatorAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            TranslatorAction::EditSource(source) => {
                state.clear_result();
                state.source = source;
            }
            TranslatorAction::SetDirection(direction) => {
                state.clear_result();
                state.direction = direction;
            }
            TranslatorAction::Clear => {
                state.clear_result();
                state.source.clear();
            }
            TranslatorAction::BeginRequest => {
                state.clear_result();
                state.busy = true;
                state.progress_phase = "Generating translation and analysis".to_owned();
            }
            TranslatorAction::Progress {
                phase,
                received_chars,
            } => {
                if state.busy {
                    state.progress_phase = phase;
                    state.progress_chars = received_chars;
                }
            }
            TranslatorAction::Resolve {
                text,
                analysis,
                verified,
                warning,
            } => {
                state.analysis_warning = warning;
                state.result = text;
                state.analysis = analysis;
                state.verified = verified;
                state.busy = false;
            }
            TranslatorAction::Fail(error) => {
                state.error = error;
                state.busy = false;
            }
            TranslatorAction::CopyStatus(status) => state.copy_status = status,
        }
        Rc::new(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_source_clears_stale_translation_and_error() {
        let mut previous = TranslatorState::default();
        previous.result = "old result".to_owned();
        previous.error = "old error".to_owned();
        previous.busy = true;
        let next = Rc::new(previous).reduce(TranslatorAction::EditSource("新的句子".to_owned()));
        assert_eq!(next.source, "新的句子");
        assert!(next.result.is_empty());
        assert!(next.error.is_empty());
        assert!(!next.busy);
    }

    #[test]
    fn changing_direction_invalidates_prior_results() {
        let mut previous = TranslatorState::default();
        previous.result = "hello".to_owned();
        let next = Rc::new(previous).reduce(TranslatorAction::SetDirection(
            TranslationDirection::EnglishToChinese,
        ));
        assert_eq!(next.direction.code(), "en-zh");
        assert!(next.result.is_empty());
    }
}
