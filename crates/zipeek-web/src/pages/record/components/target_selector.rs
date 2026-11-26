use web_sys::InputEvent;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::pages::record::state::{RecordState, TargetMode};

#[derive(Properties, PartialEq)]
pub(super) struct TargetSelectorProps {
    pub state: UseReducerHandle<RecordState>,
    pub on_mode: Callback<TargetMode>,
    pub on_sentence: Callback<InputEvent>,
    pub on_bank: Callback<(u8, u8)>,
    pub on_random: Callback<MouseEvent>,
}

#[function_component(TargetSelector)]
pub(super) fn target_selector(props: &TargetSelectorProps) -> Html {
    let state = &props.state;
    html! {
        <>
            <div class="flex items-center gap-2" role="group" aria-label="Recording target">
                {for [(TargetMode::Hsk, "HSK word"), (TargetMode::Sentence, "Custom sentence"), (TargetMode::Live, "Live") ].into_iter().map(|(mode, label)| {
                    let on_mode = props.on_mode.clone();
                    html! { <button type="button" key={label} aria-pressed={(state.mode == mode).to_string()} onclick={Callback::from(move |_| on_mode.emit(mode))} class={classes!("rounded-lg", "px-4", "py-2", "text-sm", "font-semibold", if state.mode == mode {"bg-brand-action text-white"} else {"bg-surface border border-line"})}>{label}</button> }
                })}
            </div>
            if state.mode == TargetMode::Hsk {
                <div class="flex flex-wrap items-center gap-x-4 gap-y-3 rounded-2xl border border-line bg-surface px-4 py-3" data-record-hsk-bank="">
                    <fieldset class="flex items-center gap-2">
                        <legend class="sr-only">{"HSK standard"}</legend>
                        <span class="text-xs font-semibold text-muted" aria-hidden="true">{"HSK"}</span>
                        <div role="group" aria-label="HSK standard" class="inline-flex rounded-lg border border-line bg-raised p-0.5">
                            {for [2_u8, 3_u8].into_iter().map(|system| {
                                let active = state.bank.0 == system;
                                let on_bank = props.on_bank.clone();
                                let level = state.bank.1.min(if system == 2 {6} else {7});
                                html! { <button type="button" key={system.to_string()} data-record-hsk-system={system.to_string()} aria-pressed={active.to_string()} onclick={Callback::from(move |_| on_bank.emit((system, level)))} class={classes!("min-h-9", "rounded-md", "px-3", "text-sm", "font-semibold", if active {"bg-brand-action text-white"} else {"text-muted hover:text-ink"})}>{format!("{system}.0")}</button> }
                            })}
                        </div>
                    </fieldset>
                    <fieldset class="flex min-w-0 items-center gap-1.5">
                        <legend class="sr-only">{"HSK level"}</legend>
                        <span class="mr-1 text-xs font-semibold text-muted" aria-hidden="true">{"Level"}</span>
                        {for (1_u8..=(if state.bank.0 == 2 {6} else {7})).map(|level| {
                            let active = state.bank.1 == level;
                            let on_bank = props.on_bank.clone();
                            let system = state.bank.0;
                            html! { <button type="button" key={level.to_string()} data-record-hsk-level={level.to_string()} aria-label={format!("HSK {system}.0 level {level}")} aria-pressed={active.to_string()} onclick={Callback::from(move |_| on_bank.emit((system, level)))} class={classes!("h-9", "min-w-9", "rounded-lg", "border", "text-sm", "font-semibold", if active {"border-brand-action bg-brand-action text-white"} else {"border-line bg-raised hover:bg-surface"})}>{level}</button> }
                        })}
                    </fieldset>
                    <button type="button" data-record-random="" disabled={state.loading || state.pool.is_empty()} onclick={props.on_random.clone()} aria-label="Random word" title="Random word" class="inline-flex h-9 w-9 items-center justify-center rounded-lg bg-brand-action text-white disabled:opacity-40 sm:ml-auto"><Icon data={IconData::LUCIDE_DICE_5} width="18px" height="18px" /></button>
                </div>
            } else {
                <textarea data-record-sentence="" aria-label="Mandarin sentence" rows="2" maxlength="200" placeholder="Enter a Mandarin sentence..." value={state.sentence.clone()} oninput={props.on_sentence.clone()} class="w-full resize-none rounded-xl border border-line bg-surface px-4 py-3 text-xl text-ink" />
            }
        </>
    }
}
