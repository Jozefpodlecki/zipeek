use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::{hooks::use_app_environment, services::image_generation};

#[derive(Clone, PartialEq)]
enum Feedback {
    Healthy(String),
    Saved,
    Reset,
    Failure(String),
}

#[function_component(ImageServerSettings)]
pub fn image_server_settings() -> Html {
    let environment = use_app_environment();
    let is_tauri = environment.is_tauri;
    let storage = environment.local_storage.clone();
    let url = use_state(move || image_generation::read_api_url(&storage));
    let checking = use_state(|| false);
    let generating = use_state(|| false);
    let feedback = use_state(|| None::<Feedback>);
    let preview = use_state(|| None::<String>);
    let generation = use_mut_ref(|| 0u64);
    use_effect_with((*preview).clone(), |current| {
        let previous = current.clone();
        move || {
            if let Some(url) = previous {
                image_generation::release_preview(&url);
            }
        }
    });

    let on_url_change = {
        let url = url.clone();
        let feedback = feedback.clone();
        let generation = generation.clone();
        Callback::from(move |event: InputEvent| {
            *generation.borrow_mut() += 1;
            url.set(event.target_unchecked_into::<HtmlInputElement>().value());
            feedback.set(None);
        })
    };
    let on_save = {
        let storage = environment.local_storage.clone();
        let url = url.clone();
        let feedback = feedback.clone();
        Callback::from(
            move |_: MouseEvent| match image_generation::save_api_url(&storage, &url) {
                Ok(()) => {
                    crate::contexts::service_management::notify_configuration_changed();
                    feedback.set(Some(Feedback::Saved));
                }
                Err(error) => feedback.set(Some(Feedback::Failure(error))),
            },
        )
    };
    let on_reset = {
        let storage = environment.local_storage.clone();
        let url = url.clone();
        let feedback = feedback.clone();
        let generation = generation.clone();
        Callback::from(move |_: MouseEvent| {
            *generation.borrow_mut() += 1;
            match image_generation::reset_api_url(&storage) {
                Ok(()) => {
                    url.set(image_generation::DEFAULT_URL.to_owned());
                    feedback.set(Some(Feedback::Reset));
                    crate::contexts::service_management::notify_configuration_changed();
                }
                Err(error) => feedback.set(Some(Feedback::Failure(error))),
            }
        })
    };
    let on_check = {
        let url = url.clone();
        let checking = checking.clone();
        let feedback = feedback.clone();
        let generation = generation.clone();
        Callback::from(move |_: MouseEvent| {
            if *checking {
                return;
            }
            let ticket = {
                let mut current = generation.borrow_mut();
                *current += 1;
                *current
            };
            checking.set(true);
            feedback.set(None);
            let endpoint = (*url).clone();
            let checking = checking.clone();
            let feedback = feedback.clone();
            let generation = generation.clone();
            spawn_local(async move {
                let result = image_generation::check_health(&endpoint).await;
                if *generation.borrow() != ticket {
                    return;
                }
                checking.set(false);
                feedback.set(Some(match result {
                    Ok(message) => Feedback::Healthy(message),
                    Err(error) => Feedback::Failure(error),
                }));
            });
        })
    };
    let on_generate = {
        let generating = generating.clone();
        let feedback = feedback.clone();
        let preview = preview.clone();
        let url = url.clone();
        let generation = generation.clone();
        Callback::from(move |_: MouseEvent| {
            if *generating {
                return;
            }
            let ticket = {
                let mut current = generation.borrow_mut();
                *current += 1;
                *current
            };
            generating.set(true);
            feedback.set(None);
            preview.set(None);
            let endpoint = (*url).clone();
            let generating = generating.clone();
            let feedback = feedback.clone();
            let preview = preview.clone();
            let generation = generation.clone();
            spawn_local(async move {
                let result = image_generation::generate(
                    &endpoint,
                    "A simple Chinese landscape painting of a mountain and river",
                )
                .await;
                if *generation.borrow() != ticket {
                    return;
                }
                generating.set(false);
                match result {
                    Ok(url) => preview.set(Some(url)),
                    Err(error) => feedback.set(Some(Feedback::Failure(error))),
                }
            });
        })
    };

    let button_class = "inline-flex min-h-10 items-center justify-center gap-2 rounded-lg border border-line px-4 text-sm font-medium hover:bg-raised disabled:opacity-50";
    html! {
        <section data-settings-image-server="" class="w-full rounded-2xl border border-line bg-surface p-5 shadow-sm sm:p-7">
            <div class="flex items-center gap-3">
                <Icon data={IconData::LUCIDE_IMAGE} width="22px" height="22px" />
                <h2 class="text-xl font-semibold">{"Image generation"}</h2>
            </div>
            if is_tauri {
                <div class="mt-5 flex items-center justify-between gap-3 rounded-xl border border-line bg-raised/50 p-4">
                    <div class="min-w-0">
                        <p class="font-medium">{"Zipeek image service"}</p>
                        <p class="mt-1 font-mono text-xs text-muted">{"http://127.0.0.1:1431"}</p>
                    </div>
                    <span class="shrink-0 text-xs text-muted">{"Managed locally"}</span>
                </div>
                <p class="mt-3 text-xs text-muted">{"Real image generation requires SDXL model files and a supported CUDA device. Configure ZIPEEK_IMAGE_MODEL_DIR before starting the service."}</p>
            } else {
                <label class="mt-5 flex flex-col gap-2 text-sm font-medium">
                    {"Image API URL"}
                    <input type="url" data-image-server-url="" value={(*url).clone()}
                        oninput={on_url_change} placeholder={image_generation::DEFAULT_URL}
                        class="min-h-11 w-full rounded-lg border border-line bg-raised px-3 text-ink focus-visible:outline-2 focus-visible:outline-accent" />
                </label>
            }
            <div class="mt-5 flex flex-wrap items-center gap-2">
                if !is_tauri {
                    <button type="button" data-action="save" onclick={on_save} class={button_class}>{"Save"}</button>
                    <button type="button" data-action="reset" onclick={on_reset} class={button_class}>{"Reset"}</button>
                }
                <button type="button" onclick={on_check} disabled={*checking} class={button_class} aria-label="Check image service health">
                    <Icon data={IconData::LUCIDE_REFRESH_CW} width="16px" height="16px"
                        class={if *checking {"animate-spin motion-reduce:animate-none"} else {""}} />
                    {if *checking {"Checking"} else {"Check health"}}
                </button>
                <button type="button" onclick={on_generate} disabled={*generating} class={button_class} aria-label="Generate a test image">
                    <Icon data={if *generating {IconData::LUCIDE_REFRESH_CW} else {IconData::LUCIDE_WAND_2}}
                        width="16px" height="16px" class={if *generating {"animate-spin motion-reduce:animate-none"} else {""}} />
                    {if *generating {"Generating"} else {"Test generation"}}
                </button>
            </div>
            if let Some(status) = &*feedback {
                <p class={classes!("mt-4", "text-sm", if matches!(status, Feedback::Failure(_)) {"text-amber-600 dark:text-amber-400"} else {"text-muted"})}
                    role={if matches!(status, Feedback::Failure(_)) {"alert"} else {"status"}}>
                    {match status {
                        Feedback::Healthy(message) => message.clone(),
                        Feedback::Saved => "Endpoint saved".to_owned(),
                        Feedback::Reset => "Default endpoint restored".to_owned(),
                        Feedback::Failure(message) => message.clone(),
                    }}
                </p>
            }
            if let Some(url) = &*preview {
                <div class="mt-5 space-y-2">
                    <p class="text-xs font-medium text-muted">{"Generated preview"}</p>
                    <img src={url.clone()} alt="Test image generated by the configured image service"
                        class="max-h-72 w-full rounded-xl border border-line bg-raised object-contain" />
                </div>
            }
        </section>
    }
}
