# Testing and validation

> **Current approved target behavior:** [Approved behavior acceptance](approved-behavior-acceptance.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


Rust unit/integration tests and real Chrome runs answer different questions. JavaScript source-contract tests under `scripts/` were retired by decision in the historical source. A test appearing in the repository is not proof that it has run successfully. Current acceptance state is maintained in [CURRENT](../../project-management/current.md); open gates are in [TASKS](../../project-management/tasks.md). Historical evidence belongs in [release history](../../archive-history/RELEASE_HISTORY.md).

## Commands from the workspace root

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Run affected Rust tests first for routine work. For comprehensive release acceptance use:

```powershell
cargo make e2e-full
cargo make pack-e2e
```

`e2e-full` first checks WASM, compiles all release E2E targets, then runs the managed Rust/Chrome suites serially. The pack task prepares evidence for review. Browser suites use a managed server, an isolated Chrome profile and scenario-specific artifacts; see [E2E](../../archive-history/audits/e2e.md) and [E2E architecture](../e2e/e2e-architecture.md).

## Native Qwen3-TTS synthesis acceptance

The real `/v1/tts` endpoint prefers the local Qwen3-TTS-12Hz-0.6B-CustomVoice model under `models/speech/` over eSpeak, while preserving eSpeak as an optional fallback when the neural checkpoint is absent. This pathway executes the talker, code predictor and neural codec decoder to produce PCM WAV, not test-tone audio. `ZIPEEK_QWEN_TTS_MODEL` overrides the checkpoint path, and `ZIPEEK_TTS_DEVICE` selects `cuda` (default) or `cpu` in the current Candle-native build; `ZIPEEK_STT_DEVICE` defaults to `cpu`. Do not mistake asset preflight for successful inference.

The new dependency requires lockfile resolution and review before accepted use. After that, run:

```powershell
cargo test --locked -p zipeek-tts-stt --test neural_tts -- --ignored --nocapture
cargo test --locked -p zipeek-tts-stt --test speech_tts_stt_roundtrip -- --ignored --nocapture
```

The first test saves `target/speech-e2e/mandarin_nihao_shijie.wav` and `target/speech-e2e/mandarin_nihao_shijie.opus` when neural inference succeeds. Listen to this output before accepting exact STT text comparison. Neural outputs must be non-silent and at 24 kHz; compilation alone does not establish model accuracy. Kokoro remains future work.

## R662 speech worker reliability and performance acceptance

The native speech service uses one persistent TTS worker and one independent STT worker. A request holds its backend permit through worker completion even when the caller abandons it. Cancellation is cooperative, not a forced CUDA interruption. `/v1/capabilities` reports `neural_tts_status`, `neural_stt_status`, and non-sensitive `neural_tts_failure`/`neural_stt_failure` categories (`none`, `model_load`, `inference`). Explicit shutdown transitions workers to `stopped`. Internal diagnostics are logged, not exposed in HTTP bodies.

Run the checkpoint-free unit and HTTP checks and then the ignored model-backed acceptance tests on Windows with the actual installed checkpoints. The 20-request tests print warm latency observations; compare measured p50/p95, VRAM usage and transcripts with the R661 baseline before attempting KV-cache or CUDA kernel optimizations. Stochastic TTS output has no fixed duration. The generated neural reference WAV is 24 kHz, while direct WAV STT input must be mono 16 kHz PCM16; the speech worker acceptance test resamples its reference fixture before transcription. This fixture conversion does not relax production input validation.

```powershell
cargo fmt --all -- --check
cargo check --locked -p zipeek-tts-stt
cargo clippy --locked -p zipeek-tts-stt --all-targets -- -D warnings
cargo test --locked -p zipeek-tts-stt
cargo test --locked -p zipeek-tts-stt --test speech_worker_acceptance -- --ignored --nocapture
```

Additionally verify timeout during active CUDA inference, prompt cancellation, shutdown while a request is executing, failed-load retry, worker-thread exit after all runtime clones drop, and correct audio playback using the Windows runtime. A passed synthetic permit test alone does not establish those end-to-end behaviours.

## Native speech Rust integration and end-to-end tests

The canonical native speech integration suite lives under `crates/zipeek-tts-stt/tests/`. Its shared `common/` harness launches the **actual compiled Rust server** on an ephemeral loopback port, sends HTTP requests, checks responses and kills/reaps the process on test completion. It requires no additional dependencies. Unit tests remain in `src/` for internal functions; Rust integration tests validate the binary and public boundaries.

From the workspace root, run the non-model-dependent suite:

```powershell
cargo test --locked -p zipeek-tts-stt --test speech_http_e2e --test speech_security --test model_loading
```

`model_loading` has one fast missing-file test and one ignored checkpoint-backed test. The baseline HTTP test can accept an explicit `503` when eSpeak is absent, but never mistakes this for synthesized speech. Baseline HTTP/security tests explicitly launch the server with an absent STT checkpoint path so they can verify a deterministic `503` regardless of local assets.

Run real eSpeak synthesis acceptance (requires a working installed eSpeak NG executable and a parent directory containing `espeak-ng-data`):

```powershell
$env:ZIPEEK_ESPEAK_BIN = 'C:\path\to\espeak-ng.exe'
$env:ZIPEEK_ESPEAK_DATA = 'C:\path\to\resources'
cargo test --locked -p zipeek-tts-stt --test tts_inference -- --ignored --nocapture
```

Run full Qwen3-ASR checkpoint and spoken-Mandarin HTTP acceptance separately. The test embeds the checked-in 16 kHz mono PCM16 Mandarin audio fixture with `include_bytes!` and compares the transcript to the fixed text `你好世界`. It is generated with eSpeak and must be listened to for Mandarin pronunciation verification; the local synthesis tool reported a missing full Chinese dictionary. Unlike a sine wave, it contains a speech synthesis signal, but this does not independently prove accurate speech content:

```powershell
$env:ZIPEEK_CANDLE_STT_MODEL = (Resolve-Path .\models\speech\Qwen3-ASR-0.6B\model.safetensors).Path
cargo test --locked -p zipeek-tts-stt --test model_loading -- --ignored --nocapture
cargo test --locked -p zipeek-tts-stt --test stt_inference -- --ignored --nocapture
```

A successful HTTP 200 and nonempty text are insufficient: the strict STT test compares the normalized transcript with the provided reference. Model-backed tests are ignored by default because the safetensors are not included in source archives and can require substantial memory/time. Passing a test is evidence only of that specific check; it does not establish numerical model parity, multiple-speaker accuracy, or full product/browser acceptance. The Rust toolchain and model files must exist on the testing machine.

### Speech acceptance

The retired Node speech harness is no longer part of the source tree. Validate speech with Rust `zipeek-tts-stt` tests and existing Rust E2E suites. The removed harness provided separate HTTP protocol coverage; do not count that coverage as passing until an equivalent Rust test is established.

## Current browser coverage

- Shared catalog: 34 concrete route cases under root and the `/zipeek/` deployment base, plus separate accessibility and 320/390/768px responsive checks.
- Behavioral suites: Browse/Word Sets, Settings/Debug, Practice setup and persistence, courses, games, speaking, translation, writing, shell navigation and errors. See test targets under `crates/zipeek-e2e/tests/`.
- Sentence Translation: deterministic Rust AI fixture and optional live provider journey through Settings, Practice setup and the UUID-backed generated exercise. Fixture tests cover canonical tiles, finite completion, restore and Endless behavior.
- AI fixture: `crates/zipeek-e2e/src/harness/ai_fixture.rs`; the old Node `ai-server.cjs` is removed.

**Acceptance limitation:** The user reported successful Windows accessibility, Browse and an earlier live translation test. The newest UUID translation implementation and full suite still need their own execution. An ignored test is not a pass; route presence is not equivalent to complete interaction or WCAG compliance.

## Live AI test

Provide a running OpenAI-compatible endpoint and exact model ID, then run:

```powershell
$env:ZIPEEK_E2E_AI_ENDPOINT = 'http://127.0.0.1:1234/v1/'
$env:ZIPEEK_E2E_AI_MODEL = 'your-loaded-model-id'
cargo test --release --locked -p zipeek-e2e --test ai_learner -- --ignored --nocapture
```

The live test is intentionally excluded from deterministic CI. It sends actual requests to the locally configured provider; it is not a general assertion of model correctness.

## Related focused guides

[Browser database](../e2e/browser-db.md), [matching](../e2e/matching.md), [MCQ](../e2e/mcq.md), [startup](../e2e/startup.md), [shell](../e2e/shell.md), [E2E recording](../e2e/e2e-recording.md), [evidence](../e2e/evidence.md), [UI acceptance](ui-acceptance.md).

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/testing-quality-acceptance-testing.md). Dated implementation claims in that snapshot are not current build or browser verification.

## Record automatic microphone termination (R673)

Record route microphone capture may end automatically after sustained speech and a trailing silent interval. Its adaptive energy detector uses browser Web Audio analyser PCM with an 80 ms sampling callback. Speech must be sustained for at least 200 ms to arm stop detection; use 900 ms trailing silence for HSK words and 1500 ms for custom sentences. Maximum capture duration is 9 seconds for words and 28 seconds for sentences. The detector does not perform neural voice classification or know whether the target word has been completed. Manual stop and cancellation must remain functional. Recording bytes remain in browser memory and STT is called only after capture completes.

Acceptance requires noisy and quiet microphones, starting speech immediately or after silence, naturally spaced sentences, no-speech recordings, manually stopped recordings, permission refusal, service failure, target switching during capture, cancellation after stop and repeated recording/replay. Verify that no STT request is made when automatic capture ends without confirmed speech. Run the pure Rust `zipeek-browser` unit tests as well as the locked WASM build and actual Chrome interaction tests.

## Tauri frontend/database separation checks

After staging the native dictionary and sidecars with `cargo make desktop-sidecars`, run:

```powershell
cargo test --locked -p zipeek-db --lib
cargo test --locked -p zipeek-cli desktop_frontend
cargo test --locked -p zipeek-cli native_selection_uses_verified_manifest_without_a_browser_gzip
cargo check --locked -p zipeek-web --target wasm32-unknown-unknown
cargo check --locked -p zipeek-app
cargo test --locked -p zipeek-app --lib
cargo run --locked -p zipeek-cli -- desktop-frontend
```

Native tests require the pinned DuckDB DLL on PATH. Desktop output must omit database preload links, manifest/download files and DuckDB-WASM/Arrow vendor files, while retaining shared worker/assets and notices. A missing native bridge or failed native open must surface an error without requesting browser database assets. Browser builds must still load the dictionary at both `/` and `/zipeek/`. Local Chrome with an injected IPC fixture establishes frontend behavior only; actual WebView/installed acceptance still requires `cargo make desktop-dev` and `cargo make desktop-build` on Windows.

## Settings reset regression

`cargo test --locked -p zipeek-app --lib` covers native personal-file/WAL removal, reopen with empty history, dictionary preservation, repeat reset and invalid/removal-failure paths. The focused browser suite is:

```powershell
cargo test --locked -p zipeek-e2e --test app_reset -- --test-threads=1 --nocapture
```

It uses fresh managed Chrome profiles and the existing screenshot/video harness. Both root and Pages suites cover cancellation, full owned-storage cleanup, retention of unrelated data, fresh landing, unconfirmed reset URL, another active app tab, and mocked native failure/retry. The harness reinjects its silent-speech session flag on each document; reset assertions account for this test-only reinjection while verifying all seeded user data is gone. Native frontend fixture checks are not actual installed WebView acceptance. No tests clear the developer's real app profile.
