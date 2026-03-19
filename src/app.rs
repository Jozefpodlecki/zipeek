use yew::*;
use yew_router::{HashRouter, Switch};

use crate::{models::AppState, route::{switch, Route}};


#[function_component(App)]
pub fn app() -> Html {

    html! {
        <HashRouter>
            <Switch<Route> render={switch} />
        </HashRouter>
    }
}
