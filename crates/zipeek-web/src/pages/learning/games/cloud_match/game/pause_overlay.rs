use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::cloud_match::CloudGame;

#[derive(Properties, PartialEq)]
pub(crate) struct CloudPauseOverlayProps {
    pub round: CloudGame,
    pub on_resume: Callback<MouseEvent>,
}

#[function_component(CloudPauseOverlay)]
pub(crate) fn cloud_pause_overlay(props: &CloudPauseOverlayProps) -> Html {
    let resume_button = use_node_ref();
    {
        let resume_button = resume_button.clone();
        use_effect_with((), move |_| {
            if let Some(button) = resume_button.cast::<web_sys::HtmlElement>() {
                let _ = button.focus();
            }
        });
    }
    let on_dialog_keydown = {
        let resume_button = resume_button.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Tab" {
                event.prevent_default();
                if let Some(button) = resume_button.cast::<web_sys::HtmlElement>() {
                    let _ = button.focus();
                }
            }
        })
    };
    let matched = props.round.matched_total;
    let points = props.round.score;
    let mistakes = props.round.mistakes;

    html! {
        <div
            class="absolute inset-0 z-20 grid place-items-center bg-[#07152aa3] px-4 py-8 backdrop-blur-[7px] motion-reduce:backdrop-blur-none"
            data-cloud-match-pause=""
            role="dialog"
            aria-modal="true"
            aria-labelledby="cloud-match-pause-title"
            aria-describedby="cloud-match-pause-description"
            onkeydown={on_dialog_keydown}
        >
            <div
                class="w-full max-w-sm rounded-3xl border border-white/25 bg-[linear-gradient(145deg,rgba(24,48,80,.96),rgba(12,29,54,.98))] p-6 text-center shadow-[0_26px_90px_rgba(2,15,32,.42)] sm:p-8"
            >
                <span
                    class="mx-auto mb-4 grid size-14 place-items-center rounded-2xl border border-white/20 bg-white/10 text-white"
                    aria-hidden="true"
                >
                    <Icon
                        data={IconData::LUCIDE_PAUSE}
                        width="26"
                        height="26"
                    />
                </span>
                <h2
                    id="cloud-match-pause-title"
                    class="text-2xl font-bold tracking-tight text-white"
                >
                    {"Game paused"}
                </h2>
                <p
                    id="cloud-match-pause-description"
                    class="mt-2 text-sm text-blue-100/80"
                >
                    {"Your game is paused. Resume when you're ready."}
                </p>
                <dl
                    class="my-6 grid grid-cols-3 divide-x divide-white/15 rounded-2xl border border-white/10 bg-white/5 py-3 text-white"
                >
                    <div class="px-2">
                        <dt class="text-xs text-blue-100/70">{if props.round.length.target().is_some() { "Pairs" } else { "Pairs matched" }}</dt>
                        <dd class="mt-1 text-lg font-bold tabular-nums"><span class="inline-flex items-center gap-1">
                            {if let Some(target) = props.round.length.target() {
                                html! { <>{matched}{" / "}{target}</> }
                            } else {
                                html! { <>{matched}</> }
                            }}
                        </span></dd>
                    </div>
                    <div class="px-2">
                        <dt class="text-xs text-blue-100/70">{"Score"}</dt>
                        <dd class="mt-1 text-lg font-bold tabular-nums">{points}</dd>
                    </div>
                    <div class="px-2">
                        <dt class="text-xs text-blue-100/70">{"Mistakes"}</dt>
                        <dd class="mt-1 text-lg font-bold tabular-nums">{mistakes}</dd>
                    </div>
                </dl>
                <button
                    ref={resume_button}
                    type="button"
                    class="flex w-full items-center justify-center gap-2 rounded-xl bg-brand-action px-5 py-3 font-semibold text-white shadow-lg shadow-blue-950/25 hover:brightness-110 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-blue-200"
                    onclick={props.on_resume.clone()}
                >
                    <Icon
                        data={IconData::LUCIDE_PLAY}
                        width="19"
                        height="19"
                    />
                    {"Resume game"}
                </button>
            </div>
        </div>
    }
}
