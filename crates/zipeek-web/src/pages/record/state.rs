use std::rc::Rc;
use yew::Reducible;
use zipeek_core::Word;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordingId(pub(super) u64);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RecordingStage {
    Idle,
    Recording,
    Recognizing,
}

fn pick_word(words: &[Word], previous: Option<&Word>) -> Option<Word> {
    if words.is_empty() {
        return None;
    }
    let mut index = (js_sys::Math::random() * words.len() as f64) as usize;
    index = index.min(words.len() - 1);
    if words.len() > 1 && previous.is_some_and(|last| last.id == words[index].id) {
        index = (index + 1) % words.len();
    }
    Some(words[index].clone())
}

pub(super) fn comparable(text: &str) -> String {
    text.chars().filter(|character| !character.is_whitespace() && !character.is_ascii_punctuation() && !matches!(character, '。' | '，' | '！' | '？' | '、' | '；' | '：' | '“' | '”' | '（' | '）')).collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TargetMode { Hsk, Sentence, Live }

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Playback {
    None,
    Reference,
    Recording,
}

#[derive(Clone, PartialEq)]
pub(super) struct RecordState {
    pub(super) mode: TargetMode,
    pub(super) sentence: String,
    pub(super) bank: (u8, u8),
    pub(super) pool: Vec<Word>,
    pub(super) selected: Option<Word>,
    pub(super) loading: bool,
    pub(super) error: Option<String>,
    pub(super) transcript: Option<String>,
    pub(super) playback: Playback,
    pub(super) stage: RecordingStage,
    pub(super) recording_id: Option<RecordingId>,
}

impl Default for RecordState {
    fn default() -> Self {
        Self {
            mode: TargetMode::Hsk,
            sentence: String::new(),
            bank: (3, 1),
            pool: Vec::new(),
            selected: None,
            loading: true,
            error: None,
            transcript: None,
            playback: Playback::None,
            stage: RecordingStage::Idle,
            recording_id: None,
        }
    }
}

pub(super) enum RecordAction {
    ChangeMode(TargetMode),
    ChangeSentence(String),
    ChangeBank((u8, u8)),
    LoadWords(Result<Vec<Word>, String>),
    NextWord,
    BeginPlayback(Playback),
    EndPlayback(Option<String>),
    BeginRecording,
    Recognizing,
    CompleteRecording { transcript: Result<String, String>, recording_id: Option<RecordingId> },
    Cancel,
    Error(String),
}

impl Reducible for RecordState {
    type Action = RecordAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            RecordAction::ChangeMode(mode) => {
                if next.mode == mode { return self; }
                next.mode = mode;
                next.clear_speech();
            }
            RecordAction::ChangeSentence(sentence) => {
                next.sentence = sentence;
                next.clear_speech();
            }
            RecordAction::ChangeBank(bank) => {
                if next.bank == bank { return self; }
                next.bank = bank;
                next.loading = true;
                next.pool.clear();
                next.selected = None;
                next.clear_speech();
            }
            RecordAction::LoadWords(result) => {
                next.loading = false;
                match result {
                    Ok(words) if !words.is_empty() => {
                        next.selected = pick_word(&words, None);
                        next.pool = words;
                        next.error = None;
                    }
                    Ok(_) => next.error = Some("No words found for this HSK bank.".into()),
                    Err(message) => next.error = Some(message),
                }
            }
            RecordAction::NextWord => {
                next.selected = pick_word(&next.pool, next.selected.as_ref());
                next.clear_speech();
            }
            RecordAction::BeginPlayback(kind) => {
                if next.stage == RecordingStage::Idle {
                    next.playback = kind;
                    next.error = None;
                }
            }
            RecordAction::EndPlayback(error) => {
                next.playback = Playback::None;
                next.error = error;
            }
            RecordAction::BeginRecording => {
                next.clear_speech();
                next.stage = RecordingStage::Recording;
            }
            RecordAction::Recognizing => {
                if next.stage == RecordingStage::Recording {
                    next.stage = RecordingStage::Recognizing;
                }
            }
            RecordAction::CompleteRecording { transcript, recording_id } => {
                next.stage = RecordingStage::Idle;
                next.recording_id = recording_id;
                match transcript {
                    Ok(value) => next.transcript = Some(value),
                    Err(error) => next.error = Some(error),
                }
            }
            RecordAction::Cancel => {
                next.playback = Playback::None;
                next.stage = RecordingStage::Idle;
            }
            RecordAction::Error(error) => next.error = Some(error),
        }
        Rc::new(next)
    }
}

impl RecordState {
    pub(super) fn target(&self) -> Option<&str> {
        match self.mode {
            TargetMode::Hsk => self.selected.as_ref().map(|word| word.simplified.as_str()),
            TargetMode::Live => None,
            TargetMode::Sentence => {
                let text = self.sentence.trim();
                if text.is_empty() { None } else { Some(text) }
            }
        }
    }

    fn clear_speech(&mut self) {
        self.transcript = None;
        self.error = None;
        self.playback = Playback::None;
        self.stage = RecordingStage::Idle;
        self.recording_id = None;
    }
}


#[cfg(test)]
mod tests {
    use super::{comparable, Playback, RecordAction, RecordState, RecordingId, RecordingStage, TargetMode};
    use std::rc::Rc;
    use yew::Reducible;

    #[test]
    fn recording_remains_replayable_when_transcription_fails() {
        let state = Rc::new(RecordState::default()).reduce(RecordAction::BeginRecording);
        let state = state.reduce(RecordAction::Recognizing);
        let state = state.reduce(RecordAction::CompleteRecording {
            transcript: Err("Service unavailable".into()),
            recording_id: Some(RecordingId(1)),
        });
        assert_eq!(state.recording_id, Some(RecordingId(1)));
        assert_eq!(state.stage, RecordingStage::Idle);
        assert_eq!(state.error.as_deref(), Some("Service unavailable"));
        let state = state.reduce(RecordAction::BeginPlayback(Playback::Recording));
        assert_eq!(state.playback, Playback::Recording);
        let state = state.reduce(RecordAction::Cancel);
        assert_eq!(state.recording_id, Some(RecordingId(1)));
        assert_eq!(state.playback, Playback::None);
    }

    #[test]
    fn changing_bank_clears_pending_recording_and_results() {
        let state = Rc::new(RecordState::default()).reduce(RecordAction::CompleteRecording {
            transcript: Ok("你好".into()),
            recording_id: Some(RecordingId(1)),
        });
        let state = state.reduce(RecordAction::ChangeBank((2, 3)));
        assert_eq!(state.bank, (2, 3));
        assert!(state.loading);
        assert!(state.recording_id.is_none());
        assert!(state.transcript.is_none());
        assert!(state.selected.is_none());
    }

    #[test]
    fn sentence_edit_clears_stale_recording() {
        let state = Rc::new(RecordState::default()).reduce(RecordAction::ChangeMode(TargetMode::Sentence));
        let state = state.reduce(RecordAction::ChangeSentence("你好世界".into()));
        assert_eq!(state.target(), Some("你好世界"));
        let state = state.reduce(RecordAction::CompleteRecording { transcript: Ok("你好世界".into()), recording_id: Some(RecordingId(1)) });
        let state = state.reduce(RecordAction::ChangeSentence("早上好".into()));
        assert!(state.transcript.is_none());
        assert!(state.recording_id.is_none());
        assert_eq!(state.target(), Some("早上好"));
    }

    #[test]
    fn comparison_ignores_punctuation_not_wording() {
        assert_eq!(comparable("你好，世界。"), comparable("你好世界"));
        assert_ne!(comparable("你好世界"), comparable("您好世界"));
    }
}
