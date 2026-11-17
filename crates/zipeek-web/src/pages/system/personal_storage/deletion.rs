use crate::services::personal_data::PersonalDataService;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use super::personal_storage_categories::CATEGORIES;

#[derive(Properties, PartialEq)]
pub(super) struct DeletionPanelProps {
    pub message: UseStateHandle<String>,
}

#[function_component(DeletionPanel)]
pub(super) fn deletion_panel(props: &DeletionPanelProps) -> Html {
    let environment = crate::hooks::use_app_environment();
    let personal_data = PersonalDataService::new(
        environment.local_storage.clone(),
        environment.document.clone(),
        environment.navigator.clone(),
    );
    let message = props.message.clone();
    let selected = use_state(Vec::<String>::new);
    let confirmation = use_state(String::new);
    let deleting = use_state(|| false);
    let deletion_running = use_mut_ref(|| false);
    let on_confirm = {
        let confirmation = confirmation.clone();
        Callback::from(move |event: InputEvent| {
            confirmation.set(event.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };

    let delete = {
        let personal_data = personal_data.clone();
        let selected = selected.clone();
        let confirmation = confirmation.clone();
        let message = message.clone();
        let deleting = deleting.clone();
        let deletion_running = deletion_running.clone();
        Callback::from(move |_| {
            if *confirmation != "DELETE" || selected.is_empty() {
                return;
            }
            {
                let mut running = deletion_running.borrow_mut();
                if *running {
                    return;
                }
                *running = true;
            }
            deleting.set(true);
            let categories = (*selected).clone();
            let message = message.clone();
            let confirmation = confirmation.clone();
            let selected = selected.clone();
            let deleting = deleting.clone();
            let deletion_running = deletion_running.clone();
            let personal_data = personal_data.clone();
            spawn_local(async move {
                match personal_data.delete_selected(&categories).await {
                    Ok(result) => {
                        message.set(result);
                        confirmation.set(String::new());
                        selected.set(Vec::new());
                    }
                    Err(error) => {
                        message.set(error.to_string());
                    }
                }
                *deletion_running.borrow_mut() = false;
                deleting.set(false);
            });
        })
    };
    html! {
            <div
                class="space-y-3 rounded-xl border border-red-500/30 p-4"
            >
                <h3
                    class="font-semibold"
                >
                    {"Delete learning data"}
                </h3>
                <p
                    class="text-sm text-muted"
                >
                    {"Choose what to erase. This also removes any private snapshot so deleted records cannot be restored from an old local snapshot. Download a backup first if needed."}
                </p>
                {for CATEGORIES.iter().map(|(id, label)| {
                    let value = (*id).to_owned();
                    let checked = selected.contains(&value);
                    let selected = selected.clone();
                    let confirmation = confirmation.clone();
                    let on_toggle_category = Callback::from(move |event: Event| {
                        let checked = event.target_unchecked_into::<web_sys::HtmlInputElement>().checked();
                        let mut next = (*selected).clone();
                        next.retain(|item| item != &value);
                        if checked { next.push(value.clone()); }
                        selected.set(next);
                        confirmation.set(String::new());
                    });
                    html! {
                        <label
                            class="flex items-center gap-2 text-sm"
                        >
                            <input
                                type="checkbox"
                                checked={checked}
                                onchange={on_toggle_category}
                            />{*label}
                            </label>}
                })}
                <label
                    class="block text-sm"
                    for="personal-delete-confirm"
                >
                    {"Type DELETE to confirm permanent removal"}
                </label>
                <textarea
                    id="personal-delete-confirm"
                    rows="1"
                    data-personal-delete-confirm=""
                    class="w-full rounded-lg border border-line bg-raised p-3"
                    value={(*confirmation).clone()}
                    oninput={on_confirm}
                />
                <button
                    type="button"
                    data-personal-delete=""
                    disabled={*deleting || *confirmation != "DELETE" || selected.is_empty()}
                    class="rounded-lg bg-red-700 px-4 py-3 font-semibold text-white disabled:opacity-50"
                    onclick={delete}
                >
                    {"Delete selected learning data"}
                </button>
            </div>
    }
}
