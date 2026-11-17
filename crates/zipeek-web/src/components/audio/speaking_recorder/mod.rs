mod bindings;
mod state;

use wasm_bindgen::{JsValue, closure::Closure};
use yew::prelude::*;

use self::{
    bindings::RecorderBindings,
    state::{RecorderAction, RecorderState, RecordingPhase},
};
use super::audio_waveform::AudioWaveform;
use crate::hooks::use_app_environment;

#[derive(Properties, PartialEq)]
struct RecorderControlsProps {
    phase: RecordingPhase,
    initialized: bool,
    ready: bool,
    on_start: Callback<MouseEvent>,
    on_stop: Callback<MouseEvent>,
    on_discard: Callback<MouseEvent>,
}

#[function_component(RecorderControls)]
fn recorder_controls(props: &RecorderControlsProps) -> Html {
    let recording = props.phase == RecordingPhase::Recording;
    let busy = props.phase.is_busy();
    html! {
        <div class="flex flex-wrap gap-3">
            if recording {
                <button
                    type="button"
                    data-action="stop-recording"
                    onclick={props.on_stop.clone()}
                    class="rounded-lg bg-red-600 px-5 py-3 font-semibold text-white"
                >
                    {"Stop recording"}
                </button>
            } else {
                <button
                    type="button"
                    data-action="start-recording"
                    onclick={props.on_start.clone()}
                    disabled={busy || !props.initialized}
                    class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-50"
                >
                    {if busy { "Preparing…" } else { "Record" }}
                </button>
            }
            if props.ready {
                <button
                    type="button"
                    data-action="discard-recording"
                    onclick={props.on_discard.clone()}
                    class="rounded-lg border border-line px-5 py-3 font-semibold"
                >
                    {"Discard"}
                </button>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct RecorderPlaybackProps {
    audio_url: String,
}

#[function_component(RecorderPlayback)]
fn recorder_playback(props: &RecorderPlaybackProps) -> Html {
    html! {
        <audio
            data-component="speaking-playback"
            controls=true
            src={props.audio_url.clone()}
            class="w-full"
            aria-label="Play your recording"
        />
    }
}

#[function_component(SpeakingRecorder)]
pub fn speaking_recorder() -> Html {
    let environment = use_app_environment();
    let recorder = use_reducer(RecorderState::default);
    let bindings = use_mut_ref(|| None::<RecorderBindings>);

    {
        let recorder = recorder.clone();
        let bindings = bindings.clone();
        let window = environment.window.clone();
        use_effect_with((), move |_| {
            let levels_callback = {
                let recorder = recorder.clone();
                Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
                    if let Ok(levels) = serde_wasm_bindgen::from_value::<Vec<f32>>(value) {
                        recorder.dispatch(RecorderAction::LevelsChanged(levels));
                    }
                })
            };
            let status_callback = {
                let recorder = recorder.clone();
                Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
                    let status = value.as_string().unwrap_or_default();
                    recorder.dispatch(RecorderAction::PhaseChanged(RecordingPhase::from_bridge(
                        &status,
                    )));
                })
            };
            let finished_callback = {
                let recorder = recorder.clone();
                Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
                    recorder.dispatch(RecorderAction::RecordingFinished(
                        value.as_string().unwrap_or_default(),
                    ));
                })
            };
            let error_callback = {
                let recorder = recorder.clone();
                Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
                    let message = value
                        .as_string()
                        .filter(|text| !text.is_empty())
                        .unwrap_or_else(|| "Microphone recording failed.".to_owned());
                    recorder.dispatch(RecorderAction::Failed(message));
                })
            };

            *bindings.borrow_mut() = Some(RecorderBindings::new(
                window,
                levels_callback,
                status_callback,
                finished_callback,
                error_callback,
            ));
            recorder.dispatch(RecorderAction::Initialized);

            move || {
                if let Some(active) = bindings.borrow_mut().take() {
                    active.dispose();
                }
                recorder.dispatch(RecorderAction::Disposed);
            }
        });
    }

    let on_start = {
        let recorder = recorder.clone();
        let bindings = bindings.clone();
        Callback::from(move |_: MouseEvent| {
            recorder.dispatch(RecorderAction::StartRequested);
            if let Some(active) = bindings.borrow().as_ref() {
                active.start();
            } else {
                recorder.dispatch(RecorderAction::Failed(
                    "Microphone recorder is not initialized.".to_owned(),
                ));
            }
        })
    };
    let on_stop = {
        let bindings = bindings.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(active) = bindings.borrow().as_ref() {
                active.stop();
            }
        })
    };
    let on_discard = {
        let recorder = recorder.clone();
        let bindings = bindings.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(active) = bindings.borrow().as_ref() {
                active.discard();
            }
            recorder.dispatch(RecorderAction::Discarded);
        })
    };
    let recording = recorder.phase == RecordingPhase::Recording;
    let ready = recorder.phase == RecordingPhase::Ready && !recorder.audio_url.is_empty();

    html! {
        <section
            data-component="speaking-recorder"
            data-recorder-phase={recorder.phase.as_data_value()}
            data-recorder-initialized={recorder.initialized.to_string()}
            data-recorder-action-count={recorder.action_count.to_string()}
            class="rounded-2xl border border-line bg-surface p-5"
            aria-label="Pronunciation Practice"
        >
            <div class="space-y-4">
                <AudioWaveform
                    levels={recorder.levels.clone()}
                    recording={recording}
                />
                <p role="status" aria-live="polite" class="text-sm text-muted">
                    {recorder.phase.status()}
                </p>
                if !recorder.error.is_empty() {
                    <p role="alert" class="text-sm text-red-600">
                        {&recorder.error}
                    </p>
                }
                <RecorderControls
                    phase={recorder.phase}
                    initialized={recorder.initialized}
                    ready={ready}
                    on_start={on_start}
                    on_stop={on_stop}
                    on_discard={on_discard}
                />
                if ready {
                    <RecorderPlayback audio_url={recorder.audio_url.clone()} />
                }
                <p class="text-xs text-muted">
                    {"Recordings stay in this browser tab. Audio is not uploaded or graded. Recording stops automatically after 30 seconds."}
                </p>
            </div>
        </section>
    }
}
