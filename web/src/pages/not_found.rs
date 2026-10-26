use yew::prelude::*;
use yew_router::hooks::use_navigator;

use crate::route::Route;
#[function_component(NotFound)]
pub fn not_found() -> Html {
    let navigator = use_navigator().expect("router context");

    let go_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Home))
    };

    html! {
        <section class="flex min-h-screen flex-col items-center justify-center gap-6 p-8 text-center">
            <h1 class="text-6xl font-bold">{"404"}</h1>
            <p class="text-xl">{"Page not found."}</p>
            <button
                type="button"
                onclick={go_home}
                class="rounded border border-current px-4 py-2 hover:opacity-80"
            >
                {"Back to home"}
            </button>
        </section>
    }
}
