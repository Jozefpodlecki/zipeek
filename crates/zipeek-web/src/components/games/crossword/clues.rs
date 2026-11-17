use yew::prelude::*;
use zipeek_exercises::crossword::Crossword;

#[derive(Properties, PartialEq)]
pub(crate) struct CrosswordCluesProps {
    pub game: Crossword,
    pub active_clue: Option<u8>,
    pub on_activate: Callback<u8>,
}

#[function_component(CrosswordClues)]
pub(crate) fn crossword_clues(props: &CrosswordCluesProps) -> Html {
    html! {
        <section
            class="flex min-h-0 min-w-0 flex-1 flex-col rounded-2xl border border-line bg-surface p-2 sm:p-3"
            aria-label="Crossword clues"
        >
            <div
                class="grid min-h-0 flex-1 content-start grid-cols-1 gap-2 overflow-y-auto overscroll-contain pr-1 xl:grid-cols-2"
            >
                {for props.game.clues().iter().map(|clue| {
                    let number = clue.number;
                    let completed = props.game.clue_completed(clue);
                    let active = !completed && props.active_clue == Some(number);
                    let on_activate = props.on_activate.clone();
                    let on_activate_clue = Callback::from(move |_| on_activate.emit(number));
                    let element_classes_1 = classes!(
                                    "min-h-[4.25rem]", "border-l-[3px]", "border-l-transparent", "hover:enabled:bg-raised", "hover:enabled:border-line",
                                    "w-full",
                                    "rounded-xl",
                                    "border",
                                    "border-line",
                                    "p-3",
                                    "text-left",
                                    "transition-colors",
                                    "focus-visible:outline-2",
                                    "focus-visible:outline-offset-2",
                                    "focus-visible:outline-brand-action",
                                    if active { classes!("!border-accent", "!border-l-accent", "!bg-[color-mix(in_srgb,var(--accent)_16%,var(--surface))]") } else { Classes::new() },
                                    if completed { classes!("disabled:cursor-default", "disabled:border-line", "disabled:bg-[color-mix(in_srgb,var(--surface)_90%,var(--raised))]", "disabled:text-muted", "disabled:opacity-55") } else { Classes::new() },
                                );
                    html! {
                        <button
                            key={number}
                            type="button"
                            class={element_classes_1}
                            aria-pressed={active.to_string()}
                            disabled={completed}
                            aria-disabled={completed.to_string()}
                            onclick={on_activate_clue}
                        >
                            <span
                                class="sr-only"
                            >
                                {format!("Clue {}, {}", number, if clue.direction == zipeek_exercises::crossword::Direction::Across { "across" } else { "down" })}
                            </span>
                            <span
                                class="block text-sm font-medium leading-snug"
                            >
                                {clue.english.split('|').next().unwrap_or(&clue.english).trim().to_owned()}
                            </span>
                            <span
                                class="mt-1 block text-xs text-muted"
                            >
                                {clue.pinyin.clone()}
                            </span>
                        </button>
                    }
                })}
            </div>
        </section>
    }
}
