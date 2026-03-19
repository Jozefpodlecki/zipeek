use yew::*;
use yew_router::Routable;

use crate::pages::*;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/*path")]
    Home { path: String },
    #[at("/random")]
    Random,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home { path } => html! { <Home path={path} /> },
        Route::Random => html! { <Random/> },
    }
}