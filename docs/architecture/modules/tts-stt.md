# Local Mandarin speech service

`zipeek-tts-stt` is a native, loopback-only speech process. It does not expose a public network listener, ship eSpeak/Whisper binaries or models, or automatically enable a browser provider. The approved D56 browser/local provider selection and microphone UX remain separate integration work.

## Runtime

Start with `cargo run --locked -p zipeek-tts-stt`. The listener binds to `127.0.0.1:8799`; `ZIPEEK_SPEECH_PORT` may select another nonzero TCP port. Network-facing bind addresses are intentionally not configurable.

Resource paths can be selected with `ZIPEEK_ESPEAK_BIN`, `ZIPEEK_ESPEAK_DATA`, and `ZIPEEK_WHISPER_MODEL`. Otherwise resources are located beside the executable under `resources/`. The `espeak-ng-data` directory must reside under the configured data root. Whisper must be a multilingual model. The service must not silently install/download third-party models. Verify their licenses separately.

Native HTTP endpoints:

| Endpoint | Method | Request | Response |
| --- | --- | --- | --- |
| `/health` | GET | None | `ok` |
| `/v1/capabilities` | GET | None | Plain-text availability flags and `zh-CN` |
| `/v1/tts` | POST | `text/plain; charset=utf-8`, max 4 KiB | `audio/wav` |
| `/v1/stt` | POST | `audio/wav`, mono 16 kHz PCM16, max 30 s | UTF-8 transcript |

The speech engine uses eSpeak NG Mandarin (`cmn`) and Whisper language `zh`, with translation disabled. Requests do not award pronunciation mastery. Each engine admits one job at a time and returns HTTP 429 while busy. Missing resources return HTTP 503. Malformed input returns HTTP 400. Engine errors return HTTP 500. WAV is parsed and strictly bounded before model loading. Text and audio are neither logged nor persisted.

Native speech POST requests require a `Host` header naming `localhost` or `127.0.0.1` (with an optional nonzero port) and do not permit requests carrying a browser Origin header. It is **not** an approved web-audio endpoint or a remote transport. A future trusted desktop bridge must perform microphone permission handling, provider selection, interruption handling, and source-origin enforcement before exposing the capability to frontend consumers. Never connect it directly to a hosted browser page without an approved security design.

## Verification

Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, and `cargo test --locked -p zipeek-tts-stt`. Runtime acceptance additionally requires licensed local eSpeak NG assets, a multilingual Whisper model, a Mandarin audio fixture, and an explicit reviewer check that the service binds only to loopback. Tests and runtime acceptance were not run in the archive-authoring environment.

## R534 native protocol hardening

The POST endpoints reject non-loopback `Host` headers, malformed or missing hosts, and any browser `Origin` header before engine invocation. This is a defense against cross-site browser access and misleading Host routing, not a substitute for desktop-bridge authentication. Do not weaken these checks to make hosted Pages clients call the native service.

Synthesis now fails promptly when a child process emits output beyond the configured bound. The process is terminated and reaped on any failed or timed-out synthesis operation rather than waiting for a blocked stdout pipe. Tests cover malformed audio framing and HTTP handler error statuses without requiring external speech models.

## R536 trusted local development bridge

The browser bridge is intentionally restricted to the local Zipeek development origin
`http://localhost:1420` or `http://127.0.0.1:1420`. The process still binds to loopback only.
Remote hosting, other local ports and cross-origin web pages are rejected. An exact-origin
CORS preflight allows only POST and the Content-Type header for `/v1/tts` and `/v1/stt`.
The browser never receives credentials; local-provider selection is an explicit, in-memory
choice in Settings and resets on page reload. Browser speech remains the default.

The bridge transmits text to local eSpeak and samples from a five-second microphone
capture as mono 16 kHz PCM16 WAV to local Whisper. Audio is not persisted. Recording
is initiated explicitly, and tracks are stopped after capture, cancellation or errors.
The backend validates WAV. Hosted `/zipeek/` cannot use this bridge. This remains a
**development-only** connection, not a general desktop security boundary.

Run `cargo run --locked -p zipeek-tts-stt`, open `http://localhost:1420`,
choose Local Mandarin speech in Settings, and confirm capabilities before tests.
This integration requires Windows/browser acceptance and review of browser private
network access behavior, microphone capture, cancellation, and audio playback.

Existing practice recordings may be explicitly transcribed with **Transcribe locally**
when the user has opted in to the local speech provider. This decodes the
browser's in-memory blob, mixes up to two channels, resamples to mono 16 kHz
PCM16 and sends only to the loopback service. A transcript does not award
pronunciation mastery. Existing recording/playback controls remain unchanged.

## R538 local browser speech Rust/WASM migration

The browser-side speech bridge is now implemented in `zipeek-browser/src/local_speech/` using `wasm-bindgen`, `js-sys` and browser APIs, with standalone PCM16 WAV encoding and native unit tests in `zipeek-browser/src/speech_wav.rs`. `zipeek-web` Settings, the shared speech service and Speaking Recorder call this Rust API directly instead of the old JavaScript module. Browser speech remains the default, and local service use requires explicit in-memory opt-in from exactly `http://localhost:1420` or `http://127.0.0.1:1420`.

The new browser capture path uses MediaRecorder followed by AudioContext decoding and Rust channel mixing/resampling. Object URLs, microphone tracks and abort controllers are owned by the Rust state. Hosted origins remain blocked; no credentials, recorded audio or transcript history are persisted. This remains development-only, and needs Chrome/Firefox microphone permission, cancellation and engine integration acceptance before release.
