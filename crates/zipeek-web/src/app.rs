use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, PromiseRejectionEvent};
use yew::*;
use yew_router::{HashRouter, Switch};

use crate::components::desktop_lifecycle::DesktopLifecycle;
use crate::{
    AppEnvironment,
    components::{
        AppShell, DatabaseProvider, PlannedSessionProvider, QuizSessionProvider, ThemeProvider,
        ToastProvider,
    },
    contexts::service_management::ServiceProvider,
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
        <ContextProvider<AppEnvironment> context={props.clone()}>
        <ThemeProvider
            manager={props.theme_manager.clone()}
        >
            <DesktopLifecycle>
            <ToastProvider>
            <ServiceProvider>
            <DatabaseProvider>
                <QuizSessionProvider>
                    <PlannedSessionProvider>
                    <HashRouter>
                    <AppShell>
                        <Switch<Route> render={switch} />
                    </AppShell>
                    </HashRouter>
                    </PlannedSessionProvider>
                    </QuizSessionProvider>
            </DatabaseProvider>
            </ServiceProvider>
            </ToastProvider>
            </DesktopLifecycle>
        </ThemeProvider>
        </ContextProvider<AppEnvironment>>
    }
}
