use web_sys::DragEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct AvailableTilesProps {
    pub words: Vec<String>,
    pub solution_positions: Vec<Option<usize>>,
    pub order: Vec<usize>,
    pub selected: Vec<usize>,
    pub disabled: bool,
    pub on_add: Callback<usize>,
    pub on_remove: Callback<usize>,
    pub dragging: UseStateHandle<Option<usize>>,
    pub target_language: String,
}

#[function_component(AvailableTiles)]
pub(super) fn available_tiles(props: &AvailableTilesProps) -> Html {
    let dragging = props.dragging.clone();
    let on_drop_bank = {
        let dragging = dragging.clone();
        let remove = props.on_remove.clone();
        let disabled = props.disabled;
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            if !disabled {
                if let Some(index) = *dragging {
                    remove.emit(index);
                }
            }
            dragging.set(None);
        })
    };
    let allow_drop = Callback::from(|event: DragEvent| event.prevent_default());
    html! {
            <div
                class="flex flex-col gap-2"
            >
                <p
                    class="text-sm font-semibold"
                >
                    {format!("Available {} tiles", if props.target_language == "zh" { "Chinese" } else { "English" })}
                </p>
                <div
                    data-translation-word-bank=""
                    class="flex min-h-20 flex-wrap gap-2 rounded-xl border border-line bg-surface p-3"
                    ondragover={allow_drop.clone()}
                    ondrop={on_drop_bank}
                >
                    {for props.order.iter().filter(|index| !props.selected.contains(index)).map(|index| {
                        let index = *index;
                        let canonical_order = props.solution_positions.get(index).copied().flatten();
                        let is_first = canonical_order == Some(0);
                        let is_last = canonical_order.map(|position| position + 1 == props.words.len()).unwrap_or(false);
                        let add = props.on_add.clone();
                        let dragging_start = dragging.clone();
                        let dragging_end = dragging.clone();
                        let on_add_tile = Callback::from(move |_| add.emit(index));
                        let on_start_drag = Callback::from(move |_: DragEvent| dragging_start.set(Some(index)));
                        let on_end_drag = Callback::from(move |_: DragEvent| dragging_end.set(None));
                        html! {
                            <button
                                key={index}
                                type="button"
                                draggable={(!props.disabled).to_string()}
                                data-bank-tile={index.to_string()}
                                data-solution-order={canonical_order.map(|position| position.to_string()).unwrap_or_else(|| "-1".to_owned())}
                                data-solution-first={is_first.to_string()}
                                data-solution-last={is_last.to_string()}
                                disabled={props.disabled}
                                class="min-h-11 rounded-lg border border-line bg-surface px-3 py-2 font-medium shadow-sm hover:border-blue-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600"
                                onclick={on_add_tile}
                                ondragstart={on_start_drag}
                                ondragend={on_end_drag}
                            >
                                {&props.words[index]}
                            </button>
                        }
                    })}
                </div>
            </div>

    }
}
