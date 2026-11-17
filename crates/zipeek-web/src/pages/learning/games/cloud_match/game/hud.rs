use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::cloud_match::CloudGame;

pub(crate) fn view(
    round: &CloudGame,
    feedback: &str,
    on_pause: Callback<MouseEvent>,
    on_stop: Callback<MouseEvent>,
    on_finish_endless: Callback<MouseEvent>,
) -> Html {
    html! {
        <div
            data-component="cloud-match-hud"
            class="relative z-[5] flex min-h-16 flex-wrap items-center justify-between gap-3 border-b border-sky-200/20 bg-[#10243e]/95 px-4 py-3 text-white shadow-[0_6px_25px_rgba(2,15,36,.22)] sm:px-7 max-sm:gap-2"
        >
            <div
                class="flex flex-wrap items-center gap-x-3 gap-y-2 sm:gap-x-4"
            >
                <span
                    key={round.score}
                    class="inline-flex items-center gap-2 rounded-xl border border-white/15 bg-white/10 px-3 py-1.5 text-lg font-bold tabular-nums shadow-sm"
                    aria-label={format!("Score {}", round.score)}
                >
                    <Icon data={IconData::LUCIDE_STAR} width="17" height="17" />
                    {round.score}
                </span>
                <span
                    class="rounded-lg border border-sky-200/10 bg-sky-200/5 px-2.5 py-1.5 text-sm font-semibold tabular-nums text-sky-200"
                >
                    {format!("Combo ×{}", round.combo)}
                </span>
                <span
                    key={round.lives}
                    class="inline-flex items-center gap-1.5 rounded-lg border border-rose-200/10 bg-rose-200/5 px-2.5 py-1.5 text-sm font-semibold text-rose-200"
                    aria-label={if round.infinite_lives {"Unlimited lives".to_owned()} else {format!("{} lives remaining", round.lives)}}
                >
                    <Icon data={IconData::LUCIDE_HEART} width="15" height="15" />
                    {if round.infinite_lives { html! { <span class="text-xs text-rose-100">{"Unlimited"}</span> } } else { html! { <>{round.lives}</> } }}
                </span>
                <span
                    class="inline-flex items-center gap-1.5 rounded-lg border border-sky-200/10 px-2.5 py-1.5 text-sm font-semibold tabular-nums text-sky-100"
                    aria-label={if round.is_continuous() {
                        format!("{} seconds elapsed", round.elapsed_total)
                    } else {
                        format!("{} seconds remaining", round.remaining_seconds())
                    }}
                >
                    <Icon data={IconData::LUCIDE_CLOCK} width="15" height="15" />
                    {if round.is_continuous() {
                        format!("{}s", round.elapsed_total)
                    } else {
                        format!("{}s", round.remaining_seconds())
                    }}
                </span>
                <span
                    class="inline-flex items-center gap-1 rounded-full border border-sky-200/25 bg-sky-200/10 px-3 py-1 text-xs font-semibold tabular-nums text-[#d7ecff] max-sm:text-[.65rem]"
                >
                    {if round.is_continuous() { "Continuous · ".to_owned() } else { format!("Wave {} · ", round.wave + 1) }}
                    {round.matched_total}
                    {" / "}
                    {if let Some(target) = round.length.target() {
                        html! { <>{target}</> }
                    } else {
                        html! { <span>{"Endless"}</span> }
                    }}
                    {" matched"}
                </span>
                <span
                    key={format!("{}-{}-{}", round.score, round.lives, feedback)}
                    role="status"
                    aria-live="polite"
                    class="min-w-0 text-xs font-semibold text-[#d9edff] transition-opacity duration-200 max-sm:text-[.7rem]"
                >
                    {feedback.to_owned()}
                </span>
            </div>
            <div
                class="flex items-center gap-2"
            >
                <button
                    type="button"
                    class="grid size-10 place-items-center rounded-xl border border-white/30 bg-white/10 text-lg font-bold text-white transition-colors hover:enabled:bg-white/25 focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-sky-300 disabled:opacity-50"
                    aria-label={if round.paused {"Resume game"} else {"Pause game"}}
                    title={if round.paused {"Resume"} else {"Pause"}}
                    disabled={round.finished}
                    onclick={on_pause}
                >
                    {if round.paused { html! { <Icon data={IconData::LUCIDE_PLAY} width="19" height="19" /> } } else { html! { <Icon data={IconData::LUCIDE_PAUSE} width="19" height="19" /> } }}
                </button>
                {if (matches!(round.length, zipeek_exercises::cloud_match::GameLength::Endless) || round.is_continuous()) && !round.finished {
                    html! {<button type="button" class="rounded-lg border border-sky-200/50 px-3 py-2 text-sm font-semibold text-sky-100"
                        onclick={on_finish_endless}>{"Finish"}</button>}
                } else { Html::default() }}
                <button
                    type="button"
                    class="grid size-10 place-items-center rounded-xl border border-white/30 bg-white/10 text-lg font-bold text-white transition-colors hover:enabled:bg-white/25 focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-sky-300 disabled:opacity-50"
                    aria-label="Stop game and return to setup"
                    title="Stop game"
                    onclick={on_stop}
                >
                    <Icon data={IconData::LUCIDE_X} width="19" height="19" />
                </button>
            </div>
        </div>
    }
}
