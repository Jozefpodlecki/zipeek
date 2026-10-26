use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    // pub path: String
}

#[function_component(Home)]
pub fn home(_props: &Props) -> Html {
    html! {
        <div data-home="" class="">
            <h1>{"Zipeek"}</h1>
        </div>
    }
}
