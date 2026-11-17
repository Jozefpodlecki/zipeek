mod available_tiles;
use self::available_tiles::AvailableTiles;
use web_sys::DragEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SentenceWordBankProps {
    pub words: Vec<String>,
    pub solution_positions: Vec<Option<usize>>,
    #[prop_or_default]
    pub target_language: String,
    pub order: Vec<usize>,
    pub selected: Vec<usize>,
    pub disabled: bool,
    pub on_add: Callback<usize>,
    pub on_remove: Callback<usize>,
    pub on_reorder: Callback<(usize, usize)>,
}

#[function_component(SentenceWordBank)]
pub fn sentence_word_bank(props: &SentenceWordBankProps) -> Html {
    let dragging = use_state(|| None::<usize>);
    let on_drop_answer = {
        let dragging = dragging.clone();
        let selected = props.selected.clone();
        let disabled = props.disabled;
        let add = props.on_add.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if !disabled {
                if let Some(index) = *dragging {
                    if !selected.contains(&index) {
                        add.emit(index);
                    }
                }
            }
            dragging.set(None);
        })
    };
    let allow_drop = Callback::from(|event: DragEvent| event.prevent_default());
    let selected_len = props.selected.len();
    html! {
        <div
            data-component="sentence-word-bank"
            class="flex flex-col gap-5"
        >
            <div
                class="flex flex-col gap-2"
            >
                <p
                    class="text-sm font-semibold"
                >
                    {format!("Your {} translation", if props.target_language == "zh" { "Chinese" } else { "English" })}
                </p>
                <div
                    data-translation-assembly=""
                    class="flex min-h-24 flex-wrap items-center gap-2 rounded-xl border-2 border-dashed border-blue-400 bg-surface p-3"
                    ondragover={allow_drop.clone()}
                    ondrop={on_drop_answer}
                >
                    <span
                        class="rounded-md border border-blue-500 bg-blue-50 px-2 py-2 text-xs font-bold text-blue-800"
                        aria-label="Start of sentence"
                    >
                        {"START"}
                    </span>
                    {for props.selected.iter().enumerate().map(|(position, index)| {
                        let index = *index;
                        let canonical_order = props.solution_positions.get(index).copied().flatten();
                        let is_first = canonical_order == Some(0);
                        let is_last = canonical_order.map(|position| position + 1 == props.words.len()).unwrap_or(false);
                        let remove = props.on_remove.clone();
                        let reorder = props.on_reorder.clone();
                        let dragging_start = dragging.clone();
                        let dragging_drop = dragging.clone();
                        let disabled = props.disabled;
                        let end = position + 1 == selected_len;
                        let move_left = if position > 0 { Some(props.selected[position - 1]) } else { None };
                        let move_right = if position + 1 < selected_len { Some(props.selected[position + 1]) } else { None };
                        let reorder_left = reorder.clone();
                        let reorder_right = reorder.clone();
                        let on_drag_start = Callback::from(move |_: DragEvent| dragging_start.set(Some(index)));
                        let on_drag_end = {
                            let dragging = dragging.clone();
                            Callback::from(move |_: DragEvent| dragging.set(None))
                        };
                        let on_drop = Callback::from(move |event: DragEvent| {
                            event.prevent_default();
                            event.stop_propagation();
                            if !disabled {
                                if let Some(from) = *dragging_drop {
                                    reorder.emit((from, index));
                                }
                            }
                            dragging_drop.set(None);
                        });
                        let on_remove_tile = Callback::from(move |_| remove.emit(index));
                        let on_add_separator = Callback::from(move |_| {
                                            if let Some(target) = move_left { reorder_left.emit((index, target)); }
                                        });
                        let on_add_space = Callback::from(move |_| {
                                            if let Some(next) = move_right { reorder_right.emit((next, index)); }
                                        });
                        let element_classes_1 = classes!(
                                            "min-h-11",
                                            "rounded-lg",
                                            "border",
                                            "px-3",
                                            "py-2",
                                            "font-medium",
                                            "focus-visible:outline-2",
                                            "focus-visible:outline-offset-2",
                                            "focus-visible:outline-blue-600",
                                            if end {"border-emerald-600"} else {"border-line"},
                                        );
                        html! {
                            <div
                                key={index}
                                class="flex items-center gap-1"
                            >
                                <button
                                    type="button"
                                    draggable={(!props.disabled).to_string()}
                                    data-answer-tile={index.to_string()}
                                    data-solution-order={canonical_order.map(|position| position.to_string()).unwrap_or_else(|| "-1".to_owned())}
                                    data-solution-first={is_first.to_string()}
                                    data-solution-last={is_last.to_string()}
                                    disabled={props.disabled}
                                    class={element_classes_1}
                                    aria-label={format!("Remove {} from answer", props.words[index])}
                                    onclick={on_remove_tile}
                                    ondragstart={on_drag_start}
                                    ondragend={on_drag_end}
                                    ondragover={allow_drop.clone()}
                                    ondrop={on_drop}
                                >
                                    {&props.words[index]}
                                </button>
                                <div
                                    class="flex flex-col gap-1"
                                >
                                    <button
                                        type="button"
                                        data-tile-move-left={index.to_string()}
                                        class="rounded border border-line px-2 text-xs disabled:opacity-40"
                                        disabled={props.disabled || move_left.is_none()}
                                        aria-label={format!("Move {} left", props.words[index])}
                                        onclick={on_add_separator}
                                    >
                                        {"←"}
                                    </button>
                                    <button
                                        type="button"
                                        data-tile-move-right={index.to_string()}
                                        class="rounded border border-line px-2 text-xs disabled:opacity-40"
                                        disabled={props.disabled || move_right.is_none()}
                                        aria-label={format!("Move {} right", props.words[index])}
                                        onclick={on_add_space}
                                    >
                                        {"→"}
                                    </button>
                                </div>
                            </div>
                        }
                    })}
                    <span
                        class="rounded-md border border-emerald-600 bg-emerald-50 px-2 py-2 text-xs font-bold text-emerald-800"
                        aria-label="End of sentence"
                    >
                        {"END"}
                    </span>
                </div>
                <p
                    class="text-xs text-muted"
                >
                    {"Select words in order. Drag tiles or use the arrow buttons to reorder; select an answer word to return it."}
                </p>
            </div>
            <AvailableTiles
                words={props.words.clone()}
                solution_positions={props.solution_positions.clone()}
                order={props.order.clone()}
                selected={props.selected.clone()}
                disabled={props.disabled}
                on_add={props.on_add.clone()}
                on_remove={props.on_remove.clone()}
                dragging={dragging}
                target_language={props.target_language.clone()}
            />
        </div>
    }
}
