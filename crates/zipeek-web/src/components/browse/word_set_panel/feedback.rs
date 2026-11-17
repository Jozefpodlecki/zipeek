use crate::reducers::word_sets::WordSetsFeedback;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct WordSetMessagesProps {
    pub load_error: Option<String>,
    pub feedback: Option<WordSetsFeedback>,
    pub on_reload: Callback<MouseEvent>,
}

#[function_component(WordSetMessages)]
pub(super) fn word_set_messages(props: &WordSetMessagesProps) -> Html {
    html! {
        <>
            {props.load_error.as_ref().map(|error| html! {
                <div role="alert" class="text-sm text-red-600">
                    <p>{format!("Word Sets unavailable: {error}")}</p>
                    <button
                        type="button"
                        class="min-h-10 underline"
                        onclick={props.on_reload.clone()}
                    >
                        {"Retry loading"}
                    </button>
                </div>
            }).unwrap_or_default()}
            {match props.feedback.as_ref() {
                Some(WordSetsFeedback::Error(message)) => html! {
                    <p role="alert" class="text-sm text-red-600">{message}</p>
                },
                Some(WordSetsFeedback::Success(message)) => html! {
                    <p role="status" class="text-sm text-muted">{message}</p>
                },
                None => Html::default(),
            }}
        </>
    }
}
