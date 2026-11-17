mod backup;
mod feedback;
mod manager;
use self::{backup::WordSetBackup, feedback::WordSetMessages, manager::WordSetManager};
use crate::hooks::use_word_sets;
use crate::route::Route;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew_icons::{Icon, IconData};
use yew::prelude::*;
use yew_router::prelude::use_navigator;

use crate::{
    components::WordSetsCommand,
    reducers::word_sets::{WordSetsAction, WordSetsFeedback},
    services::word_sets,
};

#[function_component(WordSetPanel)]
pub(crate) fn word_set_panel() -> Html {
    let sets = use_word_sets();
    let navigator = use_navigator();
    let open_selected_in_practice = {
        let navigator = navigator.clone();
        let target = sets.state.target.clone();
        Callback::from(move |_| {
            if let (Some(navigator), Some(target)) = (&navigator, &target) {
                let source = format!("saved:{target}");
                if let Err(error) =
                    navigator.push_with_query(&Route::PracticeSetup, &[("source", source.as_str())])
                {
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
        Callback::from(move |event: InputEvent| {
            set_name.set(event.target_unchecked_into::<HtmlInputElement>().value())
        })
    };
    let on_renamed_set = {
        let renamed_set = renamed_set.clone();
        Callback::from(move |event: InputEvent| {
            renamed_set.set(event.target_unchecked_into::<HtmlInputElement>().value())
        })
    };
    let on_rename = {
        let sets = sets.clone();
        let renamed_set = renamed_set.clone();
        Callback::from(move |_: MouseEvent| {
            sets.execute
                .emit(WordSetsCommand::Rename((*renamed_set).clone()));
            renamed_set.set(String::new());
        })
    };
    let on_delete = {
        let sets = sets.clone();
        let confirm_delete = confirm_delete.clone();
        Callback::from(move |_: MouseEvent| {
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
                        sets.state
                            .dispatch(WordSetsAction::Feedback(WordSetsFeedback::Success(
                                "Backup generated".into(),
                            )));
                    }
                    Err(error) => sets
                        .state
                        .dispatch(WordSetsAction::Feedback(WordSetsFeedback::Error(error))),
                }
            }
        })
    };
    let on_import = {
        let sets = sets.clone();
        let backup_text = backup_text.clone();
        Callback::from(move |_| {
            sets.execute
                .emit(WordSetsCommand::Import((*backup_text).clone()))
        })
    };
    let on_backup_text = {
        let backup_text = backup_text.clone();
        Callback::from(move |event: InputEvent| {
            backup_text.set(event.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };
    let on_reload = {
        let reload = sets.reload.clone();
        Callback::from(move |_| reload.emit(()))
    };
    let target_word_content = sets.state.target.is_some().then(|| {
        html! {
            <WordSetManager
                renamed_set={(*renamed_set).clone()}
                confirm_delete={*confirm_delete}
                on_renamed_set={on_renamed_set}
                {on_rename}
                {on_delete}
            />
        }
    });
    let selected_word_set_details = if let (Ok(saved), Some(id)) =
        (&sets.state.sets, &sets.state.target)
    {
        if let Some(set) = word_sets::find(saved, id) {
            html! {
                <button
                    type="button"
                    data-practice-selected-set=""
                    class="min-h-10 rounded-lg bg-brand-action px-3 text-sm font-semibold text-white disabled:opacity-50"
                    onclick={open_selected_in_practice}
                    disabled={set.word_ids.is_empty()}
                >
                    {format!("Use {} in Practice", set.name)}
                </button>
            }
        } else {
            Html::default()
        }
    } else {
        Html::default()
    };
    let selected_label = if let (Ok(saved), Some(id)) = (&sets.state.sets, &sets.state.target) {
        word_sets::find(saved, id).map(|set| set.name.clone())
    } else {
        None
    };
    let saved_word_set_details = if let Ok(saved) = &sets.state.sets {
        html! {
            <div class="space-y-1" role="group" aria-label="Available Word Sets">
                {for saved.iter().map(|set| {
                    let id = set.id.clone();
                    let chosen = sets.state.target.as_ref() == Some(&id);
                    let select = sets.select.clone();
                    let confirm_delete = confirm_delete.clone();
                    let renamed_set = renamed_set.clone();
                    let on_select = Callback::from(move |_| {
                        select.emit(Some(id.clone()));
                        confirm_delete.set(false);
                        renamed_set.set(String::new());
                    });
                    html! {
                        <button
                            type="button"
                            key={set.id.to_string()}
                            data-word-set-option={set.id.to_string()}
                            data-selected={chosen.to_string()}
                            aria-current={chosen.then_some("true")}
                            class={classes!("flex", "min-h-11", "w-full", "items-center", "justify-between", "gap-2", "rounded-lg", "px-3", "py-2", "text-left", "text-sm", "transition-colors", "hover:bg-raised", "focus-visible:ring-2", "focus-visible:ring-accent", if chosen { "bg-accent-soft" } else { "" })}
                            onclick={on_select}
                        >
                            <span class="min-w-0 truncate">{&set.name}</span>
                            <span class="shrink-0 text-xs text-muted">{format!("{} words", set.word_ids.len())}</span>
                        </button>
                    }
                })}
            </div>
        }
    } else {
        Html::default()
    };
    html! {
        <aside
            data-component="word-set-panel"
            class="flex flex-col gap-4 rounded-2xl border border-line bg-surface p-4 shadow-sm backdrop-blur-xl sm:p-5 lg:sticky lg:top-4"
        >
            <div class="flex items-center justify-between gap-2">
                <h2 class="text-lg font-semibold">{"My sets"}</h2>
                <Icon data={IconData::LUCIDE_BOOK_OPEN} width="19px" height="19px" />
            </div>
            <form class="space-y-2" onsubmit={Callback::from({
                let sets = sets.clone();
                let set_name = set_name.clone();
                move |event: SubmitEvent| {
                    event.prevent_default();
                    if !set_name.trim().is_empty() {
                        sets.execute.emit(WordSetsCommand::Create((*set_name).clone()));
                        set_name.set(String::new());
                    }
                }
            })}>
                <label class="block text-xs font-semibold text-muted" for="new-word-set-name">{"New Word Set"}</label>
                <input id="new-word-set-name" data-new-word-set="" class="min-h-11 w-full rounded-lg border border-line bg-surface px-3 text-sm" maxlength="64" value={(*set_name).clone()} oninput={on_set_name} placeholder="My vocabulary" />
                <button type="submit" data-create-word-set="" class="inline-flex min-h-10 w-full items-center justify-center gap-2 rounded-lg bg-brand-action px-3 text-sm font-semibold text-white disabled:opacity-50" disabled={sets.state.sets.is_err() || set_name.trim().is_empty()}>
                    <Icon data={IconData::LUCIDE_PLUS} width="17px" height="17px" />
                    {"New set"}
                </button>
            </form>
            <WordSetMessages load_error={sets.state.sets.as_ref().err().cloned()} feedback={sets.state.feedback.clone()} on_reload={on_reload}/>
            <div class="space-y-2" data-component="word-set-selector">
                <div data-target-word-set="" data-selected-word-set-id={sets.state.target.as_ref().map(ToString::to_string).unwrap_or_default()} class="text-xs font-semibold text-muted">
                    {selected_label.unwrap_or_else(|| "Choose a personal set".to_owned())}
                </div>
                <nav data-word-set-options="" aria-label="Available Word Sets">
                    {saved_word_set_details}
                </nav>
                <p class="text-xs text-muted">{"Words saved from Dictionary go to this set. Your selection is remembered; Practice chooses its vocabulary separately."}</p>
            </div>
            {selected_word_set_details}
            {target_word_content}
            <details class="rounded-lg border border-line p-3" data-word-set-data-tools="">
                <summary class="flex cursor-pointer items-center gap-2 text-sm font-semibold">
                    <Icon data={IconData::LUCIDE_SETTINGS} width="17px" height="17px" />
                    {"Data tools"}
                </summary>
                <div class="mt-3">
                    <WordSetBackup disabled={sets.state.sets.is_err()} backup_text={(*backup_text).clone()} {on_export} {on_import} {on_backup_text}/>
                </div>
            </details>
        </aside>
    }
}
