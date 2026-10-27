use yew::prelude::*;

#[function_component(DailyWordCard)]
pub fn daily_word_card() -> Html {
    html! {
        <section class="daily-word-card" aria-labelledby="daily-word-title" data-daily-word="unavailable">
            <div class="daily-word-heading">
                <h2 id="daily-word-title">{"Word of the Day"}</h2>
            </div>
            <div class="daily-word-empty">
                <span class="daily-word-decoration" aria-hidden="true"><span /><span /><span /></span>
                <p>{"No word available"}</p>
            </div>
        </section>
    }
}
