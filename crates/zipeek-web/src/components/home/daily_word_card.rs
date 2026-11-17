use yew::prelude::*;

#[function_component(DailyWordCard)]
pub fn daily_word_card() -> Html {
    html! {
        <section
            aria-labelledby="daily-word-heading"
            data-component="daily-word-card"
            class="mx-auto w-full max-w-3xl space-y-3"
        >
            <div
                class="w-full"
            >
                <h1
                    id="daily-word-heading"
                    class="text-xl font-bold tracking-tight sm:text-2xl"
                >
                    {"Word of the Day"}
                </h1>
            </div>
            <crate::components::WordSpotlight daily={true} />
        </section>
    }
}
