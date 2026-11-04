use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {}

#[function_component(Loader)]
pub fn loader(_props: &Props) -> Html {
    html! {
        <div data-loading="yew" class="loading-screen loading-screen-inline" role="status" aria-live="polite">
            <div class="loading-panel">
                <span class="loading-spinner" aria-hidden="true"></span>
                <p>{"Loading"}</p>
            </div>
        </div>
    }
}
