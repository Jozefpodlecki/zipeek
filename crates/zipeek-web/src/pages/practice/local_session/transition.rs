use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use animato::{Easing, Tween, Update};
use gloo::timers::callback::Interval;
use js_sys::{Date, Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

use crate::hooks::use_app_environment;

#[derive(Properties, PartialEq)]
pub(super) struct ExerciseTransitionProps {
    pub step_id: String,
    pub children: Children,
}

fn prefers_reduced_motion(window: &web_sys::Window) -> bool {
    let Ok(method) = Reflect::get(window, &JsValue::from_str("matchMedia")) else {
        return false;
    };
    let Ok(method) = method.dyn_into::<Function>() else {
        return false;
    };
    let Ok(media) = method.call1(
        window,
        &JsValue::from_str("(prefers-reduced-motion: reduce)"),
    ) else {
        return false;
    };
    Reflect::get(&media, &JsValue::from_str("matches"))
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

#[function_component(ExerciseTransition)]
pub(super) fn exercise_transition(props: &ExerciseTransitionProps) -> Html {
    let environment = use_app_environment();
    let region = use_node_ref();
    let displayed = use_state(|| (props.step_id.clone(), props.children.clone()));
    let requested = props.step_id.clone();
    let previously_rendered = use_mut_ref(|| props.step_id.clone());
    {
        let region = region.clone();
        let previously_rendered = previously_rendered.clone();
        use_effect_with(displayed.0.clone(), move |rendered_step| {
            let changed = *previously_rendered.borrow() != *rendered_step;
            *previously_rendered.borrow_mut() = rendered_step.clone();
            if changed {
                if let Some(element) = region.cast::<web_sys::HtmlElement>() {
                    let _ = element.focus();
                }
            }
        });
    }
    {
        let displayed = displayed.clone();
        let region = region.clone();
        let next_content = props.children.clone();
        let window = environment.window.clone();
        use_effect_with(requested, move |next_step| {
            let next_step = next_step.clone();
            if displayed.0 == next_step {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            }
            if prefers_reduced_motion(&window) {
                displayed.set((next_step, next_content));
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            }
            let mut tween = Tween::new(0.0_f32, 1.0)
                .duration(0.32)
                .easing(Easing::EaseOutCubic)
                .build();
            let swapped = Rc::new(Cell::new(false));
            let last_frame = Rc::new(Cell::new(Date::now()));
            let timer = Rc::new(RefCell::new(None::<Interval>));
            let timer_for_tick = timer.clone();
            let tick_region = region.clone();
            let interval = Interval::new(16, move || {
                let now = Date::now();
                let elapsed = ((now - last_frame.replace(now)) / 1000.0).clamp(0.0, 0.1);
                tween.update(elapsed as f32);
                let progress = tween.value();
                if progress >= 0.5 && !swapped.replace(true) {
                    displayed.set((next_step.clone(), next_content.clone()));
                }
                if let Some(element) = tick_region.cast::<web_sys::Element>() {
                    let opacity = if progress < 0.5 {
                        1.0 - progress * 2.0
                    } else {
                        (progress - 0.5) * 2.0
                    };
                    let offset = if progress < 0.5 {
                        -8.0 * progress * 2.0
                    } else {
                        8.0 * (1.0 - progress) * 2.0
                    };
                    let _ = element.set_attribute(
                        "style",
                        &format!("opacity: {opacity:.3}; transform: translateY({offset:.2}px);"),
                    );
                }
                if tween.is_complete() {
                    if let Some(element) = tick_region.cast::<web_sys::Element>() {
                        let _ = element.remove_attribute("style");
                    }
                    timer_for_tick.borrow_mut().take();
                }
            });
            *timer.borrow_mut() = Some(interval);
            Box::new(move || {
                timer.borrow_mut().take();
                if let Some(element) = region.cast::<web_sys::Element>() {
                    let _ = element.remove_attribute("style");
                }
            }) as Box<dyn FnOnce()>
        });
    }
    let transitioning = displayed.0 != props.step_id;
    let content = if !transitioning {
        props.children.clone()
    } else {
        displayed.1.clone()
    };
    html! {
        <div ref={region} tabindex="-1" role="region" aria-label="Exercise content" data-practice-exercise-transition="" class={classes!("w-full", "motion-reduce:transform-none", transitioning.then_some("pointer-events-none"))}>
            {for content.iter()}
        </div>
    }
}
