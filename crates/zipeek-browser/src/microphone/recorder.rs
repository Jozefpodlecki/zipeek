use js_sys::{Array, Function};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use web_sys::Window;

use super::{BAR_COUNT, RecordingPhase, State, describe_capture_error, start_capture};

#[derive(Clone)]
pub struct MicrophoneRecorder(Rc<RefCell<State>>);

impl MicrophoneRecorder {
    pub(super) fn from_shared(state: Rc<RefCell<State>>) -> Self {
        Self(state)
    }

    pub fn new(
        window: Window,
        levels: &Function,
        status: &Function,
        finished: &Function,
        error: &Function,
    ) -> Self {
        Self(Rc::new(RefCell::new(State {
            window,
            levels_callback: levels.clone(),
            status_callback: status.clone(),
            finished_callback: finished.clone(),
            error_callback: error.clone(),
            phase: RecordingPhase::Idle,
            generation: 0,
            disposed: false,
            stream: None,
            context: None,
            analyser: None,
            recorder: None,
            chunks: Array::new(),
            object_url: None,
            frame: None,
            deadline: None,
            last_sample: 0.0,
            levels: vec![0.0; BAR_COUNT],
            on_data: None,
            on_stop: None,
            on_error: None,
            on_frame: None,
            on_deadline: None,
        })))
    }

    pub fn start(&self) {
        let generation = {
            let mut state = self.0.borrow_mut();
            if state.disposed
                || matches!(
                    state.phase,
                    RecordingPhase::Requesting
                        | RecordingPhase::Recording
                        | RecordingPhase::Processing
                )
            {
                return;
            }
            state.generation += 1;
            state.discard_audio();
            state.levels.fill(0.0);
            state.emit_levels();
            state.status(RecordingPhase::Requesting);
            state.generation
        };
        let weak = Rc::downgrade(&self.0);
        spawn_local(async move {
            if let Err(error) = start_capture(weak.clone(), generation).await {
                if let Some(inner) = weak.upgrade() {
                    let mut state = inner.borrow_mut();
                    if !state.disposed && state.generation == generation {
                        state.release_capture();
                        let message = describe_capture_error(&error);
                        state.fail(&message);
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        let mut state = self.0.borrow_mut();
        if state.disposed || state.phase != RecordingPhase::Recording {
            return;
        }
        state.status(RecordingPhase::Processing);
        if let Some(recorder) = state.recorder.clone() {
            if recorder.state() == web_sys::RecordingState::Recording {
                if recorder.stop().is_err() {
                    state.release_capture();
                    state.fail("Could not stop recording. Please try again.");
                }
            }
        }
    }

    pub fn discard(&self) {
        let mut state = self.0.borrow_mut();
        state.generation += 1;
        if let Some(recorder) = state.recorder.take() {
            if recorder.state() == web_sys::RecordingState::Recording {
                let _ = recorder.stop();
            }
            recorder.set_ondataavailable(None);
            recorder.set_onstop(None);
            recorder.set_onerror(None);
        }
        state.release_capture();
        state.on_data = None;
        state.on_stop = None;
        state.on_error = None;
        state.discard_audio();
        state.levels.fill(0.0);
        if !state.disposed {
            state.emit_levels();
            state.status(RecordingPhase::Idle);
        }
    }

    pub fn dispose(&self) {
        self.discard();
        self.0.borrow_mut().disposed = true;
    }
}
