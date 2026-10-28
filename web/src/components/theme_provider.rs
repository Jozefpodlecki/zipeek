use web_sys::{Document, Storage};
use yew::prelude::*;

use crate::{
    reducers::theme::{ThemeAction, ThemeState},
    theme::{Theme, ThemeCommands},
};

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub document: Document,
    pub storage: Option<Storage>,
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    let storage = props.storage.clone();
    let theme = use_reducer_eq(move || ThemeState::new(Theme::load(storage.as_ref())));
    let toggle = use_callback(theme.dispatcher(), |(), dispatch| {
        dispatch.dispatch(ThemeAction::Toggle)
    });
    {
        let document = props.document.clone();
        let storage = props.storage.clone();
        use_effect_with(
            (theme.current, theme.modified, document, storage),
            move |(current, modified, document, storage)| {
                current.apply(document);
                if *modified {
                    current.save(storage.as_ref());
                }
            },
        );
    }
    html! {
        <ContextProvider<ThemeCommands> context={ThemeCommands::new(toggle)}>
            <ContextProvider<Theme> context={theme.current}>
                {props.children.clone()}
            </ContextProvider<Theme>>
        </ContextProvider<ThemeCommands>>
    }
}
