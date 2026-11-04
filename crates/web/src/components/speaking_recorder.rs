use js_sys::Function;
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
use yew::prelude::*;

use super::AudioWaveform;

#[wasm_bindgen(module = "/src/services/microphone.js")]
extern "C" {
    type MicrophoneRecorder;

    #[wasm_bindgen(js_name = createMicrophoneRecorder)]
    fn create_recorder(
        levels: &Function,
        status: &Function,
        finished: &Function,
        error: &Function,
    ) -> MicrophoneRecorder;

    #[wasm_bindgen(method)]
    fn start(this: &MicrophoneRecorder);

    #[wasm_bindgen(method)]
    fn stop(this: &MicrophoneRecorder);

    #[wasm_bindgen(method)]
    fn discard(this: &MicrophoneRecorder);

    #[wasm_bindgen(method)]
    fn dispose(this: &MicrophoneRecorder);
}

struct RecorderBindings {
    recorder: MicrophoneRecorder,
    _levels: Closure<dyn FnMut(JsValue)>,
    _status: Closure<dyn FnMut(String)>,
    _finished: Closure<dyn FnMut(String)>,
    _error: Closure<dyn FnMut(String)>,
}

#[function_component(SpeakingRecorder)]
pub fn speaking_recorder() -> Html {
    let levels = use_state(|| vec![0.0; 40]);
    let phase = use_state(|| "idle".to_owned());
    let audio_url = use_state(String::new);
    let error = use_state(String::new);
    let bindings = use_mut_ref(|| None::<RecorderBindings>);

    {
        let levels = levels.clone();
        let phase = phase.clone();
        let audio_url = audio_url.clone();
        let error = error.clone();
        let bindings = bindings.clone();
        use_effect_with((), move |_| {
            let levels_callback = Closure::<dyn FnMut(JsValue)>::new(move |value| {
                if let Ok(samples) = serde_wasm_bindgen::from_value::<Vec<f32>>(value) {
                    levels.set(samples);
                }
            });
            let status_callback = Closure::<dyn FnMut(String)>::new(move |value| {
                phase.set(value);
            });
            let finished_callback = Closure::<dyn FnMut(String)>::new(move |value| {
                audio_url.set(value);
            });
            let error_callback = Closure::<dyn FnMut(String)>::new(move |message| {
                error.set(message);
            });
            let recorder = create_recorder(
                levels_callback.as_ref().unchecked_ref(),
                status_callback.as_ref().unchecked_ref(),
                finished_callback.as_ref().unchecked_ref(),
                error_callback.as_ref().unchecked_ref(),
            );
            *bindings.borrow_mut() = Some(RecorderBindings {
                recorder,
                _levels: levels_callback,
                _status: status_callback,
                _finished: finished_callback,
                _error: error_callback,
            });
            move || {
                if let Some(active) = bindings.borrow_mut().take() {
                    active.recorder.dispose();
                }
            }
        });
    }

    let on_start = {
        let bindings = bindings.clone();
        let audio_url = audio_url.clone();
        let error = error.clone();
        Callback::from(move |_: MouseEvent| {
            audio_url.set(String::new());
            error.set(String::new());
            if let Some(active) = bindings.borrow().as_ref() {
                active.recorder.start();
            }
        })
    };
    let on_stop = {
        let bindings = bindings.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(active) = bindings.borrow().as_ref() {
                active.recorder.stop();
            }
        })
    };
    let on_discard = {
        let bindings = bindings.clone();
        let audio_url = audio_url.clone();
        let error = error.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(active) = bindings.borrow().as_ref() {
                active.recorder.discard();
            }
            audio_url.set(String::new());
            error.set(String::new());
        })
    };
    let busy = *phase == "requesting" || *phase == "processing";
    let recording = *phase == "recording";
    let ready = *phase == "ready" && !audio_url.is_empty();
    let status = match phase.as_str() {
        "requesting" => "Waiting for microphone permission…",
        "recording" => "Recording speech",
        "processing" => "Preparing audio playback…",
        "ready" => "Recording ready for playback",
        "error" => "Microphone unavailable",
        _ => "Microphone is off",
    };

    html! {
        <section data-component="speaking-recorder" class="rounded-2xl border border-line bg-surface p-5" aria-label="Speaking recorder">
            <div class="space-y-4">
                <AudioWaveform levels={(*levels).clone()} recording={recording} />
                <p role="status" aria-live="polite" class="text-sm text-muted">{status}</p>
                if !error.is_empty() {
                    <p role="alert" class="text-sm text-red-600">{(*error).clone()}</p>
                }
                <div class="flex flex-wrap gap-3">
                    if recording {
                        <button type="button" data-action="stop-recording" onclick={on_stop} class="rounded-lg bg-red-600 px-5 py-3 font-semibold text-white">
                            {"Stop recording"}
                        </button>
                    } else {
                        <button type="button" data-action="start-recording" onclick={on_start} disabled={busy} class="rounded-lg bg-blue-600 px-5 py-3 font-semibold text-white disabled:opacity-50">
                            {if busy { "Preparing…" } else { "Record" }}
                        </button>
                    }
                    if ready {
                        <button type="button" data-action="discard-recording" onclick={on_discard} class="rounded-lg border border-line px-5 py-3 font-semibold">
                            {"Discard"}
                        </button>
                    }
                </div>
                if ready {
                    <audio data-component="speaking-playback" controls=true src={(*audio_url).clone()} class="w-full" aria-label="Play your recording" />
                }
                <p class="text-xs text-muted">
                    {"Recordings stay in this browser tab. Audio is not uploaded or graded. Recording stops automatically after 30 seconds."}
                </p>
            </div>
        </section>
    }
}
