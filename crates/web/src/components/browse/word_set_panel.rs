use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::use_navigator;
use crate::route::Route;

use crate::{
    components::{WordSetsCommand, WordSetsContext},
    reducers::word_sets::{WordSetsAction, WordSetsFeedback},
    services::word_sets,
};

#[function_component(WordSetPanel)]
pub(crate) fn word_set_panel() -> Html {
    let sets = use_context::<WordSetsContext>().unwrap();
    let navigator = use_navigator();
    let open_selected_in_practice = {
        let navigator = navigator.clone();
        let target = sets.state.target.clone();
        Callback::from(move |_| {
            if let Some(navigator) = &navigator {
                let source = format!("saved:{target}");
                if let Err(error) = navigator.push_with_query(&Route::PracticeSetup, &[("source", source.as_str())]) {
                    log::error!("Could not open selected Word Set in Practice: {error}");
                }
            }
        })
    };
    let set_name = use_state(String::new);
    let renamed_set = use_state(String::new);
    let confirm_delete = use_state(|| false);
    let backup_text = use_state(String::new);

    let on_set_name = {
        let set_name = set_name.clone();
        Callback::from(move |event: InputEvent| set_name.set(event.target_unchecked_into::<HtmlInputElement>().value()))
    };
    let on_create_set = {
        let sets = sets.clone();
        let set_name = set_name.clone();
        Callback::from(move |_| {
            if !set_name.trim().is_empty() {
                sets.execute.emit(WordSetsCommand::Create((*set_name).clone()));
                set_name.set(String::new());
            }
        })
    };
    let on_target_set = {
        let state = sets.state.clone();
        let confirm_delete = confirm_delete.clone();
        Callback::from(move |event: Event| {
            state.dispatch(WordSetsAction::Select(event.target_unchecked_into::<HtmlSelectElement>().value()));
            confirm_delete.set(false);
        })
    };
    let on_renamed_set = {
        let renamed_set = renamed_set.clone();
        Callback::from(move |event: InputEvent| renamed_set.set(event.target_unchecked_into::<HtmlInputElement>().value()))
    };
    let on_rename = {
        let sets = sets.clone();
        let renamed_set = renamed_set.clone();
        Callback::from(move |_| {
            sets.execute.emit(WordSetsCommand::Rename((*renamed_set).clone()));
            renamed_set.set(String::new());
        })
    };
    let on_delete = {
        let sets = sets.clone();
        let confirm_delete = confirm_delete.clone();
        Callback::from(move |_| {
            if *confirm_delete {
                sets.execute.emit(WordSetsCommand::Delete);
                confirm_delete.set(false);
            } else {
                confirm_delete.set(true);
            }
        })
    };
    let on_export = {
        let sets = sets.clone();
        let backup_text = backup_text.clone();
        Callback::from(move |_| {
            if let Ok(saved) = &sets.state.sets {
                match word_sets::export(saved) {
                    Ok(raw) => {
                        backup_text.set(raw);
                        sets.state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Success("Backup generated".into())));
                    },
                    Err(error) => sets.state.dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error(error))),
                }
            }
        })
    };
    let on_import = {
        let sets = sets.clone();
        let backup_text = backup_text.clone();
        Callback::from(move |_| sets.execute.emit(WordSetsCommand::Import((*backup_text).clone())))
    };
    let on_backup_text = {
        let backup_text = backup_text.clone();
        Callback::from(move |event: InputEvent| backup_text.set(event.target_unchecked_into::<HtmlTextAreaElement>().value()))
    };
    let on_reload = {
        let reload = sets.reload.clone();
        Callback::from(move |_| reload.emit(()))
    };
    html! {
        <aside data-component="word-set-panel" class="flex flex-col gap-4 rounded-2xl border border-line bg-surface p-5 shadow-sm backdrop-blur-xl">
            <h2 class="text-xl font-semibold">{"My Word Sets"}</h2>
            <div class="flex flex-wrap items-end gap-3">
                <label class="flex flex-col gap-1 text-sm">{"New Word Set"}
                    <input data-new-word-set="" class="min-h-11 rounded-lg border border-line bg-surface px-3" maxlength="64" value={(*set_name).clone()} oninput={on_set_name} placeholder="My vocabulary" />
                </label>
                <button type="button" data-create-word-set="" class="min-h-11 rounded-lg border border-line px-4" onclick={on_create_set} disabled={sets.state.sets.is_err()}>{"Create set"}</button>
            </div>
            {if let Err(error) = &sets.state.sets {html! {<div role="alert" class="text-sm text-red-600"><p>{format!("Word Sets unavailable: {error}")}</p><button type="button" class="min-h-10 underline" onclick={on_reload}>{"Retry loading"}</button></div>}} else {Html::default()}}
            {if let Some(feedback) = &sets.state.feedback {match feedback {
                WordSetsFeedback::Error(message) => html! {<p role="alert" class="text-sm text-red-600">{message}</p>},
                WordSetsFeedback::Success(message) => html! {<p role="status" class="text-sm text-muted">{message}</p>},
            }} else {Html::default()}}
            <label class="flex flex-col gap-1 text-sm">{"Selected Word Set"}
                <select data-target-word-set="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={sets.state.target.clone()} onchange={on_target_set}>
                    <option value="">{"Choose a personal set"}</option>
                    {if let Ok(saved) = &sets.state.sets {html! {<>{for saved.iter().enumerate().map(|(index, set)| html! {<option value={index.to_string()}>{&set.name}</option>})}</>}} else {Html::default()}}
                </select>
            </label>
            {if let Ok(saved) = &sets.state.sets {
                if let Ok(index) = sets.state.target.parse::<usize>() {
                    if let Some(set) = saved.get(index) {
                        html! { <button type="button" data-practice-selected-set="" class="min-h-10 rounded-lg bg-blue-600 px-3 text-sm font-semibold text-white disabled:opacity-50" onclick={open_selected_in_practice} disabled={set.word_ids.is_empty()}> {format!("Use {} in Practice", set.name)} </button> }
                    } else { Html::default() }
                } else { Html::default() }
            } else { Html::default() }}
            {if !sets.state.target.is_empty() {html! {
                <div data-component="word-set-manager" class="flex flex-wrap items-end gap-3 rounded-lg border border-line p-3" data-manage-word-set="">
                    <label class="flex flex-col gap-1 text-sm">{"Rename selected set"}
                        <input data-rename-word-set="" class="min-h-11 rounded-lg border border-line bg-surface px-3" maxlength="64" value={(*renamed_set).clone()} oninput={on_renamed_set} placeholder="New name" />
                    </label>
                    <button type="button" data-confirm-rename-word-set="" class="min-h-11 rounded-lg border border-line px-3" onclick={on_rename}>{"Rename"}</button>
                    <button type="button" data-delete-word-set="" class="min-h-11 rounded-lg border border-line px-3" onclick={on_delete}>{if *confirm_delete {"Confirm deletion"} else {"Delete set"}}</button>
                </div>
            }} else {Html::default()}}
            <details data-component="word-set-backup" class="rounded-lg border border-line p-3" data-word-set-backup="">
                <summary class="cursor-pointer font-semibold">{"Back up or restore personal Word Sets"}</summary>
                <p class="mt-2 text-sm text-muted">{"Export as JSON and copy it somewhere safe. Paste a backup below to merge sets with matching names. Word IDs are specific to the dictionary build; check restored words after a database update."}</p>
                <div class="mt-3 flex flex-wrap gap-2">
                    <button type="button" data-export-word-sets="" class="min-h-11 rounded-lg border border-line px-3" disabled={sets.state.sets.is_err()} onclick={on_export}>{"Generate backup"}</button>
                    <button type="button" data-import-word-sets="" class="min-h-11 rounded-lg border border-line px-3" disabled={sets.state.sets.is_err() || backup_text.trim().is_empty()} onclick={on_import}>{"Merge pasted backup"}</button>
                </div>
                <label class="mt-3 flex flex-col gap-1 text-sm">{"Backup JSON"}
                    <textarea data-word-set-backup-json="" class="min-h-36 rounded-lg border border-line bg-surface p-3 font-mono text-xs" value={(*backup_text).clone()} oninput={on_backup_text} spellcheck="false" />
                </label>
            </details>
        </aside>
    }
}
