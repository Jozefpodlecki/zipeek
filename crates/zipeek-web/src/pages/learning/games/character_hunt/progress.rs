use yew::prelude::*;

use super::{state::HuntSessionContext, summary::HuntSummary};
use crate::components::ListenButton;

#[function_component(HuntProgress)]
pub(super) fn hunt_progress() -> Html {
    let context = use_context::<HuntSessionContext>()
        .expect("HuntProgress must be rendered within HuntSession");
    let session = &context.0;
    let Some(board) = session.board.as_ref() else {
        return Html::default();
    };
    let found = board.found.len();
    let total = board.total_words();
    let percentage = if total == 0 { 0 } else { found * 100 / total };

    let assistance_message = {
        if session.consecutive_misses >= 3 {
            let assistance_warning = {
                if session.consecutive_misses >= 7 {
                    "Reveal word is available."
                } else {
                    "Use Hint to highlight an unfinished word."
                }
            };
            html! { <p
                data-hunt-assistance=""
                class="text-xs text-accent"
            >
                {assistance_warning}
            </p> }
        } else {
            Html::default()
        }
    };

    let hint_content = {
        if session.hint_visible {
            board
                .hint()
                .map(|(first, count)| {
                    html! {
                        <p
                            data-hunt-hint=""
                            class="text-xs text-accent"
                        >
                            {format!("Try a {count}-character word beginning with {first}.")}
                        </p>
                    }
                })
                .unwrap_or_default()
        } else {
            Html::default()
        }
    };
    let final_hunt_round_notice =
        if context.1.is_none() && board.complete() && session.is_last_round() {
            html! {<div
                class="mt-4"
            >
                <HuntSummary />
            </div>}
        } else {
            Html::default()
        };
    html! {
        <aside
            data-hunt-progress-panel=""
            class="flex min-h-0 flex-col overflow-hidden rounded-2xl border border-line bg-surface lg:h-full"
        >
            <div
                class="shrink-0 space-y-3 border-b border-line p-4"
            >
                <div
                    class="flex items-center justify-between gap-3"
                >
                    <h2
                        class="font-semibold"
                    >
                        {"Words found"}
                    </h2>
                    <span
                        class="text-sm tabular-nums"
                        data-hunt-progress=""
                    >
                        {format!("{found} / {total}")}
                    </span>
                </div>
                <div
                    role="progressbar"
                    aria-label="Words found"
                    aria-valuemin="0"
                    aria-valuemax={total.to_string()}
                    aria-valuenow={found.to_string()}
                    class="h-2 overflow-hidden rounded-full bg-raised"
                >
                    <div
                        class="h-full rounded-full bg-brand-action transition-[width] duration-150"
                        style={format!("width: {percentage}%")}
                    >
                    </div>
                </div>
                <p
                    class="text-xs text-muted"
                >
                    {format!("{} incorrect attempts", board.mistakes)}
                </p>
                {assistance_message}
                <p
                    data-hunt-feedback=""
                    role="status"
                    aria-live="polite"
                    class="text-sm"
                >
                    {session.feedback.clone()}
                </p>
                {hint_content}
            </div>
            <div
                data-hunt-completed-scroll=""
                class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-4"
            >
                <h3
                    class="mb-3 font-semibold"
                >
                    {"Completed words"}
                </h3>
                <ul
                    class="grid gap-2"
                    data-hunt-found=""
                >
                    {for board.completed_words().into_iter().map(|word| html! {
                        <li
                            key={word.hanzi.clone()}
                            class="flex items-center justify-between gap-2 rounded-xl border border-line bg-raised px-3 py-2"
                        >
                            <div
                                class="min-w-0 flex-1"
                            >
                                <span
                                    class="flex flex-wrap items-baseline gap-x-2"
                                >
                                    <span
                                        class="text-lg font-semibold"
                                        data-hunt-found-word=""
                                    >
                                        {word.hanzi.clone()}
                                    </span>
                                    {if let Some(reading) = word.pinyin {
                                        html! {<span
                                            class="text-xs text-muted"
                                            data-hunt-found-pinyin=""
                                        >
                                            {reading}
                                        </span>}
                                    } else { Html::default() }}
                                </span>
                                <span
                                    class="block text-sm text-muted"
                                    data-hunt-found-meaning=""
                                >
                                    {word.meaning.unwrap_or_else(|| "Definition unavailable".to_owned())}
                                </span>
                            </div>
                            <span
                                data-hunt-word-audio=""
                                class="shrink-0"
                            >
                                <ListenButton
                                    hanzi={word.hanzi}
                                    compact={true}
                                />
                            </span>
                        </li>
                    })}
                </ul>
                {final_hunt_round_notice}
            </div>
        </aside>
    }
}
