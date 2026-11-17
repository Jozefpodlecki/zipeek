use yew::prelude::*;
use yew_icons::{Icon, IconData};

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
                class="inline-flex min-h-12 items-center justify-center gap-2 rounded-xl bg-brand-action px-6 py-3 font-semibold text-white hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
                <Icon
                    data={IconData::LUCIDE_REFRESH_CW}
                    width="18px"
                    height="18px"
                />
                {"Retry"}
            </button>
        }
    } else {
        Html::default()
    };

    html! {
        <section
            class="flex min-h-[65vh] items-center justify-center px-4 py-12"
        >
            <div
                class="w-full max-w-xl rounded-3xl border border-line bg-surface/95 px-6 py-10 text-center shadow-2xl sm:px-10 sm:py-12"
            >
                <div
                    aria-hidden="true"
                    class="mx-auto flex size-16 items-center justify-center rounded-2xl border border-line bg-raised text-accent"
                >
                    <Icon
                        data={IconData::LUCIDE_INFO}
                        width="30px"
                        height="30px"
                    />
                </div>
                <h1
                    class="mt-6 text-3xl font-bold leading-tight tracking-tight text-ink"
                >
                    {title}
                </h1>
                <p
                    class="mx-auto mt-4 max-w-md text-base leading-relaxed text-muted"
                >
                    {message}
                </p>
                <div
                    class="mt-7"
                >
                    {retry_button}
                </div>
            </div>
        </section>
    }
}
