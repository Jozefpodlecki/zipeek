use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, PromiseRejectionEvent};
use yew::*;
use yew_router::{HashRouter, Switch};

use crate::{
    AppEnvironment,
    components::{AppShell, DatabaseProvider, ThemeProvider},
    route::*,
};

#[function_component(App)]
pub fn app(props: &AppEnvironment) -> Html {
    let AppEnvironment { window, .. } = props;

    {
        let window = window.clone();
        use_effect_with((), move |_| {
            let error_listener = EventListener::new(&window, "error", move |event| {
                let error_event = event.clone().unchecked_into::<ErrorEvent>();
                let message = error_event.message();
                web_sys::console::error_1(&format!("Fatal: {}", message).into());
            });

            let rejection_listener =
                EventListener::new(&window, "unhandledrejection", move |event| {
                    let rejection = event.clone().unchecked_into::<PromiseRejectionEvent>();
                    let reason = rejection.reason();
                    web_sys::console::error_2(&"Unhandled rejection:".into(), &reason);
                });

            move || {
                drop(error_listener);
                drop(rejection_listener);
            }
        });
    }

    html! {
        <ThemeProvider document={props.document.clone()} storage={props.local_storage.clone()}>
            <HashRouter>
                <AppShell>
                    <DatabaseProvider><Switch<Route> render={switch} /></DatabaseProvider>
                </AppShell>
            </HashRouter>
        </ThemeProvider>
    }
}
