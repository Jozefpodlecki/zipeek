use yew::prelude::*;

#[function_component(Background)]
pub fn background() -> Html {

    html! {
        <img
            class="fixed inset-0 w-full h-full object-cover -z-10 brightness-40"
            src={"public/background.webp"}
            alt="background" />
    }
}