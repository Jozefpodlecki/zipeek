use yew::prelude::*;
use zipeek_exercises::character_hunt::{Candidate, Difficulty};

use super::{
    board::HuntBoard,
    controls::HuntControls,
    progress::HuntProgress,
    state::{HuntAction, HuntSessionContext, HuntState},
};

#[derive(Properties, PartialEq)]
pub(super) struct HuntSessionProps {
    pub candidates: Vec<Candidate>,
    pub difficulty: Difficulty,
    pub rounds: Option<u16>,
    #[prop_or_default]
    pub instance_seed: Option<u64>,
    #[prop_or_default]
    pub on_completed: Option<Callback<()>>,
}

#[function_component(HuntSession)]
pub(super) fn hunt_session(props: &HuntSessionProps) -> Html {
    let candidates = props.candidates.clone();
    let difficulty = props.difficulty;
    let rounds = props.rounds;
    let seed = props
        .instance_seed
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64);
    let session = use_reducer(move || HuntState::new(candidates, difficulty, rounds, seed));

    {
        let session = session.clone();
        use_effect_with(
            (session.highlighted.clone(), session.round),
            move |(highlighted, _)| {
                let timeout = if highlighted.is_empty() {
                    None
                } else {
                    let session = session.clone();
                    Some(gloo::timers::callback::Timeout::new(950, move || {
                        session.dispatch(HuntAction::ClearHint);
                    }))
                };
                move || drop(timeout)
            },
        );
    }

    let hunt_board_content = {
        if session.board.is_some() {
            html! {
                <div
                    class="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(280px,0.72fr)]"
                >
                    <div
                        class="flex min-h-0 items-start justify-center overflow-hidden"
                    >
                        <HuntBoard />
                    </div>
                    <HuntProgress />
                </div>
            }
        } else {
            html! {<p
                role="alert"
                class="rounded-xl border border-line bg-surface p-5"
            >
                {session.feedback.clone()}
            </p>}
        }
    };
    html! {
        <ContextProvider<HuntSessionContext> context={HuntSessionContext(session.clone(), props.on_completed.clone())}>
            <div
                data-hunt-session=""
                class="flex min-h-0 flex-col gap-3 lg:h-[calc(100dvh-12rem)]"
            >
                <div
                    data-hunt-session-info=""
                    class="flex shrink-0 items-center justify-between gap-3 text-sm text-muted"
                >
                    <span>
                        {format!(
                        "Round {}{}",
                        session.round,
                        session.total_rounds.map(|count| format!(" / {count}")).unwrap_or_default(),
                    )}
                    </span>
                    <span>
                        {session.difficulty.label()}
                    </span>
                </div>
                {hunt_board_content}
                {if session.board.is_some() {
                    html! {<HuntControls />}
                } else { Html::default() }}
            </div>
        </ContextProvider<HuntSessionContext>>
    }
}
