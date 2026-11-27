use std::{ops::Deref, rc::Rc};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::sentence_builder::{
    self, BuilderAction, BuilderState, DropTarget, SentencePrompt, TileId, Verdict,
};

use super::tile::{BankTile, SelectedTile};
use crate::pages::practice::feedback::{ExerciseFeedback, ExerciseReview, FeedbackVerdict};
use crate::pages::practice::instruction::InstructionIcon;
use crate::pages::practice::surface::ExerciseSurface;
use crate::{
    models::SentenceBuilderOptions,
    services::practice_session_store::{ExerciseAssessment, ExerciseVerdict},
};

#[derive(Properties, PartialEq)]
pub(super) struct Props {
    pub seed: u64,
    pub prompt: &'static SentencePrompt,
    pub options: SentenceBuilderOptions,
    pub on_completed: Callback<()>,
    #[prop_or_default]
    pub on_assessed: Option<Callback<ExerciseAssessment>>,
}

#[derive(PartialEq)]
struct ReducerState(BuilderState);

impl Deref for ReducerState {
    type Target = BuilderState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Reducible for ReducerState {
    type Action = BuilderAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = self.0.clone();
        next.apply(action);
        if next == self.0 {
            self
        } else {
            Rc::new(Self(next))
        }
    }
}

#[function_component(SentenceBuilderExercise)]
pub(super) fn sentence_builder_exercise(props: &Props) -> Html {
    let prompt = props.prompt;
    let state = use_reducer_eq(|| {
        ReducerState(BuilderState::new(
            prompt,
            props.options.automatic_check,
            props.options.retry_incorrect,
        ))
    });
    let bank = use_memo(props.seed, move |seed| {
        sentence_builder::shuffled_tile_ids(prompt, *seed)
    });
    {
        let on_completed = props.on_completed.clone();
        let on_assessed = props.on_assessed.clone();
        let result = state.completed().then(|| ExerciseAssessment {
            verdict: if state.verdict() == Some(Verdict::Correct) {
                ExerciseVerdict::Correct
            } else {
                ExerciseVerdict::Incorrect
            },
            attempts: state.checks(),
            mistakes: state.mistakes(),
            hints: 0,
            assisted: false,
            item_evidence: Vec::new(),
        });
        use_effect_with(result, move |result| {
            if let Some(assessment) = result {
                if let Some(on_assessed) = &on_assessed {
                    on_assessed.emit(assessment.clone());
                } else {
                    on_completed.emit(());
                }
            }
        });
    }
    let on_action = {
        let state = state.clone();
        Callback::from(move |action| state.dispatch(action))
    };
    let undo = {
        let on_action = on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Undo))
    };
    let clear = {
        let on_action = on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Clear))
    };
    let check = {
        let on_action = on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Check))
    };
    let retry = {
        let on_action = on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Retry))
    };
    let continue_session = {
        let on_action = on_action.clone();
        Callback::from(move |_: MouseEvent| on_action.emit(BuilderAction::Continue))
    };
    let drop_at_end = {
        let on_action = on_action.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            on_action.emit(BuilderAction::Drop(DropTarget::End));
        })
    };
    let allow_drop = Callback::from(|event: DragEvent| event.prevent_default());
    let selected = state.selected();
    let locked = state.locked();
    html! {
        <div class="w-full" data-practice-sentence-builder="" data-auto-check={props.options.automatic_check.to_string()} data-retry-incorrect={props.options.retry_incorrect.to_string()}>
            <ExerciseSurface icon={InstructionIcon::Arrange} instruction="Arrange the tiles to build the sentence">
            <p class="text-center text-lg font-medium text-ink sm:text-xl">{prompt.meaning}</p>
            <div class="flex min-h-20 flex-wrap items-center justify-center gap-2 rounded-xl border border-line bg-raised/50 p-3"
                aria-label="Constructed Chinese sentence" lang="zh-CN" data-sentence-builder-selection=""
                ondragover={allow_drop} ondrop={drop_at_end}>
                {for selected.iter().enumerate().map(|(index, &tile)| html! {
                    <SelectedTile key={tile.index().to_string()} tile={tile} label={prompt.tiles[tile.index()]} locked={locked}
                        can_move_left={index > 0} can_move_right={index + 1 < selected.len()} on_action={on_action.clone()} />
                })}
            </div>
            <div class="flex min-h-14 flex-wrap items-center justify-center gap-2" role="group" aria-label="Available Chinese phrase tiles" lang="zh-CN">
                {for bank.iter().map(|&index| {
                    let tile = TileId::new(index);
                    html! { <BankTile key={index.to_string()} tile={tile} label={prompt.tiles[index]} disabled={locked || selected.contains(&tile)} on_action={on_action.clone()} /> }
                })}
            </div>
            <div class="flex flex-wrap items-center justify-center gap-2" data-sentence-builder-actions="">
                if !locked {
                    <button type="button" onclick={undo} disabled={selected.is_empty()} class="inline-flex size-11 items-center justify-center rounded-lg border border-line text-ink hover:bg-raised disabled:opacity-30" aria-label="Undo" title="Undo">
                        <Icon data={IconData::LUCIDE_UNDO_2} width="20" height="20" />
                    </button>
                    <button type="button" onclick={clear} disabled={selected.is_empty()} class="inline-flex size-11 items-center justify-center rounded-lg border border-line text-ink hover:bg-raised disabled:opacity-30" aria-label="Clear" title="Clear">
                        <Icon data={IconData::LUCIDE_TRASH_2} width="20" height="20" />
                    </button>
                    if !props.options.automatic_check {
                        <button type="button" data-sentence-builder-check="" onclick={check} disabled={!state.complete_order()}
                            class="inline-flex size-11 items-center justify-center rounded-lg bg-brand-action text-white disabled:opacity-30" aria-label="Check answer" title="Check answer">
                            <Icon data={IconData::LUCIDE_CHECK} width="20" height="20" />
                        </button>
                    }
                }
            </div>
            if let Some(verdict) = state.verdict() {
                if verdict == Verdict::Incorrect && props.options.retry_incorrect {
                    <ExerciseFeedback verdict={FeedbackVerdict::Incorrect}>
                        <p class="text-sm text-ink">{"Try another order"}</p>
                        <button type="button" onclick={retry} class="rounded-lg border border-line px-3 py-2 text-sm text-ink hover:bg-surface">{"Try again"}</button>
                    </ExerciseFeedback>
                } else {
                    <ExerciseReview
                        verdict={if verdict == Verdict::Correct { FeedbackVerdict::Correct } else { FeedbackVerdict::Incorrect }}
                        on_continue={continue_session}
                        disabled={!state.can_continue()}
                        sentence_builder=true
                    >
                        if verdict == Verdict::Incorrect {
                            if let Some(solution) = prompt.accepted_orders.first().and_then(|order| sentence_builder::sentence(prompt, order)) {
                                <p lang="zh-CN">{solution}</p>
                            }
                        }
                        <p class="text-sm text-muted">{prompt.explanation}</p>
                    </ExerciseReview>
                }
            }
            </ExerciseSurface>
        </div>
    }
}
