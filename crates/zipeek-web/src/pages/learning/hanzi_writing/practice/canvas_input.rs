use super::{
    super::canvas::normalized_pointer,
    controller::{WritingAction, WritingState},
};
use web_sys::{HtmlCanvasElement, PointerEvent};
use yew::prelude::*;

pub(super) struct CanvasPointerHandlers {
    pub down: Callback<PointerEvent>,
    pub move_pointer: Callback<PointerEvent>,
    pub up: Callback<PointerEvent>,
    pub cancel: Callback<PointerEvent>,
}

pub(super) fn canvas_pointer_handlers(
    canvas: NodeRef,
    state: UseReducerHandle<WritingState>,
    enabled: bool,
) -> CanvasPointerHandlers {
    let down = {
        let canvas = canvas.clone();
        let state = state.clone();
        Callback::from(move |event: PointerEvent| {
            if !enabled || !event.is_primary() || event.button() != 0 {
                return;
            }
            let Some(element) = canvas.cast::<HtmlCanvasElement>() else {
                return;
            };
            let Some(point) = normalized_pointer(&event, &element) else {
                return;
            };

            if element.set_pointer_capture(event.pointer_id()).is_err() {
                return;
            }
            event.prevent_default();
            state.dispatch(WritingAction::Begin {
                pointer: i64::from(event.pointer_id()),
                point,
            });
        })
    };
    let move_pointer = {
        let canvas = canvas.clone();
        let state = state.clone();
        Callback::from(move |event: PointerEvent| {
            let Some(element) = canvas.cast::<HtmlCanvasElement>() else {
                return;
            };
            if !element.has_pointer_capture(event.pointer_id()) {
                return;
            }
            let Some(point) = normalized_pointer(&event, &element) else {
                return;
            };
            event.prevent_default();
            state.dispatch(WritingAction::Move {
                pointer: i64::from(event.pointer_id()),
                point,
            });
        })
    };
    let up = {
        let canvas = canvas.clone();
        let state = state.clone();
        Callback::from(move |event: PointerEvent| {
            let Some(element) = canvas.cast::<HtmlCanvasElement>() else {
                return;
            };
            if !element.has_pointer_capture(event.pointer_id()) {
                return;
            }
            let point = normalized_pointer(&event, &element);
            event.prevent_default();
            state.dispatch(WritingAction::Finish {
                pointer: i64::from(event.pointer_id()),
                point,
            });
            if element.has_pointer_capture(event.pointer_id()) {
                let _ = element.release_pointer_capture(event.pointer_id());
            }
        })
    };
    let cancel = {
        let canvas = canvas.clone();
        let state = state.clone();
        Callback::from(move |event: PointerEvent| {
            state.dispatch(WritingAction::Cancel {
                pointer: i64::from(event.pointer_id()),
            });
            if let Some(element) = canvas.cast::<HtmlCanvasElement>() {
                if element.has_pointer_capture(event.pointer_id()) {
                    let _ = element.release_pointer_capture(event.pointer_id());
                }
            }
        })
    };
    CanvasPointerHandlers {
        down,
        move_pointer,
        up,
        cancel,
    }
}
