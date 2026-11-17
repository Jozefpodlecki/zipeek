use std::{cell::RefCell, rc::Weak};
use wasm_bindgen::{JsCast, closure::Closure};

use super::{BAR_COUNT, RecordingPhase, SAMPLE_INTERVAL_MS, State};

pub(super) fn schedule_frame(weak: &Weak<RefCell<State>>, generation: u64) {
    let Some(inner) = weak.upgrade() else {
        return;
    };
    let next = weak.clone();
    let callback = Closure::<dyn FnMut(f64)>::new(move |time| {
        let Some(inner) = next.upgrade() else {
            return;
        };
        {
            let mut state = inner.borrow_mut();
            if state.disposed
                || state.phase != RecordingPhase::Recording
                || state.generation != generation
            {
                return;
            }
            if time - state.last_sample >= SAMPLE_INTERVAL_MS {
                if let Some(analyser) = state.analyser.as_ref() {
                    let mut samples = vec![0.0_f32; analyser.fft_size() as usize];
                    analyser.get_float_time_domain_data(&mut samples);
                    let rms =
                        (samples.iter().map(|x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
                    state.levels.rotate_left(1);
                    state.levels[BAR_COUNT - 1] = (rms * 6.0).min(1.0);
                    state.emit_levels();
                    state.last_sample = time;
                }
            }
        }
        let mut state = inner.borrow_mut();
        if let Some(callback) = state.on_frame.as_ref() {
            if let Ok(frame) = state
                .window
                .request_animation_frame(callback.as_ref().unchecked_ref())
            {
                state.frame = Some(frame);
            }
        }
    });
    let mut state = inner.borrow_mut();
    state.on_frame = Some(callback);
    if let Some(callback) = state.on_frame.as_ref() {
        if let Ok(frame) = state
            .window
            .request_animation_frame(callback.as_ref().unchecked_ref())
        {
            state.frame = Some(frame);
        }
    }
}
