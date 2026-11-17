use std::{cell::RefCell, rc::Weak};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioContext, Blob, BlobEvent, BlobPropertyBag, MediaRecorder, MediaStream,
    MediaStreamConstraints, Url,
};

use super::{MAX_DURATION_MS, MicrophoneRecorder, RecordingPhase, State, schedule_frame};

pub(super) async fn start_capture(
    weak: Weak<RefCell<State>>,
    generation: u64,
) -> Result<(), JsValue> {
    let window = weak
        .upgrade()
        .ok_or_else(|| JsValue::from_str("Recorder is no longer available"))?
        .borrow()
        .window
        .clone();
    if !window.is_secure_context() {
        return Err(JsValue::from_str(
            "Microphone recording requires HTTPS or localhost.",
        ));
    }
    let media_devices = window.navigator().media_devices()?;
    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    constraints.set_video(&JsValue::FALSE);
    let stream = JsFuture::from(media_devices.get_user_media_with_constraints(&constraints)?)
        .await?
        .dyn_into::<MediaStream>()?;
    let Some(inner) = weak.upgrade() else {
        stop_tracks(&stream);
        return Ok(());
    };
    if inner.borrow().disposed || inner.borrow().generation != generation {
        stop_tracks(&stream);
        return Ok(());
    }
    inner.borrow_mut().stream = Some(stream.clone());
    let context = AudioContext::new()?;
    inner.borrow_mut().context = Some(context.clone());
    let source = context.create_media_stream_source(&stream)?;
    let analyser = context.create_analyser()?;
    analyser.set_fft_size(1024);
    source.connect_with_audio_node(&analyser)?;
    inner.borrow_mut().analyser = Some(analyser.clone());
    JsFuture::from(context.resume()?).await?;
    if inner.borrow().disposed || inner.borrow().generation != generation {
        inner.borrow_mut().release_capture();
        return Ok(());
    }
    let recorder = MediaRecorder::new_with_media_stream(&stream)?;
    let data_weak = weak.clone();
    let on_data = Closure::<dyn FnMut(BlobEvent)>::new(move |event: BlobEvent| {
        if let Some(inner) = data_weak.upgrade() {
            let state = inner.borrow();
            if !state.disposed && state.generation == generation {
                if let Some(blob) = event.data().filter(|blob| blob.size() > 0.0) {
                    state.chunks.push(blob.as_ref());
                }
            }
        }
    });
    recorder.set_ondataavailable(Some(on_data.as_ref().unchecked_ref()));
    let stop_weak = weak.clone();
    let on_stop = Closure::<dyn FnMut()>::new(move || {
        if let Some(inner) = stop_weak.upgrade() {
            let mut state = inner.borrow_mut();
            if state.disposed || state.generation != generation {
                return;
            }
            let mime = state
                .recorder
                .as_ref()
                .map(MediaRecorder::mime_type)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "audio/webm".to_owned());
            let options = BlobPropertyBag::new();
            options.set_type(&mime);
            match Blob::new_with_blob_sequence_and_options(&state.chunks, &options) {
                Ok(blob) if blob.size() > 0.0 => match Url::create_object_url_with_blob(&blob) {
                    Ok(url) => {
                        state.release_capture();
                        state.object_url = Some(url.clone());
                        state.status(RecordingPhase::Ready);
                        let _ = state
                            .finished_callback
                            .call1(&JsValue::NULL, &JsValue::from_str(&url));
                    }
                    Err(_) => state.fail("Could not prepare the recording for playback."),
                },
                _ => state.fail("No audio was recorded. Please try again."),
            }
        }
    });
    recorder.set_onstop(Some(on_stop.as_ref().unchecked_ref()));
    let error_weak = weak.clone();
    let on_error = Closure::<dyn FnMut()>::new(move || {
        if let Some(inner) = error_weak.upgrade() {
            let mut state = inner.borrow_mut();
            if !state.disposed && state.generation == generation {
                state.release_capture();
                state.fail("Recording failed. Please try again.");
            }
        }
    });
    recorder.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    recorder.start_with_time_slice(250)?;
    {
        let mut state = inner.borrow_mut();
        state.recorder = Some(recorder);
        state.on_data = Some(on_data);
        state.on_stop = Some(on_stop);
        state.on_error = Some(on_error);
        state.status(RecordingPhase::Recording);
        state.last_sample = 0.0;
    }
    schedule_frame(&weak, generation);
    let timeout_weak = weak.clone();
    let timeout = Closure::<dyn FnMut()>::new(move || {
        if let Some(state) = timeout_weak.upgrade() {
            MicrophoneRecorder::from_shared(state).stop();
        }
    });
    let deadline = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        timeout.as_ref().unchecked_ref(),
        MAX_DURATION_MS,
    )?;
    let mut state = inner.borrow_mut();
    state.deadline = Some(deadline);
    state.on_deadline = Some(timeout);
    Ok(())
}

pub(super) fn stop_tracks(stream: &MediaStream) {
    for track in stream.get_tracks().iter() {
        if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
            track.stop();
        }
    }
}
