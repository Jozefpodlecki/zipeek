use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::hooks::use_app_environment;

#[derive(Properties, PartialEq)]
pub(super) struct HeaderDragonProps {
    pub recording_enabled: bool,
}

#[function_component(HeaderDragon)]
pub(super) fn header_dragon(props: &HeaderDragonProps) -> Html {
    let open = use_state(|| false);
    let root = use_node_ref();
    let environment = use_app_environment();
    {
        let document = environment.document.clone();
        let root = root.clone();
        let open = open.clone();
        use_effect_with(*open, move |visible| {
            let outside = if *visible {
                let open = open.clone();
                let root = root.clone();
                Some(EventListener::new(&document, "pointerdown", move |event| {
                    let inside = root
                        .cast::<web_sys::Element>()
                        .zip(
                            event
                                .target()
                                .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
                        )
                        .is_some_and(|(element, target)| element.contains(Some(&target)));
                    if !inside {
                        open.set(false);
                    }
                }))
            } else {
                None
            };
            let escape = if *visible {
                let open = open.clone();
                Some(EventListener::new(&document, "keydown", move |event| {
                    if event
                        .dyn_ref::<web_sys::KeyboardEvent>()
                        .is_some_and(|key| key.key() == "Escape")
                    {
                        open.set(false);
                    }
                }))
            } else {
                None
            };
            move || {
                drop(outside);
                drop(escape);
            }
        });
    }
    let hosted = environment
        .window
        .location()
        .pathname()
        .ok()
        .is_some_and(|path| path == "/zipeek" || path.starts_with("/zipeek/"));
    let source = if hosted {
        "/zipeek/practice/header-dragon-unassessed.webp"
    } else {
        "/practice/header-dragon-unassessed.webp"
    };
    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };
    html! {
        <div ref={root} data-learning-dragon="" data-evidence-state="unavailable"
            class="relative flex min-w-0 flex-1 self-center justify-center">
            <button type="button" onclick={toggle} aria-expanded={open.to_string()}
                aria-label="Dragon learning progress status" aria-controls="dragon-progress-status"
                class="flex h-10 min-w-0 max-w-full items-center justify-center rounded-lg outline-none focus-visible:outline-2 focus-visible:outline-accent">
                <img src={source} alt="" aria-hidden="true" draggable="false"
                    class="h-8 w-56 max-w-full object-contain object-center opacity-55 saturate-50" />
            </button>
            <div id="dragon-progress-status" role="status" aria-hidden={(!*open).to_string()}
                class={classes!(
                    "absolute", "left-1/2", "top-full", "z-50", "mt-1", "w-[min(17rem,80vw)]",
                    "-translate-x-1/2", "rounded-lg", "border", "border-line", "bg-surface",
                    "px-3", "py-2", "text-xs", "leading-relaxed", "shadow-lg",
                    "transition-opacity", "duration-150", "motion-reduce:transition-none",
                    if *open { "opacity-100" } else { "pointer-events-none opacity-0" },
                )}>
                <strong class="block font-semibold text-ink">{"Dragon progress unavailable"}</strong>
                <p class="mt-1 text-muted">{"Verified pedagogical evidence will power this indicator once connected."}</p>
                <p class="mt-1 text-muted">{if props.recording_enabled {
                    "Practice history saving: on"
                } else {
                    "Practice history saving: off or unavailable"
                }}</p>
            </div>
        </div>
    }
}
