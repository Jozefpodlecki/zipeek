use yew::prelude::*;

#[function_component(DailyWordCard)]
pub fn daily_word_card() -> Html {
    html! { <crate::components::WordSpotlight daily={true} /> }
}
