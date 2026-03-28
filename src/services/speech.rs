use wasm_bindgen::JsCast;
use web_sys::{window, Headers, Request, RequestInit, RequestMode, Response, SpeechSynthesisUtterance, SpeechSynthesisVoice, Window};
use zipseek_core::{HashToLexemeMap, LexemeHash};

use crate::models::AppError;

pub struct SpeechSynthesisApi();

impl SpeechSynthesisApi {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn speak(text: &str) -> Result<(), AppError> {
        Ok(())
    }    
}

pub fn speak(text: &str) -> Result<(), AppError> {
    let lang = "zh-CN";
    let win = window().unwrap();
    let synth = win.speech_synthesis()?;

    let utterance = SpeechSynthesisUtterance::new()?;
    utterance.set_rate(0.9);
    utterance.set_pitch(1.0);
    utterance.set_text(text);

    let voices = synth.get_voices();

    for i in 0..voices.length() {
        let voice = voices.get(i).unchecked_into::<SpeechSynthesisVoice>();
        
        if voice.lang() == lang {
            utterance.set_voice(Some(&voice));
            break;
        }
    }

    // synth.add_event_listener_with_callback(type_, listener)

    synth.speak(&utterance);

    Ok(())
}
