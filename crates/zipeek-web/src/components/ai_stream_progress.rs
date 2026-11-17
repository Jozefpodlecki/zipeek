use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct AiStreamProgressProps {
    pub phase: AttrValue,
    pub received_chars: usize,
}

#[function_component(AiStreamProgress)]
pub(crate) fn ai_stream_progress(props: &AiStreamProgressProps) -> Html {
    let stream_progress_text = if props.received_chars == 0 {
        "Waiting for the first response from the model…".to_owned()
    } else if props.phase.contains("reasoning") {
        format!(
            "Model is active ({} reasoning characters received).",
            props.received_chars
        )
    } else {
        format!(
            "Received {} generated characters. Processing continues…",
            props.received_chars
        )
    };

    html! {
        <div
            data-ai-stream-progress=""
            data-ai-stream-chars={props.received_chars.to_string()}
            role="status"
            aria-live="polite"
            aria-atomic="true"
            class="flex items-center gap-3 rounded-xl border border-line bg-surface px-4 py-3 text-sm"
        >
            <span
                aria-hidden="true"
                class="h-5 w-5 shrink-0 animate-spin rounded-full border-2 border-line border-t-blue-500 motion-reduce:animate-none"
            />
            <div
                class="min-w-0"
            >
                <p
                    class="font-medium"
                >
                    {props.phase.clone()}
                </p>
                <p
                    class="mt-1 text-xs text-muted"
                >
                    {stream_progress_text}
                </p>
            </div>
        </div>
    }
}
