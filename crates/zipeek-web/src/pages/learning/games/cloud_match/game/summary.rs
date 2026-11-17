use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::cloud_match::CloudGame;

pub(crate) fn view(
    game: &CloudGame,
    replay: Callback<MouseEvent>,
    back: Callback<MouseEvent>,
) -> Html {
    let heading = if game.completed_all_pairs() {
        "All pairs matched"
    } else {
        "Game ended"
    };
    let review = game.review_with_reasons().collect::<Vec<_>>();
    html! {
        <section
            data-component="cloud-match-summary"
            role="region"
            aria-label="Cloud Match results"
            class="absolute inset-0 z-20 overflow-y-auto bg-[#102646]/95 px-5 py-6 text-white"
        >
            <div class="mx-auto flex max-w-xl flex-col items-center gap-4 text-center">
                <h2 class="text-3xl font-bold">{heading}</h2>
                <p class="text-lg font-semibold">
                    {format!("{} / {} pairs matched · {} points", game.matched_total, game.length.target().map_or_else(|| "∞".to_owned(), |n| n.to_string()), game.score)}
                </p>
                <dl class="grid w-full grid-cols-2 gap-3 text-sm sm:grid-cols-4">
                    <div class="rounded-lg bg-white/10 p-3">
                        <dt>{"Mistakes"}</dt>
                        <dd class="text-xl font-bold tabular-nums">{game.mistakes}</dd>
                    </div>
                    <div class="rounded-lg bg-white/10 p-3">
                        <dt>{"Timed-out waves"}</dt>
                        <dd class="text-xl font-bold tabular-nums">{game.timed_out_waves}</dd>
                    </div>
                    <div class="rounded-lg bg-white/10 p-3">
                        <dt>{"Best combo"}</dt>
                        <dd class="text-xl font-bold tabular-nums">{game.best_combo}</dd>
                    </div>
                    <div class="rounded-lg bg-white/10 p-3">
                        <dt>{"Active time"}</dt>
                        <dd class="text-xl font-bold tabular-nums">{format!("{}s", game.elapsed_total)}</dd>
                    </div>
                </dl>
                {if review.is_empty() {
                    html! {<p role="status">{"No pairs need review."}</p>}
                } else {
                    html! {
                        <div class="w-full text-left">
                            <h3 class="mb-2 text-lg font-semibold">{"Pairs to review"}</h3>
                            <ul class="max-h-52 space-y-2 overflow-y-auto" data-cloud-match-review="true">
                                {for review.iter().map(|(pair, reason)| {
                                    let (question, answer) = pair.labels(game.mode);
                                    html! {
                                        <li key={pair.id.to_string()} class="rounded-lg bg-white/10 px-4 py-2">
                                            <span class="font-semibold">{question}</span>
                                            <span class="px-2" aria-hidden="true">{"—"}</span>
                                            <span>{answer}</span>
                                            <span class="ml-2 text-sm text-white/80">
                                                {match (reason.incorrect_pair, reason.wave_expired) {
                                                    (true, true) => "Incorrect attempt · Wave expired",
                                                    (true, false) => "Incorrect attempt",
                                                    (false, true) => "Wave expired",
                                                    (false, false) => "Review",
                                                }}
                                            </span>
                                        </li>
                                    }
                                })}
                            </ul>
                        </div>
                    }
                }}
                <div class="flex flex-wrap justify-center gap-3">
                    <button
                        type="button"
                        class="inline-flex items-center gap-2 rounded-xl bg-blue-500 px-5 py-3 font-bold text-white focus-visible:outline-4"
                        onclick={replay}
                    ><Icon data={IconData::LUCIDE_PLAY} width="18" height="18" />{"Play again"}</button>
                    <button
                        type="button"
                        class="inline-flex items-center gap-2 rounded-xl border border-white/50 px-5 py-3 font-semibold text-white focus-visible:outline-4"
                        onclick={back}
                    ><Icon data={IconData::LUCIDE_ARROW_LEFT} width="18" height="18" />{"Back to setup"}</button>
                </div>
            </div>
        </section>
    }
}
