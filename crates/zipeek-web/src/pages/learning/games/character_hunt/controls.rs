use wasm_bindgen::JsCast;

use yew::prelude::*;

use super::{
    exit::HuntExit,
    state::{HuntAction, HuntSessionContext},
};

#[function_component(HuntControls)]
pub(super) fn hunt_controls() -> Html {
    let context = use_context::<HuntSessionContext>()
        .expect("HuntControls must be rendered within HuntSession");
    let session = &context.0;

    let completed = session.board.as_ref().is_some_and(|board| board.complete());
    let has_selection = session
        .board
        .as_ref()
        .is_some_and(|board| !board.selected.is_empty());
    let on_action = {
        let session = session.clone();
        Callback::from(move |event: MouseEvent| {
            let action = event
                .current_target()
                .map(|target| target.unchecked_into::<web_sys::HtmlElement>())
                .and_then(|element| element.dataset().get("action"));
            match action.as_deref() {
                Some("hint") => session.dispatch(HuntAction::ShowHint),
                Some("reveal") => session.dispatch(HuntAction::RevealWord),
                Some("next") => session.dispatch(HuntAction::NextRound(
                    chrono::Utc::now().timestamp_millis() as u64,
                )),
                _ => log::warn!("Unknown Character Hunt action: {action:?}"),
            }
        })
    };
    let last_round = session.is_last_round();
    let on_host_continue = {
        let on_completed = context.1.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(callback) = &on_completed {
                callback.emit(());
            }
        })
    };

    let completion_action = {
        if completed && context.1.is_some() {
            html! {
                <button
                    type="button"
                    data-hunt-session-continue=""
                    class="min-h-12 rounded-xl bg-brand-action px-6 py-2 font-semibold text-white"
                    onclick={on_host_continue}
                >
                    {"Continue"}
                </button>
            }
        } else if completed && !last_round {
            html! {
                <button
                    type="button"
                    data-hunt-next=""
                    data-action="next"
                    onclick={on_action.clone()}
                    class="min-h-12 rounded-xl bg-brand-action px-6 py-2 font-semibold text-white"
                >
                    {"Continue"}
                </button>
            }
        } else if completed && last_round {
            html! {
                <a
                    href="#/practice"
                    data-hunt-new-practice=""
                    class="inline-flex min-h-12 items-center rounded-xl bg-brand-action px-6 py-2 font-semibold text-white no-underline"
                >
                    {"Practice again"}
                </a>
            }
        } else {
            Html::default()
        }
    };
    let reveal_control = {
        if session.consecutive_misses >= 7 && !completed {
            html! {
                <button
                    type="button"
                    data-hunt-reveal=""
                    data-action="reveal"
                    onclick={on_action.clone()}
                    class="min-h-11 rounded-lg border border-line px-4 py-2 hover:border-accent"
                >
                    {"Reveal word"}
                </button>
            }
        } else {
            Html::default()
        }
    };

    html! {
        <footer
            data-hunt-actions=""
            class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-surface px-4 py-3"
        >
            <div
                class="flex flex-wrap items-center gap-2"
            >
                <button
                    type="button"
                    data-hunt-hint=""
                    data-action="hint"
                    onclick={on_action.clone()}
                    disabled={completed || has_selection}
                    class="min-h-11 rounded-lg border border-line px-4 py-2 hover:border-accent disabled:opacity-40"
                >
                    {"Hint"}
                </button>
                {reveal_control}
                {has_selection.then(|| html! {
                    <span
                        class="text-xs text-muted"
                    >
                        {"Tap a selected tile to deselect it"}
                    </span>
                }).unwrap_or_default()}
            </div>
            <div
                class="flex items-center gap-2"
            >
                {if context.1.is_none() { html! { <HuntExit /> } } else { Html::default() }}
                {completion_action}
            </div>
        </footer>
    }
}
