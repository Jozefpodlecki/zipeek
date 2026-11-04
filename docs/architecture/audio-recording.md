# Speaking capture, waveform and contextual targets (R66)

**Status:** browser microphone capture, live amplitude visualization, local playback,
and discard are implemented on the experimental `#/practice/speaking` route.
No pronunciation assessment, persistence, or upload exists. The
`zipeek-exercises::definition::SPEAKING` contract is reserved for the future scored exercise.

## Domain boundaries

- `zipeek-exercises`: speaking exercise definition/instance and authoritative result;
  pronunciation evidence must identify the intended reading, context, and
  what was actually assessed.
- `crates/web`: permission request, capture lifecycle, accessible controls,
  live visualization, cancellation, release of microphone tracks and browser
  compatibility errors. Keep microphone samples and audio blobs out of the
  exercise/session domain model.
- Future speech-assessment adapter: explicit opt-in upload/transcription and
  pronunciation feedback; do not score pronunciation based on waveform height.

## Implemented browser boundary

- `crates/web/src/services/microphone.js` owns the `getUserMedia`, `AudioContext`,
  `AnalyserNode`, `MediaRecorder`, timer, track and object-URL lifecycle. It accepts
  callbacks and holds no Yew or AI dependency. The JS module is imported by the
  Rust WebAssembly component using `wasm_bindgen`'s local-module binding.
- `crates/web/src/components/speaking_recorder.rs` owns browser UI controls,
  status/errors, the recording's current playback URL, and lifecycle disposal.
  It feeds samples into the existing `AudioWaveform` pure renderer.
- `zipeek-exercises/src/speaking.rs` defines validated, unambiguous
  speaking targets and produces exercise instances with learner-visible input.
  It contains no browser audio or scoring code.
- `crates/web/src/pages/speaking.rs` offers three curated speaking targets with
  explicit pinyin and focus-word readings (including polyphonic `行` and `乐`).
  Switching targets keys/remounts the recorder so its cleanup releases the
  previous capture and playback URL. The preview does not require DuckDB
  readiness and is not a graded quiz session.
- `scripts/microphone-recorder.test.cjs` simulates browser permission, capture,
  recording, disposal, and permission denial without requiring hardware.

## Microphone-to-waveform pipeline

1. Ask for microphone access only after pressing Record; support permission
   denial, unavailable devices, insecure origins and concurrent recording.
2. Attach the `MediaStream` to a Web Audio `AudioContext` and `AnalyserNode`
   with a small FFT window. Sample time-domain data with
   `getFloatTimeDomainData`; calculate windowed RMS or peak amplitude.
3. At `requestAnimationFrame` cadence, append an amplitude observation to a
   bounded ring buffer. Render vertical bars centered on a baseline, with
   approximate 20–40 updates per second. This is an **amplitude waveform**,
   not an FFT/spectrogram (frequency visualization is different).
4. Capture audio separately through `MediaRecorder`, with runtime MIME support
   detection and a bounded recording duration. `MediaRecorder` and Web Audio
   may share the same `MediaStream`, but their lifecycles are independent.
5. On Stop, navigate-away or component unmount, stop media tracks and release
   the recorder/audio context/animation frame. Do not assume cancelling a
   visualizer also stops a microphone recording.
6. Preserve the last amplitude bars with browser-native playback controls after
   recording. A player may seek through the captured audio; amplitude bars are
   purely presentational and must not be treated as a measure of speaking skill.
   Elapsed-time display and waveform seeking are follow-up tasks.

## UI and accessibility

A SoundCloud-like row of centered vertical bars can visualize the user's
speech. Provide visible Record, Stop, Play and Discard controls plus elapsed
recording time and a textual status for screen readers. Use a neutral idle
waveform, respect reduced motion, display a clear recording/permission state,
and provide a fallback when Web Audio is unsupported. Always distinguish
**recorded** from **evaluated**; never imply that a decorative waveform is an
assessment. Avoid autoplay and automatic cloud uploads.

## API / dependency policy

No Rust crate, npm package, `web-sys` feature flag, or dependency pin was added.
The JavaScript adapter uses built-in browser APIs and is imported through the
already approved `wasm-bindgen` dependency. Chromium browser and wasm32
compilation still require local verification. Extract a separate audio crate
only if reusable signal processing and capture-domain contracts warrant it;
`zipeek-exercises` must not depend on microphone APIs.

`crates/web/src/components/audio_waveform.rs` remains a presentational 40-bar
SVG renderer with clamped amplitude input. Microphone samples are never stored
as vocabulary proficiency evidence, and there is no implicit speech upload.
