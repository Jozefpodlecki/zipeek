use crate::appearance::Appearance;
use yew::prelude::*;

#[function_component(Background)]
pub fn background() -> Html {
    let appearance =
        use_context::<Appearance>().expect("Appearance provider missing from background");
    let source = format!("public/backgrounds/{}", appearance.preset.background());
    html! {
        <img
            class="fixed inset-0 w-full h-full object-cover -z-10 brightness-40"
            src={source}
            alt=""
            aria-hidden="true"
        />
    }
}
