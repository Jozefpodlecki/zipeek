use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct HeaderDragonProps {
    pub recording_enabled: bool,
}

#[function_component(HeaderDragon)]
pub(super) fn header_dragon(props: &HeaderDragonProps) -> Html {
    let recording_status = if props.recording_enabled {
        "Practice history saving is on."
    } else {
        "Practice history saving is off or unavailable."
    };
    let hosted = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .is_some_and(|path| path.starts_with("/zipeek/"));
    let source = if hosted {
        "/zipeek/practice/header-dragon-unassessed.webp"
    } else {
        "/practice/header-dragon-unassessed.webp"
    };
    html! {
        <details data-learning-dragon="" data-evidence-state="unavailable"
            class="group relative min-w-0 flex-1 self-center" >
            <summary aria-label="Dragon learning progress unavailable. Show details"
                class="flex h-12 min-w-0 cursor-pointer list-none items-center justify-center rounded-lg outline-none focus-visible:outline-2 focus-visible:outline-accent [&::-webkit-details-marker]:hidden">
                <img src={source} alt="" aria-hidden="true" draggable="false"
                    class="h-11 max-w-full object-contain opacity-55 saturate-50 motion-reduce:transition-none" />
            </summary>
            <div class="absolute left-1/2 top-full z-50 mt-2 w-[min(22rem,85vw)] max-w-[calc(100vw-2rem)] -translate-x-1/2 rounded-xl border border-line bg-surface p-4 text-sm shadow-xl"
                role="status">
                <strong class="block font-semibold text-ink">{"Dragon progress is not yet available"}</strong>
                <p class="mt-2 text-muted">{"The dragon will reflect verified, persisted pedagogical evidence when the assessment pipeline is connected. It does not represent completed sessions or recording preference."}</p>
                <p class="mt-2 text-muted">{recording_status}</p>
            </div>
        </details>
    }
}
