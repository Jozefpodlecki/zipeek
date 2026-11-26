use super::super::canvas::paint;
use crate::{
    components::hanzi_stroke_reference::HanziStrokeReference,
    hooks::use_app_environment,
    models::{StrokeAssistance, WriteHanziOptions, WritingGuidance},
};
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

use std::{cell::Cell, rc::Rc};

use super::board::WritingBoard;
use super::controller::{AttemptFeedback, WritingAction, WritingState};
use super::controls::WritingControls;
use super::standalone::{HanziTargetPicker, StrokeProgress};
use super::targets::TARGETS;

#[derive(Properties, PartialEq)]
pub struct HanziWritingProps {
    #[prop_or_default]
    pub selected: Option<(String, String, String)>,
    #[prop_or_default]
    pub on_completed: Option<Callback<()>>,
    #[prop_or_default]
    pub on_completion_change: Option<Callback<bool>>,
    #[prop_or_default]
    pub options: Option<WriteHanziOptions>,
}

#[function_component(HanziWriting)]
pub fn hanzi_writing(props: &HanziWritingProps) -> Html {
    let environment = use_app_environment();
    let is_practice = props.options.is_some();
    let practice_options = props.options.unwrap_or(WriteHanziOptions {
        guidance: WritingGuidance::FullGuide,
        animation: true,
        show_pinyin: true,
        show_meaning: true,
        playable_audio: true,
        assistance: StrokeAssistance::AfterThree,
    });
    let canvas = use_node_ref();
    let state = use_reducer(WritingState::default);
    let retry_epoch = use_state(|| 0u64);
    let resize_epoch = super::lifecycle::use_resize_epoch(environment.window.clone());
    let (character, pinyin, meaning) = if let Some((character, pinyin, meaning)) = &props.selected {
        (character.as_str(), pinyin.as_str(), meaning.as_str())
    } else {
        TARGETS[state.target]
    };

    {
        let state = state.clone();
        use_effect_with(
            (character.to_owned(), state.target, *retry_epoch),
            move |(character, target, _)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                state.dispatch(WritingAction::Select {
                    target: *target,
                    character: character.clone(),
                });
                state.dispatch(WritingAction::Loading {
                    character: character.clone(),
                });
                let character = character.clone();
                let state = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = super::stroke_loading::load(&character).await;
                    if pending.get() {
                        state.dispatch(WritingAction::Load { character, result });
                    }
                });
                move || alive.set(false)
            },
        );
    }
    super::lifecycle::use_replay_completion(state.clone());
    super::lifecycle::use_assist_completion(state.clone());
    let completed_character = state.character == character
        && matches!(&state.study, Some(Ok(study)) if study.session.is_complete());
    {
        let completion_change = props.on_completion_change.clone();
        use_effect_with(
            (character.to_owned(), completed_character),
            move |(_, complete)| {
                if let Some(callback) = &completion_change {
                    callback.emit(*complete);
                }
            },
        );
    }
    {
        let state = state.clone();
        use_effect_with(practice_options.assistance, move |assistance| {
            state.dispatch(WritingAction::ConfigureAssistance(*assistance));
        });
    }
    let on_retry = {
        let retry_epoch = retry_epoch.clone();
        Callback::from(move |_: MouseEvent| retry_epoch.set((*retry_epoch).wrapping_add(1)))
    };

    let visible_study = if state.character == character {
        state.study.clone()
    } else {
        None
    };
    let study = &visible_study;
    let visible_feedback = if state.character == character {
        state.feedback
    } else {
        None
    };
    let enabled = state.character == character && state.ready();
    let progress = if state.assist_epoch.is_some() {
        "Showing this stroke before continuing automatically.".to_owned()
    } else if state.replay_epoch.is_some() {
        "Watching stroke order. Your practice progress is unchanged.".to_owned()
    } else {
        match study {
            None => "Loading character stroke data...".to_owned(),
            Some(Err(error)) => error.clone(),
            Some(Ok(current)) if current.session.is_complete() => format!(
                "Character complete. {} handwritten, {} assisted, {} mistakes.",
                current.session.completed() - current.session.assisted_strokes(),
                current.session.assisted_strokes(),
                current.session.mistakes()
            ),
            Some(Ok(current)) => {
                let next = current.session.completed() + 1;
                match current.session.last_rejection() {
                    Some(reason) => format!(
                        "Stroke {next} of {}. {}",
                        current.session.total(),
                        reason.message()
                    ),
                    None => format!(
                        "Trace stroke {next} of {} in the highlighted direction.",
                        current.session.total()
                    ),
                }
            }
        }
    };

    {
        let canvas = canvas.clone();
        let ink = state.ink.clone();
        let device_pixel_ratio = environment.window.device_pixel_ratio();
        use_effect_with((ink, resize_epoch), move |(ink, _)| {
            if let Some(canvas) = canvas.cast::<HtmlCanvasElement>() {
                let bounds = canvas.get_bounding_client_rect();
                let dpr = device_pixel_ratio;
                let side = (bounds.width().max(1.0) * dpr).round() as u32;
                let side = side.clamp(1, 2048);
                if canvas.width() != side || canvas.height() != side {
                    canvas.set_width(side);
                    canvas.set_height(side);
                }
                paint(&canvas, ink);
            }
        });
    }
    let pointer =
        super::canvas_input::canvas_pointer_handlers(canvas.clone(), state.clone(), enabled);
    let on_workspace_action = {
        let state = state.clone();
        Callback::from(move |action: WorkspaceAction| match action {
            WorkspaceAction::Replay => state.dispatch(WritingAction::Replay),
            WorkspaceAction::Undo => state.dispatch(WritingAction::Undo),
            WorkspaceAction::Clear => state.dispatch(WritingAction::Clear),
            WorkspaceAction::ToggleGuide => state.dispatch(WritingAction::ToggleGuide),
            WorkspaceAction::NextCharacter => {
                let target = (state.target + 1) % TARGETS.len();
                state.dispatch(WritingAction::Select {
                    target,
                    character: TARGETS[target].0.to_owned(),
                });
            }
        })
    };
    let on_replay = action_callback(&on_workspace_action, WorkspaceAction::Replay);
    let on_undo = action_callback(&on_workspace_action, WorkspaceAction::Undo);
    let on_clear = action_callback(&on_workspace_action, WorkspaceAction::Clear);
    let on_toggle_guide = action_callback(&on_workspace_action, WorkspaceAction::ToggleGuide);
    let on_next_character = action_callback(&on_workspace_action, WorkspaceAction::NextCharacter);
    let on_target = {
        let state = state.clone();
        Callback::from(move |index: usize| {
            if let Some((hanzi, _, _)) = TARGETS.get(index) {
                state.dispatch(WritingAction::Select {
                    target: index,
                    character: (*hanzi).to_owned(),
                });
            }
        })
    };
    let stroke_reference_content = if props.selected.is_none() {
        html! { <HanziStrokeReference character={character.to_owned()} /> }
    } else {
        Html::default()
    };
    let writing_practice_content = if props.selected.is_none() {
        html! { <HanziTargetPicker selected={state.target} on_target={on_target.clone()} /> }
    } else {
        Html::default()
    };
    let writing_page_header = if props.selected.is_none() {
        super::standalone::standalone_header()
    } else {
        Html::default()
    };
    let empty_character_workspace = if props.selected.is_none() {
        super::standalone::target_summary(character, pinyin, meaning)
    } else {
        Html::default()
    };
    let on_completion_clicked = props
        .on_completed
        .clone()
        .map(|callback| Callback::from(move |_: MouseEvent| callback.emit(())));
    html! {
        <main
            data-component="hanzi-writing-page"
            data-hanzi-feedback={visible_feedback.map(|feedback| feedback.kind()).unwrap_or("idle")}
            data-hanzi-strokes-assisted={match study { Some(Ok(current)) => current.session.assisted_strokes().to_string(), _ => "0".to_owned() }}
            data-hanzi-strokes-completed={match study { Some(Ok(current)) => current.session.completed().to_string(), _ => "0".to_owned() }}
            class={if props.selected.is_some() {"flex w-full flex-col gap-2"} else {"mx-auto flex w-full max-w-5xl flex-col gap-6 p-4 sm:p-6"}}
        >
            {writing_page_header}
            <div
                class={if props.selected.is_some() {"grid items-start gap-6"} else {"grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(240px,300px)]"}}
            >
                <section
                    class="write-hanzi-board rounded-2xl border border-line bg-surface p-3 space-y-2"
                    aria-label="Hanzi drawing board"
                >
                    {empty_character_workspace}
                    <WritingBoard
                        canvas={canvas.clone()}
                        character={character.to_owned()}
                        study={(*study).clone()}
                        {enabled}
                        guidance={if is_practice { practice_options.guidance } else if state.guide { WritingGuidance::FullGuide } else { WritingGuidance::None }}
                        drawing={state.ink.is_drawing()}
                        rejection_count={state.rejection_count}
                        replay_epoch={state.replay_epoch}
                        assist_epoch={state.assist_epoch}
                        assist_index={state.assist_index}
                        pointer_down={pointer.down}
                        pointer_move={pointer.move_pointer}
                        pointer_up={pointer.up}
                        pointer_cancel={pointer.cancel}
                    />
                    <WritingControls
                        replaying={state.replay_epoch.is_some() || state.assist_epoch.is_some()}
                        study={(*study).clone()}
                        drawing={state.ink.is_drawing()}
                        guide={state.guide}
                        show_next={props.selected.is_none()}
                        {on_replay} {on_undo} {on_clear} {on_toggle_guide} {on_next_character}
                        allow_animation={!is_practice || practice_options.animation}
                        allow_guide_toggle={!is_practice}
                    />
                    if props.selected.is_some() {
                        <div class="flex min-h-[6.5rem] flex-col items-center justify-center gap-2" data-hanzi-completion-slot="">
                            if matches!(study, Some(Ok(current)) if current.session.is_complete()) {
                                if let Some(Ok(current)) = study {
                                    <p data-hanzi-completion-summary="" class="text-center text-sm text-muted">
                                        {format!("{} handwritten, {} assisted strokes, {} mistakes",
                                            current.session.completed() - current.session.assisted_strokes(),
                                            current.session.assisted_strokes(), current.session.mistakes())}
                                    </p>
                                }
                                if let Some(on_completed) = &on_completion_clicked {
                                    <button type="button" data-action="complete-hanzi-exercise"
                                        onclick={on_completed.clone()}
                                        class="min-h-11 rounded-xl bg-brand-action px-6 py-3 font-semibold text-white">
                                        {"Continue"}
                                    </button>
                                }
                            }
                        </div>
                    }
                    if matches!(study, Some(Err(_))) {
                        <button type="button" data-action="retry-hanzi-reference"
                            onclick={on_retry}
                            class="rounded-lg border border-line bg-raised px-4 py-2 text-sm font-semibold">
                            {"Retry loading reference"}
                        </button>
                    }
                    if !is_practice {
                        <StrokeProgress study={(*study).clone()} replaying={state.replay_epoch.is_some() || state.assist_epoch.is_some()} message={progress.clone()} />
                    } else {
                        <div class="min-h-5" data-hanzi-training-status-slot="">
                            if let Some(Err(error)) = study {
                                <p role="alert" data-hanzi-training-status="error" class="text-sm text-muted">{error.clone()}</p>
                            } else if matches!(visible_feedback, Some(AttemptFeedback::Rejected(_) | AttemptFeedback::Assisting | AttemptFeedback::Assisted)) {
                                <p role="status" aria-live="polite" data-hanzi-training-status="rejected" class="text-sm text-muted">{visible_feedback.map(AttemptFeedback::label).unwrap_or_default()}</p>
                            } else if let Some(feedback) = visible_feedback {
                                <p role="status" aria-live="polite" data-hanzi-training-status={feedback.kind()} class="sr-only">{feedback.label()}</p>
                            }
                        </div>
                    }
                </section>
                {stroke_reference_content}
                {writing_practice_content}
            </div>
        </main>
    }
}

#[derive(Clone, Copy)]
enum WorkspaceAction {
    Replay,
    Undo,
    Clear,
    ToggleGuide,
    NextCharacter,
}

fn action_callback(
    dispatch: &Callback<WorkspaceAction>,
    action: WorkspaceAction,
) -> Callback<MouseEvent> {
    let dispatch = dispatch.clone();
    Callback::from(move |_: MouseEvent| dispatch.emit(action))
}
