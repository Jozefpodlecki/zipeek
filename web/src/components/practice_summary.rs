use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use yew::prelude::*;

#[wasm_bindgen(module = "/src/services/summary.js")]
extern "C" {
    #[wasm_bindgen(catch,js_name=createSummary)]
    async fn create_summary(
        canvas: JsValue,
        count: JsValue,
        correct: u32,
        total: u32,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=destroySummary)]
    fn destroy_summary(handle: &JsValue) -> Result<(), JsValue>;
}

#[derive(Properties, PartialEq)]
pub(crate) struct PracticeSummaryProps {
    pub correct: u32,
    pub total: u32,
    pub mistakes: u64,
    pub elapsed_ms: u64,
    pub highest_streak: u64,
    pub restart: Callback<()>,
    pub back_to_setup: Callback<()>,
    pub navigation_failed: bool,
}

#[function_component(PracticeSummary)]
pub(crate) fn practice_summary(props: &PracticeSummaryProps) -> Html {
    let canvas = use_node_ref();
    let count = use_node_ref();
    let heading = use_node_ref();
    let chart_failed = use_state(|| false);
    {
        let canvas = canvas.clone();
        let count = count.clone();
        let heading = heading.clone();
        let chart_failed = chart_failed.clone();
        use_effect_with((props.correct, props.total), move |(correct, total)| {
            if let Some(heading) = heading.cast::<HtmlElement>()
                && let Err(error) = heading.focus()
            {
                log::warn!("Could not focus summary: {error:?}");
            }
            let disposed = Rc::new(Cell::new(false));
            let handle = Rc::new(RefCell::new(None));
            if let (Some(canvas), Some(count)) =
                (canvas.cast::<HtmlElement>(), count.cast::<HtmlElement>())
            {
                let disposed = disposed.clone();
                let handle = handle.clone();
                let correct = *correct;
                let total = *total;
                wasm_bindgen_futures::spawn_local(async move {
                    match create_summary(canvas.into(), count.into(), correct, total).await {
                        Ok(chart) if disposed.get() => destroy_chart(&chart),
                        Ok(chart) => *handle.borrow_mut() = Some(chart),
                        Err(error) => {
                            log::error!("Summary ring unavailable: {error:?}");
                            if !disposed.get() {
                                chart_failed.set(true);
                            }
                        }
                    }
                });
            }
            move || {
                disposed.set(true);
                if let Some(chart) = handle.borrow_mut().take() {
                    destroy_chart(&chart);
                }
            }
        });
    }
    let restart = props.restart.clone();
    let back = props.back_to_setup.clone();
    html! {
        <section data-practice-summary="" class="flex flex-col gap-8">
            <h1 ref={heading} tabindex="-1" class="text-3xl font-bold">{"Session complete"}</h1>
            <div class="flex flex-col items-center gap-8 rounded-2xl border border-line bg-surface p-6 lg:flex-row">
                <div class="relative size-48 shrink-0"><canvas ref={canvas} aria-hidden="true" data-summary-ring="" /></div>
                <dl class="grid w-full grid-cols-2 gap-6 lg:grid-cols-4">
                    <div><dt class="text-sm text-muted">{"First-attempt"}</dt><dd class="mt-2 text-2xl font-semibold" data-summary-first-attempt={format!("{} / {}",props.correct,props.total)}><span ref={count}>{props.correct}</span>{format!(" / {}",props.total)}</dd></div>
                    <div><dt class="text-sm text-muted">{"Mistakes"}</dt><dd data-summary-mistakes="" class="mt-2 text-2xl font-semibold">{props.mistakes}</dd></div>
                    <div><dt class="text-sm text-muted">{"Time"}</dt><dd data-summary-time={props.elapsed_ms.to_string()} class="mt-2 text-2xl font-semibold">{format_time(props.elapsed_ms)}</dd></div>
                    <div><dt class="text-sm text-muted">{"Highest streak"}</dt><dd data-summary-highest-streak="" class="mt-2 text-2xl font-semibold">{props.highest_streak}</dd></div>
                </dl>
            </div>
            {if *chart_failed {html!{<p data-summary-chart-error="" class="text-sm text-muted">{"Chart unavailable. Session results are shown above."}</p>}} else {Html::default()}}
            {if props.navigation_failed {html!{<p class="text-sm text-muted" role="alert">{"Couldn't start another session. Try again."}</p>}} else {Html::default()}}
            <div class="flex flex-col gap-3 sm:flex-row">
                <button type="button" data-summary-restart="" class="min-h-12 rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white hover:bg-blue-700" onclick={Callback::from(move |_|restart.emit(()))}>{"Play again"}</button>
                <button type="button" data-summary-setup="" class="min-h-12 rounded-xl border border-line bg-surface px-5 py-3 font-semibold text-ink hover:bg-raised" onclick={Callback::from(move |_|back.emit(()))}>{"Back to setup"}</button>
            </div>
        </section>
    }
}

fn destroy_chart(chart: &JsValue) {
    if let Err(error) = destroy_summary(chart) {
        log::warn!("Could not destroy summary ring: {error:?}");
    }
}

fn format_time(elapsed_ms: u64) -> String {
    let seconds = elapsed_ms / 1000;
    format!("{}m {}s", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elapsed_time_is_whole_minutes_and_seconds() {
        assert_eq!(format_time(134999), "2m 14s");
        assert_eq!(format_time(0), "0m 0s");
    }
}
