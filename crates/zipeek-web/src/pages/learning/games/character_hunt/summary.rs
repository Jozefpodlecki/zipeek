use yew::prelude::*;
use yew_router::prelude::Link;

use crate::route::Route;

use super::state::HuntSessionContext;

#[function_component(HuntSummary)]
pub(super) fn hunt_summary() -> Html {
    let context = use_context::<HuntSessionContext>()
        .expect("HuntSummary must be rendered within HuntSession");
    let session = &context.0;
    let Some(board) = session.board.as_ref() else {
        return Html::default();
    };
    let Some(summary) = session.stats.with_completed_round(board) else {
        return Html::default();
    };

    html! {
        <div
            role="status"
            data-hunt-finished=""
            class="space-y-3"
        >
            <p
                class="font-semibold text-accent"
            >
                {"Practice complete"}
            </p>
            <div
                data-hunt-session-summary=""
                class="space-y-1 text-sm tabular-nums"
            >
                <p>
                    {format!("{} rounds completed", summary.completed_rounds)}
                </p>
                <p>
                    {format!("{} words found", summary.found_words)}
                </p>
                <p>
                    {format!("{} incorrect attempts", summary.mistakes)}
                </p>
                <p>
                    {format!("{} words revealed with assistance", session.revealed_words)}
                </p>
            </div>
            <Link<Route>
                to={Route::PracticeSetup}
                classes="inline-flex min-h-11 items-center rounded-xl border border-line px-5"
            >
                {"Back to practice"}
            </Link<Route>>
        </div>
    }
}
