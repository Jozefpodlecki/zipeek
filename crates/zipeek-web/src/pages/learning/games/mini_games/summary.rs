use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub struct GameSummaryProps {
    pub rounds: usize,
    pub mistakes: u32,
    pub on_restart: Callback<MouseEvent>,
}

#[function_component(GameSummary)]
pub fn game_summary(props: &GameSummaryProps) -> Html {
    html! {
        <section
            data-game-complete=""
            class="space-y-5 rounded-2xl border border-line bg-raised p-6 text-center"
        >
            <Icon data={IconData::LUCIDE_TROPHY} width="48" height="48" />
            <h2 class="text-2xl font-bold">{"Game complete"}</h2>
            <p>{format!("{} rounds completed, {} {}", props.rounds, props.mistakes, if props.mistakes == 1 { "mistake" } else { "mistakes" })}</p>
            <p class="text-sm text-muted">{"Game results do not affect Practice proficiency."}</p>
            <button
                type="button"
                onclick={props.on_restart.clone()}
                class="min-h-11 rounded-xl bg-brand-action px-5 py-3 font-semibold text-white"
            >
                {"Play again"}
            </button>
        </section>
    }
}
