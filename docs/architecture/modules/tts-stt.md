## R661 WIP: Persistent native inference workers

The `speech_workers` module now owns two independent thread-backed inference loops. Each holds an optional persistent Candle engine, accepts bounded requests through Tokio MPSC, and responds through oneshot channels. The HTTP-facing `Speech` coordinator retains the eSpeak subprocess fallback, WAV processing and the two capacity semaphores, but no longer owns mutable neural models or locks over inference. A queued or running request transfers its owned permit to the worker, preventing premature admission after an HTTP timeout. Dropped requests use cooperative cancellation; the worker does not promise preemptive interruption of a GPU kernel.

The neural model status machine distinguishes not installed, installed, loading, warming, ready and failed, using an atomic snapshot for nonblocking capabilities reads. TTS startup still runs actual `你好` synthesis; ASR startup loads weights without a waveform forward pass. The capabilities boolean is true only when a neural model is ready, or when the selected eSpeak fallback is usable in the absence of complete neural assets. Checkpoint presence remains separately queryable. Model failure invalidates the worker's cached engine and a later neural request can attempt reinitialization. Device names are validated at service construction. Backend error details are logged but not returned verbatim to HTTP clients.

This is authored, uncompiled R661 source, not Windows/CUDA acceptance. In particular, confirm backend failure recovery and shutdown with actual checkpoints before claiming production reliability. The earlier R660 sampling-path behavior was not intentionally modified.

## R637 Zipeek-owned Qwen3-TTS acoustic-code and waveform inference

R637 replaces the former placeholder with a native Candle source path for Mandarin CustomVoice/Vivian speech synthesis. The Qwen3-TTS talker now runs a bounded greedy acoustic-code generation loop; the five-layer predictor emits 15 residual codebooks per frame, and the waveform decoder reconstructs all 16 quantizers, runs the published eight-layer sliding-window codec Transformer and causal convolutional upsampling/vocoder network. Its synthesis endpoint is designed to return 24 kHz mono PCM16 WAV, with the existing Opus response negotiation unchanged. No external neural TTS crate, system eSpeak dependency or additional Rust dependency is required by this neural code path. Existing eSpeak fallback remains separately available when model assets are absent.

This is **unverified source**, not proven audio synthesis: Cargo/rustfmt and the local 1.8 GB Qwen3-TTS weights are not available in the authoring environment. `NEURAL_WAVEFORM_DECODER_READY` selects the implementation when local model assets exist, but asserts **architecture wiring**, not runtime readiness or quality. The ignored `neural_tts.rs` test must execute with downloaded model files and save a non-silent `target/speech-e2e/mandarin_qwen3_tts.wav` before claiming working TTS. The codec mapping and output must be checked numerically against the upstream tokenizer; an inference error must not be reported as success. Text input support remains limited to uninterrupted Mandarin Hanzi, not general-purpose segmentation or full Unicode tokenization. GPU memory, latency, caching and reference-speech accuracy are not accepted.

The previous R634 narrative below describes the historical, now-removed `talechime-qwen3-tts` integration. It is not the current implementation or dependency.

## R634 native Qwen3-TTS CustomVoice synthesis integration

When the complete local Qwen3-TTS-12Hz-0.6B-CustomVoice directory is available under `models/speech`, `/v1/tts` now selects a native Rust/Candle inference backend: Qwen text tokenization, autoregressive talker, acoustic code predictor and neural audio-token decoder. It uses Mandarin (`Language::Chinese`) and preset speaker Vivian, converts the resulting finite non-silent 24 kHz mono waveform to PCM16 WAV, and keeps the existing Ogg Opus HTTP response negotiation. `ZIPEEK_QWEN_TTS_MODEL` accepts a checkpoint override; `ZIPEEK_TTS_DEVICE` accepts `cpu` (default) or `cuda` when the `qwen-tts-cuda` feature is built. Inference failures return explicit errors rather than silently falling back. eSpeak NG remains an optional fallback **only if** complete Qwen3-TTS assets are absent. Kokoro native inference remains a separate unfinished backend.

This integration introduces pinned `talechime-qwen3-tts =0.4.1` and a feature-gated CUDA path. Dependency approval and root lockfile resolution remain blocking before release acceptance, and Rust compilation, actual model output and spoken Mandarin quality have not been validated in the authoring environment. The `neural_tts.rs` ignored acceptance test must save a non-silent, 24 kHz waveform from real weights before the neural backend is considered working. Long-running blocking synthesis is held behind a single-job semaphore. In this checkpoint the large model is loaded for each request; caching, cancellation, memory limits and warm-up performance require subsequent runtime work.

## R630 pure-Rust audio codec integration

The native speech server now has an `audio_codec` module. It decodes MP3/FLAC with pinned Symphonia and Ogg Opus with pinned `opus-pure`, without linking native codec libraries. Decoded material is downmixed to mono, checked for finite samples and bounded to 30 seconds, then linearly resampled to the 16 kHz PCM required by Qwen3-ASR. RIFF PCM16 WAV remains the strict canonical regression and model-input format. Input `Content-Type` is required and explicitly allowlisted: `audio/wav`, `audio/x-wav`, `audio/mpeg`, `audio/flac`, `audio/x-flac`, `audio/ogg`, or `audio/opus`.

`POST /v1/tts` retains its PCM16 WAV default. Clients can request `Accept: audio/ogg` to obtain a mono 48 kHz Ogg Opus file generated using the pure-Rust `opus-pure` encoder with 24 kb/s voice settings, actual encoder pre-skip and end-trim. This is compression of the existing eSpeak output; it does not implement Kokoro or Qwen3-TTS. Unsupported explicit Accept values receive HTTP 406. There are no FFI codec bindings. For continuous work, the current linear resampler should be evaluated against a bandlimited resampler before high-fidelity audio requirements are considered accepted.

The new crates must be resolved into the existing workspace `Cargo.lock` on a machine with Cargo and crates.io access before `cargo --locked` can work. The authoring environment lacks Cargo and internet package resolution, so the source is a WIP checkpoint, not a compiled/locked release. Run `cargo check -p zipeek-tts-stt` once to update the root lockfile, then `cargo test --locked -p zipeek-tts-stt --test audio_codecs`, and the existing suite. Run the strict real speech tests independently. Existing WAV fixtures are unchanged; new small MP3/FLAC fixtures test decoding only, not lexical transcription correctness.

## R625 acoustic encoder implementation scope

R625 adds `candle_stt/encoder.rs`, with the published 0.6B audio tower's 18 noncausal Transformer layers, 14 attention heads (64 channels each), 3,584-channel GELU feed-forward path, pre-attention and pre-FFN LayerNorm, post-encoder LayerNorm and a two-stage 896-to-1,024 acoustic projector. The existing 100-Mel-frame convolution chunks feed independent eight-second attention windows; no attention crosses those window boundaries. The exact weight prefixes follow `thinker.audio_tower` and the upstream encoder layers. A checkpoint-backed ignored Rust test provides an explicit opt-in path to exercise the complete encoder on CPU. The normal library/test startup does not load weights or enable STT.

The user supplied an R624 Windows log with 44 passing Rust unit tests. This establishes R624 compile/test success, not R625 or actual weight execution. Numerical parity, tokenizer, Qwen3 text decoder and Mandarin transcription remain pending. Qwen3-TTS asset validation now requires the codec's checkpoint, configuration and preprocessing metadata, but actual TTS remains eSpeak NG; Kokoro's PyTorch format remains a separate conversion/loader requirement. No model downloads or dependencies were added.

## R619 approved Candle-native speech architecture

Candle (`candle-core`, `candle-nn`, `candle-transformers`, `candle-kernels`, pinned to 0.11.1) remains the neural execution stack. `whisper-rs` and whisper.cpp are not permitted. The primary Mandarin STT research target is **Qwen/Qwen3-ASR-0.6B** (audio encoder plus language-model decoder); SenseVoiceSmall is an evaluation candidate, not a second production backend. The practical first Mandarin TTS target is **hexgrad/Kokoro-82M-v1.1-zh**; **Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice** is a later neural codec-based TTS research target. Each model requires a distinct architecture and correct text/audio preprocessing. R620 updates the audio frontend to Qwen3-ASR's published 128-band, 16 kHz, 400-point FFT and 160-sample hop contract; coefficient parity, padding/masking semantics and encoder integration are still unverified.

`src/runtime/device.rs` supplies a CPU/CUDA Candle initialization and small tensor-operation check. `src/runtime/models.rs` identifies chosen published model repositories and verifies Qwen TTS assets. `src/candle_stt/assets.rs` validates the Qwen3-ASR distribution layout; `src/candle_tts/assets.rs` validates the **actual upstream Kokoro distribution** (`kokoro-v1_1-zh.pth`, `config.json`, `voices/zf_007.pt`). The published Kokoro `.pth` and `.pt` files are PyTorch serialized assets, **not** ready-to-load Candle safetensors. Conversion or a native compatible loader, verified against trusted weight names/shapes and reference outputs, is a separate milestone. Never load untrusted pickle files in the application or assume an arbitrary file rename converts their format.

R621 replaces fixed-length input with a 128-band F32 tensor padded to 100-Mel-frame multiples (up to `[1, 128, 3000]`); `candle_stt/weights.rs` probes the Qwen3-ASR safetensors checkpoint using memory-mapped Candle `VarBuilder`. R622 adds a three-Conv2D projection stem. R624 runs convolution over each complete zero-padded 100-frame chunk and adds non-interleaved sinusoidal positions resetting at each chunk before trimming invalid output tokens. Neither checkpoint tensor-shape verification nor numerical parity with upstream is complete; there is still no end-to-end encoder/decoder. `candle_tts/waveform.rs` prepares finite bounded neural PCM samples as RIFF PCM16 WAV, but it is not neural waveform generation. These new primitives do not change the exposed endpoint readiness flags. Only load model assets from trusted local sources.

The model registry is preflight/architecture groundwork only. No complete Qwen3-ASR encoder/decoder, Qwen3-TTS audio-token decoder, Mandarin Kokoro synthesis, pronunciation scoring or automatic model downloading is implemented. `/v1/stt` remains unavailable and `/v1/tts` still uses eSpeak NG. Do not use successful asset checks or a Candle tensor allocation as evidence of usable speech inference.

### Model downloads (external to source archives)

Recommended local directories beneath the native executable's `resources/` tree:

- `qwen3-asr-0.6b/`: snapshot of `https://huggingface.co/Qwen/Qwen3-ASR-0.6B` including `model.safetensors`, `config.json`, `generation_config.json`, `preprocessor_config.json`, `vocab.json`, `merges.txt` and tokenizer metadata. This is the primary STT checkpoint.
- `kokoro-zh/`: snapshot of `https://huggingface.co/hexgrad/Kokoro-82M-v1.1-zh` including `kokoro-v1_1-zh.pth`, `config.json`, and the `voices/` directory, at least `zf_007.pt`. This is the first TTS target. The upstream checkpoint format needs a safe conversion/load strategy before any Candle inference.
- `qwen3-tts-0.6b/`: optional later snapshot of `https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice` including the entire `speech_tokenizer/` subdirectory and root safetensors/config/tokenizer files. It is **not** required for the current milestone.

Snapshot whole repositories using `hf download <repository> --local-dir <directory>` from the installed Hugging Face CLI, rather than choosing a single weights file. Model directories are local runtime assets; do not put several gigabytes of weights in the project source ZIP. Review each repository's licence and download a pinned revision for reproducible development.

`ZIPEEK_CANDLE_STT_MODEL` defaults to the Qwen model path under the native executable's `resources/`. `ZIPEEK_CANDLE_TTS_MODEL` selects the Chinese Kokoro checkpoint; its current default is a project-relative path. `ZIPEEK_CANDLE_DEVICE=cpu|cuda` is recognized by the new explicit runtime device API, but no model request yet invokes it. GPU errors should be reported, never silently treated as successful CUDA execution.

### Implementation stages

1. Validate local snapshots and choose/initialize a Candle device without loading untrusted Python-pickle assets.
2. Port Qwen3-ASR-specific audio frontend, feature shapes, weight naming, encoder, tokenization and incremental decoder with reference parity tests on Mandarin samples. Keep speech transcription separate from tone/pronunciation assessment.
3. Build the Kokoro Chinese phonemizer (word segmentation, polyphonic readings, tone sandhi), establish a verified checkpoint-conversion workflow, port neural model/voice/vocoder and synthesize PCM WAV with reference parity tests.
4. Investigate Qwen3-TTS neural speech-token generation and `speech_tokenizer` codec as an alternative later backend. Benchmark latency and peak VRAM on the user's 16 GB GPU before selecting a production configuration.
5. Enable native endpoints only after locked Rust compilation, actual model inference, Mandarin audio correctness, resource and security checks and native loopback acceptance.

## R615 Candle transition status

`whisper-rs` was explicitly rejected. The current source no longer imports Whisper.rs or expects a GGML Whisper.cpp checkpoint. `ZIPEEK_CANDLE_STT_MODEL` selects the Qwen3-ASR safetensors checkpoint under `resources/qwen3-asr-0.6b/`; asset validation and the explicit mapping probe do not establish inference readiness. There is no Candle inference pipeline yet, so `/v1/stt` returns a transparent service-unavailable error even when a checkpoint file is present. WAV input validation remains enabled. Do not advertise a speech-recognition capability until the Candle frontend, encoder/decoder, tokenizer and inference have been implemented and evaluated.

TTS still runs through the pre-existing eSpeak NG native adapter as a temporary development path; Candle Mandarin TTS has not yet replaced it. The four Candle 0.11.1/CUDA dependencies already existed in the manifest. No new crates or model binaries were added. The speech model architecture, checkpoints, Mandarin text processing, licensing, CUDA build support and real inference acceptance remain open implementation tasks.

## R618 native Candle research checkpoint

The native speech crate now contains a standalone Whisper-compatible *candidate* audio frontend at `src/candle_stt/frontend.rs`, with bounded 16 kHz mono PCM input, a 400-point radix-2/5 Fourier transform, periodic Hann window, 160-sample hop, Slaney-style triangular Mel filters, 80-by-3000 output and log-energy normalization. `speech::prepare_stt_features` connects strict WAV decoding to this frontend for model integration. The implementation includes deterministic numerical and input-validation tests, but has **not** been compared to a reference Whisper feature extractor on real Mandarin audio. Such comparison is required before it can be treated as numerically compatible with the selected model. This is not transcription or a pronunciation assessor.

`src/candle_tts/` adds two separate prerequisites for a future Chinese Kokoro implementation: bounded Mandarin punctuation-aware segmentation, and a local safetensors model/voice/vocabulary asset contract. These are experimental interfaces, **not** evidence that a published Kokoro Chinese checkpoint has exactly those files or that it is loadable by the current code. Real model loading, checkpoint conversion where necessary, text-to-phoneme mapping, tone sandhi, voice selection, prosody/acoustic inference and waveform synthesis remain unimplemented. The live `/v1/tts` service continues to use eSpeak NG.

The `ZIPEEK_CANDLE_TTS_MODEL` environment variable can select the experimental checkpoint path for asset diagnostics. Neither model weights nor voices are bundled, and no downloads occur at runtime. The current CUDA-only Candle dependency selection is unchanged.

Native verification remains required: `cargo check --locked -p zipeek-tts-stt`, `cargo test --locked -p zipeek-tts-stt`, comparison of log-Mel coefficients against a reference Whisper implementation, and native Mandarin speech/model tests. Do not advertise `/v1/stt` as available based on asset checks or successful frontend numerical tests alone.

# Local Mandarin speech service

`zipeek-tts-stt` is a native, loopback-only speech process. It does not expose a public network listener, ship eSpeak/Whisper binaries or models, or automatically enable a browser provider. The approved D56 browser/local provider selection and microphone UX remain separate integration work.

## Runtime

Start with `cargo run --locked -p zipeek-tts-stt`. The listener binds to `127.0.0.1:8799`; `ZIPEEK_SPEECH_PORT` may select another nonzero TCP port. Network-facing bind addresses are intentionally not configurable.

Resource paths can be selected with `ZIPEEK_ESPEAK_BIN`, `ZIPEEK_ESPEAK_DATA`, and `ZIPEEK_CANDLE_STT_MODEL`. Otherwise resources are located beside the executable under `resources/`. The `espeak-ng-data` directory must reside under the configured data root. Candle STT inference is not yet implemented. The service must not silently install/download third-party models. Verify their licenses separately.

Native HTTP endpoints:

| Endpoint | Method | Request | Response |
| --- | --- | --- | --- |
| `/health` | GET | None | `ok` |
| `/v1/capabilities` | GET | None | Plain-text availability flags and `zh-CN` |
| `/v1/tts` | POST | `text/plain; charset=utf-8`, max 4 KiB | `audio/wav` |
| `/v1/stt` | POST | `audio/wav`, mono 16 kHz PCM16, max 30 s | UTF-8 transcript |

The speech engine uses eSpeak NG Mandarin (`cmn`) for TTS; STT is currently unavailable. Requests do not award pronunciation mastery. Each engine admits one job at a time and returns HTTP 429 while busy. Missing resources return HTTP 503. Malformed input returns HTTP 400. Engine errors return HTTP 500. WAV is parsed and strictly bounded before model loading. Text and audio are neither logged nor persisted.

Native speech POST requests require a `Host` header naming `localhost` or `127.0.0.1` (with an optional nonzero port) and do not permit requests carrying a browser Origin header. It is **not** an approved web-audio endpoint or a remote transport. A future trusted desktop bridge must perform microphone permission handling, provider selection, interruption handling, and source-origin enforcement before exposing the capability to frontend consumers. Never connect it directly to a hosted browser page without an approved security design.

## Verification

Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, and `cargo test --locked -p zipeek-tts-stt`. Runtime acceptance additionally requires licensed local eSpeak NG assets, a Candle model implementation and checkpoint, a Mandarin audio fixture, and an explicit reviewer check that the service binds only to loopback. Tests and runtime acceptance were not run in the archive-authoring environment.

## R534 native protocol hardening

The POST endpoints reject non-loopback `Host` headers, malformed or missing hosts, and any browser `Origin` header before engine invocation. This is a defense against cross-site browser access and misleading Host routing, not a substitute for desktop-bridge authentication. Do not weaken these checks to make hosted Pages clients call the native service.

Synthesis now fails promptly when a child process emits output beyond the configured bound. The process is terminated and reaped on any failed or timed-out synthesis operation rather than waiting for a blocked stdout pipe. Tests cover malformed audio framing and HTTP handler error statuses without requiring external speech models.

## R536 trusted local development bridge

The browser bridge is intentionally restricted to the local Zipeek development origin
`http://localhost:1420`, `http://localhost.:1420` or `http://127.0.0.1:1420`. The process still binds to loopback only.
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

The browser-side speech bridge is now implemented in `zipeek-browser/src/local_speech/` using `wasm-bindgen`, `js-sys` and browser APIs, with standalone PCM16 WAV encoding and native unit tests in `zipeek-browser/src/speech_wav.rs`. `zipeek-web` Settings, the shared speech service and Speaking Recorder call this Rust API directly instead of the old JavaScript module. Browser speech remains the default, and local service use requires explicit in-memory opt-in from only `http://localhost:1420`, `http://localhost.:1420`, or `http://127.0.0.1:1420`. `zipeek-web` passes the origin from `AppEnvironment.window` during application setup; `zipeek-browser` caches only the trusted/untrusted decision, not an independently retrieved global Window.

The new browser capture path uses MediaRecorder followed by AudioContext decoding and Rust channel mixing/resampling. Object URLs, microphone tracks and abort controllers are owned by the Rust state. Hosted origins remain blocked; no credentials, recorded audio or transcript history are persisted. This remains development-only, and needs Chrome/Firefox microphone permission, cancellation and engine integration acceptance before release.

## R594 native client API boundary

`zipeek-tts-stt-client` is a separate dependency-free native library for the existing loopback HTTP protocol. It exposes `SpeechClient` (`health`, `capabilities`, `synthesize`, `transcribe`), typed capability and error values, a configurable nonzero port, and a bounded timeout. The transport connects only to `127.0.0.1`; it does not accept remote addresses, credentials, arbitrary URLs, or a browser Origin. HTTP response framing and content lengths are bounded, and protocol responses are checked before passing data to callers. This is a **blocking native client**; callers must not run it on a browser UI thread or within asynchronous executor tasks without an appropriate blocking boundary.

The existing `zipeek-browser/src/local_speech/` WASM microphone and browser HTTP bridge remains the browser implementation. It is not replaced by the native client, which deliberately depends on no `reqwest`, `tokio`, `web-sys`, `whisper-rs` or other external crate. The server does not require a third-party dependency change or a new network listener. The new crate has local protocol and loopback mock tests, but the actual Windows server, eSpeak engine, Whisper model and browser integrations still require runtime acceptance. No speech recording or transcript is persisted by the native client.

Build checks: `cargo test --locked -p zipeek-tts-stt-client -p zipeek-tts-stt`, `cargo check --locked -p zipeek-tts-stt-client`, and the existing speech server/Chrome acceptance matrix. These commands were not executable in the R594 authoring environment.

## R595 native client protocol verification boundary

The native `zipeek-tts-stt-client` now checks successful HTTP `Content-Type` against the endpoint contract, rejects duplicate or absent success media-type headers, and strips control characters from bounded service-error text. The client checks full WAV RIFF size, PCM16 format, mono channel count, rate and byte alignment rather than trusting magic bytes; STT uploads are restricted to mono 16 kHz and at most 30 seconds. Framing arithmetic for chunked responses is checked for overflow. Server-side validation remains authoritative. This is protocol validation of bytes returned by a loopback process, not proof of model quality or audio permissions.

Run `cargo test --locked -p zipeek-tts-stt-client -p zipeek-tts-stt` and use a real native service for GET and POST acceptance. The existing browser bridge remains a separate permission-controlled integration, and neither native model assets nor Chrome microphone tests are supplied by this checkpoint.


## Native speech client validation boundary

The native loopback speech client validates endpoint-specific HTTP Content-Type, WAV RIFF framing, PCM16 mono parameters, sample alignment, nonempty data and bounded error text. STT additionally requires 16 kHz input and no more than 30 seconds of samples. Chunked-length arithmetic is checked. This is a blocking, native `127.0.0.1` client and does not govern browser microphone permissions or make external model/runtime availability claims. Locked native tests and end-to-end audio validation remain required.

## R617 native Whisper/Candle model preflight

The local service now requires a nonempty `model.safetensors` checkpoint, `config.json`, and `tokenizer.json` alongside the checkpoint before its model-assets diagnostic reports a complete bundle. This preflight does not parse or instantiate the model, validate model/tokenizer compatibility, generate log-Mel audio features, or enable transcription. `recognition_available` remains false and the STT endpoint returns unavailable. Native Candle inference, a tested Windows CUDA toolchain and a properly verified checkpoint bundle are pending. The current eSpeak TTS path is unchanged.


## R621 Qwen3-ASR variable-length acoustic input

The STT acoustic frontend has been corrected from R620's fixed `[1, 128, 3000]` tensor to a variable-length Candle tensor `[1, 128, padded_frames]` and an accompanying `[1, padded_frames]` U32 attention mask. Audio shorter than 8,000 samples receives zero sample padding before features are extracted. The Mel time axis is right-padded to a multiple of 100 frames, without replacing the observed sample duration with an artificial 30-second input. Only valid frames participate in spectral normalization; post-feature padding is zero.

The encoder layout maps each 100-frame Mel chunk to 13 output tokens, and partitions tokens into attention windows of no more than eight chunks. `candle_stt/layout.rs` contains a deterministic, separately testable window plan. Neither the convolutional encoder nor neural decoding is integrated. These boundaries must be checked against Hugging Face reference tensor values and attention masks before implementing acoustic layers. See https://huggingface.co/docs/transformers/model_doc/qwen3_asr and https://github.com/huggingface/transformers/blob/main/src/transformers/models/qwen3_asr/feature_extraction_qwen3_asr.py.


## R622 WIP Qwen3-ASR convolutional stem

`candle_stt/stem.rs` introduces a checkpoint-backed Candle Conv2D stem for the 0.6B model: three 3x3 convolutions with stride 2, padding 1 and GELU, followed by a bias-free 7680-to-896 linear projection. The loader uses the published `thinker.audio_tower.conv2d1`, `conv2d2`, `conv2d3` and `conv_out` weight namespaces. A chunk is processed separately for every 100 valid Mel frames, with a short final chunk cropped to its actual valid frame count before convolution. The returned embeddings have shape `[token_count, 896]`; a short 50-frame chunk yields 7 tokens, not 13. The window planner now accepts **valid frames** rather than padded frames, so final attention-window lengths agree with the convolution's output. The right-padded Mel features and input mask remain available to the rest of the pipeline.

This is not yet a usable Qwen3-ASR audio encoder: the sinusoidal per-chunk positional embedding, 18 Transformer encoder blocks, final projections, decoder and tokenizer are not implemented. The new stem has not been tested with a real safetensors checkpoint or compiled on the required toolchain. This WIP does not enable the STT endpoint. Reference: https://github.com/QwenLM/Qwen3-ASR/blob/main/qwen_asr/core/transformers_backend/modeling_qwen3_asr.py.

## Local speech model discovery (R631 WIP)

The native speech server uses `models/speech` under the working directory, or searches ancestor directories of the executable to find an existing `models/speech` directory. This supports a source checkout started from `C:\repos\zipeek` and a binary under `target/debug/deps`. The packaged eSpeak executable and voice-data defaults remain executable-adjacent `resources`, independent of neural checkpoint discovery.

Default checkpoint paths are `models/speech/Qwen3-ASR-0.6B/model.safetensors` for recognition and `models/speech/Kokoro-82M-v1.1-zh/kokoro-v1_1-zh.pth` for planned neural synthesis. `ZIPEEK_CANDLE_STT_MODEL` and `ZIPEEK_CANDLE_TTS_MODEL` explicitly override those defaults. An existing model directory alone does not demonstrate inference readiness; model asset validation and real checkpoint execution remain separate gates.
