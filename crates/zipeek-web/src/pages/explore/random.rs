use yew::prelude::*;

#[function_component(Random)]
pub fn random() -> Html {
    html! {
        <main
            data-component="random-page"
            class="mx-auto flex w-full max-w-5xl flex-col items-center gap-8 py-5 sm:py-10"
        >
            <header
                class="flex flex-col items-center gap-3 text-center"
            >
                <h1
                    class="text-3xl font-bold tracking-tight sm:text-4xl"
                >
                    {"Word discovery"}
                </h1>
                <p
                    class="max-w-xl text-sm text-muted sm:text-base"
                >
                    {"Discover a random Chinese word and expand your vocabulary."}
                </p>
            </header>
            <crate::components::WordSpotlight />
        </main>
    }
}
