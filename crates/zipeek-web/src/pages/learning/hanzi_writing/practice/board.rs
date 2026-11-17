use crate::models::WritingGuidance;

use super::{
    direction_guide::{DirectionGuide, replay_median_path},
    stroke_loading::StrokeStudy,
};
use web_sys::PointerEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct WritingBoardProps {
    pub canvas: NodeRef,
    pub character: String,
    pub study: Option<Result<StrokeStudy, String>>,
    pub enabled: bool,
    pub guidance: WritingGuidance,
    pub drawing: bool,
    pub rejection_count: usize,
    pub replay_epoch: Option<u64>,
    pub assist_epoch: Option<u64>,
    pub assist_index: Option<usize>,
    pub pointer_down: Callback<PointerEvent>,
    pub pointer_move: Callback<PointerEvent>,
    pub pointer_up: Callback<PointerEvent>,
    pub pointer_cancel: Callback<PointerEvent>,
}

#[derive(Properties, PartialEq)]
struct CanonicalStrokeOverlayProps {
    study: StrokeStudy,
    guidance: WritingGuidance,
    drawing: bool,
    rejection_count: usize,
    replay_epoch: Option<u64>,
    assist_epoch: Option<u64>,
    assist_index: Option<usize>,
}

#[function_component(CanonicalStrokeOverlay)]
fn canonical_stroke_overlay(props: &CanonicalStrokeOverlayProps) -> Html {
    let completed = props.study.session.completed();
    let replaying = props.replay_epoch.is_some();
    let assisting = props.assist_epoch.is_some();
    html! {
        <svg
            class="pointer-events-none absolute inset-0 h-full w-full"
            viewBox="0 0 1024 1024"
            aria-hidden="true"
        >
            if let Some(epoch) = props.replay_epoch {
                <g key={format!("replay-{epoch}")} data-hanzi-replay="" transform="scale(1 -1) translate(0 -900)">
                    {for props.study.paths.iter().enumerate().map(|(index, outline)| {
                        let id = format!("hanzi-replay-{epoch}-{index}");
                        let path = props.study.session.reference_strokes().get(index).and_then(replay_median_path);
                        let delay = index * 850;
                        html! {
                            <g key={index}>
                                if let Some(path) = path {
                                    <mask id={id.clone()} maskUnits="userSpaceOnUse" maskContentUnits="userSpaceOnUse" style="mask-type: alpha" x="0" y="-124" width="1024" height="1024">
                                        <path d={path} stroke="white" stroke-width="200" fill="none"
                                            stroke-linecap="round" stroke-linejoin="round"
                                            pathLength="100" stroke-dasharray="100" stroke-dashoffset="100"
                                            class="animate-[hanzi-replay-reveal_750ms_linear_both] motion-reduce:[animation-duration:1ms]" style={format!("animation-delay: {delay}ms")}/>
                                    </mask>
                                    <path d={outline.clone()} fill="#79b7ff" mask={format!("url(#{id})")} data-replay-outline={index.to_string()}/>
                                }
                            </g>
                        }
                    })}
                </g>
            }
            if let (Some(epoch), Some(index)) = (props.assist_epoch, props.assist_index) {
                if let (Some(outline), Some(path)) = (
                    props.study.paths.get(index),
                    props.study.session.reference_strokes().get(index).and_then(replay_median_path),
                ) {
                    <g key={format!("assist-{epoch}")} data-hanzi-assist="" transform="scale(1 -1) translate(0 -900)">
                        <mask id={format!("hanzi-assist-{epoch}")} maskUnits="userSpaceOnUse" maskContentUnits="userSpaceOnUse" style="mask-type: alpha" x="0" y="-124" width="1024" height="1024">
                            <path d={path} stroke="white" stroke-width="200" fill="none"
                                stroke-linecap="round" stroke-linejoin="round" pathLength="100"
                                stroke-dasharray="100" stroke-dashoffset="100"
                                class="animate-[hanzi-replay-reveal_750ms_linear_both] motion-reduce:[animation-duration:1ms]" />
                        </mask>
                        <path d={outline.clone()} fill="#79b7ff" mask={format!("url(#hanzi-assist-{epoch})")}/>
                    </g>
                }
            }
            <g transform="scale(1 -1) translate(0 -900)">
                {for props.study.paths.iter().enumerate().map(|(index, path)| {
                    let done = index < completed;
                    let expected = index == completed;
                    let fill = if done { "#79b7ff" } else if expected { "#fbbf24" } else { "#53637b" };
                    let opacity = if replaying {
                        "0"
                    } else if done {
                        "1"
                    } else if !assisting && props.guidance == WritingGuidance::FullGuide && expected {
                        "0.5"
                    } else if props.guidance == WritingGuidance::FullGuide {
                        "0.14"
                    } else {
                        "0"
                    };
                    html! { <path key={index} d={path.clone()} fill={fill} opacity={opacity}/> }
                })}
            </g>
            if props.guidance != WritingGuidance::None && !props.drawing && !replaying && !assisting {
                if let Some(expected) = props.study.session.expected_stroke() {
                    <DirectionGuide
                        stroke={expected.clone()}
                        stroke_index={completed}
                        rejection_count={props.rejection_count}
                    />
                }
            }
        </svg>
    }
}

#[function_component(WritingBoard)]
pub(super) fn writing_board(props: &WritingBoardProps) -> Html {
    let assessment = if props.enabled {
        "ready"
    } else if props.study.is_none() {
        "loading"
    } else if matches!(&props.study, Some(Err(_))) {
        "unavailable"
    } else {
        "complete"
    };

    html! {
        <div
            class="relative mx-auto w-full max-w-md"
            data-hanzi-guidance={match props.guidance {
                WritingGuidance::None => "none",
                WritingGuidance::StartCue => "start",
                WritingGuidance::FullGuide => "full",
            }}
        >
            <canvas
                ref={props.canvas.clone()}
                width="640"
                height="640"
                aria-label={format!("Writing canvas for {}", props.character)}
                data-hanzi-canvas=""
                data-hanzi-character={props.character.clone()}
                data-hanzi-assessment={assessment}
                class="block aspect-square w-full rounded-xl border border-line cursor-crosshair touch-none writing-desktop:w-[min(100%,47dvh,440px)]"
                onpointerdown={props.pointer_down.clone()}
                onpointermove={props.pointer_move.clone()}
                onpointerup={props.pointer_up.clone()}
                onpointercancel={props.pointer_cancel.clone()}
                onlostpointercapture={props.pointer_cancel.clone()}
            />
            if let Some(Ok(study)) = &props.study {
                <CanonicalStrokeOverlay
                    study={study.clone()}
                    guidance={props.guidance}
                    drawing={props.drawing}
                    rejection_count={props.rejection_count}
                    replay_epoch={props.replay_epoch}
                    assist_epoch={props.assist_epoch}
                    assist_index={props.assist_index}
                />
            }
        </div>
    }
}
