use super::state::AiSettingsState;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

pub(super) fn status(state: &AiSettingsState) -> Html {
    if state.message.is_empty() || state.test_busy() {
        return Html::default();
    }
    let failed = state.test_ok == Some(false);
    html! {
        <div data-ai-diagnostic-status="" role={if failed {"alert"} else {"status"}}
            class={classes!("mt-3", "flex", "items-start", "gap-2", "rounded-lg", "px-3", "py-2", "text-sm", if failed {"bg-amber-500/10 text-amber-700 dark:text-amber-300"} else {"bg-raised/70 text-muted"})}>
            if failed {
                <Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="16px" height="16px" />
            }
            <span>{&state.message}</span>
        </div>
    }
}

pub(super) fn trace(state: &AiSettingsState) -> Html {
    let ai_trace = &state.trace;
    if let Some(trace) = &*ai_trace {
        html! {
            <details
                data-ai-connection-trace=""
                class="rounded-xl border border-line bg-raised p-4 text-sm"
            >
                <summary
                    class="cursor-pointer font-semibold"
                >
                    {"Inspect diagnostic request and response"}
                </summary>
                <p
                    class="mt-3 text-xs text-muted"
                >
                    {"Sent to the selected model. The API key and Authorization header are never included here."}
                </p>
                <p
                    class="mt-2 font-semibold"
                >
                    {format!("Model: {}", trace.model)}
                </p>
                {for trace.messages.iter().map(|message| html! {
                    <div
                        class="mt-2"
                    >
                        <p
                            class="text-xs font-semibold text-muted"
                        >
                        {&message.role}
                        </p><pre
                            class="whitespace-pre-wrap break-words text-xs"
                        >
                            {&message.content}
                        </pre>
                        </div>
                })}
                <p
                    class="mt-3 text-xs font-semibold text-muted"
                >
                    {"Model reply"}
                </p>
                <pre
                    class="whitespace-pre-wrap break-words text-xs"
                >
                    {&trace.reply}
                </pre>
                {if trace.truncated {html! {
                    <p
                        class="text-xs text-muted"
                    >
                        {"Reply truncated to 600 characters."}
                    </p>}} else {Html::default()}}
            </details>
        }
    } else {
        Html::default()
    }
}
