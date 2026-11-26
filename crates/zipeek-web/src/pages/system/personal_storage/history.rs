use crate::hooks::use_app_environment;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[function_component(PersonalLearningHistoryPanel)]
pub(super) fn personal_learning_history_panel() -> Html {
    let environment = use_app_environment();
    let confirmation = use_state(String::new);
    let running = use_state(|| false);
    let feedback = use_state(String::new);
    let on_input = {
        let confirmation = confirmation.clone();
        Callback::from(move |event: InputEvent| {
            confirmation.set(event.target_unchecked_into::<HtmlTextAreaElement>().value());
        })
    };
    let on_erase = {
        let document = environment.document.clone();
        let confirmation = confirmation.clone();
        let running = running.clone();
        let feedback = feedback.clone();
        Callback::from(move |_: MouseEvent| {
            if *running || *confirmation != "DELETE HISTORY" {
                return;
            }
            running.set(true);
            let document = document.clone();
            let confirmation = confirmation.clone();
            let running = running.clone();
            let feedback = feedback.clone();
            spawn_local(async move {
                match crate::services::learning_history::erase(&document).await {
                    Ok(()) => {
                        confirmation.set(String::new());
                        feedback.set("Saved Practice history erased. Active temporary sessions are unchanged.".into());
                    }
                    Err(error) => {
                        feedback.set(format!("Practice history deletion failed: {error}"))
                    }
                }
                running.set(false);
            });
        })
    };
    html! {
        <section data-personal-learning-history="" class="rounded-lg border border-red-500/30 p-3 space-y-3">
            <h4 class="font-semibold">{"Erase saved Practice history"}</h4>
            <p class="text-sm text-muted">{"This deletes the personal Practice outcomes database records. The portable JSON backup does not include this history."}</p>
            <label class="block text-sm font-medium" for="delete-learning-history-confirmation">{"Enter DELETE HISTORY to erase these records"}</label>
            <textarea id="delete-learning-history-confirmation" rows="1" class="w-full rounded-lg border border-line bg-raised p-2" value={(*confirmation).clone()} oninput={on_input} data-personal-history-delete-confirmation="" />
            <button type="button" disabled={*running || *confirmation != "DELETE HISTORY"} onclick={on_erase} data-personal-history-delete="" class="min-h-10 rounded-lg bg-red-700 px-4 py-2 text-sm font-semibold text-white disabled:opacity-50">{"Erase Practice history"}</button>
            if !feedback.is_empty() { <p role="status" class="text-sm">{(*feedback).clone()}</p> }
        </section>
    }
}
