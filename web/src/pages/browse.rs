use yew::prelude::*;

#[function_component(Browse)]
pub fn browse() -> Html {
    html! {
        <section class="flex min-h-screen flex-col items-center justify-center gap-6 p-8 text-center">
            <h1 class="text-3xl font-bold">{"Browse"}</h1>
            <p class="text-lg">{"Search coming soon."}</p>
        </section>
    }
}
