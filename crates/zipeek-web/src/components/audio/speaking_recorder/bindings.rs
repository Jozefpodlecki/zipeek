use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::Window;
use zipeek_browser::microphone::MicrophoneRecorder;

pub(super) struct RecorderBindings {
    recorder: MicrophoneRecorder,
    _levels: Closure<dyn FnMut(JsValue)>,
    _status: Closure<dyn FnMut(JsValue)>,
    _finished: Closure<dyn FnMut(JsValue)>,
    _error: Closure<dyn FnMut(JsValue)>,
}

impl RecorderBindings {
    pub(super) fn new(
        window: Window,
        levels: Closure<dyn FnMut(JsValue)>,
        status: Closure<dyn FnMut(JsValue)>,
        finished: Closure<dyn FnMut(JsValue)>,
        error: Closure<dyn FnMut(JsValue)>,
    ) -> Self {
        let recorder = MicrophoneRecorder::new(
            window,
            levels.as_ref().unchecked_ref(),
            status.as_ref().unchecked_ref(),
            finished.as_ref().unchecked_ref(),
            error.as_ref().unchecked_ref(),
        );
        Self {
            recorder,
            _levels: levels,
            _status: status,
            _finished: finished,
            _error: error,
        }
    }

    pub(super) fn start(&self) {
        self.recorder.start();
    }

    pub(super) fn stop(&self) {
        self.recorder.stop();
    }

    pub(super) fn discard(&self) {
        self.recorder.discard();
    }

    pub(super) fn dispose(&self) {
        self.recorder.dispose();
    }
}
