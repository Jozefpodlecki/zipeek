use std::{collections::BTreeSet, rc::Rc};

use yew::Reducible;
use zipeek_exercises::image_questions::{ImageQuestion, ImageQuestionMode};

use super::generation::GeneratedQuestion;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImageVocabularyState {
    pub mode: ImageQuestionMode,
    pub phase: String,
    pub busy: bool,
    pub error: String,
    pub question: Option<ImageQuestion>,
    pub scene_sentence: String,
    pub image_url: Option<String>,
    pub approved: bool,
    pub image_loaded: bool,
    pub image_failed: bool,
    pub selected: BTreeSet<i32>,
    pub checked: bool,
}

impl Default for ImageVocabularyState {
    fn default() -> Self {
        Self {
            mode: ImageQuestionMode::OneOfThree,
            phase: "Loading eligible HSK vocabulary".to_owned(),
            busy: true,
            error: String::new(),
            question: None,
            scene_sentence: String::new(),
            image_url: None,
            approved: false,
            image_loaded: false,
            image_failed: false,
            selected: BTreeSet::new(),
            checked: false,
        }
    }
}

pub(super) enum Action {
    Start,
    ChooseMode(ImageQuestionMode),
    Progress(String),
    Generated(GeneratedQuestion),
    Failed(String),
    ImageLoaded,
    ImageFailed,
    Approve,
    Toggle(i32),
    Check,
}

impl Reducible for ImageVocabularyState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            Action::Start => {
                return Rc::new(Self {
                    mode: self.mode,
                    ..Self::default()
                });
            }
            Action::ChooseMode(mode) => {
                if mode == self.mode {
                    return self;
                }
                return Rc::new(Self {
                    mode,
                    ..Self::default()
                });
            }
            Action::Progress(message) if next.busy => next.phase = message,
            Action::Generated(generated) if next.busy => {
                next.scene_sentence = generated.sentence;
                next.question = Some(generated.question);
                next.image_url = Some(generated.image_url);
                next.phase = "Review generated image".to_owned();
                next.busy = false;
            }
            Action::Failed(message) if next.busy => {
                next.error = message;
                next.busy = false;
            }
            Action::ImageLoaded if next.image_url.is_some() => {
                next.image_failed = false;
                next.image_loaded = true;
            }
            Action::ImageFailed if next.image_url.is_some() => {
                next.image_loaded = false;
                next.image_failed = true;
                next.approved = false;
                next.selected.clear();
                next.checked = false;
            }
            Action::Approve
                if next.image_loaded && !next.image_failed && next.question.is_some() =>
            {
                next.approved = true;
            }
            Action::Toggle(id) if next.approved && !next.checked => {
                if let Some(question) = next.question.as_ref() {
                    if question.mode == next.mode
                        && question.options.iter().any(|word| word.id == id)
                    {
                        next.selected = toggle_selection(&next.selected, next.mode, id);
                    }
                }
            }
            Action::Check if next.approved && !next.checked => {
                if next
                    .question
                    .as_ref()
                    .is_some_and(|question| question.grade(&next.selected).is_ok())
                {
                    next.checked = true;
                }
            }
            _ => {}
        }
        Rc::new(next)
    }
}

pub(super) fn toggle_selection(
    current: &BTreeSet<i32>,
    mode: ImageQuestionMode,
    id: i32,
) -> BTreeSet<i32> {
    let mut next = current.clone();
    match mode {
        ImageQuestionMode::OneOfThree => {
            next.clear();
            next.insert(id);
        }
        ImageQuestionMode::SelectAll => {
            if !next.insert(id) {
                next.remove(&id);
            }
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_exercises::image_questions::VisualWord;

    fn generated() -> GeneratedQuestion {
        GeneratedQuestion {
            question: ImageQuestion {
                mode: ImageQuestionMode::OneOfThree,
                options: (1..=3)
                    .map(|id| VisualWord {
                        id,
                        hanzi: format!("字{id}"),
                        english: format!("word{id}"),
                    })
                    .collect(),
                correct_ids: BTreeSet::from([1]),
                prompt: "A visible object".to_owned(),
            },
            sentence: "A scene".to_owned(),
            image_url: "http://localhost/test.png".to_owned(),
        }
    }

    #[test]
    fn choosing_one_answer_replaces_previous_selection() {
        let selected = BTreeSet::from([1, 2]);
        assert_eq!(
            toggle_selection(&selected, ImageQuestionMode::OneOfThree, 3),
            BTreeSet::from([3])
        );
        assert_eq!(selected, BTreeSet::from([1, 2]));
    }

    #[test]
    fn select_all_toggles_without_changing_other_choices() {
        let selected = BTreeSet::from([1, 2]);
        assert_eq!(
            toggle_selection(&selected, ImageQuestionMode::SelectAll, 2),
            BTreeSet::from([1])
        );
        assert_eq!(
            toggle_selection(&selected, ImageQuestionMode::SelectAll, 3),
            BTreeSet::from([1, 2, 3])
        );
    }

    #[test]
    fn approval_requires_loaded_image_and_valid_question() {
        let state = Rc::new(ImageVocabularyState::default()).reduce(Action::Generated(generated()));
        let state = state.reduce(Action::Approve);
        assert!(!state.approved);
        let state = state.reduce(Action::ImageLoaded).reduce(Action::Approve);
        assert!(state.approved);
    }

    #[test]
    fn missing_or_invalid_selection_cannot_submit() {
        let state = Rc::new(ImageVocabularyState::default())
            .reduce(Action::Generated(generated()))
            .reduce(Action::ImageLoaded)
            .reduce(Action::Approve)
            .reduce(Action::Toggle(999))
            .reduce(Action::Check);
        assert!(!state.checked);
        assert!(state.selected.is_empty());
        let state = state.reduce(Action::Toggle(1)).reduce(Action::Check);
        assert!(state.checked);
    }

    #[test]
    fn image_failure_revokes_approval_and_answers() {
        let state = Rc::new(ImageVocabularyState::default())
            .reduce(Action::Generated(generated()))
            .reduce(Action::ImageLoaded)
            .reduce(Action::Approve)
            .reduce(Action::Toggle(1))
            .reduce(Action::ImageFailed);
        assert!(state.image_failed);
        assert!(!state.approved);
        assert!(state.selected.is_empty());
        assert!(!state.checked);
    }

    #[test]
    fn selecting_the_existing_mode_preserves_the_current_question() {
        let state = Rc::new(ImageVocabularyState::default())
            .reduce(Action::Generated(generated()))
            .reduce(Action::ImageLoaded)
            .reduce(Action::Approve);
        let same = state
            .clone()
            .reduce(Action::ChooseMode(ImageQuestionMode::OneOfThree));
        assert!(Rc::ptr_eq(&state, &same));
        assert!(same.approved);
    }

    #[test]
    fn switching_modes_resets_the_question_and_keeps_selected_mode() {
        let state = Rc::new(ImageVocabularyState::default())
            .reduce(Action::Generated(generated()))
            .reduce(Action::ImageLoaded)
            .reduce(Action::Approve)
            .reduce(Action::ChooseMode(ImageQuestionMode::SelectAll))
            .reduce(Action::Start);
        assert_eq!(state.mode, ImageQuestionMode::SelectAll);
        assert!(state.busy);
        assert!(!state.approved);
        assert!(state.question.is_none());
    }

    #[test]
    fn new_generation_clears_all_previous_question_state() {
        let state = Rc::new(ImageVocabularyState::default())
            .reduce(Action::Generated(generated()))
            .reduce(Action::ImageLoaded)
            .reduce(Action::Approve)
            .reduce(Action::Toggle(1))
            .reduce(Action::Check)
            .reduce(Action::Start);
        assert_eq!(*state, ImageVocabularyState::default());
    }
}
