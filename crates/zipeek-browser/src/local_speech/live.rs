use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use js_sys::{Array, Float32Array, Function, Object, Promise, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

use super::{
    allowed,
    browser_api::{assign, invoke, new_object, property, wait},
    enabled,
};

const SERVICE_URL: &str = "ws://127.0.0.1:8799/v1/stt/live";
const MAX_BUFFERED_BYTES: f64 = 65_536.0;
const FRAME_SAMPLES: usize = 1_600;
const WORKLET: &str = r#"
class ZipeekLiveCapture extends AudioWorkletProcessor {
  constructor() {
    super(); this.pending = []; this.done = false;
    this.port.onmessage = event => {
      if (event.data === 'finish') {
        if (this.pending.length) this.port.postMessage(new Float32Array(this.pending));
        this.pending = [];
        this.done = true;
        this.port.postMessage('finished');
      }
    };
  }
  process(inputs, outputs) {
    if (this.done) return false;
    const channel = inputs[0] && inputs[0][0];
    if (channel) {
      for (let i = 0; i < channel.length; i++) this.pending.push(channel[i]);
      if (this.pending.length >= 2048) {
        this.port.postMessage(new Float32Array(this.pending));
        this.pending = [];
      }
    }
    for (const output of outputs) for (const channel of output) channel.fill(0);
    return true;
  }
}
registerProcessor('zipeek-live-capture', ZipeekLiveCapture);
"#;

#[derive(Clone, PartialEq)]
pub enum LiveEvent {
    Connected,
    Ready,
    Waveform(Vec<f32>),
    Partial(String),
    Final(String),
    Error(String),
    Closed,
}

struct Packetizer {
    sample_rate: f64,
    input_position: u64,
    next_position: f64,
    previous: f32,
    pcm: Vec<i16>,
}

impl Packetizer {
    fn new(sample_rate: f64) -> Result<Self, JsValue> {
        if !sample_rate.is_finite() || !(16_000.0..=192_000.0).contains(&sample_rate) {
            return Err(JsValue::from_str("Unsupported microphone sample rate"));
        }
        Ok(Self {
            sample_rate,
            input_position: 0,
            next_position: 0.0,
            previous: 0.0,
            pcm: Vec::new(),
        })
    }

    fn append(&mut self, samples: &[f32]) {
        let step = self.sample_rate / 16_000.0;
        for &sample in samples {
            let position = self.input_position as f64;
            while self.next_position <= position {
                let t = (self.next_position - (position - 1.0)).clamp(0.0, 1.0) as f32;
                let value = self.previous * (1.0 - t) + sample * t;
                self.pcm
                    .push((value.clamp(-1.0, 1.0) * 32767.0).round() as i16);
                self.next_position += step;
            }
            self.previous = sample;
            self.input_position += 1;
        }
    }

    fn take_packet(&mut self, finish: bool) -> Option<Vec<u8>> {
        if self.pcm.len() < FRAME_SAMPLES && (!finish || self.pcm.is_empty()) {
            return None;
        }
        let count = self.pcm.len().min(FRAME_SAMPLES);
        let mut frame = Vec::with_capacity(count * 2 + 1);
        frame.push(3);
        for sample in self.pcm.drain(..count) {
            frame.extend_from_slice(&sample.to_le_bytes());
        }
        Some(frame)
    }
}

fn send(socket: &JsValue, bytes: &[u8]) -> Result<(), JsValue> {
    invoke(socket, "send", &[Uint8Array::from(bytes).into()]).map(|_| ())
}

fn drain(socket: &JsValue, packetizer: &mut Packetizer, finish: bool) -> Result<(), JsValue> {
    while let Some(packet) = packetizer.take_packet(finish) {
        if property(socket, "bufferedAmount")?
            .as_f64()
            .unwrap_or(f64::INFINITY)
            > MAX_BUFFERED_BYTES
        {
            return Err(JsValue::from_str("Live audio transport is congested"));
        }
        send(socket, &packet)?;
    }
    Ok(())
}

fn stop_tracks(stream: &JsValue) {
    if let Ok(tracks) = invoke(stream, "getTracks", &[]) {
        for track in Array::from(&tracks).iter() {
            let _ = invoke(&track, "stop", &[]);
        }
    }
}

fn message(data: &JsValue) -> Result<LiveEvent, JsValue> {
    let bytes = Uint8Array::new(data).to_vec();
    match bytes.as_slice() {
        [0x81, 0x80, 0x3e, 0, 0, 30, 0, 1] => Ok(LiveEvent::Connected),
        [0x82] => Ok(LiveEvent::Ready),
        [kind @ (0x83 | 0x84), _, _, _, _, text @ ..] if text.len() <= 8192 => {
            let text = std::str::from_utf8(text)
                .map_err(|_| JsValue::from_str("Invalid live transcript UTF-8"))?
                .to_owned();
            if *kind == 0x83 {
                Ok(LiveEvent::Partial(text))
            } else {
                Ok(LiveEvent::Final(text))
            }
        }
        [0x85, error] => Ok(LiveEvent::Error(
            match error {
                1 => "No speech detected",
                2 => "Recognition unavailable",
                3 => "Recognition worker failed",
                4 => "Live session timed out",
                5 => "Invalid session transition",
                6 => "Unexpected protocol message",
                7 => "Recording is too long",
                8 => "Invalid audio frame",
                _ => "Live recognition failed",
            }
            .into(),
        )),
        _ => Err(JsValue::from_str("Unexpected live speech response")),
    }
}

pub struct LiveConnection {
    socket: JsValue,
    stream: JsValue,
    context: JsValue,
    source: JsValue,
    worklet: JsValue,
    sink: JsValue,
    module_url: String,
    packetizer: Rc<RefCell<Packetizer>>,
    streaming: Rc<Cell<bool>>,
    finishing: Rc<Cell<bool>>,
    on_audio: Closure<dyn FnMut(JsValue)>,
    on_message: Closure<dyn FnMut(JsValue)>,
    on_close: Closure<dyn FnMut(JsValue)>,
    on_error: Closure<dyn FnMut(JsValue)>,
}

impl LiveConnection {
    pub async fn connect(on_event: Rc<dyn Fn(LiveEvent)>) -> Result<Self, JsValue> {
        if !allowed() || !enabled() {
            return Err(JsValue::from_str("Local speech is not enabled"));
        }
        let socket = new_object("WebSocket", &[JsValue::from_str(SERVICE_URL)])?;
        assign(&socket, "binaryType", &JsValue::from_str("arraybuffer"))?;
        let on_open = Rc::new(RefCell::new(None::<Function>));
        let opened = Promise::new(&mut {
            let on_open = on_open.clone();
            move |resolve, _reject| {
                *on_open.borrow_mut() = Some(resolve);
            }
        });
        let open_handler = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            if let Some(callback) = on_open.borrow().as_ref() {
                let _ = callback.call0(&JsValue::NULL);
            }
        });
        assign(&socket, "onopen", open_handler.as_ref())?;
        let on_failed = Rc::new(RefCell::new(None::<Function>));
        let failed = Promise::new(&mut {
            let on_failed = on_failed.clone();
            move |resolve, _| *on_failed.borrow_mut() = Some(resolve)
        });
        let fail_handler = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            if let Some(callback) = on_failed.borrow().as_ref() {
                let _ = callback.call0(&JsValue::NULL);
            }
        });
        assign(&socket, "onerror", fail_handler.as_ref())?;
        assign(&socket, "onclose", fail_handler.as_ref())?;
        let race = Array::new();
        race.push(&opened);
        race.push(&failed);
        let _ = wait(Promise::race(&race).into()).await?;
        assign(&socket, "onopen", &JsValue::NULL)?;
        assign(&socket, "onerror", &JsValue::NULL)?;
        assign(&socket, "onclose", &JsValue::NULL)?;
        drop(open_handler);
        drop(fail_handler);
        if property(&socket, "readyState")?.as_f64() != Some(1.0) {
            let _ = invoke(&socket, "close", &[]);
            return Err(JsValue::from_str(
                "Could not connect to local speech service",
            ));
        }

        let create = async {
            let navigator = property(&js_sys::global(), "navigator")?;
            let devices = property(&navigator, "mediaDevices")?;
            let constraints = Object::new();
            assign(&constraints, "audio", &JsValue::TRUE)?;
            assign(&constraints, "video", &JsValue::FALSE)?;
            let stream = wait(invoke(&devices, "getUserMedia", &[constraints.into()])?).await?;
            let setup = async {
                let context = new_object("AudioContext", &[])?;
                let context_setup = async {
                    let sample_rate = property(&context, "sampleRate")?.as_f64().unwrap_or(0.0);
                    let packetizer = Rc::new(RefCell::new(Packetizer::new(sample_rate)?));
                    let parts = Array::new();
                    parts.push(&JsValue::from_str(WORKLET));
                    let options = Object::new();
                    assign(&options, "type", &JsValue::from_str("text/javascript"))?;
                    let blob = new_object("Blob", &[parts.into(), options.into()])?;
                    let url_api = property(&js_sys::global(), "URL")?;
                    let module_url = invoke(&url_api, "createObjectURL", &[blob])?
                        .as_string()
                        .ok_or_else(|| JsValue::from_str("Could not create audio worklet"))?;
                    let setup_module = async {
                        let audio_worklet = property(&context, "audioWorklet")?;
                        wait(invoke(
                            &audio_worklet,
                            "addModule",
                            &[JsValue::from_str(&module_url)],
                        )?)
                        .await?;
                        let worklet = new_object(
                            "AudioWorkletNode",
                            &[context.clone(), JsValue::from_str("zipeek-live-capture")],
                        )?;
                        let source =
                            invoke(&context, "createMediaStreamSource", &[stream.clone()])?;
                        let sink = invoke(&context, "createGain", &[])?;
                        assign(&property(&sink, "gain")?, "value", &JsValue::from_f64(0.0))?;
                        invoke(&source, "connect", &[worklet.clone()])?;
                        invoke(&worklet, "connect", &[sink.clone()])?;
                        invoke(&sink, "connect", &[property(&context, "destination")?])?;
                        let port = property(&worklet, "port")?;
                        let streaming = Rc::new(Cell::new(false));
                        let finishing = Rc::new(Cell::new(false));
                        let on_audio = {
                            let socket = socket.clone();
                            let stream_for_stop = stream.clone();
                            let context_for_stop = context.clone();
                            let packetizer = packetizer.clone();
                            let streaming = streaming.clone();
                            let on_event = on_event.clone();
                            Closure::<dyn FnMut(JsValue)>::new(move |event| {
                                if !streaming.get() {
                                    return;
                                }
                                let result = (|| {
                                    let value = property(&event, "data")?;
                                    if value.as_string().as_deref() == Some("finished") {
                                        let mut audio = packetizer.borrow_mut();
                                        drain(&socket, &mut audio, true)?;
                                        send(&socket, &[2])?;
                                        streaming.set(false);
                                        stop_tracks(&stream_for_stop);
                                        let _ = invoke(&context_for_stop, "close", &[]);
                                        return Ok::<_, JsValue>(());
                                    }
                                    let mut audio = packetizer.borrow_mut();
                                    let samples = Float32Array::new(&value).to_vec();
                                    let chunk = (samples.len() / 32).max(1);
                                    let peaks = samples
                                        .chunks(chunk)
                                        .take(32)
                                        .map(|slice| {
                                            let rms = (slice.iter().map(|x| x * x).sum::<f32>()
                                                / slice.len() as f32)
                                                .sqrt();
                                            if rms < 0.007 {
                                                0.0
                                            } else {
                                                (rms * 4.0).clamp(0.0, 1.0)
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    on_event(LiveEvent::Waveform(peaks));
                                    audio.append(&samples);
                                    drain(&socket, &mut audio, false)
                                })();
                                if result.is_err() {
                                    streaming.set(false);
                                    stop_tracks(&stream_for_stop);
                                    let _ = invoke(&context_for_stop, "close", &[]);
                                    on_event(LiveEvent::Error(
                                        "Audio streaming interrupted; try again".into(),
                                    ));
                                }
                            })
                        };
                        assign(&port, "onmessage", on_audio.as_ref())?;
                        let final_seen = Rc::new(Cell::new(false));
                        let on_message = {
                            let stream_for_final = stream.clone();
                            let context_for_final = context.clone();
                            let final_seen = final_seen.clone();
                            let on_event = on_event.clone();
                            let streaming = streaming.clone();
                            Closure::<dyn FnMut(JsValue)>::new(move |event| {
                                let result =
                                    property(&event, "data").and_then(|data| message(&data));
                                match result {
                                    Ok(LiveEvent::Ready) => {
                                        streaming.set(true);
                                        on_event(LiveEvent::Ready);
                                    }
                                    Ok(LiveEvent::Final(text)) => {
                                        final_seen.set(true);
                                        streaming.set(false);
                                        stop_tracks(&stream_for_final);
                                        let _ = invoke(&context_for_final, "close", &[]);
                                        on_event(LiveEvent::Final(text));
                                    }
                                    Ok(event) => on_event(event),
                                    Err(_) => on_event(LiveEvent::Error(
                                        "Unexpected speech protocol message".into(),
                                    )),
                                }
                            })
                        };
                        assign(&socket, "onmessage", on_message.as_ref())?;
                        let on_close = {
                            let stream_for_close = stream.clone();
                            let context_for_close = context.clone();
                            let final_seen = final_seen.clone();
                            let on_event = on_event.clone();
                            let streaming = streaming.clone();
                            Closure::<dyn FnMut(JsValue)>::new(move |_| {
                                streaming.set(false);
                                stop_tracks(&stream_for_close);
                                let _ = invoke(&context_for_close, "close", &[]);
                                if !final_seen.get() {
                                    on_event(LiveEvent::Closed);
                                }
                            })
                        };
                        assign(&socket, "onclose", on_close.as_ref())?;
                        let on_error = {
                            let on_event = on_event.clone();
                            Closure::<dyn FnMut(JsValue)>::new(move |_| {
                                on_event(LiveEvent::Error("Live speech connection failed".into()))
                            })
                        };
                        assign(&socket, "onerror", on_error.as_ref())?;
                        send(&socket, &[1, 0x80, 0x3e, 0, 0])?;
                        Ok::<_, JsValue>(Self {
                            socket: socket.clone(),
                            stream: stream.clone(),
                            context: context.clone(),
                            source,
                            worklet,
                            sink,
                            module_url: module_url.clone(),
                            packetizer,
                            streaming,
                            finishing,
                            on_audio,
                            on_message,
                            on_close,
                            on_error,
                        })
                    }
                    .await;
                    if setup_module.is_err() {
                        let _ = invoke(
                            &url_api,
                            "revokeObjectURL",
                            &[JsValue::from_str(&module_url)],
                        );
                    }
                    setup_module
                }
                .await;
                if context_setup.is_err() {
                    let _ = invoke(&context, "close", &[]);
                }
                context_setup
            }
            .await;
            if setup.is_err() {
                stop_tracks(&stream);
            }
            setup
        }
        .await;
        if create.is_err() {
            let _ = invoke(&socket, "close", &[]);
        }
        create
    }

    pub fn finish(&self) -> Result<(), JsValue> {
        if !self.streaming.get() || self.finishing.replace(true) {
            return Err(JsValue::from_str("Live microphone is not ready"));
        }
        let port = property(&self.worklet, "port")?;
        invoke(&port, "postMessage", &[JsValue::from_str("finish")])?;
        Ok(())
    }
}

impl Drop for LiveConnection {
    fn drop(&mut self) {
        self.streaming.set(false);
        let _ = assign(&self.socket, "onmessage", &JsValue::NULL);
        let _ = assign(&self.socket, "onclose", &JsValue::NULL);
        let _ = assign(&self.socket, "onerror", &JsValue::NULL);
        if let Ok(port) = property(&self.worklet, "port") {
            let _ = assign(&port, "onmessage", &JsValue::NULL);
        }
        let _ = invoke(&self.source, "disconnect", &[]);
        let _ = invoke(&self.worklet, "disconnect", &[]);
        let _ = invoke(&self.sink, "disconnect", &[]);
        stop_tracks(&self.stream);
        let _ = invoke(&self.context, "close", &[]);
        let _ = invoke(&self.socket, "close", &[]);
        if let Ok(api) = property(&js_sys::global(), "URL") {
            let _ = invoke(
                &api,
                "revokeObjectURL",
                &[JsValue::from_str(&self.module_url)],
            );
        }
        let _ = (
            &self.on_audio,
            &self.on_message,
            &self.on_close,
            &self.on_error,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packetizer_bounds_frame_sizes_and_preserves_duration() {
        let mut packetizer = Packetizer::new(48_000.0).unwrap();
        packetizer.append(&vec![0.25; 48_000]);
        let mut bytes = 0;
        while let Some(packet) = packetizer.take_packet(true) {
            assert_eq!(packet[0], 3);
            assert!(packet.len() <= 3_201);
            bytes += packet.len() - 1;
        }
        assert!((bytes / 2).abs_diff(16_000) <= 1);
    }
}
