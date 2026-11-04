use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::practice_draft::PracticeDraft;

#[derive(Properties, PartialEq)]
pub struct PracticeBuilderProps {
    pub draft: PracticeDraft,
    pub active_tab: String,
    pub on_toggle: Callback<&'static str>,
    pub on_tab: Callback<String>,
    pub section: String,
}

#[function_component(PracticeBuilder)]
pub fn practice_builder(props: &PracticeBuilderProps) -> Html {
    let selected_count = props.draft.exercise_ids.len();
    let exercises = [
        ("matching", "Matching", "Pair Chinese words with meanings or readings."),
        ("multiple-choice", "Multiple choice", "Choose the correct word or reading."),
        ("sentence-translation", "Sentence translation", "Translate sentences with a word bank."),
        ("speaking", "Speaking recorder", "Read aloud and replay a local recording."),
        ("listening-choice", "Listening", "Audio-based choices are not implemented yet."),
    ];
    html! {
        <div data-component="practice-builder" class="flex flex-col gap-5">
            {if props.section == "exercises" { html! { <section class="flex flex-col gap-3">
                <div>
                    <h2 class="text-lg font-semibold">{"Choose exercise types"}</h2>
                    <p class="text-sm text-muted">{"Select one or more. Settings become available for the types you select."}</p>
                </div>
                <div role="group" aria-label="Choose exercise types" data-practice-tabs="" class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4">
                    {for exercises.into_iter().map(|(id, title, description)| {
                        let selected = props.draft.has_exercise(id);
                        let disabled = id == "listening-choice";
                        let on_toggle = props.on_toggle.clone();
                        html! {
                            <button key={id} type="button" data-practice-exercise={id}
                                data-practice-tab={if id == "sentence-translation" {"translation"} else if id == "speaking" {"speaking"} else {id}}
                                aria-pressed={selected.to_string()} disabled={disabled}
                                class={classes!("flex", "min-h-28", "flex-col", "gap-2", "rounded-xl", "border", "p-4", "text-left", "transition-colors", "disabled:cursor-not-allowed", "disabled:opacity-50", if selected {"border-blue-400 bg-blue-950/40"} else {"border-line bg-surface hover:border-blue-400"})}
                                onclick={Callback::from(move |_| {
                                    on_toggle.emit(id);
                                })}>
                                <span class="flex w-full items-center justify-between gap-3 text-sm font-semibold">
                                    <span class="flex items-center gap-2">
                                        <Icon data={match id {
                                            "matching" => IconData::LUCIDE_SHUFFLE,
                                            "multiple-choice" => IconData::LUCIDE_GRADUATION_CAP,
                                            "sentence-translation" => IconData::LUCIDE_BOOK_OPEN,
                                            "speaking" => IconData::LUCIDE_VOLUME_2,
                                            _ => IconData::LUCIDE_SETTINGS,
                                        }} width="18px" height="18px" />
                                        {title}
                                    </span>
                                    <span aria-hidden="true" class={classes!("flex", "h-5", "w-5", "items-center", "justify-center", "rounded", "border", if selected {"border-blue-400 bg-blue-600 text-white"} else {"border-muted"})}>{if selected {"✓"} else {""}}</span>
                                </span>
                                <span class="text-sm text-muted">{description}</span>
                                {if id == "speaking" {html!{<span class="text-xs text-muted">{"Unscored preview"}</span>}} else {Html::default()}}
                                {if disabled {html!{<span class="text-xs text-muted">{"Coming later"}</span>}} else {Html::default()}}
                            </button>
                        }
                    })}
                </div>
                {if selected_count == 0 {
                    html! {<p role="alert" class="text-sm text-muted">{"Choose at least one exercise type to configure practice."}</p>}
                } else if selected_count > 1 {
                    html! {<p role="status" class="text-sm text-muted">{"Multiple types selected. This release retains the existing separate launch flows; one mixed Start action will follow the shared session planner."}</p>}
                } else {Html::default()}}
            </section> } } else { Html::default() }}
            {if props.section == "configure" { html! { <section class="flex flex-col gap-3">
                <h2 class="text-lg font-semibold">{"Configure exercises"}</h2>
                <div role="tablist" aria-label="Exercise settings" data-practice-config-tabs="" class="flex gap-2 overflow-x-auto rounded-xl border border-line bg-surface p-2">
                    {for props.draft.configuration_tabs().into_iter().map(|tab| {
                        let selected = props.active_tab == tab;
                        let on_tab = props.on_tab.clone();
                        let label = match tab {"general" => "General", "matching" => "Matching", "multiple-choice" => "Multiple choice", "translation" => "Sentence translation", "speaking" => "Speaking", _ => "Listening"};
                        html! {
                            <button key={tab} type="button" role="tab" aria-selected={selected.to_string()}
                                data-practice-config-tab={tab}
                                class={classes!("min-h-11", "shrink-0", "rounded-lg", "px-4", "text-sm", "font-semibold", if selected {"bg-blue-600 text-white"} else {"text-muted hover:bg-raised hover:text-ink"})}
                                onclick={Callback::from(move |_| on_tab.emit(tab.to_owned()))}>{label}</button>
                        }
                    })}
                </div>
            </section> } } else { Html::default() }}
        </div>
    }
}
