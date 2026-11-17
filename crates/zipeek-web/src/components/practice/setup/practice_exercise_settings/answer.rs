use yew::prelude::*;

use crate::models::VocabularyExerciseOptions;
use crate::reducers::practice_setup_flow::PracticeSetupAction;

use super::super::practice_answer_settings::PracticeAnswerSettings;

#[derive(Properties, PartialEq)]
pub(super) struct AnswerSettingsProps {
    pub options: VocabularyExerciseOptions,
    pub on_action: Callback<PracticeSetupAction>,
}

#[function_component(AnswerSettings)]
pub(super) fn answer_settings(props: &AnswerSettingsProps) -> Html {
    let builder = matches!(
        props.options,
        VocabularyExerciseOptions::CharacterBuilder(_)
    );
    let on_retry = {
        let on_action = props.on_action.clone();
        Callback::from(move |value| {
            on_action.emit(if builder {
                PracticeSetupAction::SetCharacterBuilderRetry(value)
            } else {
                PracticeSetupAction::SetTypePinyinRetry(value)
            })
        })
    };
    let on_audio = {
        let on_action = props.on_action.clone();
        Callback::from(move |value| {
            on_action.emit(PracticeSetupAction::SetCharacterBuilderAudio(value));
        })
    };
    let on_auto = {
        let on_action = props.on_action.clone();
        Callback::from(move |value| {
            on_action.emit(PracticeSetupAction::SetCharacterBuilderAutoCheck(value));
        })
    };
    let on_hint = {
        let on_action = props.on_action.clone();
        Callback::from(move |value| {
            on_action.emit(PracticeSetupAction::SetTypePinyinHint(value));
        })
    };
    html! {
        <PracticeAnswerSettings
            options={props.options}
            on_retry_incorrect={on_retry}
            on_show_audio={on_audio}
            on_auto_check={on_auto}
            on_pinyin_hint={on_hint}
        />
    }
}
