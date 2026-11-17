use std::{cell::Cell, rc::Rc};

use crate::hanzi_references::{self, GlyphReference};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HanziStrokeReferenceProps {
    pub character: String,
}

#[function_component(HanziStrokeReference)]
pub fn hanzi_stroke_reference(props: &HanziStrokeReferenceProps) -> Html {
    let reference = use_state(|| None::<Result<GlyphReference, String>>);
    let visible = use_state(|| 1usize);
    let character = props.character.clone();

    {
        let reference = reference.clone();
        let visible = visible.clone();
        use_effect_with(character.clone(), move |character| {
            let alive = Rc::new(Cell::new(true));
            let active = alive.clone();
            visible.set(1);
            reference.set(None);
            let character = character.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result: Result<GlyphReference, String> = async {
                    let mut characters = character.chars();
                    let hanzi = characters
                        .next()
                        .ok_or_else(|| "No character selected".to_owned())?;
                    if characters.next().is_some() {
                        return Err("Select a single character".to_owned());
                    }
                    let parsed = hanzi_references::load(hanzi).await?;
                    Ok(parsed)
                }
                .await;
                if active.get() {
                    reference.set(Some(result));
                }
            });
            move || alive.set(false)
        });
    }

    let previous = {
        let visible = visible.clone();
        Callback::from(move |_| visible.set((*visible).saturating_sub(1).max(1)))
    };
    let next = {
        let visible = visible.clone();
        Callback::from(move |_| visible.set(*visible + 1))
    };

    let stroke_reference_content = {
        match &*reference {
            None => html! {
            <p
                role="status"
                class="text-xs text-muted"
            >
                {"Checking local stroke references…"}
            </p>},
            Some(Err(error)) => html! {
            <p
                class="text-xs text-muted"
                role="status"
            >
                {error}
            </p>},
            Some(Ok(reference)) => {
                let max = reference.strokes.len();
                let shown = (*visible).min(max);
                html! {
                    <>
                    <svg
                        viewBox="0 0 1024 1024"
                        role="img"
                        aria-label={format!("First {shown} of {max} reference strokes for {}", props.character)}
                        class="mx-auto aspect-square w-full max-w-56 rounded-lg border border-line bg-surface"
                    >
                        <g transform="scale(1 -1) translate(0 -900)">
                            {for reference.strokes.iter().enumerate().map(|(index, stroke)| html! {
                                <path
                                    key={index}
                                    d={stroke.clone()}
                                    fill={if index < shown {"#79b7ff"} else {"#37465e"}}
                                    opacity={if index < shown {"1"} else {"0.2"}}
                                />
                            })}
                        </g>
                    </svg>
                    <div
                        class="flex items-center justify-between gap-3"
                    >
                        <button
                            type="button"
                            onclick={previous}
                            disabled={shown<=1}
                            class="rounded-lg border border-line px-3 py-2 text-xs font-semibold disabled:opacity-40"
                        >
                            {"Previous"}
                        </button>
                        <span
                            aria-live="polite"
                            class="text-xs text-muted"
                        >
                            {format!("Stroke {shown} / {max}")}
                        </span>
                        <button
                            type="button"
                            onclick={next}
                            disabled={shown>=max}
                            class="rounded-lg border border-line px-3 py-2 text-xs font-semibold disabled:opacity-40"
                        >
                            {"Next"}
                        </button>
                    </div>
                    <p
                        class="text-xs text-muted"
                    >
                        {"Source: Hanzi Writer Data / Make Me a Hanzi · Arphic Public License"}
                    </p>
                </>}
            }
        }
    };
    html! {
        <section
            data-component="hanzi-stroke-reference"
            class="rounded-xl border border-line bg-raised p-4 space-y-3"
            aria-label="Reference stroke order"
        >
            <div
                class="space-y-1"
            >
                <h3
                    class="text-sm font-semibold"
                >
                    {"Reference stroke order"}
                </h3>
                <p
                    class="text-xs text-muted"
                >
                    {"Explore canonical stroke shapes and their drawing order."}
                </p>
            </div>
            {stroke_reference_content}
        </section>
    }
}
