use yew::prelude::*;

use super::controller::{WritingAction, WritingState};

#[hook]
pub(super) fn use_resize_epoch(window: web_sys::Window) -> u64 {
    let epoch = use_state(|| 0u64);
    let counter = use_mut_ref(|| 0u64);
    {
        let epoch = epoch.clone();
        let counter = counter.clone();
        use_effect_with((), move |_| {
            let listener = gloo::events::EventListener::new(&window, "resize", move |_| {
                let next = {
                    let mut value = counter.borrow_mut();
                    *value = value.wrapping_add(1);
                    *value
                };
                epoch.set(next);
            });
            move || drop(listener)
        });
    }
    *epoch
}

#[hook]
pub(super) fn use_replay_completion(state: UseReducerHandle<WritingState>) {
    let replay_epoch = state.replay_epoch;
    let stroke_count = match &state.study {
        Some(Ok(study)) => study.session.total(),
        _ => 0,
    };
    use_effect_with((replay_epoch, stroke_count), move |(epoch, count)| {
        let timeout = epoch.map(|epoch| {
            let state = state.clone();
            let duration = count.saturating_mul(850).saturating_add(100);
            gloo::timers::callback::Timeout::new(
                duration.min(u32::MAX as usize) as u32,
                move || state.dispatch(WritingAction::ReplayFinished { epoch }),
            )
        });
        move || drop(timeout)
    });
}

#[hook]
pub(super) fn use_assist_completion(state: UseReducerHandle<WritingState>) {
    let epoch = state.assist_epoch;
    use_effect_with(epoch, move |epoch| {
        let timeout = epoch.map(|epoch| {
            let state = state.clone();
            gloo::timers::callback::Timeout::new(850, move || {
                state.dispatch(WritingAction::AssistFinished { epoch });
            })
        });
        move || drop(timeout)
    });
}
