use yew::prelude::*;

use crate::hooks::{use_app_environment, use_stt_service, use_tts_service};
use crate::services::browser_stt;

#[function_component(SpeechSettings)]
pub(super) fn speech_settings() -> Html {
    let environment = use_app_environment();
    let tts = use_tts_service();
    let stt = use_stt_service();
    let tts_feedback = use_state(|| None::<Result<String, String>>);
    let stt_feedback = use_state(|| None::<Result<String, String>>);
    let listening = use_state(|| false);

    let on_tts_test = {
        let feedback = tts_feedback.clone();
        let speech = environment.speech.clone();
        Callback::from(move |_: MouseEvent| {
            let result = speech
                .speak("你好，欢迎学习中文。", "zh-CN")
                .map(|_| {
                    "Playback requested. Check whether you heard the Mandarin phrase.".to_owned()
                })
                .map_err(|error| format!("Speech synthesis failed: {error:?}"));
            feedback.set(Some(result));
        })
    };
    let on_stt_test = {
        let feedback = stt_feedback.clone();
        let listening = listening.clone();
        let window = environment.window.clone();
        Callback::from(move |_: MouseEvent| {
            if *listening {
                return;
            }
            listening.set(true);
            feedback.set(None);
            let feedback_out = feedback.clone();
            let listening_out = listening.clone();
            let finished = Callback::from(move |result: Result<String, String>| {
                feedback_out.set(Some(result));
                listening_out.set(false);
            });
            if let Err(error) = browser_stt::test_mandarin(&window, finished) {
                feedback.set(Some(Err(error)));
                listening.set(false);
            }
        })
    };
    let stt_button_label = if *listening {
        "Listening…"
    } else {
        "Test microphone"
    };
    let provider_note = if tts.desktop || stt.desktop {
        html! { <p class="text-sm text-muted">{"Native speech providers are not yet implemented. Browser speech remains the active provider."}</p> }
    } else {
        Html::default()
    };
    let tts_result = match &*tts_feedback {
        Some(Ok(message)) => {
            html! { <p role="status" data-speech-tts-status="" class="text-sm text-muted">{message.clone()}</p> }
        }
        Some(Err(error)) => {
            html! { <p role="alert" data-speech-tts-status="" class="text-sm text-red-600">{error.clone()}</p> }
        }
        None => Html::default(),
    };
    let stt_result = match &*stt_feedback {
        Some(Ok(text)) => {
            html! { <p role="status" data-speech-stt-status="" class="text-sm">{"Recognized: "}<span lang="zh-CN" class="font-semibold">{text.clone()}</span></p> }
        }
        Some(Err(error)) => {
            html! { <p role="alert" data-speech-stt-status="" class="text-sm text-red-600">{error.clone()}</p> }
        }
        None => Html::default(),
    };
    html! {
        <section data-settings-speech="" class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8">
            <h2 class="text-xl font-semibold">{"Speech"}</h2>
            <p class="mt-2 text-sm text-muted">{"Test your browser's Mandarin speech capabilities."}</p>
            <div class="mt-5 space-y-5">
                <div class="flex flex-wrap items-center justify-between gap-3 border-b border-line pb-5">
                    <div class="min-w-0 space-y-1">
                        <h3 class="font-semibold">{"Text-to-speech"}</h3>
                        <p class="text-sm text-muted">{"Browser speech synthesis"}</p>
                        {tts_result}
                    </div>
                    <button type="button" data-speech-test-tts="" onclick={on_tts_test}
                        class="min-h-11 rounded-lg border border-line px-4 font-semibold hover:bg-raised">{"Test voice"}</button>
                </div>
                <div class="flex flex-wrap items-center justify-between gap-3">
                    <div class="min-w-0 space-y-1">
                        <h3 class="font-semibold">{"Speech-to-text"}</h3>
                        <p class="text-sm text-muted">{"Browser speech recognition · Mandarin (zh-CN). Your browser may send microphone audio to its speech-recognition provider after you choose Test microphone."}</p>
                        {stt_result}
                    </div>
                    <button type="button" data-speech-test-stt="" onclick={on_stt_test}
                        disabled={*listening}
                        class="min-h-11 rounded-lg border border-line px-4 font-semibold hover:bg-raised disabled:opacity-50">
                        {stt_button_label}
                    </button>
                </div>
                {provider_note}
            </div>
        </section>
    }
}
