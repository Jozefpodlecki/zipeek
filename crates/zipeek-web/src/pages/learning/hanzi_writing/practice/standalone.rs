use super::stroke_loading::StrokeStudy;
use super::targets::TARGETS;
use yew::prelude::*;
use yew_router::prelude::Link;
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub(super) struct StrokeProgressProps {
    pub replaying: bool,
    pub study: Option<Result<StrokeStudy, String>>,
    pub message: String,
}

#[function_component(StrokeProgress)]
pub(super) fn stroke_progress(props: &StrokeProgressProps) -> Html {
    let (completed, total, mistakes) = match &props.study {
        Some(Ok(study)) => (
            study.session.completed(),
            study.session.total(),
            study.session.mistakes(),
        ),
        _ => (0, 0, 0),
    };
    let visible_completed = if props.replaying { 0 } else { completed };
    let percentage = if total == 0 {
        0
    } else {
        visible_completed * 100 / total
    };
    html! {
        <div class="space-y-2" data-hanzi-training-progress="">
            if total > 0 {
                <div class="flex items-center justify-between gap-3 text-xs text-muted">
                    <span>{if props.replaying { format!("Watching {total} strokes") } else { format!("{completed} / {total} strokes") }}</span>
                    if !props.replaying { <span>{format!("{mistakes} mistakes")}</span> }
                </div>
                <div
                    role="progressbar"
                    aria-label="Accepted stroke progress"
                    aria-valuemin="0"
                    aria-valuemax={total.to_string()}
                    aria-valuenow={visible_completed.to_string()}
                    aria-disabled={props.replaying.to_string()}
                    class="h-2 overflow-hidden rounded-full bg-raised"
                >
                    <div
                        class="h-full rounded-full bg-blue-500 transition-[width] duration-150"
                        style={format!("width: {percentage}%")}
                    />
                </div>
            }
            <p
                role="status"
                aria-live="polite"
                data-hanzi-training-status=""
                class="text-xs text-muted"
            >
                {if props.replaying { "Watch the stroke order and direction." } else { &props.message }}
            </p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct HanziTargetPickerProps {
    pub selected: usize,
    pub on_target: Callback<usize>,
}

#[function_component(HanziTargetPicker)]
pub(super) fn hanzi_target_picker(props: &HanziTargetPickerProps) -> Html {
    let target = props.selected;
    let on_target = props.on_target.clone();
    html! {

            <aside
                class="rounded-2xl border border-line bg-surface p-5 space-y-4"
            >
            <h2
                class="text-lg font-semibold"
            >
                {"Choose a character"}
            </h2>
            <div
                class="grid grid-cols-4 gap-2"
            >
                {for TARGETS.iter().enumerate().map(|(index, (hanzi, _, _))| {
                    let on_target = on_target.clone();
                    let on_select_hanzi = Callback::from(move |_| on_target.emit(index));
                    let element_classes_1 = classes!(
                                    "aspect-square",
                                    "rounded-lg",
                                    "border",
                                    "text-2xl",
                                    "font-semibold",
                                    if target == index {"border-blue-500 bg-blue-600/20 text-ink"} else {"border-line bg-raised text-muted hover:text-ink"},
                                );
                    html! {
                        <button
                            type="button"
                            key={*hanzi}
                            data-action="select-hanzi"
                            data-hanzi-target={index.to_string()}
                            aria-pressed={(target == index).to_string()}
                            onclick={on_select_hanzi}
                            class={element_classes_1}
                        >
                            {hanzi}
                        </button>}
                })}
            </div>
            <div
                class="rounded-xl border border-line bg-raised p-4 text-sm text-muted space-y-2"
            >
                <p
                    class="font-semibold text-ink"
                >
                    {"Practice mode"}
                </p>
                <p>
                    {"Trace the highlighted stroke from its starting point. Accepted strokes snap to the reference shape."}
                </p>
                <p>
                    {"Stroke order, direction, shape and position are checked. Missing reference data prevents grading."}
                </p>
            </div>
        </aside>
    }
}


pub(super) fn standalone_header() -> Html {
    html! {
        <header class="space-y-2">
            <div class="flex items-center gap-3">
                <h1 class="text-3xl font-bold">{"Hanzi Writing"}</h1>
                <span class="rounded-full border border-line px-3 py-1 text-xs text-muted">{"Guided stroke practice"}</span>
            </div>
            <p class="text-muted">{"Follow each stroke in order. Accurate strokes snap to the reference; misplaced strokes are rejected."}</p>
            <Link<Route> to={Route::PracticeSetup} classes="inline-flex text-sm font-semibold text-accent hover:underline">{"Back to Practice"}</Link<Route>>
        </header>
    }
}

pub(super) fn target_summary(character: &str, pinyin: &str, meaning: &str) -> Html {
    html! {
        <div class="flex items-center justify-between gap-3">
            <div>
                <p class="text-xs font-semibold uppercase tracking-wider text-muted">{"Draw this character"}</p>
                <p lang="zh-Hans" class="text-4xl font-semibold">{character.to_owned()}</p>
            </div>
            <div class="text-right">
                <p class="font-medium text-accent">{pinyin.to_owned()}</p>
                <p class="text-sm text-muted">{meaning.to_owned()}</p>
            </div>
        </div>
    }
}
