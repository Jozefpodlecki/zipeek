mod audio;
mod components;
mod controller;
mod readings;
mod state;

use components::RecordView;
use controller::use_record_controller;
use yew::prelude::*;

#[function_component(RecordPage)]
pub fn record_page() -> Html {
    let controller = use_record_controller();
    html! { <RecordView controller={controller} /> }
}
