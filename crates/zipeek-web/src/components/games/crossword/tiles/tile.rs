use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct CrosswordTileProps {
    pub index: usize,
    pub character: char,
    pub pronunciation: String,
    pub selected: bool,
    pub rejected: bool,
    pub on_select: Callback<usize>,
}

#[function_component(CrosswordTile)]
pub(super) fn crossword_tile(props: &CrosswordTileProps) -> Html {
    let on_select = {
        let on_select = props.on_select.clone();
        let index = props.index;
        Callback::from(move |_: MouseEvent| on_select.emit(index))
    };
    let on_start_drag = {
        let index = props.index;
        Callback::from(move |event: DragEvent| {
            if let Some(data) = event.data_transfer() {
                let _ = data.set_data("text/plain", &index.to_string());
            }
        })
    };
    let pronunciation = if props.pronunciation.is_empty() {
        "pronunciation unavailable"
    } else {
        &props.pronunciation
    };
    let element_classes_1 = classes!(
        "bg-surface",
        "text-ink",
        "border-line",
        "cursor-grab",
        "hover:bg-[color-mix(in_srgb,var(--accent)_27%,var(--surface))]",
        "hover:border-accent",
        "flex",
        "min-h-18",
        "min-w-18",
        "shrink-0",
        "flex-col",
        "items-center",
        "justify-center",
        "rounded-xl",
        "border",
        "focus-visible:outline-2",
        if props.selected {
            classes!(
                "!bg-[color-mix(in_srgb,var(--accent)_27%,var(--surface))]",
                "!border-accent"
            )
        } else {
            Classes::new()
        },
        if props.rejected {
            classes!(
                "motion-safe:animate-[crossword-reject_360ms_ease-in-out]",
                "motion-reduce:!border-[#ec6877]"
            )
        } else {
            Classes::new()
        },
    );
    html! {
        <button
            type="button"
            draggable="true"
            aria-pressed={props.selected.to_string()}
            data-crossword-tile=""
            aria-label={format!("{}, {pronunciation}", props.character)}
            class={element_classes_1}
            onclick={on_select}
            ondragstart={on_start_drag}
        >
            <span class="text-4xl font-bold">{props.character}</span>
            <span class="text-xs text-muted">{&props.pronunciation}</span>
        </button>
    }
}
