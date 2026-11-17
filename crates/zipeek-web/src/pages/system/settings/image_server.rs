use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{hooks::use_app_environment, services::image_generation};

#[derive(Clone, PartialEq)]
enum ConnectionFeedback {
    Saved,
    Reset,
    Checking,
    Connected,
    InvalidUrl(String),
    HttpError(u16),
    Unreachable(String),
}

#[function_component(ImageServerSettings)]
pub fn image_server_settings() -> Html {
    let environment = use_app_environment();
    let initial_storage = environment.local_storage.clone();
    let url = use_state(move || image_generation::read_api_url(&initial_storage));
    let feedback = use_state(|| None::<ConnectionFeedback>);
    let busy = use_state(|| false);
    let epoch = use_mut_ref(|| 0u64);

    let on_change = {
        let url = url.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let epoch = epoch.clone();
        Callback::from(move |event: InputEvent| {
            busy.set(false);
            *epoch.borrow_mut() += 1;
            url.set(event.target_unchecked_into::<HtmlInputElement>().value());
            feedback.set(None);
        })
    };
    let on_save = {
        let url = url.clone();
        let feedback = feedback.clone();
        let storage = environment.local_storage.clone();
        Callback::from(
            move |_: MouseEvent| match image_generation::save_api_url(&storage, &url) {
                Ok(()) => {
                    crate::contexts::service_management::notify_configuration_changed();
                    feedback.set(Some(ConnectionFeedback::Saved));
                }
                Err(error) => feedback.set(Some(ConnectionFeedback::InvalidUrl(error))),
            },
        )
    };
    let on_reset = {
        let url = url.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let epoch = epoch.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |_: MouseEvent| {
            *epoch.borrow_mut() += 1;
            if let Err(error) = image_generation::reset_api_url(&storage) {
                feedback.set(Some(ConnectionFeedback::InvalidUrl(error)));
                return;
            }
            busy.set(false);
            crate::contexts::service_management::notify_configuration_changed();
            url.set(image_generation::DEFAULT_URL.to_owned());
            feedback.set(Some(ConnectionFeedback::Reset));
        })
    };
    let on_test = {
        let url = url.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if *busy {
                return;
            }
            let candidate = url.trim().trim_end_matches('/').to_owned();
            if let Err(error) = image_generation::validate_api_url(&candidate) {
                feedback.set(Some(ConnectionFeedback::InvalidUrl(error.to_owned())));
                return;
            }
            let ticket = {
                let mut value = epoch.borrow_mut();
                *value += 1;
                *value
            };
            busy.set(true);
            feedback.set(Some(ConnectionFeedback::Checking));
            let feedback = feedback.clone();
            let busy = busy.clone();
            let epoch = epoch.clone();
            spawn_local(async move {
                let result = gloo::net::http::Request::get(&format!("{candidate}/health"))
                    .send()
                    .await;
                if *epoch.borrow() != ticket {
                    return;
                }
                busy.set(false);
                match result {
                    Ok(response) if response.ok() => {
                        feedback.set(Some(ConnectionFeedback::Connected));
                    }
                    Ok(response) => {
                        feedback.set(Some(ConnectionFeedback::HttpError(response.status())));
                    }
                    Err(error) => {
                        feedback.set(Some(ConnectionFeedback::Unreachable(error.to_string())));
                    }
                }
            });
        })
    };

    let health_url = format!("{}/health", url.trim().trim_end_matches('/'));
    let feedback_panel = match &*feedback {
        Some(ConnectionFeedback::Saved) => html! {
            <div data-image-server-feedback="saved" role="status" class="mt-4 rounded-xl border border-line bg-raised px-4 py-3 text-sm text-ink">
                <p class="font-semibold">{"URL saved"}</p>
                <p class="mt-1 text-muted">{"Use Test connection to check that the image server is reachable."}</p>
            </div>
        },
        Some(ConnectionFeedback::Reset) => html! {
            <div data-image-server-feedback="reset" role="status" class="mt-4 rounded-xl border border-line bg-raised px-4 py-3 text-sm text-ink">
                <p class="font-semibold">{"Default address restored"}</p>
                <p class="mt-1 text-muted">{"The image server has not yet been checked."}</p>
            </div>
        },
        Some(ConnectionFeedback::Checking) => html! {
            <div data-image-server-feedback="checking" role="status" class="mt-4 rounded-xl border border-line bg-raised px-4 py-3 text-sm text-ink">
                <p class="font-semibold">{"Checking image server…"}</p>
                <p class="mt-1 text-muted">{"Requesting the local health endpoint."}</p>
            </div>
        },
        Some(ConnectionFeedback::Connected) => html! {
            <div data-image-server-feedback="connected" role="status" class="mt-4 rounded-xl border border-line bg-raised px-4 py-3 text-sm text-ink">
                <p class="font-semibold">{"Image server connected"}</p>
                <p class="mt-1 text-muted">{"The health endpoint responded successfully. This does not confirm that the image-generation model is loaded."}</p>
            </div>
        },
        Some(ConnectionFeedback::InvalidUrl(reason)) => html! {
            <div data-image-server-feedback="invalid" role="alert" class="mt-4 rounded-xl border border-amber-500/40 bg-amber-950/10 px-4 py-3 text-sm text-ink">
                <p class="font-semibold">{"Check the server address"}</p>
                <p class="mt-1 text-muted">{reason}</p>
            </div>
        },
        Some(ConnectionFeedback::HttpError(code)) => html! {
            <div data-image-server-feedback="http-error" role="alert" class="mt-4 rounded-xl border border-amber-500/40 bg-amber-950/10 px-4 py-3 text-sm text-ink">
                <p class="font-semibold">{"The image server responded with an error"}</p>
                <p class="mt-1 text-muted">{format!("Health check returned HTTP {code}. Verify that the image API is running at this address.")}</p>
            </div>
        },
        Some(ConnectionFeedback::Unreachable(details)) => html! {
            <div data-image-server-feedback="unreachable" role="alert" class="mt-4 space-y-3 rounded-xl border border-amber-500/40 bg-amber-950/10 p-4 text-sm text-ink">
                <div>
                    <p class="font-semibold">{"Browser connection blocked"}</p>
                    <p class="mt-1 leading-6 text-muted">{"The browser could not read the server response. The server may be offline, or its CORS policy may not allow this browser address."}</p>
                </div>
                <div class="rounded-lg border border-line bg-raised px-3 py-2">
                    <p class="font-medium">{"Check the connection"}</p>
                    <ol class="mt-1 list-inside list-decimal space-y-1 text-muted">
                        <li>{"Start the local image server and keep it running."}</li>
                        <li>{"Open its health endpoint below. If it responds there but this check fails, inspect the browser console for a CORS error."}</li>
                        <li>{"If the URL contains localhost. (with a trailing dot), restart with the updated image server that recognizes this origin."}</li>
                    </ol>
                </div>
                <a href={health_url} target="_blank" rel="noopener noreferrer" class="inline-flex rounded-lg border border-line bg-raised px-3 py-2 font-semibold text-ink no-underline hover:border-accent focus-visible:outline-2 focus-visible:outline-accent">
                    {"Open health endpoint"}
                </a>
                <details class="text-xs text-muted">
                    <summary class="cursor-pointer font-medium">{"Technical details"}</summary>
                    <p class="mt-1 break-all">{details}</p>
                </details>
            </div>
        },
        None => Html::default(),
    };

    html! {
        <section
            data-settings-image-server=""
            class="min-w-0 w-full rounded-2xl border border-line bg-surface p-4 shadow-sm sm:p-6 lg:p-8"
        >
            <h2 class="text-xl font-semibold">{"Image generation"}</h2>
            <p class="mt-2 text-sm text-muted">
                {"Connect the separate local Axum image server. The URL is stored in this browser, independently of LM Studio."}
            </p>
            <label class="mt-5 flex min-w-0 flex-col gap-2 text-sm font-semibold">
                {"Image API URL"}
                <input
                    type="url"
                    data-image-server-url=""
                    value={(*url).clone()}
                    oninput={on_change}
                    placeholder={image_generation::DEFAULT_URL}
                    class="min-h-11 min-w-0 w-full rounded-lg border border-line bg-raised px-3 font-normal text-ink focus-visible:outline-2 focus-visible:outline-accent"
                />
            </label>
            <div class="mt-4 grid grid-cols-2 gap-2 sm:flex sm:flex-wrap">
                <button type="button" data-action="save" onclick={on_save} class="min-h-11 rounded-lg bg-brand-action px-4 font-semibold text-white sm:px-5">
                    {"Save"}
                </button>
                <button type="button" data-action="test" onclick={on_test} disabled={*busy} class="min-h-11 min-w-0 rounded-lg border border-line px-3 disabled:opacity-50 sm:px-4">
                    {if *busy {"Checking…"} else {"Test connection"}}
                </button>
                <button type="button" data-action="reset" onclick={on_reset} class="min-h-11 min-w-0 rounded-lg border border-line px-3 sm:px-4">
                    {"Reset"}
                </button>
            </div>
            {feedback_panel}
            <p class="mt-4 text-xs text-muted">
                {"The image server must permit your browser's exact origin. Generated images are only used when the image exercise is enabled."}
            </p>
        </section>
    }
}
