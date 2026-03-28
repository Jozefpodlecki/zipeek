use yew::*;
use yew_router::Routable;

use crate::pages::*;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/random")]
    Random,
    #[at("/lexeme/*query")]
    Lexeme { query: String },
}

pub fn switch(routes: Route) -> Html {
    // log::info!("test");
    match routes {
        Route::Home => html! { <Home /> },
        Route::Random => html! { <Random/> },
        Route::Lexeme { query } => html! { <LexemePage query={query} /> },
    }
}