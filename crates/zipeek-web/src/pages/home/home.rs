use yew::prelude::*;

use crate::components::DailyWordCard;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {}

#[function_component(Home)]
pub fn home(_props: &Props) -> Html {
    html! {
        <main
            data-component="home-page"
            data-home=""
            class="mx-auto flex w-full max-w-5xl flex-col gap-3 py-5 sm:py-8"
        >
            <DailyWordCard />
        </main>
    }
}
