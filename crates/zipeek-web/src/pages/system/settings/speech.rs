use gloo::timers::callback::Interval;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::use_app_environment;
use crate::services::browser_stt;

use zipeek_browser::local_speech;

const VOICE_SAMPLE: &str = "今天天气晴朗。";

#[derive(Clone, Copy, PartialEq, Eq)]
enum RecordingPhase {
    Idle,
    Recording,
    Processing,
}

fn microphone_meter(level: f32) -> Html {
    let heights = [0.48, 0.72, 0.90, 0.60, 1.0, 0.68, 0.84, 0.54, 0.76];
    html! {
        <div data-speech-live-level="" role="meter" aria-label="Microphone input level" aria-valuemin="0" aria-valuemax="100" aria-valuenow={((level * 100.0).round() as u32).to_string()} class="flex h-9 items-center gap-1.5">
            {for heights.iter().enumerate().map(|(index, factor)| {
                let height = (4.0 + level * *factor * 28.0).round();
                html! {
                    <span key={index} class="w-1 rounded-full bg-brand-action motion-safe:transition-[height] motion-safe:duration-75" style={format!("height:{height}px")} />
                }
            })}
        </div>
    }
}

fn speech_error(error: JsValue, operation: &str) -> String {
    web_sys::console::error_2(&JsValue::from_str(operation), &error);
    let detail = error.as_string().unwrap_or_default().to_ascii_lowercase();
    if detail.contains("failed to fetch") || detail.contains("networkerror") {
        "Cannot reach the speech service. Check that it is running and try again.".into()
    } else if detail.contains("no transcript") || detail.contains("empty transcription") {
        "I didn’t catch that. Try again.".into()
    } else if detail.contains("permission") || detail.contains("notallowederror") {
        "Microphone permission was denied. Allow access in your browser and retry.".into()
    } else if detail.contains("500") || detail.contains("token") || detail.contains("input") {
        "Couldn’t play the sample. Try again.".into()
    } else if detail.contains("503") || detail.contains("unavailable") {
        "Local speech is unavailable. Check the service and retry.".into()
    } else {
        format!("{operation} failed. Try again.")
    }
}

fn speech_feedback(result: &Result<String, String>, transcript: bool) -> Html {
    match result {
        Ok(value) if transcript => html! {
            <div class="flex items-start gap-2 text-sm text-foreground" role="status">
                <Icon data={IconData::LUCIDE_CHECK} width="16px" height="16px" />
                <span><span class="text-muted">{"Heard: "}</span><span lang="zh-CN" class="font-semibold">{value}</span></span>
            </div>
        },
        Ok(_) => Html::default(),
        Err(message) => html! {
            <div class="flex items-start gap-2 text-sm text-red-300" role="alert">
                <span class="shrink-0"><Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="16px" height="16px" /></span>
                <span>{message}</span>
            </div>
        },
    }
}

fn speech_activity() -> Html {
    html! {
        <span class="inline-flex h-5 items-center gap-1" aria-hidden="true">
            <span class="h-2 w-1 rounded-full bg-current animate-pulse motion-reduce:animate-none" />
            <span class="h-4 w-1 rounded-full bg-current animate-pulse [animation-delay:120ms] motion-reduce:animate-none" />
            <span class="h-5 w-1 rounded-full bg-current animate-pulse [animation-delay:240ms] motion-reduce:animate-none" />
            <span class="h-4 w-1 rounded-full bg-current animate-pulse [animation-delay:360ms] motion-reduce:animate-none" />
            <span class="h-2 w-1 rounded-full bg-current animate-pulse [animation-delay:480ms] motion-reduce:animate-none" />
        </span>
    }
}

#[function_component(SpeechSettings)]
pub(super) fn speech_settings() -> Html {
    let environment = use_app_environment();
    let local = use_state(local_speech::enabled);
    let tts_feedback = use_state(|| None::<Result<String, String>>);
    let stt_feedback = use_state(|| None::<Result<String, String>>);
    let capabilities = use_state(|| None::<Result<(), String>>);
    let listening = use_state(|| false);
    let recording_phase = use_state(|| RecordingPhase::Idle);
    let microphone_level = use_state(|| 0.0_f32);
    let playing = use_state(|| false);
    let epoch = use_mut_ref(|| 0_u64);
    {
        let microphone_level = microphone_level.clone();
        let recording_phase = recording_phase.clone();
        use_effect_with(*recording_phase, move |phase| {
            let timer = if *phase == RecordingPhase::Recording {
                Some(Interval::new(80, move || {
                    if local_speech::microphone_processing() {
                        recording_phase.set(RecordingPhase::Processing);
                    } else {
                        microphone_level.set(local_speech::microphone_level().unwrap_or(0.0));
                    }
                }))
            } else {
                None
            };
            move || drop(timer)
        });
    }

    {
        use_effect_with((), move |_| {
            move || {
                local_speech::cancel();
            }
        });
    }
    let on_select = {
        let local = local.clone();
        let tts = tts_feedback.clone();
        let stt = stt_feedback.clone();
        let capabilities = capabilities.clone();
        let epoch = epoch.clone();
        let speech = environment.speech.clone();
        let playing = playing.clone();
        let listening = listening.clone();
        let recording_phase = recording_phase.clone();
        Callback::from(move |next: bool| {
            *epoch.borrow_mut() += 1;
            let _ = speech.cancel();
            tts.set(None);
            stt.set(None);
            capabilities.set(None);
            playing.set(false);
            listening.set(false);
            recording_phase.set(RecordingPhase::Idle);
            match local_speech::select(next) {
                Ok(()) => {
                    local.set(next);
                    if next {
                        let capabilities = capabilities.clone();
                        let epoch = epoch.clone();
                        let revision = *epoch.borrow();
                        wasm_bindgen_futures::spawn_local(async move {
                            let result = local_speech::capabilities()
                                .await
                                .map(|_| ())
                                .map_err(|error| speech_error(error, "Speech connection"));
                            if *epoch.borrow() == revision {
                                capabilities.set(Some(result));
                            }
                        });
                    }
                }
                Err(error) => tts.set(Some(Err(speech_error(error, "Speech selection")))),
            }
        })
    };
    let on_select_browser = {
        let on_select = on_select.clone();
        Callback::from(move |_: MouseEvent| on_select.emit(false))
    };
    let on_select_local = {
        let on_select = on_select.clone();
        Callback::from(move |_: MouseEvent| on_select.emit(true))
    };
    let on_tts_test = {
        let feedback = tts_feedback.clone();
        let playing = playing.clone();
        let local = local.clone();
        let speech = environment.speech.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if *playing {
                return;
            }
            feedback.set(None);
            if *local {
                playing.set(true);
                let feedback = feedback.clone();
                let playing = playing.clone();
                let epoch = epoch.clone();
                let revision = *epoch.borrow();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = local_speech::play(VOICE_SAMPLE)
                        .await
                        .map(|_| "".to_owned())
                        .map_err(|error| speech_error(error, "Voice test"));
                    if *epoch.borrow() == revision {
                        feedback.set(Some(result));
                        playing.set(false);
                    }
                });
            } else {
                feedback.set(Some(
                    speech
                        .speak(VOICE_SAMPLE, "zh-CN")
                        .map(|_| "".to_owned())
                        .map_err(|error| {
                            web_sys::console::error_1(&JsValue::from_str(&format!("{error:?}")));
                            "Browser voice test failed. Check your browser's speech settings."
                                .to_owned()
                        }),
                ));
            }
        })
    };
    let on_stop_recording = {
        let recording_phase = recording_phase.clone();
        let listening = listening.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if local_speech::finish_microphone_recording() {
                recording_phase.set(RecordingPhase::Processing);
            } else {
                *epoch.borrow_mut() += 1;
                local_speech::cancel();
                listening.set(false);
                recording_phase.set(RecordingPhase::Idle);
            }
        })
    };
    let on_stt_test = {
        let feedback = stt_feedback.clone();
        let listening = listening.clone();
        let recording_phase = recording_phase.clone();
        let microphone_level = microphone_level.clone();
        let local = local.clone();
        let window = environment.window.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if *listening {
                return;
            }
            listening.set(true);
            microphone_level.set(0.0);
            recording_phase.set(RecordingPhase::Recording);
            feedback.set(None);
            if *local {
                let feedback = feedback.clone();
                let listening = listening.clone();
                let recording_phase = recording_phase.clone();
                let epoch = epoch.clone();
                let revision = *epoch.borrow();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = local_speech::transcribe_microphone()
                        .await
                        .and_then(|value| {
                            if value.trim().is_empty() {
                                Err(JsValue::from_str("no transcript"))
                            } else {
                                Ok(value)
                            }
                        })
                        .map_err(|error| speech_error(error, "Microphone test"));
                    if *epoch.borrow() == revision {
                        feedback.set(Some(result));
                        listening.set(false);
                        recording_phase.set(RecordingPhase::Idle);
                    }
                });
            } else {
                let feedback_out = feedback.clone();
                let listening_out = listening.clone();
                let recording_phase_out = recording_phase.clone();
                let finished = Callback::from(move |result: Result<String, String>| {
                    feedback_out.set(Some(result));
                    listening_out.set(false);
                    recording_phase_out.set(RecordingPhase::Idle);
                });
                if let Err(error) = browser_stt::test_mandarin(&window, finished) {
                    web_sys::console::error_1(&JsValue::from_str(&error));
                    feedback.set(Some(Err(
                        "Microphone could not start. Check browser permissions.".into(),
                    )));
                    listening.set(false);
                    recording_phase.set(RecordingPhase::Idle);
                }
            }
        })
    };
    let on_cancel = {
        let playing = playing.clone();
        let listening = listening.clone();
        let recording_phase = recording_phase.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            *epoch.borrow_mut() += 1;
            local_speech::cancel();
            playing.set(false);
            listening.set(false);
            recording_phase.set(RecordingPhase::Idle);
        })
    };
    html! {
        <section data-settings-speech="" class="rounded-2xl border border-line bg-surface p-5 shadow-sm sm:p-8 space-y-6">
            <div class="space-y-2">
                <h2 class="text-xl font-semibold">{"Mandarin speech"}</h2>
            </div>
            <fieldset class="space-y-3" aria-label="Speech provider">
                <legend class="mb-2 font-semibold">{"Provider"}</legend>
                <div role="group" aria-label="Speech provider" class="grid gap-3 sm:grid-cols-2">
                    <button type="button" data-speech-provider="browser"
                        aria-pressed={(!*local).to_string()}
                        onclick={on_select_browser}
                        class={classes!("min-h-20", "rounded-xl", "border-2", "p-4", "text-left", "transition-colors", "motion-reduce:transition-none", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-accent", if !*local {"border-brand-action bg-accent-soft"} else {"border-line bg-surface hover:bg-raised"})}>
                        <span class="block font-semibold">{"Browser speech"}</span>
                        <span class="block text-sm text-muted">{"Browser-provided speech; recognition may use a cloud service."}</span>
                    </button>
                    <button type="button" data-speech-provider="local"
                        aria-pressed={(*local).to_string()}
                        disabled={!local_speech::allowed()}
                        onclick={on_select_local}
                        class={classes!("min-h-20", "rounded-xl", "border-2", "p-4", "text-left", "transition-colors", "motion-reduce:transition-none", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-accent", "disabled:cursor-not-allowed", "disabled:opacity-40", if *local {"border-brand-action bg-accent-soft"} else {"border-line bg-surface hover:bg-raised"})}>
                        <span class="block font-semibold">{"Local Mandarin speech"}</span>
                        <span class="block text-sm text-muted">{"Qwen3-TTS / Qwen3-ASR on the loopback speech server."}</span>
                    </button>
                </div>
                if !local_speech::allowed() { <p class="text-sm text-muted">{"Local speech is available only from the trusted localhost:1420 development app."}</p> }
                if let Some(Err(message)) = &*capabilities {
                    <p class="text-sm text-red-300" role="alert">{message}</p>
                }
            </fieldset>
            <div class="grid items-stretch gap-5 md:grid-cols-2">
                <div class="flex min-w-0 flex-col gap-4 rounded-xl border border-line p-4 sm:p-5">
                    <h3 class="font-semibold">{"Listen to a phrase"}</h3>
                    <p class="text-lg font-medium" lang="zh-CN">{VOICE_SAMPLE}</p>
                    <div class="mt-auto flex items-center gap-3">
                        <button type="button" data-speech-test-tts="" onclick={if *local && *playing {on_cancel.clone()} else {on_tts_test.clone()}} disabled={*playing && !*local} class="inline-flex min-h-11 items-center justify-center gap-2 rounded-lg bg-brand-action px-4 font-semibold text-white transition-colors hover:brightness-110 disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent motion-reduce:transition-none">
                            <Icon data={if *playing {IconData::LUCIDE_SQUARE} else {IconData::LUCIDE_VOLUME_2}} width="18px" height="18px" />
                            <span>{if *playing {"Stop"} else {"Play sample"}}</span>
                        </button>
                        if *playing { {speech_activity()} }
                    </div>
                    <div data-speech-tts-status="" class="min-h-5" aria-live="polite">
                        if let Some(result) = &*tts_feedback {
                            {speech_feedback(result, false)}
                        }
                    </div>
                </div>
                <div class="flex min-w-0 flex-col gap-4 rounded-xl border border-line p-4 sm:p-5">
                    <h3 class="font-semibold">{"Speak Mandarin"}</h3>
                    <p class="text-sm text-muted">{"Speak a short Mandarin phrase."}</p>
                    <div class="mt-auto flex min-h-11 flex-wrap items-center gap-3">
                        <button type="button" data-speech-test-stt="" onclick={if *local && *recording_phase == RecordingPhase::Recording {on_stop_recording.clone()} else {on_stt_test.clone()}} disabled={*listening && (!*local || *recording_phase == RecordingPhase::Processing)} class="inline-flex min-h-11 items-center justify-center gap-2 rounded-lg bg-brand-action px-4 font-semibold text-white transition-colors hover:brightness-110 disabled:cursor-wait disabled:opacity-60 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent motion-reduce:transition-none">
                            <Icon data={if *recording_phase == RecordingPhase::Recording {IconData::LUCIDE_SQUARE} else {IconData::LUCIDE_MIC}} width="18px" height="18px" />
                            <span>{if *recording_phase == RecordingPhase::Recording {"Stop recording"} else if *recording_phase == RecordingPhase::Processing {"Recognizing..."} else if *listening {"Listening..."} else {"Start speaking"}}</span>
                        </button>
                        if *recording_phase == RecordingPhase::Recording && *local {
                            {microphone_meter(*microphone_level)}
                            <span class="sr-only" role="status">{"Microphone recording in progress"}</span>
                        }
                        if *listening && !*local { {speech_activity()} }
                    </div>
                    <div data-speech-stt-status="" class="min-h-10" aria-live="polite">
                        if *recording_phase == RecordingPhase::Processing {
                            <div class="flex items-center gap-2 text-sm text-muted" role="status">
                                <span class="h-3 w-3 animate-spin rounded-full border-2 border-current border-r-transparent motion-reduce:animate-none" aria-hidden="true" />
                                {"Transcribing your speech..."}
                            </div>
                        } else if let Some(result) = &*stt_feedback {
                            {speech_feedback(result, true)}
                        }
                    </div>
                </div>
            </div>
        </section>
    }
}
