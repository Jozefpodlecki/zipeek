use yew::prelude::*;
use zipeek_exercises::province_quest::MapRegion;

#[derive(Properties, PartialEq)]
pub struct ProvinceReviewProps {
    pub correct: bool,
    pub region: MapRegion,
    pub on_continue: Callback<MouseEvent>,
}

#[function_component(ProvinceReview)]
pub fn province_review(props: &ProvinceReviewProps) -> Html {
    let status = if props.correct {
        "Correct"
    } else {
        "Not quite"
    };
    html! {
        <section aria-live="polite" class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-raised p-4" data-province-review="">
            <div class="space-y-1">
                <p class="font-semibold">{status}{" · "}{&props.region.hanzi}</p>
                <p class="text-sm text-muted">{&props.region.pinyin}{" · "}{&props.region.english}</p>
            </div>
            <button class="rounded-xl bg-brand-action px-5 py-3 font-semibold text-white" type="button" onclick={props.on_continue.clone()}> {"Continue"} </button>
        </section>
    }
}
