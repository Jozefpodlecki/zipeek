mod answer_choices;
mod generation;
mod image_review;
mod source;
mod state;

use std::{cell::Cell, collections::HashMap, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::{Link, use_location};
use zipeek_exercises::image_questions::ImageQuestionMode;

use crate::{
    hooks::{use_app_environment, use_dictionary},
    route::Route,
    services::image_generation,
};
use source::ImageSource;
use state::{Action, ImageVocabularyState};

#[function_component(ImageVocabularyPage)]
pub fn image_vocabulary_page() -> Html {
    let dictionary = use_dictionary();
    let environment = use_app_environment();
    let image_server = image_generation::read_api_url(&environment.local_storage);
    let params = use_location()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();
    let source = ImageSource::from_query(&params);
    let retry_number = use_state(|| 0u32);
    let session = use_reducer(ImageVocabularyState::default);
    let active = use_mut_ref(|| 0u64);

    {
        let dictionary = dictionary.clone();
        let image_server = image_server.clone();
        let session = session.clone();
        let active = active.clone();
        use_effect_with(
            (source, session.mode, *retry_number),
            move |(source, mode, _)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                let ticket = {
                    let mut current = active.borrow_mut();
                    *current = current.wrapping_add(1);
                    *current
                };
                session.dispatch(Action::Start);
                let source = source.clone();
                let mode = *mode;
                let generation_active = active.clone();
                let generation_session = session.clone();
                spawn_local(async move {
                    let result = async {
                        let words = source.load(&dictionary).await?;
                        if !pending.get() || *generation_active.borrow() != ticket {
                            return Err("Generation superseded".to_owned());
                        }
                        generation::generate(&words, mode, &image_server, |name, received| {
                            if pending.get() && *generation_active.borrow() == ticket {
                                generation_session.dispatch(Action::Progress(if received == 0 {
                                    name.to_owned()
                                } else {
                                    format!("{name} · {received} characters received")
                                }));
                            }
                        })
                        .await
                    }
                    .await;
                    if !pending.get() || *generation_active.borrow() != ticket {
                        return;
                    }
                    match result {
                        Ok(generated) => session.dispatch(Action::Generated(generated)),
                        Err(message) => session.dispatch(Action::Failed(message)),
                    }
                });
                move || alive.set(false)
            },
        );
    }

    let change_mode = {
        let session = session.clone();
        let active = active.clone();
        Callback::from(move |next: ImageQuestionMode| {
            if session.mode != next {
                let mut ticket = active.borrow_mut();
                *ticket = ticket.wrapping_add(1);
                session.dispatch(Action::ChooseMode(next));
            }
        })
    };
    let regenerate = {
        let retry_number = retry_number.clone();
        Callback::from(move |_| retry_number.set(retry_number.wrapping_add(1)))
    };
    let approve = {
        let session = session.clone();
        Callback::from(move |_| session.dispatch(Action::Approve))
    };
    let on_image_load = {
        let session = session.clone();
        Callback::from(move |_: Event| session.dispatch(Action::ImageLoaded))
    };
    let on_image_error = {
        let session = session.clone();
        Callback::from(move |_: Event| session.dispatch(Action::ImageFailed))
    };
    let toggle = {
        let session = session.clone();
        Callback::from(move |id: i32| session.dispatch(Action::Toggle(id)))
    };
    let check = {
        let session = session.clone();
        Callback::from(move |_| session.dispatch(Action::Check))
    };
    html! {
        <section
            data-image-vocabulary=""
            class="mx-auto flex w-full max-w-4xl flex-col gap-5 rounded-2xl border border-line bg-surface p-5"
        >
            <header class="space-y-2">
                <h1 class="text-2xl font-semibold">{"Picture vocabulary"}</h1>
                <p class="text-sm text-muted">
                    {"A scene is composed from your selected vocabulary, then illustrated. Review the image before answering."}
                </p>
            </header>
            <ModeSelector mode={session.mode} on_change={change_mode} />
            <GenerationStatus busy={session.busy} phase={session.phase.clone()} error={session.error.clone()} />
            <ImageScene
                session={(*session).clone()}
                on_approve={approve}
                on_image_load={on_image_load}
                on_image_error={on_image_error}
                on_toggle={toggle}
                on_check={check}
            />
            <PageActions busy={session.busy} on_regenerate={regenerate} />
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct ModeSelectorProps {
    mode: ImageQuestionMode,
    on_change: Callback<ImageQuestionMode>,
}

#[function_component(ModeSelector)]
fn mode_selector(props: &ModeSelectorProps) -> Html {
    html! {
        <fieldset class="flex flex-wrap items-center gap-3 rounded-xl border border-line p-3">
            <legend class="text-sm font-semibold">{"Answer mode"}</legend>
            <ModeButton
                mode={ImageQuestionMode::OneOfThree}
                current={props.mode}
                label="One of three"
                on_select={props.on_change.clone()}
            />
            <ModeButton
                mode={ImageQuestionMode::SelectAll}
                current={props.mode}
                label="Select all"
                on_select={props.on_change.clone()}
            />
        </fieldset>
    }
}

#[derive(Properties, PartialEq)]
struct ModeButtonProps {
    mode: ImageQuestionMode,
    current: ImageQuestionMode,
    label: &'static str,
    on_select: Callback<ImageQuestionMode>,
}

#[function_component(ModeButton)]
fn mode_button(props: &ModeButtonProps) -> Html {
    let selected = props.mode == props.current;
    let mode = props.mode;
    let on_select = props.on_select.clone();
    let on_click = Callback::from(move |_: MouseEvent| on_select.emit(mode));
    let action = match mode {
        ImageQuestionMode::OneOfThree => "one-of-three",
        ImageQuestionMode::SelectAll => "select-all",
    };
    let element_classes_1 = classes!(
        "rounded-lg",
        "px-3",
        "py-2",
        "focus-visible:outline-2",
        "focus-visible:outline-offset-2",
        if selected {
            "bg-brand-action text-white"
        } else {
            "border border-line"
        }
    );
    html! {
        <button
            type="button"
            data-action={action}
            aria-pressed={selected.to_string()}
            onclick={on_click}
            class={element_classes_1}
        >
            {props.label}
        </button>
    }
}

#[derive(Properties, PartialEq)]
struct GenerationStatusProps {
    busy: bool,
    phase: String,
    error: String,
}

#[function_component(GenerationStatus)]
fn generation_status(props: &GenerationStatusProps) -> Html {
    html! {
        <>
            if props.busy {
                <div
                    role="status"
                    data-image-generation-progress=""
                    class="rounded-xl border border-line bg-raised p-5"
                >
                    <p class="font-semibold">{"Preparing your picture question"}</p>
                    <p class="mt-2 text-sm text-muted">{&props.phase}</p>
                    <p class="mt-2 text-xs text-muted">
                        {"AI and image generation can take some time. This page updates as work progresses."}
                    </p>
                </div>
            }
            if !props.error.is_empty() {
                <p role="alert" class="rounded-xl border border-red-700 p-4 text-red-400">
                    {&props.error}
                </p>
            }
        </>
    }
}

#[derive(Properties, PartialEq)]
struct ImageSceneProps {
    session: ImageVocabularyState,
    on_approve: Callback<MouseEvent>,
    on_image_load: Callback<Event>,
    on_image_error: Callback<Event>,
    on_toggle: Callback<i32>,
    on_check: Callback<MouseEvent>,
}

#[function_component(ImageScene)]
fn image_scene(props: &ImageSceneProps) -> Html {
    let Some(url) = &props.session.image_url else {
        return Html::default();
    };
    html! {
        <div class="space-y-4">
            <img
                key={url.clone()}
                src={url.clone()}
                alt="AI-generated visual vocabulary scene"
                class="mx-auto max-h-[34rem] w-full rounded-xl object-contain"
                onload={props.on_image_load.clone()}
                onerror={props.on_image_error.clone()}
            />
            if props.session.approved {
                <ImageQuestionPanel
                    session={props.session.clone()}
                    on_toggle={props.on_toggle.clone()}
                    on_check={props.on_check.clone()}
                />
            } else {
                <image_review::ImageReview
                    prompt={props.session.scene_sentence.clone()}
                    image_loaded={props.session.image_loaded}
                    image_failed={props.session.image_failed}
                    on_approve={props.on_approve.clone()}
                />
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ImageQuestionPanelProps {
    session: ImageVocabularyState,
    on_toggle: Callback<i32>,
    on_check: Callback<MouseEvent>,
}

#[function_component(ImageQuestionPanel)]
fn image_question_panel(props: &ImageQuestionPanelProps) -> Html {
    let session = &props.session;
    let title = match session.mode {
        ImageQuestionMode::OneOfThree => "Which Hanzi matches the scene?",
        ImageQuestionMode::SelectAll => "Select every Hanzi shown in the scene",
    };
    let verdict = if session.checked {
        session
            .question
            .as_ref()
            .and_then(|question| question.grade(&session.selected).ok())
    } else {
        None
    };
    html! {
        <div class="space-y-4">
            <h2 class="text-lg font-semibold">{title}</h2>
            if let Some(question) = &session.question {
                <answer_choices::AnswerChoices
                    question={question.clone()}
                    selected={session.selected.clone()}
                    checked={session.checked}
                    on_toggle={props.on_toggle.clone()}
                />
            }
            if let Some(correct) = verdict {
                <p role="status" class={if correct {"text-green-500"} else {"text-red-400"}}>
                    {if correct {"Correct!"} else {"Not quite. Review the correct answers."}}
                </p>
            }
            <button
                type="button"
                data-image-check-answer="true"
                disabled={session.checked || session.selected.is_empty()}
                onclick={props.on_check.clone()}
                class="rounded-lg bg-brand-action px-4 py-2 font-semibold text-white disabled:opacity-50"
            >
                {"Check answer"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct PageActionsProps {
    busy: bool,
    on_regenerate: Callback<MouseEvent>,
}

#[function_component(PageActions)]
fn page_actions(props: &PageActionsProps) -> Html {
    html! {
        <div class="flex flex-wrap items-center gap-3">
            <button
                type="button"
                data-image-regenerate="true"
                disabled={props.busy}
                onclick={props.on_regenerate.clone()}
                class="rounded-lg bg-brand-action px-4 py-2 font-semibold text-white disabled:opacity-50"
            >
                {"Generate another scene"}
            </button>
            <Link<Route> to={Route::Settings} classes="rounded-lg border border-line px-4 py-2">
                {"AI and image server settings"}
            </Link<Route>>
        </div>
    }
}
