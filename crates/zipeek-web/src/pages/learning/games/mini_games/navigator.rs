use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::mini_games::{Direction, NavigatorState};

#[derive(Properties, PartialEq)]
pub struct NavigatorRoundProps {
    pub game: NavigatorState,
    pub on_move: Callback<Direction>,
    pub on_listen: Callback<MouseEvent>,
    pub audio_error: Option<String>,
}

#[function_component(NavigatorRound)]
pub fn navigator_round(props: &NavigatorRoundProps) -> Html {
    if props.game.current().is_none() {
        return Html::default();
    }
    html! {
        <section
            data-listening-navigator=""
            class="space-y-5 rounded-2xl border border-line bg-raised p-6 text-center"
        >
            <p class="text-muted">{"Listen to the Mandarin direction, then move through the city."}</p>
            <button
                type="button"
                data-navigator-listen=""
                onclick={props.on_listen.clone()}
                class="inline-flex min-h-11 items-center justify-center gap-2 rounded-xl bg-brand-action px-5 py-3 font-semibold text-white"
            >
                <Icon data={IconData::LUCIDE_VOLUME_2} width="20" height="20" />
                {"Listen to direction"}
            </button>
            if let Some(error) = &props.audio_error {
                <p role="alert" class="text-sm text-red-400">{error.clone()}</p>
            }
            <div class="mx-auto grid w-full max-w-xs grid-cols-3 gap-2" aria-label="City grid">
                {for (0..9).map(|index| {
                    let x = index % 3;
                    let y = index / 3;
                    let occupied = props.game.position == (x as i8, y as i8);
                    html! {
                        <div
                            key={index}
                            class={if occupied { "grid aspect-square place-items-center rounded-lg border border-line bg-brand-action text-white" } else { "grid aspect-square place-items-center rounded-lg border border-line bg-surface" }}
                        >
                            if occupied {
                                <Icon data={IconData::LUCIDE_MAP_PIN} width="24" height="24" />
                            }
                        </div>
                    }
                })}
            </div>
            <div class="grid grid-cols-4 gap-2" aria-label="Movement controls">
                {for Direction::ALL.into_iter().map(|direction| {
                    let on_move = props.on_move.reform(move |_: MouseEvent| direction);
                    html! {
                        <button
                            key={direction.label()}
                            type="button"
                            data-navigator-direction={direction.label()}
                            disabled={props.game.last_wrong_direction == Some(direction)}
                            onclick={on_move}
                            class="min-h-12 rounded-xl border border-line bg-surface font-semibold hover:border-accent focus-visible:outline-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            {direction.label()}
                        </button>
                    }
                })}
            </div>
        </section>
    }
}
