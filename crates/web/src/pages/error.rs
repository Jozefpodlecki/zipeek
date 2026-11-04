use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorPageProps {
    #[prop_or_default]
    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub message: Option<AttrValue>,
    #[prop_or_default]
    pub on_retry: Option<Callback<()>>,
}

#[function_component(ErrorPage)]
pub fn error_page(props: &ErrorPageProps) -> Html {
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| AttrValue::from("Something went wrong"));
    let message = props
        .message
        .clone()
        .unwrap_or_else(|| AttrValue::from("An unexpected error occurred."));

    let retry_button = if let Some(on_retry) = props.on_retry.clone() {
        let onclick = Callback::from(move |_| on_retry.emit(()));
        html! {
            <button
                type="button"
                {onclick}
                class="rounded border border-current px-4 py-2 hover:opacity-80"
            >
                {"Retry"}
            </button>
        }
    } else {
        Html::default()
    };

    html! {
        <section class="flex min-h-screen flex-col items-center justify-center gap-6 p-8 text-center">
            <h1 class="text-3xl font-bold">{ title }</h1>
            <p class="text-lg">{ message }</p>
            { retry_button }
        </section>
    }
}
