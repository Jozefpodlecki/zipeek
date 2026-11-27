use gloo::timers::callback::{Interval, Timeout};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use zipeek_browser::local_speech::{self, AutoStopMode};

use super::audio::AudioResources;
use super::readings::{self, ReadingGlyph};
use super::state::{Playback, RecordAction, RecordState, RecordingId, RecordingStage, TargetMode};
use crate::hooks::use_dictionary;

fn public_error(error: JsValue) -> String {
    web_sys::console::error_1(&error);
    let message = error.as_string().unwrap_or_default().to_ascii_lowercase();
    if message.contains("permission") || message.contains("notallowed") {
        "Microphone permission was denied. Allow access and try again.".into()
    } else if message.contains("fetch") || message.contains("network") {
        "Cannot reach the local speech server. Check that it is running.".into()
    } else if message.contains("no mandarin")
        || message.contains("no speech")
        || message.contains("empty")
    {
        "No speech was recognized. Try recording again.".into()
    } else {
        "Speech request failed. Check the local service and try again.".into()
    }
}

fn ensure_local_speech() -> Result<(), String> {
    if !local_speech::allowed() {
        return Err("Local speech requires the localhost:1420 development app.".into());
    }
    if !local_speech::enabled() && local_speech::select(true).is_err() {
        return Err("Could not enable local speech.".into());
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReferenceStatus {
    Idle,
    Loading,
    Ready,
    Failed,
}

#[derive(PartialEq)]
pub(super) struct RecordController {
    pub state: UseReducerHandle<RecordState>,
    pub microphone_waveform: Vec<f32>,
    pub reference_waveform: Vec<f32>,
    pub reference_status: ReferenceStatus,
    pub reading_glyphs: Vec<ReadingGlyph>,
    pub recorded_waveform: Vec<f32>,
    pub has_recording: bool,
    pub on_mode: Callback<TargetMode>,
    pub on_sentence: Callback<InputEvent>,
    pub on_bank: Callback<(u8, u8)>,
    pub on_random: Callback<MouseEvent>,
    pub on_play: Callback<MouseEvent>,
    pub on_retry: Callback<MouseEvent>,
    pub on_record: Callback<MouseEvent>,
    pub on_replay: Callback<MouseEvent>,
    pub on_cancel: Callback<MouseEvent>,
    pub on_download: Callback<MouseEvent>,
}

#[hook]
pub(super) fn use_record_controller() -> RecordController {
    let dictionary = use_dictionary();
    let state = use_reducer(RecordState::default);
    let microphone_waveform = use_state(Vec::<f32>::new);
    let reference_waveform = use_state(Vec::<f32>::new);
    let reference_status = use_state(|| ReferenceStatus::Idle);
    let retry_epoch = use_state(|| 0_u32);
    let reading_glyphs = use_state(Vec::<ReadingGlyph>::new);
    let generation = use_mut_ref(|| 0_u64);
    let audio = use_mut_ref(AudioResources::default);

    {
        let dictionary = dictionary.clone();
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        use_effect_with(state.bank, move |(system, level)| {
            *generation.borrow_mut() += 1;
            let request = *generation.borrow();
            audio.borrow_mut().clear();
            local_speech::cancel();
            let (system, level) = (*system, *level);
            let active = Rc::new(std::cell::Cell::new(true));
            let pending = active.clone();
            spawn_local(async move {
                let result = async {
                    let db = dictionary.database().map_err(|error| error.to_string())?;
                    db.game_words_for_hsk(system, level)
                        .await
                        .map_err(|error| error.to_string())
                }
                .await
                .map_err(|_| "Could not load the HSK vocabulary. Try another level.".to_owned());
                if pending.get() && *generation.borrow() == request {
                    state.dispatch(RecordAction::LoadWords(result));
                }
            });
            move || active.set(false)
        });
    }
    {
        let state = state.clone();
        let level = microphone_waveform.clone();
        use_effect_with((state.stage, state.mode), move |(phase, _mode)| {
            let interval = if *phase == RecordingStage::Recording {
                let meter = level.clone();
                Some(Interval::new(80, move || {
                    if local_speech::microphone_auto_stop_tick()
                        || local_speech::microphone_processing()
                    {
                        state.dispatch(RecordAction::Recognizing);
                    } else {
                        meter.set(local_speech::microphone_waveform().unwrap_or_default());
                    }
                }))
            } else {
                level.set(Vec::new());
                None
            };
            move || drop(interval)
        });
    }
    {
        use_effect_with((), move |_| move || local_speech::cancel());
    }
    {
        let dictionary = dictionary.clone();
        let audio = audio.clone();
        let reference_waveform = reference_waveform.clone();
        let reference_status = reference_status.clone();
        let reading_glyphs = reading_glyphs.clone();
        let key = state.target().map(str::to_owned);
        let retry = *retry_epoch;
        let mode = state.mode;
        let word_numeric = if state.mode == TargetMode::Hsk {
            state
                .selected
                .as_ref()
                .and_then(|word| word.pinyin_numeric.clone())
        } else {
            None
        };
        use_effect_with(
            (key, word_numeric, retry, mode),
            move |(key, word_numeric, _retry, mode)| {
                audio.borrow_mut().clear_reference();
                reference_waveform.set(Vec::new());
                reading_glyphs.set(Vec::new());
                reference_status.set(if key.is_some() {
                    ReferenceStatus::Loading
                } else {
                    ReferenceStatus::Idle
                });
                let pending = Rc::new(std::cell::Cell::new(true));
                let current = pending.clone();
                let timer = key.as_ref().map(|text| {
                    let text = text.clone();
                    let word_numeric = word_numeric.clone();
                    let dictionary = dictionary.clone();
                    let audio = audio.clone();
                    let reference_waveform = reference_waveform.clone();
                    let reference_status = reference_status.clone();
                    let reading_glyphs = reading_glyphs.clone();
                    let pending = pending.clone();
                    Timeout::new(if *mode == TargetMode::Hsk { 0 } else { 600 }, move || {
                        let reading_text = text.clone();
                        let reading_pending = pending.clone();
                        spawn_local(async move {
                            let resolved = async {
                                let db = dictionary.database().ok()?;
                                let candidates =
                                    db.text_reading_candidates(&reading_text).await.ok()?;
                                let mut resolved = readings::resolve(&reading_text, &candidates);
                                if let Some(numeric) = word_numeric.as_deref() {
                                    if let Some(syllables) =
                                        readings::aligned_syllables(&reading_text, numeric)
                                    {
                                        for (glyph, pinyin) in resolved.iter_mut().zip(syllables) {
                                            glyph.pinyin = Some(pinyin);
                                        }
                                    }
                                }
                                Some(resolved)
                            }
                            .await;
                            if reading_pending.get() {
                                if let Some(readings) = resolved {
                                    reading_glyphs.set(readings);
                                }
                            }
                        });
                        spawn_local(async move {
                            if ensure_local_speech().is_err() {
                                if pending.get() {
                                    reference_status.set(ReferenceStatus::Failed);
                                }
                                return;
                            }
                            let result = local_speech::prepare_record_reference_audio(&text).await;
                            match result {
                                Ok(prepared) if pending.get() => {
                                    reference_waveform.set(prepared.peaks.clone());
                                    audio.borrow_mut().store_reference(prepared);
                                    reference_status.set(ReferenceStatus::Ready);
                                }
                                Ok(prepared) => local_speech::dispose_matching_audio(&prepared),
                                Err(error) if pending.get() => {
                                    web_sys::console::error_1(&error);
                                    reference_status.set(ReferenceStatus::Failed);
                                }
                                Err(_) => {}
                            }
                        });
                    })
                });
                move || {
                    current.set(false);
                    drop(timer);
                }
            },
        );
    }
    let on_mode = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        Callback::from(move |mode: TargetMode| {
            if mode == state.mode {
                return;
            }
            *generation.borrow_mut() += 1;
            audio.borrow_mut().clear();
            local_speech::cancel();
            state.dispatch(RecordAction::ChangeMode(mode));
        })
    };
    let on_sentence = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        Callback::from(move |event: InputEvent| {
            let Some(target) = event.target() else {
                return;
            };
            let Ok(target) = target.dyn_into::<HtmlTextAreaElement>() else {
                return;
            };
            let value = target.value();
            if value.chars().count() > 200 {
                return;
            }
            *generation.borrow_mut() += 1;
            audio.borrow_mut().clear();
            local_speech::cancel();
            state.dispatch(RecordAction::ChangeSentence(value));
        })
    };
    let on_bank = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        Callback::from(move |next: (u8, u8)| {
            if next == state.bank || !matches!(next, (2, 1..=6) | (3, 1..=7)) {
                return;
            }
            *generation.borrow_mut() += 1;
            audio.borrow_mut().clear();
            local_speech::cancel();
            state.dispatch(RecordAction::ChangeBank(next));
        })
    };
    let on_random = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        Callback::from(move |_: MouseEvent| {
            *generation.borrow_mut() += 1;
            audio.borrow_mut().clear();
            local_speech::cancel();
            state.dispatch(RecordAction::NextWord);
        })
    };
    let on_play = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        let reference_status = reference_status.clone();
        Callback::from(move |_: MouseEvent| {
            if state.playback == Playback::Reference {
                *generation.borrow_mut() += 1;
                local_speech::cancel();
                state.dispatch(RecordAction::Cancel);
                return;
            }
            if state.stage != RecordingStage::Idle || state.playback != Playback::None {
                return;
            }
            if *reference_status != ReferenceStatus::Ready {
                return;
            }
            let Some(prepared) = audio.borrow().reference() else {
                return;
            };
            state.dispatch(RecordAction::BeginPlayback(Playback::Reference));
            *generation.borrow_mut() += 1;
            let request = *generation.borrow();
            let generation = generation.clone();
            let state = state.clone();
            spawn_local(async move {
                let result = local_speech::play_prepared_reference(&prepared).await;
                if *generation.borrow() == request {
                    state.dispatch(RecordAction::EndPlayback(result.err().map(public_error)));
                }
            });
        })
    };
    let on_retry = {
        let retry_epoch = retry_epoch.clone();
        Callback::from(move |_: MouseEvent| retry_epoch.set((*retry_epoch).wrapping_add(1)))
    };
    let on_record = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        Callback::from(move |_: MouseEvent| {
            if state.stage == RecordingStage::Recording {
                if local_speech::finish_microphone_recording() {
                    state.dispatch(RecordAction::Recognizing);
                }
                return;
            }
            if state.stage != RecordingStage::Idle {
                return;
            }
            if let Err(message) = ensure_local_speech() {
                state.dispatch(RecordAction::Error(message));
                return;
            }
            *generation.borrow_mut() += 1;
            let request = *generation.borrow();
            audio.borrow_mut().clear_recording();
            local_speech::cancel();
            state.dispatch(RecordAction::BeginRecording);
            let generation = generation.clone();
            let state = state.clone();
            let audio = audio.clone();
            let auto_mode = match state.mode {
                TargetMode::Hsk => AutoStopMode::HskWord,
                TargetMode::Sentence | TargetMode::Live => AutoStopMode::CustomSentence,
            };
            spawn_local(async move {
                let result = local_speech::transcribe_microphone_with_auto_stop(auto_mode).await;
                if *generation.borrow() != request {
                    return;
                }
                match result {
                    Ok(result) => {
                        audio.borrow_mut().store(RecordingId(request), result.wav);
                        state.dispatch(RecordAction::CompleteRecording {
                            transcript: result.transcript.map_err(public_error),
                            recording_id: Some(RecordingId(request)),
                        });
                    }
                    Err(error) => state.dispatch(RecordAction::CompleteRecording {
                        transcript: Err(public_error(error)),
                        recording_id: None,
                    }),
                }
            });
        })
    };
    let on_replay = {
        let state = state.clone();
        let generation = generation.clone();
        let audio = audio.clone();
        Callback::from(move |_: MouseEvent| {
            if state.playback == Playback::Recording {
                *generation.borrow_mut() += 1;
                local_speech::cancel();
                state.dispatch(RecordAction::Cancel);
                return;
            }
            if state.stage != RecordingStage::Idle || state.playback != Playback::None {
                return;
            }
            let wav = audio.borrow().replay(state.recording_id);
            let Some(wav) = wav else {
                return;
            };
            if let Err(message) = ensure_local_speech() {
                state.dispatch(RecordAction::Error(message));
                return;
            }
            state.dispatch(RecordAction::BeginPlayback(Playback::Recording));
            *generation.borrow_mut() += 1;
            let request = *generation.borrow();
            let generation = generation.clone();
            let state = state.clone();
            spawn_local(async move {
                let result = local_speech::play_recorded_wav(wav.as_ref()).await;
                if *generation.borrow() == request {
                    state.dispatch(RecordAction::EndPlayback(result.err().map(public_error)));
                }
            });
        })
    };
    let on_cancel = {
        let state = state.clone();
        let generation = generation.clone();
        Callback::from(move |_: MouseEvent| {
            *generation.borrow_mut() += 1;
            local_speech::cancel();
            state.dispatch(RecordAction::Cancel);
        })
    };
    let on_download = {
        let state = state.clone();
        let audio = audio.clone();
        Callback::from(move |_: MouseEvent| {
            if state.stage != RecordingStage::Idle {
                return;
            }
            if let Some(wav) = audio.borrow().replay(state.recording_id) {
                if let Err(error) = local_speech::download_recorded_wav(wav.as_ref()) {
                    state.dispatch(RecordAction::Error(public_error(error)));
                }
            }
        })
    };
    let recorded_waveform = audio.borrow().waveform(state.recording_id);
    let has_recording = audio.borrow().replay(state.recording_id).is_some();
    RecordController {
        state,
        microphone_waveform: (*microphone_waveform).clone(),
        reference_waveform: (*reference_waveform).clone(),
        reference_status: *reference_status,
        reading_glyphs: (*reading_glyphs).clone(),
        recorded_waveform,
        has_recording,
        on_mode,
        on_sentence,
        on_bank,
        on_random,
        on_play,
        on_retry,
        on_record,
        on_replay,
        on_cancel,
        on_download,
    }
}
