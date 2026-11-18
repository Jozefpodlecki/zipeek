use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::hooks::use_app_environment;
use crate::services::browser_stt;

use zipeek_browser::local_speech;

fn message(error: JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}

#[function_component(SpeechSettings)]
pub(super) fn speech_settings() -> Html {
    let environment = use_app_environment();
    let local = use_state(local_speech::enabled);
    let tts_feedback = use_state(|| None::<Result<String, String>>);
    let stt_feedback = use_state(|| None::<Result<String, String>>);
    let capabilities = use_state(|| None::<String>);
    let listening = use_state(|| false);
    let playing = use_state(|| false);
    let epoch = use_mut_ref(|| 0_u64);

    {
        use_effect_with((), move |_| {
            move || { local_speech::cancel(); }
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
        Callback::from(move |event: Event| {
            let next = event.target_unchecked_into::<web_sys::HtmlInputElement>().value() == "local";
            *epoch.borrow_mut() += 1;
            let _ = speech.cancel();
            tts.set(None); stt.set(None); capabilities.set(None);
            playing.set(false); listening.set(false);
            match local_speech::select(next) {
                Ok(()) => {
                    local.set(next);
                    if next {
                        let capabilities = capabilities.clone();
                        let epoch = epoch.clone();
                        let revision = *epoch.borrow();
                        wasm_bindgen_futures::spawn_local(async move {
                            let result = local_speech::capabilities().await.map(|value| JsValue::from_str(&value));
                            if *epoch.borrow() == revision {
                                capabilities.set(Some(match result {
                                    Ok(value) => value.as_string().unwrap_or_else(|| "Local service reached".into()),
                                    Err(_) => "Local service unavailable. Start cargo run --locked -p zipeek-tts-stt and install local speech resources.".into(),
                                }));
                            }
                        });
                    }
                }
                Err(error) => tts.set(Some(Err(message(error)))),
            }
        })
    };
    let on_tts_test = {
        let feedback = tts_feedback.clone();
        let playing = playing.clone();
        let local = local.clone();
        let speech = environment.speech.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if *playing { return; }
            feedback.set(None);
            if *local {
                playing.set(true);
                let feedback = feedback.clone();
                let playing = playing.clone();
                let epoch = epoch.clone();
                let revision = *epoch.borrow();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = local_speech::play("你好，欢迎学习中文。").await.map(|value| JsValue::from_str(&value)).map(|_| "Local Mandarin WAV played.".to_owned()).map_err(message);
                    if *epoch.borrow() == revision { feedback.set(Some(result)); playing.set(false); }
                });
            } else {
                feedback.set(Some(speech.speak("你好，欢迎学习中文。", "zh-CN")
                    .map(|_| "Browser synthesis requested.".to_owned())
                    .map_err(|error| format!("Browser synthesis failed: {error:?}"))));
            }
        })
    };
    let on_stt_test = {
        let feedback = stt_feedback.clone();
        let listening = listening.clone();
        let local = local.clone();
        let window = environment.window.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            if *listening { return; }
            listening.set(true); feedback.set(None);
            if *local {
                let feedback = feedback.clone(); let listening = listening.clone();
                let epoch = epoch.clone(); let revision = *epoch.borrow();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = local_speech::transcribe_microphone().await.map(|value| JsValue::from_str(&value)).and_then(|value| value.as_string().ok_or_else(|| JsValue::from_str("No transcript")))
                        .map_err(message);
                    if *epoch.borrow() == revision { feedback.set(Some(result)); listening.set(false); }
                });
            } else {
                let feedback_out = feedback.clone(); let listening_out = listening.clone();
                let finished = Callback::from(move |result: Result<String, String>| {
                    feedback_out.set(Some(result)); listening_out.set(false);
                });
                if let Err(error) = browser_stt::test_mandarin(&window, finished) {
                    feedback.set(Some(Err(error))); listening.set(false);
                }
            }
        })
    };
    let on_cancel = {
        let playing = playing.clone(); let listening = listening.clone();
        let epoch = epoch.clone();
        Callback::from(move |_: MouseEvent| {
            *epoch.borrow_mut() += 1;
            local_speech::cancel();
            playing.set(false); listening.set(false);
        })
    };
    html! {
        <section data-settings-speech="" class="rounded-2xl border border-line bg-surface p-5 shadow-sm sm:p-8 space-y-6">
            <div class="space-y-2">
                <h2 class="text-xl font-semibold">{"Mandarin speech"}</h2>
                <p class="text-sm text-muted">{"Choose how pronunciation and dictation are handled. Browser speech remains the default."}</p>
            </div>
            <fieldset class="space-y-3" aria-label="Speech provider">
                <legend class="mb-2 font-semibold">{"Provider"}</legend>
                <label class="flex items-start gap-3 rounded-xl border border-line p-4 cursor-pointer">
                    <input class="mt-1" type="radio" name="speech-provider" value="browser" checked={!*local} onchange={on_select.clone()} />
                    <span><strong>{"Browser speech"}</strong><span class="block text-sm text-muted">{"No local setup required. Browser speech recognition may use your browser vendor's cloud service."}</span></span>
                </label>
                <label class="flex items-start gap-3 rounded-xl border border-line p-4 cursor-pointer">
                    <input class="mt-1" type="radio" name="speech-provider" value="local" checked={*local} disabled={!local_speech::allowed()} onchange={on_select} />
                    <span><strong>{"Local Mandarin speech"}</strong><span class="block text-sm text-muted">{"Uses the eSpeak NG / Whisper process on this computer. Explicit opt-in, not available on hosted pages; microphone audio stays on loopback."}</span></span>
                </label>
                if !local_speech::allowed() { <p class="text-sm text-muted">{"Local speech is available only from the trusted localhost:1420 development app."}</p> }
                if let Some(status) = &*capabilities { <p class="text-sm text-muted" role="status">{status.clone()}</p> }
            </fieldset>
            <div class="grid gap-5 md:grid-cols-2">
                <div class="space-y-3 rounded-xl border border-line p-4">
                    <h3 class="font-semibold">{"Listen to a phrase"}</h3>
                    <p class="text-sm text-muted" lang="zh-CN">{"你好，欢迎学习中文。"}</p>
                    <button type="button" data-speech-test-tts="" onclick={on_tts_test} disabled={*playing} class="min-h-11 rounded-lg bg-brand-action px-4 font-semibold text-white disabled:opacity-50">{if *playing {"Playing..."} else {"Test voice"}}</button>
                    if let Some(result) = &*tts_feedback {
                        <p role={if result.is_ok() {"status"} else {"alert"}} data-speech-tts-status="" class="text-sm">{result.as_ref().map(String::as_str).unwrap_or_else(|error| error.as_str())}</p>
                    }
                </div>
                <div class="space-y-3 rounded-xl border border-line p-4">
                    <h3 class="font-semibold">{"Speak Mandarin"}</h3>
                    <p class="text-sm text-muted">{"Microphone recording starts only when you select Test microphone. Local mode records for five seconds."}</p>
                    <button type="button" data-speech-test-stt="" onclick={on_stt_test} disabled={*listening} class="min-h-11 rounded-lg bg-brand-action px-4 font-semibold text-white disabled:opacity-50">{if *listening {"Recording..."} else {"Test microphone"}}</button>
                    if let Some(result) = &*stt_feedback {
                        <p role={if result.is_ok() {"status"} else {"alert"}} data-speech-stt-status="" class="text-sm">{if result.is_ok() {"Recognized: "} else {""}}<span lang="zh-CN">{result.as_ref().map(String::as_str).unwrap_or_else(|error| error.as_str())}</span></p>
                    }
                </div>
            </div>
            if *local && (*playing || *listening) {
                <button type="button" onclick={on_cancel} class="rounded-lg border border-line px-4 py-2 font-semibold">{"Stop local speech"}</button>
            }
        </section>
    }
}
