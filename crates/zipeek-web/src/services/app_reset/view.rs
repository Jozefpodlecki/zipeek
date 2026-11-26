use std::{cell::Cell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use web_sys::Window;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub window: Window,
    pub native: bool,
    pub confirmed: bool,
}

#[function_component(ResetRunner)]
pub fn reset_runner(props: &Props) -> Html {
    let error = use_state(|| None::<String>);
    let running = use_state(|| false);
    let request = use_state(|| 0u32);
    let pending = use_mut_ref(|| false);
    {
        let window = props.window.clone();
        let native = props.native;
        let confirmed = props.confirmed;
        let error = error.clone();
        let running = running.clone();
        let pending = pending.clone();
        use_effect_with(*request, move |_| {
            let active = Rc::new(Cell::new(true));
            if confirmed && !*pending.borrow() {
                *pending.borrow_mut() = true;
                running.set(true);
                error.set(None);
                let active_request = active.clone();
                spawn_local(async move {
                    let result = super::reset(&window, native).await;
                    if active_request.get() {
                        let result = result.and_then(|()| super::reload_at(&window, "#/"));
                        if let Err(message) = result {
                            error.set(Some(message));
                            running.set(false);
                            *pending.borrow_mut() = false;
                        }
                    }
                });
            }
            move || active.set(false)
        });
    }
    let retry = {
        let request = request.clone();
        let pending = pending.clone();
        Callback::from(move |_| {
            if !*pending.borrow() {
                request.set(request.wrapping_add(1));
            }
        })
    };
    let back = {
        let window = props.window.clone();
        let error = error.clone();
        Callback::from(move |_| {
            if let Err(message) = super::reload_at(&window, "#/settings?section=reset") {
                error.set(Some(message));
            }
        })
    };
    html! {
        <main class="mx-auto flex min-h-screen max-w-xl items-center px-5 py-12" data-component="reset-runner">
            <section class="w-full space-y-4 rounded-2xl border border-line bg-surface p-6 text-ink" aria-labelledby="reset-heading" aria-busy={running.to_string()}>
                <h1 id="reset-heading" class="text-2xl font-semibold">{"Reset app"}</h1>
                if *running {
                    <p role="status">{"Clearing your data. Keep this window open."}</p>
                } else if !props.confirmed {
                    <p>{"Reset has not been confirmed. Return to Settings to start over."}</p>
                }
                if let Some(message) = &*error {
                    <div role="alert" data-reset-error="" class="space-y-2">
                        <p>{message}</p>
                        <p class="text-sm text-muted">{"Reset did not finish. Some data may already have been cleared. Retry to complete it."}</p>
                    </div>
                }
                if !*running {
                    <div class="flex flex-wrap gap-3">
                        if props.confirmed {
                            <button type="button" onclick={retry} data-reset-retry="" class="rounded-lg bg-accent px-4 py-2 text-on-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">{"Retry reset"}</button>
                        }
                        <button type="button" onclick={back} class="rounded-lg border border-line px-4 py-2 focus-visible:outline-2 focus-visible:outline-accent">{"Return to Settings"}</button>
                    </div>
                }
            </section>
        </main>
    }
}
