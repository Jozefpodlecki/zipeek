use yew::prelude::*;

#[function_component(DailyWordCard)]
pub fn daily_word_card() -> Html {
    html! {
        <section class="relative overflow-hidden rounded-[1.1rem] border border-line bg-[linear-gradient(135deg,var(--surface),var(--card-tint))] p-5 backdrop-blur-[12px] md:px-7 md:py-6" aria-labelledby="daily-word-title" data-daily-word="unavailable">
            <div class="flex items-center justify-between gap-4">
                <h2 class="text-[.85rem] font-semibold text-ink" id="daily-word-title">{"Word of the Day"}</h2>
            </div>
            <div class="flex min-h-32 flex-col items-center justify-center gap-[1.2rem] pt-8 pb-4 text-[.9rem] text-muted">
                <span class="flex h-9 items-center gap-[.4rem]" aria-hidden="true"><span class="block h-5 w-[.3rem] rounded-2xl bg-accent opacity-35" /><span class="block h-8 w-[.3rem] rounded-2xl bg-accent opacity-70" /><span class="block h-5 w-[.3rem] rounded-2xl bg-accent opacity-35" /></span>
                <p>{"No word available"}</p>
            </div>
        </section>
    }
}
