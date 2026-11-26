use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, KeyboardEvent};
use yew::prelude::*;

#[derive(Debug, PartialEq, Eq)]
enum WrapTarget {
    First,
    Last,
}

fn tab_wrap_target(shift: bool, at_first: bool, at_last: bool) -> Option<WrapTarget> {
    if shift && at_first {
        Some(WrapTarget::Last)
    } else if !shift && at_last {
        Some(WrapTarget::First)
    } else {
        None
    }
}

pub(super) fn trap_tab(event: &KeyboardEvent, first: &NodeRef, last: &NodeRef) {
    if event.key() != "Tab" {
        return;
    }
    let Some(first) = first.cast::<HtmlElement>() else {
        return;
    };
    let Some(last) = last.cast::<HtmlElement>() else {
        return;
    };
    let current = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element());
    let at_first = current
        .as_ref()
        .is_some_and(|element| JsValue::from(element.clone()) == JsValue::from(first.clone()));
    let at_last = current
        .as_ref()
        .is_some_and(|element| JsValue::from(element.clone()) == JsValue::from(last.clone()));
    match tab_wrap_target(event.shift_key(), at_first, at_last) {
        Some(WrapTarget::First) => {
            event.prevent_default();
            let _ = first.focus();
        }
        Some(WrapTarget::Last) => {
            event.prevent_default();
            let _ = last.focus();
        }
        None => {}
    }
}

#[hook]
pub(super) fn use_dialog_initial_focus(first: NodeRef) {
    use_effect_with((), move |_| {
        let previous = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .and_then(|element| element.dyn_into::<HtmlElement>().ok());
        if let Some(element) = first.cast::<HtmlElement>() {
            let _ = element.focus();
        }
        move || {
            if let Some(element) = previous {
                let _ = element.focus();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{WrapTarget, tab_wrap_target};

    #[test]
    fn forward_tab_wraps_from_last_element() {
        assert_eq!(tab_wrap_target(false, false, true), Some(WrapTarget::First));
        assert_eq!(tab_wrap_target(false, true, false), None);
    }

    #[test]
    fn reverse_tab_wraps_from_first_element() {
        assert_eq!(tab_wrap_target(true, true, false), Some(WrapTarget::Last));
        assert_eq!(tab_wrap_target(true, false, true), None);
    }
}
