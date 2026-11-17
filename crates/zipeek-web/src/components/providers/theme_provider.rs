use yew::prelude::*;

use crate::{
    appearance::AppearanceProvider,
    reducers::theme::{ThemeAction, ThemeState},
    theme::{Theme, ThemeCommands, ThemeManager},
};

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub manager: ThemeManager,
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    let manager = props.manager.clone();
    let theme = use_reducer_eq(move || ThemeState::new(manager.load()));
    let toggle = use_callback(theme.dispatcher(), |(), dispatch| {
        dispatch.dispatch(ThemeAction::Toggle)
    });
    {
        let manager = props.manager.clone();
        use_effect_with(
            (theme.current, theme.modified, manager),
            move |(current, modified, manager)| {
                if let Err(error) = manager.apply(*current) {
                    log::warn!("Could not apply document theme: {error}");
                }
                if *modified {
                    if let Err(error) = manager.save(*current) {
                        log::warn!("Could not save theme preference: {error}");
                    }
                }
            },
        );
    }
    html! {
        <AppearanceProvider
            document={props.manager.browser_document().document().clone()}
            browser_document={props.manager.browser_document().clone()}
            storage={props.manager.storage().clone()}
        >
        <ContextProvider<ThemeCommands> context={ThemeCommands::new(toggle)}>
            <ContextProvider<Theme> context={theme.current}>
                {props.children.clone()}
            </ContextProvider<Theme>>
        </ContextProvider<ThemeCommands>>
        </AppearanceProvider>
    }
}
