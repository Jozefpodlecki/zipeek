use yew::prelude::*;
use yew_router::prelude::Link;

use crate::route::Route;
#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <crate::components::RoutePlaceholder title="Page not found" message="This page does not exist.">
            <Link<Route> to={Route::Home} classes="mt-6 rounded px-3 py-2 text-accent hover:bg-raised">{"Back to home"}</Link<Route>>
        </crate::components::RoutePlaceholder>
    }
}
