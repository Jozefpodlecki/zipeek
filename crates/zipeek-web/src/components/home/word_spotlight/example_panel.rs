use yew::prelude::*;

pub(super) fn render(
    show: bool,
    busy: bool,
    result: &Option<Result<String, String>>,
    sense_order: Option<i32>,
) -> Html {
    if !show {
        return Html::default();
    }
    let example_generation_status = if busy {
        html! {
            <p
                class="text-muted"
            >
                {"Generating an example from your configured AI model…"}
            </p>
        }
    } else {
        match result {
            Some(Ok(sentence)) => html! {
                <p
                    class="whitespace-pre-line"
                >
                    {sentence}
                </p>
            },
            Some(Err(error)) => html! {
                <p
                    role="alert"
                    class="text-red-500"
                >
                    {error}
                </p>
            },
            None => html! {
                <p
                    class="text-muted"
                >
                    {"Ready to generate."}
                </p>
            },
        }
    };

    html! {
        <div
            data-ai-example=""
            role="status"
            class="w-full rounded-xl border border-line bg-raised p-4 text-left text-sm leading-relaxed"
        >
            <p
                class="mb-2 text-xs font-semibold text-accent"
            >
                {"AI-generated example · not a verified dictionary citation"}
            </p>
            if let Some(order) = sense_order {
                <p
                    data-example-sense-order={order.to_string()}
                    class="mb-2 text-xs text-muted"
                >
                    {format!("Based on meaning {order}")}
                </p>
            }
            {example_generation_status}
        </div>
    }
}
