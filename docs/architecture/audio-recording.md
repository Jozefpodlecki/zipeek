# Speaking capture, waveform and contextual targets

**Status:** browser microphone capture, live amplitude visualization, local playback,
and discard are implemented on the experimental `#/practice/speaking` route.
No pronunciation assessment, persistence, or upload exists. The
`zipeek-exercises::definition::SPEAKING` contract is reserved for the future scored exercise.

## Domain boundaries

- `zipeek-exercises`: speaking exercise definition/instance and future pronunciation evidence; a waveform is not a pronunciation score.
- `zipeek-browser/src/microphone/`: browser permission, capture, MediaRecorder, Web Audio analysis, 30-second limit, playback URL, generation checks and release of tracks. This is Rust using `web-sys`; no JavaScript recorder module remains.
- `zipeek-web/src/components/audio/speaking_recorder/`: Yew controls, status, errors, playback UI and callback lifetimes. The bindings adapter passes callback functions to `zipeek_browser::microphone::MicrophoneRecorder` and disposes it on unmount.
- Future opt-in speech assessment is outside this local-only recorder.

The recorder is a single-field tuple struct wrapping shared Rust lifecycle state.
The Rust implementation is divided into `recorder.rs` (public API), `state.rs`
(resource ownership and callbacks), `capture.rs` (MediaRecorder and permission
flow), `waveform.rs` (frame sampling), and `error.rs` (browser error mapping).
`mod.rs` exports the recorder without implementing all of those concerns.
The lifecycle phase is a typed `RecordingPhase` enum internally, converted to
legacy status strings only when invoking the Yew callback.
`BlobEvent::data()` returns `Option<Blob>` in the pinned `web-sys`; missing or
empty blobs are ignored rather than unwrapped before adding chunks. It retains the original `idle`, `requesting`, `recording`, `processing`, `ready` and `error` states. The permission request starts only from a user gesture. Late permission grants are rejected using an increasing generation token; tracks from obsolete grants are stopped immediately. Unmount/discard releases MediaRecorder handlers, audio context, animation frame, deadline and object URL.

The previous `crates/zipeek-web/src/services/microphone.js` has been removed. `scripts/translation.test.cjs` asserts the Rust contracts; actual permission denial, capture and stop/discard behavior still need browser validation.

## Microphone-to-waveform pipeline

1. Ask for microphone access only after pressing Record; support permission
   denial, unavailable devices, insecure origins and concurrent recording.
2. Attach the `MediaStream` to a Web Audio `AudioContext` and `AnalyserNode`
   with a small FFT window. Sample time-domain data with
   `getFloatTimeDomainData`; calculate windowed RMS or peak amplitude.
3. At `requestAnimationFrame` cadence, append an amplitude observation to a
   bounded ring buffer. Render vertical bars centered on a baseline, with
   approximate 20-40 updates per second. This is an **amplitude waveform**,
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
The Rust browser adapter uses `web-sys` browser APIs and the existing
`wasm-bindgen` dependency. Chromium browser and wasm32
compilation still require local verification. Extract a separate audio crate
only if reusable signal processing and capture-domain contracts warrant it;
`zipeek-exercises` must not depend on microphone APIs.

`crates/zipeek-web/src/components/audio/audio_waveform.rs` remains a presentational 40-bar
SVG renderer with clamped amplitude input. Microphone samples are never stored
as vocabulary proficiency evidence, and there is no implicit speech upload.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../archive-history/audits/maintained-source-snapshots/architecture-audio-recording.md). Dated implementation claims in that snapshot are not current build or browser verification.
