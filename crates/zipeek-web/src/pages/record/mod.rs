mod audio;
mod readings;
mod controller;
mod state;
mod components;

use yew::prelude::*;
use controller::use_record_controller;
use components::RecordView;

#[function_component(RecordPage)]
pub fn record_page() -> Html {
    let controller = use_record_controller();
    html! { <RecordView controller={controller} /> }
}
