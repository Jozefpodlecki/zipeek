use yew::prelude::*;

use crate::appearance::Appearance;

#[function_component(SessionBackground)]
pub(super) fn session_background() -> Html {
    let appearance = use_context::<Appearance>()
        .expect("Appearance provider missing from practice session background");
    let source = format!("public/backgrounds/{}", appearance.preset.background());

    html! {
        <div data-practice-theme-background="" aria-hidden="true" class="pointer-events-none absolute inset-0 z-0 overflow-hidden">
            <img src={source} alt="" class="h-full w-full object-cover opacity-65 transition-opacity duration-300 motion-reduce:transition-none" />
            <div class="absolute inset-0 bg-canvas/75" />
        </div>
    }
}
