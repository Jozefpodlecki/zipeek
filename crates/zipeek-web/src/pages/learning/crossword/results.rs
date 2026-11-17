use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::{crossword::Crossword, practice_draft::HskBand};

#[derive(Properties, PartialEq)]
pub(super) struct CrosswordResultsProps {
    pub game: Crossword,
    pub bands: Vec<HskBand>,
    pub on_reset: Callback<MouseEvent>,
}

#[function_component(CrosswordResults)]
pub(super) fn crossword_results(props: &CrosswordResultsProps) -> Html {
    html! {
        <main
            class="flex min-h-0 flex-1 items-center justify-center px-4 py-8"
            aria-label="Crossword results"
        >
            <div
                class="w-full max-w-2xl rounded-3xl border border-line bg-raised px-6 py-8 text-center shadow-xl sm:px-10 sm:py-10"
            >
                <div
                    class="mx-auto mb-5 flex size-16 items-center justify-center rounded-2xl bg-brand-action/15 text-brand-action"
                >
                    <Icon
                        data={IconData::LUCIDE_TROPHY}
                        width="36"
                        height="36"
                    />
                </div>
                <p
                    class="text-xs font-bold uppercase tracking-widest text-muted"
                >
                    {"Puzzle completed"}
                </p>
                <h2
                    role="status"
                    aria-live="polite"
                    class="mt-2 text-3xl font-bold sm:text-4xl"
                >
                    {"Well done!"}
                </h2>
                <p
                    class="mx-auto mt-2 max-w-sm text-sm text-muted"
                >
                    {"Every character is in place. Ready for another challenge?"}
                </p>
                <div
                    class="my-7 grid grid-cols-3 gap-2 sm:gap-4"
                >
                    <div
                        class="rounded-2xl border border-line bg-surface px-2 py-5"
                    >
                        <strong
                            class="block text-2xl font-bold sm:text-3xl"
                        >
                            {props.game.score}
                        </strong>
                        <span
                            class="text-xs text-muted sm:text-sm"
                        >
                            {"Points"}
                        </span>
                    </div>
                    <div
                        class="rounded-2xl border border-line bg-surface px-2 py-5"
                    >
                        <strong
                            class="block text-2xl font-bold sm:text-3xl"
                        >
                            {props.game.clues().len()}
                        </strong>
                        <span
                            class="text-xs text-muted sm:text-sm"
                        >
                            {"Words"}
                        </span>
                    </div>
                    <div
                        class="rounded-2xl border border-line bg-surface px-2 py-5"
                    >
                        <strong
                            class="block text-2xl font-bold sm:text-3xl"
                        >
                            {props.game.mistakes}
                        </strong>
                        <span
                            class="text-xs text-muted sm:text-sm"
                        >
                            {"Mistakes"}
                        </span>
                    </div>
                </div>
                <p
                    class="mb-6 text-xs text-muted"
                >
                    {format!("HSK {}", props.bands.iter().map(|b| format!("{}.{}",b.system,b.level)).collect::<Vec<_>>().join(", "))}
                </p>
                <div
                    class="flex flex-wrap justify-center gap-3"
                >
                    <button
                        data-testid="crossword-play-again"
                        type="button"
                        class="inline-flex min-h-12 items-center gap-2 rounded-xl bg-brand-action px-6 py-3 font-semibold text-white hover:bg-brand-hover"
                        onclick={props.on_reset.clone()}
                    >
                        <Icon
                            data={IconData::LUCIDE_PLAY}
                            width="18"
                            height="18"
                        />{"Play again"}
                    </button>
                    <a
                        class="inline-flex min-h-12 items-center rounded-xl border border-line px-6 py-3 font-semibold hover:bg-surface"
                        href="#/games"
                    >
                        {"All games"}
                    </a>
                </div>
            </div>
        </main>
    }
}
