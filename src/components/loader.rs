use log::*;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {}

#[function_component(Loader)]
pub fn loader(props: &Props) -> Html {

    html! {
        <div data-loading="" class="loader-wrapper">
			<div class="loader">
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
				<div class="square"></div>
			</div>
		</div>
    }
}