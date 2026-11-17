use js_sys::{Array, Function};
use wasm_bindgen::{JsValue, closure::Closure};
use web_sys::{AnalyserNode, AudioContext, BlobEvent, MediaRecorder, MediaStream, Url, Window};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RecordingPhase {
    Idle,
    Requesting,
    Recording,
    Processing,
    Ready,
    Error,
}

impl RecordingPhase {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Requesting => "requesting",
            Self::Recording => "recording",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Error => "error",
        }
    }
}

pub(super) struct State {
    pub(super) window: Window,
    pub(super) levels_callback: Function,
    pub(super) status_callback: Function,
    pub(super) finished_callback: Function,
    pub(super) error_callback: Function,
    pub(super) phase: RecordingPhase,
    pub(super) generation: u64,
    pub(super) disposed: bool,
    pub(super) stream: Option<MediaStream>,
    pub(super) context: Option<AudioContext>,
    pub(super) analyser: Option<AnalyserNode>,
    pub(super) recorder: Option<MediaRecorder>,
    pub(super) chunks: Array,
    pub(super) object_url: Option<String>,
    pub(super) frame: Option<i32>,
    pub(super) deadline: Option<i32>,
    pub(super) last_sample: f64,
    pub(super) levels: Vec<f32>,
    pub(super) on_data: Option<Closure<dyn FnMut(BlobEvent)>>,
    pub(super) on_stop: Option<Closure<dyn FnMut()>>,
    pub(super) on_error: Option<Closure<dyn FnMut()>>,
    pub(super) on_frame: Option<Closure<dyn FnMut(f64)>>,
    pub(super) on_deadline: Option<Closure<dyn FnMut()>>,
}

impl State {
    pub(super) fn status(&mut self, phase: RecordingPhase) {
        self.phase = phase;
        if !self.disposed {
            let _ = self
                .status_callback
                .call1(&JsValue::NULL, &JsValue::from_str(phase.as_str()));
        }
    }

    pub(super) fn emit_levels(&self) {
        if !self.disposed {
            if let Ok(value) = serde_wasm_bindgen::to_value(&self.levels) {
                let _ = self.levels_callback.call1(&JsValue::NULL, &value);
            }
        }
    }

    pub(super) fn fail(&mut self, message: &str) {
        self.status(RecordingPhase::Error);
        let _ = self
            .error_callback
            .call1(&JsValue::NULL, &JsValue::from_str(message));
    }

    pub(super) fn release_capture(&mut self) {
        if let Some(frame) = self.frame.take() {
            let _ = self.window.cancel_animation_frame(frame);
        }
        if let Some(deadline) = self.deadline.take() {
            self.window.clear_timeout_with_handle(deadline);
        }
        if let Some(stream) = self.stream.take() {
            super::capture::stop_tracks(&stream);
        }
        if let Some(context) = self.context.take() {
            let _ = context.close();
        }
        self.analyser = None;
        self.on_frame = None;
        self.on_deadline = None;
    }

    pub(super) fn discard_audio(&mut self) {
        if let Some(url) = self.object_url.take() {
            let _ = Url::revoke_object_url(&url);
        }
        self.chunks = Array::new();
    }
}
