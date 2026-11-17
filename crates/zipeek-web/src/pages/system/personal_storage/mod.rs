use crate::hooks::use_app_environment;
use crate::services::personal_data::PersonalDataService;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod backup;
mod deletion;
mod history;
mod personal_storage_categories;

#[function_component(PersonalStoragePanel)]
pub fn personal_storage_panel() -> Html {
    let environment = use_app_environment();
    let personal_data = PersonalDataService::new(
        environment.local_storage.clone(),
        environment.document.clone(),
        environment.navigator.clone(),
    );
    let message = use_state(String::new);
    let status = use_state(|| "Checking learning storage…".to_owned());
    {
        let status = status.clone();
        let personal_data = personal_data.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match personal_data.storage_status().await {
                    Ok(description) => status.set(description),
                    Err(error) => status.set(error.to_string()),
                }
            });
            || ()
        });
    }
    let storage_feedback = {
        if !message.is_empty() {
            html! {
            <p
                role="status"
                class="rounded-lg bg-raised p-3 text-sm"
                data-personal-feedback=""
            >
                {(*message).clone()}
            </p>}
        } else {
            Html::default()
        }
    };
    html! {
        <section
            data-component="personal-storage"
            class="flex flex-col gap-5 rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
        >
            <div
                class="space-y-2"
            >
                <h2
                    class="text-xl font-semibold"
                >
                {"Learning data"}
                </h2><p
                    class="text-sm text-muted"
                >
                    {"Record, review, export and delete your learning activity. App appearance and the dictionary are separate."}
                </p>
                </div>
            <p
                class="rounded-lg bg-raised px-3 py-3 text-sm text-muted"
                data-storage-status=""
            >
                {(*status).clone()}
            </p>
            <history::PersonalLearningHistoryPanel />
            <backup::BackupPanel message={message.clone()} />
            <deletion::DeletionPanel message={message.clone()} />
            {storage_feedback}
        </section>
    }
}
