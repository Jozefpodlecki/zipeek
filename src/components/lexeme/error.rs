use yew::prelude::*;

use crate::models::AppError;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub error: AppError,
}

#[function_component(Error)]
pub fn error(props: &Props) -> Html {

    html! {
        <p>{"Error"}</p>
    }
}