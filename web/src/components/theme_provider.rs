use web_sys::{Document, Storage};
use yew::prelude::*;

use crate::theme::{Theme, ThemeContext};

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub document: Document,
    pub storage: Option<Storage>,
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    let storage = props.storage.clone();
    let theme = use_state(move || Theme::load(storage.as_ref()));
    {
        let document = props.document.clone();
        use_effect_with(*theme, move |theme| {
            theme.apply(&document);
        });
    }
    html! {
        <ContextProvider<ThemeContext> context={ThemeContext { state: theme, storage: props.storage.clone() }}>
            {props.children.clone()}
        </ContextProvider<ThemeContext>>
    }
}
