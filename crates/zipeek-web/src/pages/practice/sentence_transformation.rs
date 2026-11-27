use super::feedback::{ExercisePrimaryAction, ExerciseReview, FeedbackVerdict};
use super::instruction::InstructionIcon;
use super::surface::ExerciseSurface;
use std::rc::Rc;
use yew::TargetCast;
use yew::prelude::*;
use zipeek_exercises::sentence_transformation::{
    self, AnswerMode, TransformationAction, TransformationItem, TransformationState,
};

use crate::{
    models::SentenceTransformationOptions,
    services::practice_session_store::{ExerciseAssessment, ExerciseVerdict},
};

#[derive(Properties, PartialEq)]
pub struct SentenceTransformationProps {
    pub seed: u64,
    pub options: SentenceTransformationOptions,
    pub on_completed: Callback<()>,
    #[prop_or_default]
    pub on_assessed: Option<Callback<ExerciseAssessment>>,
}

#[function_component(SentenceTransformation)]
pub fn sentence_transformation(props: &SentenceTransformationProps) -> Html {
    let Some(item) =
        sentence_transformation::item_for_seed(props.seed, props.options.pattern_index)
    else {
        return html! { <p role="alert">{"The selected grammar pattern has no authored question."}</p> };
    };
    html! { <TransformationExercise key={format!("{}-{}-{:?}", item.id, props.seed, props.options.answer_mode)}
    item={item} seed={props.seed} mode={props.options.answer_mode} on_completed={props.on_completed.clone()} on_assessed={props.on_assessed.clone()} /> }
}

#[derive(Properties, PartialEq)]
struct TransformationExerciseProps {
    item: &'static TransformationItem,
    seed: u64,
    mode: AnswerMode,
    on_completed: Callback<()>,
    #[prop_or_default]
    on_assessed: Option<Callback<ExerciseAssessment>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TransformationUiState(TransformationState);

impl std::ops::Deref for TransformationUiState {
    type Target = TransformationState;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl yew::Reducible for TransformationUiState {
    type Action = (
        TransformationAction,
        &'static TransformationItem,
        AnswerMode,
    );
    fn reduce(self: Rc<Self>, (action, item, mode): Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        if next.0.apply(item, mode, action) {
            Rc::new(next)
        } else {
            self
        }
    }
}

#[function_component(TransformationExercise)]
fn transformation_exercise(props: &TransformationExerciseProps) -> Html {
    let item = props.item;
    let mode = props.mode;
    let state = use_reducer(|| TransformationUiState(TransformationState::new(mode)));
    let selected = &state.selected;
    let typed = &state.typed;
    let checked = state.checked;
    let use_tiles = state.use_tiles;
    let order = sentence_transformation::shuffled_ids(item, props.seed);
    let correct = state.is_correct(item);
    let add_tile = {
        let state = state.clone();
        Callback::from(move |id: usize| {
            state.dispatch((TransformationAction::AddTile(id), item, mode))
        })
    };
    let remove_tile = {
        let state = state.clone();
        Callback::from(move |position: usize| {
            state.dispatch((TransformationAction::RemoveTile(position), item, mode))
        })
    };
    let undo = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch((TransformationAction::Undo, item, mode))
        })
    };
    let clear = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch((TransformationAction::ResetTiles, item, mode))
        })
    };
    let update_typed = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                state.dispatch((TransformationAction::SetTyped(input.value()), item, mode));
            }
        })
    };
    let check = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch((TransformationAction::Check, item, mode))
        })
    };
    let use_tiles_mode = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch((TransformationAction::UseTiles(true), item, mode))
        })
    };
    let use_typing_mode = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch((TransformationAction::UseTiles(false), item, mode))
        })
    };
    let continue_session = {
        let on_completed = props.on_completed.clone();
        let on_assessed = props.on_assessed.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(on_assessed) = &on_assessed {
                on_assessed.emit(ExerciseAssessment {
                    verdict: if correct {
                        ExerciseVerdict::Correct
                    } else {
                        ExerciseVerdict::Incorrect
                    },
                    attempts: 1,
                    mistakes: u16::from(!correct),
                    hints: 0,
                    assisted: false,
                    item_evidence: Vec::new(),
                });
            } else {
                on_completed.emit(());
            }
        })
    };
    let can_check = state.can_check(item);

    html! {
        <div class="w-full" data-practice-sentence-transformation="" data-item-id={item.id} data-transformation-method={if use_tiles {"tiles"} else {"typing"}} data-transformation-status={if checked {if correct {"correct"} else {"incorrect"}} else {"editing"}}>
            <ExerciseSurface icon={InstructionIcon::Rewrite} instruction="Rewrite using the requested grammar pattern">
            <p lang="zh-CN" class="text-lg">{item.original}</p>
            <p><strong>{"Target construction: "}</strong><span lang="zh-CN">{item.target_structure}</span></p>
            <p>{item.instruction}</p>
            {if mode == AnswerMode::Both { html! {
                <div role="group" aria-label="Answer method" data-transformation-method-switch="" class="flex flex-wrap gap-2">
                    <button type="button" data-transformation-select-method="tiles" onclick={use_tiles_mode} disabled={checked} aria-pressed={use_tiles.to_string()}
                        class={classes!("rounded-lg", "px-4", "py-2", if use_tiles {"bg-brand-action text-white"} else {"border border-line bg-raised"})}>
                        {"Word tiles"}
                    </button>
                    <button type="button" data-transformation-select-method="typing" onclick={use_typing_mode} disabled={checked} aria-pressed={(!use_tiles).to_string()}
                        class={classes!("rounded-lg", "px-4", "py-2", if !use_tiles {"bg-brand-action text-white"} else {"border border-line bg-raised"})}>
                        {"Type answer"}
                    </button>
                </div>
            }} else {Html::default()}}
            {if use_tiles { html! { <>
                <div class="flex min-h-14 flex-wrap items-center gap-2 rounded-lg border border-line p-3" role="group"
                    aria-label="Constructed sentence, select a tile to remove it" lang="zh-CN" data-sentence-transformation-selection="">
                    {for selected.iter().enumerate().map(|(position, &id)| {
                        let on_click = {
                            let remove_tile = remove_tile.clone();
                            Callback::from(move |_: MouseEvent| remove_tile.emit(position))
                        };
                        html! {<button key={format!("{position}-{id}")} type="button" data-transformation-selected-tile={id.to_string()} data-transformation-tile-position={position.to_string()} onclick={on_click} disabled={checked}
                            aria-label={format!("Remove {} from position {}", item.bank[id], position + 1)}
                            class="rounded-md bg-raised px-3 py-2 disabled:opacity-70">{item.bank[id]}</button>}
                    })}
                    {if selected.is_empty() {html! {<span class="text-muted">{"Select Chinese tiles below"}</span>}} else {Html::default()}}
                </div>
                <div class="flex flex-wrap gap-2" role="group" aria-label="Available Chinese word-bank tiles" lang="zh-CN" data-transformation-word-bank="">
                    {for order.iter().map(|&id| {
                        let on_click = {
                            let add_tile = add_tile.clone();
                            Callback::from(move |_: MouseEvent| add_tile.emit(id))
                        };
                        html! {<button key={id.to_string()} type="button" data-transformation-bank-tile={id.to_string()} onclick={on_click}
                            disabled={checked || selected.contains(&id)}
                            aria-label={format!("Add {}", item.bank[id])}
                            class="rounded-lg border border-line bg-raised px-4 py-3 disabled:opacity-40">{item.bank[id]}</button>}
                    })}
                </div>
            </> }} else {html! {
                <div class="space-y-2">
                    <label for="transformation-typed-answer" class="block text-sm font-semibold">{"Your Chinese sentence"}</label>
                    <textarea id="transformation-typed-answer" lang="zh-CN" rows="3" value={typed.to_owned()}
                        oninput={update_typed} disabled={checked} data-transformation-typed-answer=""
                        placeholder="Type the complete Chinese sentence" class="w-full rounded-lg border border-line bg-raised p-3 text-lg disabled:opacity-70" />
                    <p class="text-xs text-muted">{"Spaces and final punctuation are optional. Other variations must match an authored accepted answer."}</p>
                </div>
            }}}
            {if !checked {html! {
                <div class="flex flex-wrap gap-3">
                    {if use_tiles {html! {<>
                        <button type="button" data-transformation-undo="" onclick={undo} disabled={selected.is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Undo"}</button>
                        <button type="button" data-transformation-reset="" onclick={clear} disabled={selected.is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Reset"}</button>
                    </>}} else {Html::default()}}
                    <ExercisePrimaryAction label="Check answer" on_activate={check} disabled={!can_check} sentence_transformation=true />
                </div>
            }} else {html! {
                <ExerciseReview verdict={if correct { FeedbackVerdict::Correct } else { FeedbackVerdict::Incorrect }} on_continue={continue_session} sentence_transformation=true>
                    <span data-transformation-feedback={if correct {"correct"} else {"incorrect"}} />
                    <p class="text-sm">{if correct {item.explanation} else {item.incorrect_feedback}}</p>
                    <p><strong>{"Accepted answer: "}</strong><span lang="zh-CN">{sentence_transformation::answer(item, item.accepted_orders[0]).unwrap_or_default()}</span></p>
                </ExerciseReview>
            }}}
            </ExerciseSurface>
        </div>
    }
}
