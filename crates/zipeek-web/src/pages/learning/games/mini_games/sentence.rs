use yew::prelude::*;
use zipeek_exercises::mini_games::SentenceState;

#[derive(Properties, PartialEq)]
pub struct SentenceRoundProps {
    pub game: SentenceState,
    pub on_select: Callback<usize>,
    pub on_remove: Callback<usize>,
    pub on_undo: Callback<MouseEvent>,
    pub on_clear: Callback<MouseEvent>,
    pub on_check: Callback<MouseEvent>,
}

#[function_component(SentenceRound)]
pub fn sentence_round(props: &SentenceRoundProps) -> Html {
    let Some(question) = props.game.current() else {
        return Html::default();
    };
    html! {
        <section
            data-sentence-relay=""
            class="space-y-6 rounded-2xl border border-line bg-raised p-6"
        >
            <p class="text-sm text-muted">{"Arrange the tiles into a Chinese sentence."}</p>
            <h2 class="text-xl font-semibold">{question.english}</h2>
            <div
                class="flex min-h-16 flex-wrap items-center gap-2 rounded-xl border border-line bg-surface p-3"
                aria-label="Your sentence"
            >
                {for props.game.selected.iter().map(|&index| {
                    let on_remove = props.on_remove.reform(move |_: MouseEvent| index);
                    html! {
                        <button
                            key={index}
                            type="button"
                            aria-label={format!("Remove {} from your sentence", question.tokens[index])}
                            title="Remove tile"
                            data-sentence-selected-tile={index.to_string()}
                            onclick={on_remove}
                            class="min-h-11 rounded-lg bg-brand-action px-3 py-2 text-lg text-white focus-visible:outline-2 focus-visible:outline-accent"
                        >
                            {question.tokens[index]}
                        </button>
                    }
                })}
            </div>
            <div class="flex flex-wrap gap-2" aria-label="Available words">
                {for question.tokens[..question.count].iter().enumerate().map(|(index, word)| {
                    let selected = props.game.selected.contains(&index);
                    let on_select = props.on_select.reform(move |_: MouseEvent| index);
                    html! {
                        <button
                            key={index}
                            type="button"
                            data-sentence-tile={index.to_string()}
                            disabled={selected}
                            onclick={on_select}
                            class="min-h-12 rounded-xl border border-line bg-surface px-4 text-xl hover:border-accent disabled:opacity-40"
                        >
                            {*word}
                        </button>
                    }
                })}
            </div>
            <div class="flex flex-wrap gap-2">
                <button
                    type="button"
                    onclick={props.on_undo.clone()}
                    disabled={props.game.selected.is_empty()}
                    class="min-h-11 rounded-lg border border-line px-4 disabled:opacity-40"
                >
                    {"Undo"}
                </button>
                <button
                    type="button"
                    onclick={props.on_clear.clone()}
                    disabled={props.game.selected.is_empty()}
                    class="min-h-11 rounded-lg border border-line px-4 disabled:opacity-40"
                >
                    {"Clear"}
                </button>
                <button
                    type="button"
                    data-sentence-check=""
                    onclick={props.on_check.clone()}
                    disabled={props.game.selected.len() != question.count
                        || props.game.notice == Some("Tap any selected word to change its position")}
                    class="min-h-11 rounded-lg bg-brand-action px-5 font-semibold text-white disabled:opacity-40"
                >
                    {"Check sentence"}
                </button>
            </div>
        </section>
    }
}
