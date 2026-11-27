## R706: Image generation readiness

- [x] Stop silently treating the preview-only backend as generated artwork; default to SDXL and distinguish readiness from health.
- [x] Reject placeholder image generation in both desktop and browser clients; explain required SDXL files.
- [ ] Validate SDXL startup/inference on Windows CUDA with actual model weights; run locked native and WASM checks.

## R706: Profile and saved-session UX refinement (2026-11-27)

- [x] Dictionary-backed custom exclusion autocomplete (Hanzi, pinyin, English) with exact Word ID selection and keyboard controls.
- [x] Remove repeated Profile/Word Sets/Drawer instructional labels and show navigation icons.
- [x] Compact saved-session resume with progress bar, Play icon and direct delete; suppress duplicate reminder on Practice setup.
- [ ] Compile WASM and test desktop/browser interaction, keyboard focus, fast typing, persistence and session recovery.

## R706: Profile configuration hub (2026-11-27)

- [x] Add `#/profile` with explicit `pages/profile/mod.rs` and separate excluded-word and drawer modules; replace Word Sets drawer link with Profile user icon.
- [x] Reuse existing Word Sets provider and data, with an icon-only create action.
- [x] Read and update existing excluded-word IDs through the shared exclusion service; display resolved vocabulary, search, add dictionary-backed Hanzi/pinyin/English autocomplete and direct removal.
- [x] Add persisted drawer item visibility using existing preferences and centralized navigation definitions; Profile and Settings cannot be hidden. Refresh same-tab and across tabs.
- [ ] Run locked wasm32 compile and browser acceptance for tabs, word sets, exclusion add/removal, menu toggles, persistence and multiple tabs.
- [ ] Verify R706 numbered-major release gate separately; this source is a WIP checkpoint.

## R705: Native bridge consolidation (2026-11-27)

- [x] Introduce explicit `services/tauri/mod.rs` module with typed `TauriService`, structured `TauriError`, command contracts and separately owned event subscriptions.
- [x] Migrate database/history, service controls, native AI, and image IPC to one bridge implementation; centralize desktop capability detection and remove redundant per-service reflection.
- [x] Add command naming, argument encoding and native storage-response tests without changing native commands or adding dependencies.
- [ ] Verify `cargo check -p zipeek-web --target wasm32-unknown-unknown --locked`, `cargo test -p zipeek-web --target wasm32-unknown-unknown --locked` where supported, and actual Tauri database/service/AI/image/event runtime behavior. No native event consumer has been migrated because none currently exists.

## R705: Practice UX and speech resilience (2026-11-27)

- [x] Remove redundant round/mistake and session-position counters from exercise gameplay; retain metrics in persisted assessments.
- [x] Remove redundant vocabulary card nesting and Character Builder instructional prompt.
- [x] Add slot-aware mouse and touch drag/reorder with click and keyboard-operable fallback, plus placement unit tests.
- [x] Add native Mandarin speech fallback with browser voice gating, backoff, stale-request guards, one-time notice and developer logging.
- [ ] Validate WASM compilation, browser touch/mouse dragging, pronunciation voice availability and notification behavior using the user's toolchain.

## R705: Per-target vocabulary assessment and truthful partial outcomes (2026-11-27)

- [x] Emit observational target-specific evidence for Type Pinyin, including visible-answer assistance and retry counts, without altering Knowledge Points.
- [x] Preserve Word Mosaics pair identities and the two targets involved in a wrong match, and emit qualified clean/partial per-pair outcomes.
- [x] Preserve up to 20 eight-item mosaic rounds in the assessed evidence and observation journal; align aggregate verdicts with mistakes and assistance.
- [ ] Run WASM compilation and browser acceptance for Type Pinyin hint-enabled/disabled, retry, Word Mosaics wrong-pair tracking and 20-round persistence.
- [ ] Connect authorized knowledge manifests and validated pedagogical events to a durable transactional KnowledgeLedger processor; current capabilities do not automatically grant points.
- [ ] Complete full lossless Endless ledger/reader and remove 1,000-step limit only when verified.
- [ ] Satisfy the numbered major-release 20-qualifying-Rust-file acceptance gate without unrelated filler changes, or keep R705 as an unverified candidate.

## R704: Library warnings and durable observation inbox modularity (2026-11-27)

- [x] Apply explicit `allow(unused)` and `allow(unused_imports)` to all 22 crate library roots; do not hide unrelated compiler errors.
- [x] Gate the pedagogy-worker implementation modules behind `wasm32` so native builds do not compile unused WASM-only worker internals.
- [x] Split the existing consented Practice observation journal into event-model validation, deterministic inbox reading, storage and reconciliation modules, preserving step-keyed deduplication and history erasure.
- [x] Test observation round trips, unassessed routing, invalid identity, inverted timestamps and contradictory evaluator evidence.
- [ ] Connect independently qualified catalog targets, signed-off manifests and modality to `KnowledgeLedger`, with atomic durable apply receipts. The observation inbox alone never awards Knowledge Points.
- [ ] Build on Windows and wasm32, check any remaining binary-only warnings and verify two-tab/out-of-storage race behavior.

## R704: Cross-exercise Endless selection, observation recording and review integration (2026-11-27)

- [x] Expose Endless beside fixed lengths for all 19 registered exercise IDs and any supported combination. Make translation-only count selection update `SessionLength` and ensure launch uses `SessionLength` as the only mode/count authority; allow a 20-sentence translation session.
- [x] Add an opt-in, step-identified, immutable Practice observation journal. Persist evidence after an exercise result is committed, reconcile on resume, reject conflicting rewrites and erase it with saved Practice history. Do not falsely promote exercise verdicts into target-level knowledge mastery.
- [x] Migrate generated Sentence Translation and the non-retry Sentence Builder review branch to shared `ExerciseReview`, preserving existing gameplay and selectors.
- [ ] Replace the hard 1,000-step limit with a bounded session window and lossless chunked evidence ledger, including history readers and completion summary. Local-storage observations alone are not a substitute for that redesign.
- [ ] Connect qualifying per-item assessments to the real pedagogy `KnowledgeLedger` using validated target identity, modality, context and independence, plus durable projection readback. No new unqualified mastery updates are allowed.
- [ ] Validate WASM compilation, generated translation review, Sentence Builder retry, all exercise setup modes, consent-off recording, out-of-date observation reconciliation, storage quota failures, history erasure and two-tab races in the user's environment.

## R704: Atomic Endless-session progression (2026-11-27)

- [x] Preflight deterministic Endless scheduling before mutating the current exercise; keep completed outcomes and revision unchanged on scheduler failure.
- [x] Make replenishment all-or-nothing and reject revision overflow before advancing or finishing sessions.
- [x] Add regression tests for invalid quiz configuration, invalid scheduling inputs and revision saturation.
- [ ] Replace the existing 1,000-step ceiling with a lossless, bounded evidence ledger and reader; this milestone does not change the persisted data format.
- [ ] Verify with `cargo test --locked --target wasm32-unknown-unknown -p zipeek-web` and the browser session flow.

## R704: Source pack safety and I/O performance (2026-11-27)

- [x] Exclude nested destination archives from source collection, prevent failed packs from overwriting a previous archive, clean temporary output, and skip recompressing common binary media/fonts and uppercase extensions.
- [x] Add targeted regression tests for nested-output pack, failure preservation and compression selection.
- [ ] Verify locked CLI archive tests and benchmark repeated Windows pack/unpack cycles; check no `.unpack-backups` or successful staging leftovers.

## R704: No persistent unpack backups (2026-11-27)

- [x] Source unpack uses a temporary rollback area under `target/zipeek-unpack/rollback` and removes it on success or successful rollback; no `.unpack-backups` directory is created.
- [x] Refuses a new unpack if an incomplete rollback area remains, preserving recoverable files instead of deleting them.
- [ ] Run `cargo test -p zipeek-cli --locked` on Windows and verify no rollback artifacts after successful unpack.

## R704: Source packaging and assessed-evidence compatibility (2026-11-27)

- [x] Use buffered streams for source packing and reject lossy Unicode path encoding; install extracted files by same-volume moves, preserving prior contents temporarily for rollback without leaving backup artifacts, and clean successful extraction staging.
- [x] Update the database learning-history summary parser for typed assessed outcomes, including partial verdicts and trusted attempt validation.
- [ ] Run `cargo test -p zipeek-cli -p zipeek-db --locked` and measure source archive unpack throughput and filesystem churn on Windows; no benchmarks have been run.
- [ ] Implement lossless Endless history checkpointing before lifting the 1,000-step cap; preserve granular attempt evidence with a durable bounded session and a versioned/history-reader-compatible evidence ledger.

## R704: Mixed Endless and target verification (2026-11-27)

- [x] Implement deterministic mixed-exercise Endless replenishment and selected quiz/AI settings with regression tests. The existing 1,000-step limit is retained.
- [x] Isolate the browser-only `zipeek-web` modules from native `cargo test --workspace` compilation; browser-specific code and tests must be built for wasm32.
- [ ] Remove the Endless step/history cap through an approved bounded evidence persistence design, with reader and finalization changes together; do not discard or silently truncate completed outcomes.
- [ ] Run `cargo fmt --all -- --check`, `cargo test -p zipeek-exercises --locked`, and `cargo check -p zipeek-web --locked --target wasm32-unknown-unknown` on the user toolchain.
- [ ] Resolve the separate MSVC runtime clash in E2E (`llama-cpp-sys-2` vs `esaxx-rs`) after a native dependency-feature/CRT review; no dependency or lockfile change was made in this milestone.

## R704: Review lifecycle and stale callbacks (2026-11-27)

- [x] Centralize feedback and Continue through `ExerciseReview` for the directly reviewed authored, listening, image, transformation, spot-mistake, multiple-choice and vocabulary exercise components, without re-scoring or changing gameplay.
- [x] Scope exercise outcome/retry callbacks to the rendered step to protect against late asynchronous signals crossing an `animato` transition.
- [ ] Extend common review lifecycle to conditional retry and generated feedback flows; verify selectors and focus on mobile and desktop browsers.
- [ ] Build `zipeek-web` for wasm32 and run browser/regression acceptance; the source and archive checks are not compilation verification.

## R704: Evaluator outcome adoption (2026-11-27)

- [x] Add assessed outcome callbacks in Fill Blank, Audio Dictation, Spot Mistake (including partial), Sentence Transformation and Listening Choice; preserve non-hosted completion callbacks and allow assessed storage/restore for those IDs.
- [ ] Verify all five new outcome callbacks in the browser and on the wasm32 target before claiming acceptance.

## R704 Practice UI acceptance

- [ ] Build and visually verify the new shared `ExerciseSurface` across all 19 registered exercise IDs at desktop/mobile widths, in both fixed and Endless sessions, including focus/Enter, animated step swaps, audio/canvas/grid sizing, and existing selector compatibility. The shared surface is not yet verified by Rust or browser tests.
- [x] Establish typed `ExerciseAssessment` and `ExerciseVerdict` at the registry boundary, with attempt/mistake/hint/assistance fields, validation, session persistence, legacy-score deserialization, and honest verdict counts in the session summary (source-level implementation; pending build/browser verification).
- [ ] Complete the remaining exercise-family evaluators (authored/listening/transformation, quiz/builder, and all three vocabulary IDs now emit real assessments); do not fabricate verdicts for completion-only callbacks. Finish shared feedback/action lifecycle ownership and per-exercise retry/hint evidence collection.

- [ ] Run WASM compilation and browser checks for Docs article links, table of contents, source browsing, and Statistics dialog focus trapping/restoration after AppEnvironment injection.

- [ ] Compile and verify typed registry outcome callbacks, binary true/false conversion, explicit partial verdict/attempt evidence propagation and unscored completion across hosted exercises.
- [ ] Verify rolling pre-generation in browser: next translation ready before Continue, slow/failed AI generation, leaving and resuming, and Finish while a future step is generating. Verify fixed-session prefetch as well.
- [ ] Extend `SessionMode::Endless` beyond translation only after approving a deterministic next-step policy for multi-exercise playlists and bounded storage/history, removing the automatic 1,000-step finish. Preserve progress evidence and explicitly define zero-completion Finish semantics.
- [ ] Verify the new in-panel Finish session confirmation, Keep practicing cancellation, persisted summary, and Leave Practice/resume distinction in Chrome and Tauri. Verify reducer snapshot restore, error, generation, and completion behavior.


- [ ] Compile and verify extracted registry adapter modules, `ExerciseFrame` session-position announcements, and the single in-session Leave control for Type Pinyin, Character Builder and Word Mosaics.
- [ ] Verify generated narrative instruction/action presentation, conversation reducer end-of-turn scoring, keyboard interaction and repeat-submit behavior in browser.
- [ ] Verify shared primary Check and form Submit controls in generated Image Vocabulary, Sentence Translation, Spot Mistake, Sentence Transformation, Fill Blank/Audio Dictation, Character Builder and Type Pinyin, including Enter-key composition handling, disabled states and preserved selectors.
- [ ] Continue migration beyond Spot Mistake, Sentence Transformation, Fill Blank, Audio Dictation, Sentence Builder, Local Quiz, generated Image Vocabulary, generated Sentence Translation, Type Pinyin, Word Mosaics, Character Builder, Listening Choice and Write Hanzi to shared instructional, feedback and action ownership; keep gameplay-specific layouts and typed outcome evidence intact.
- [ ] Verify the first shared feedback/action adapter and elapsed-time transition under correct/partial/incorrect results, retries, back-to-back navigation, reduced motion and screen-reader announcements. Confirm focus arrives after the new exercise DOM render and confirm finish confirmation resets on advancement.

- [ ] Compile and verify trait-based exercise registry coverage and configuration-failure fallbacks across all supported IDs; check component identity on resumed steps.

- [ ] Compile and exercise Diagnostics simulator reducer transitions, including Stop during an in-flight write and verification after reload.
- [ ] Compile and exercise Local Quiz reducer for MCQ, matching, wrong-pair highlighting, retries, audio preparation and completion.
- [ ] Verify AppEnvironment-based header dragon and SQL examples menu at hosted `/zipeek/` and local `/` deployment.

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises`, `cargo test --locked -p zipeek-web` and locked WASM compilation after unpack.
- [ ] Verify drag-and-drop from word bank and between selected positions, touch/click and keyboard reorder, and multi-step prompt uniqueness.
- [ ] Verify automatic check and retry combinations persist in resumed sessions, including wrong answers and accepted alternate orders.
- [ ] Check the top creature/progress in every theme, portrait and landscape, compact desktop windows and reduced-motion mode.
- [ ] Consider separately approved vocabulary-conditioned/AI-generated Sentence Builder content and broader shared Practice component primitives after acceptance.

## R703 AI narrative Practice acceptance

- [ ] Run `cargo test --locked -p zipeek-exercises` and `cargo test --locked -p zipeek-web`; fix any compilation and test failures.
- [ ] Build `zipeek-web` for `wasm32-unknown-unknown`; test each 5–10 sentence setting and the independent 5–10 conversation-turn setting.
- [ ] Test real generation on local GGUF and LM Studio, including malformed content, retry, slow model and cancellation.
- [ ] Implement adaptive conversation turns with server-generated follow-up based on the selected reply and persistent mid-dialogue state.
- [ ] Add in-exercise answer review, expanded grammar-pattern selection and human evaluation of generated Mandarin content.

## R702 image transport acceptance

- [ ] Windows: compile `zipeek-app` and `zipeek-web` for wasm32; run the native image command tests.
- [ ] Start the Image service in Tauri, verify native health and actual test image generation show no WebView CORS request; verify PNG/SVG preview and URL cleanup.
- [ ] Test browser mode against the configured HTTP image endpoint at `/` and `/zipeek/`; confirm desktop asset URL opened outside Tauri fails with an explicit message.
- [ ] Validate image generation during Picture Vocabulary and after a service restart; ensure saved/serialized blob URLs are not mistaken for durable image assets.

## R701 Practice activity and UI acceptance

- [x] Preview correct Character Builder answers in creature progress before Continue, retaining persisted completion only on Continue.
- [x] Remove duplicate builder title, hide completed-session Leave navigation and use Yew icons for completion and review actions.
- [x] Add read-only, bounded real Practice activity to Statistics using browser OPFS/native Tauri evidence, kept separate from demo dashboard and mastery.
- [x] Correct reqwest public `rustls` feature configuration and keep existing lockfile.
- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-ai-client -p zipeek-app`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown` and relevant Rust/E2E tests on Windows.
- [ ] Verify browser and Tauri saved outcome recording, Statistics refresh, empty/disabled/error cases and themed creature progress.
- [ ] Later: a full real Statistics projection replacing the separate demo dashboard requires qualified pedagogical evidence and validated query contracts.

## R700 native desktop transport acceptance

- [ ] Verify Tauri native IPC AI speech, local AI, LM Studio and generated image rendering under Windows.
- [ ] Verify WebSocket handshake for Tauri dev and packaged origins; check no arbitrary origins pass.
- [ ] Evaluate cancellable streamed AI updates, HTTPS remote providers and object-URL lifecycle separately, retaining existing HTTP browser access.

## R699 Tauri image transport and Settings (WIP)

- [x] Use native Tauri IPC for image health and generation in desktop builds, with fixed-loopback and bounded response checks.
- [x] Keep hosted browser image URL configuration, remove editable URL from Tauri Image Settings, and show backend-aware health and test generation feedback.
- [x] Make Local AI endpoint discoverable, change Speech Settings navigation to a microphone, and simplify inactive pedagogical status.
- [x] Keep Tauri process lifecycle in the Services section and preserve image practice URL contracts.
- [ ] Run locked `zipeek-app` and WASM compilation plus native command unit tests on Windows.
- [ ] Verify desktop WebView health, test generation and picture vocabulary without CORS; verify hosted image URL settings and `/` / `/zipeek/` browser journeys.

## R698 SQL query workspace and header layout (WIP)

- [x] Put editor and result in each closable SQL query tab instead of separate stacked result tabs.
- [x] Close example picker on outside pointer input, Escape and example selection.
- [x] Reduce unnecessary desktop page height while keeping result data scrollable inside its panel.
- [x] Widen the header dragon without stretching the artwork.
- [ ] Verify locked WASM compilation and web tests on Windows.
- [ ] Verify diagnostics layout, keyboard/pointer interactions and dragon crop at `/` and `/zipeek/` in Chrome/Tauri.

## R697 Development diagnostics workspace UX (WIP)

- [x] Stabilize diagnostics width and minimum height across Overview, DuckDB, SQL and OPFS tabs.
- [x] Use Yew icons for tabs and integrate the user-provided DuckDB asset.
- [x] Remove redundant editor labels and group query examples in a collapsed menu.
- [x] Add closable per-query results with preserved output/error, bounded tab retention, and selected-result CSV export.
- [x] Retain debug-only routing, SQL query restrictions, OPFS read-only inspection and 100-row result cap.
- [ ] Run `cargo fmt --all -- --check`, locked WASM check and web tests on Windows.
- [ ] Browser acceptance at `/` and `/zipeek/`, testing empty results, failed queries, close/evict behavior and viewport height consistency.

## R696 OPFS inspector and SQL scope (WIP)

- [x] Add a separate debug-only OPFS root tab with an on-demand Refresh and bounded read-only enumeration.
- [x] Allow SELECT from additional dictionary tables without a hardcoded four-table list.
- [x] Keep SQL row caps and explicit external I/O restrictions; do not read or export OPFS file contents.
- [ ] Run locked WASM checks and browser acceptance at `/` and `/zipeek/` including unsupported-OPFS and empty-root states.
- [ ] Verify native Tauri SQL validation for a newly exposed dictionary table.

## R695 debug SQL terminal (WIP)

- [x] Expand SQL editor and width; remove repeated debug/AI explanatory copy.
- [x] Replace text-labelled Run SELECT with an accessible icon and in-flight feedback.
- [x] Accept one final semicolon and enable dictionary `word_relations` queries.
- [x] Add relationship resolution example and query-validator regression assertions.
- [ ] Run locked WASM compile and Rust unit tests on Windows.
- [ ] Verify the screenshot query returns relationship counts in the browser or Tauri debug page.

## R694 AI Settings provider UX (WIP)

- [x] Replace the noisy selected-provider check icon with a subtle border and active surface.
- [x] Consolidate LM Studio model discovery, manual fallback and optional credentials.
- [x] Keep save/test actions in a consistent panel position, with spinner and inline test feedback.
- [x] Add focused reducer tests for stale discovery and error feedback.
- [ ] Compile locked WASM and run Yew unit tests on Windows.
- [ ] Validate browser and Tauri provider selection, endpoint failures, model discovery and test request lifecycle.

## R693 local AI and LM Studio provider (WIP)

- [x] Replace production fixture executable with llama.cpp GGUF startup and chat completion generation.
- [x] Keep fixture-only server injection in tests, not as a runtime mode.
- [x] Add Settings provider selection, LM Studio endpoint and model discovery.
- [x] Update Tauri service launch, labels and startup allowance.
- [ ] Compile and test the real `llama-cpp-2` model implementation on Windows with an installed GGUF, and verify SSE, cancellation, tokenization and structured translation fallback.
- [ ] Verify LM Studio LAN connectivity from Tauri WebView; add a native proxy if CORS blocks requests.
- [ ] Implement approved optional GGUF file picker and persisted Tauri-side model path configuration. For now, the path comes from an environment variable or models/ai.
- [ ] Update legacy E2E fixtures that still launch the obsolete `--fixture` executable flag.
- [ ] Validate installed Tauri sidecar startup, model loading and restart after model changes.

## R692 Settings service clarity and layout stability (WIP)

- [x] Identify actual Tauri-managed AI backend and label its deterministic fixture behaviour honestly.
- [x] Remove duplicate healthy/stopped status details while retaining actionable errors and in-progress feedback.
- [x] Reserve consistent desktop Settings content height between short and tall sections.
- [ ] Run locked WASM build and inspect Services/Appearance switching at common Tauri viewport sizes.
- [ ] Separately design and approve management of real Llama.cpp inference; do not silently replace the fixture backend.

## R691 Services UI feedback and shell footer removal (WIP)

- [x] Use existing Yew Lucide icons and service-specific status visuals in the desktop-only Settings > Services panel.
- [x] Give Start/Stop operation-specific spinner/disabled feedback and visible health-check in-flight state.
- [x] Remove redundant managed-process explanation; center Settings on desktop.
- [x] Remove general shell footers and leave project attribution in documentation rather than the Settings UI.
- [x] Add focused reducer transition assertions for health refresh and operation lifecycle.
- [ ] Verify locked Rust/WASM build, actual health button responses, pending service lifecycle and responsive spacing in Tauri/Chrome.

## R690 service module structure and build-once workflows (WIP)

- [x] Split Tauri service runtime into public API, registry transitions, owned processes, health checks, errors, and tests without changing IPC shapes.
- [x] Add platform-aware existing-CLI launcher; build missing CLI only for pack/unpack.
- [x] Reuse debug AI, image and speech executables during sidecar staging; build each missing binary only when needed.
- [x] Preserve explicit release-service and release-CLI rebuild steps for installer/verified packaging.
- [x] Add checkpoint-free launcher tests.
- [ ] Run locked native checks, service tests and Cargo Make tasks on Windows; inspect installed Tauri sidecar staging.

## R689 desktop service controls and command errors (WIP)

- [x] Keep Tauri `CommandError` as the IPC boundary with automatic typed service/database error conversions and explicit imports.
- [x] Move AI fixture, Image and Speech service controls into a dedicated Settings section that is visible and routable only in Tauri.
- [x] Provide individual Start, Stop and Check health actions with lifecycle status, ownership gating and actionable errors; do not duplicate process controls in feature-specific settings.
- [x] Add direct-navigation guard test for desktop-only Services section.
- [ ] Run native/WASM checks on Windows and verify three services start/stop/health checks in an installed WebView.
- [ ] Verify hosted-browser Settings omits Services, and direct `?section=services` falls back safely on both `/` and `/zipeek/`.
- [ ] Check speech model readiness independently through `/v1/capabilities`.

## R688 Tauri managed speech service and command boundary (WIP)

- [x] Add speech sidecar to staging, release build and Tauri bundle without adding dependencies.
- [x] Add speech to the existing managed service registry and expose it in Settings with the shared TTS/STT process boundary.
- [x] Distinguish speech plaintext `/health` from JSON health checks, with a separate neural cold-start timeout and external-process ownership protection.
- [x] Replace metadata-only enum with a static service descriptor table; use tuple-based shared registry ownership.
- [x] Move blocking task scheduling inside `NativeServices` and keep Tauri commands as explicit, thin async adapters returning typed `ServiceError`.
- [x] Correct WASM `Promise::race` conversion to `JsValue`.
- [x] Author service-key, process ownership, plaintext health, shared Arc and browser endpoint tests.
- [ ] Run native Rust, WASM and sidecar staging checks on Windows; accept startup/shutdown and real TTS/STT/Live STT use from Tauri.
- [ ] Verify engine-specific readiness via `/v1/capabilities` independently of process liveness; do not claim readiness merely from `/health`.

## R687 Live speech client and binary transport (WIP)

- [x] Replace R686 JSON WebSocket frames with version-specific compact binary opcodes and typed payloads.
- [x] Add Yew Live tab with partial/final transcript, microphone status, failure and cleanup.
- [x] Implement browser AudioWorklet PCM capture, streaming sample-rate conversion, bounded PCM16 packets and transport backpressure.
- [x] Update server handshake tests for binary frames, wrong origin, no speech, invalid sample rate and audio before start.
- [ ] Regenerate root Cargo.lock if required by Axum ws feature; perform native and WASM compilation and unit/integration tests.
- [ ] Run Chrome/WebView microphone, CSP, cancellation, partial/final CUDA recognition and backpressure acceptance.
- [ ] Complete true Zipformer encoder parity and replace snapshot mode only after acceptance.

## R686 experimental WebSocket speech transport and mainline reconciliation (WIP)

- [x] Restore R685 Candle encoder-embedding source, reference generator and parity executable into current Tauri mainline.
- [x] Implement bounded localhost WebSocket PCM16 session protocol on the existing speech server with restricted browser origins.
- [x] Implement snapshot-coalesced Qwen3-ASR partial/final recognition, session limits, typed errors and disconnect cleanup.
- [x] Add checkpoint-independent protocol, audio-frame and real handshake regression tests.
- [ ] Regenerate root Cargo.lock after activating Axum WebSocket feature, then pass locked Windows checks.
- [ ] Test real microphone streaming and partial/final text under GPU load; verify cancellation/shutdown behavior.
- [ ] Complete Zipformer embedding numerical parity, six-stage encoder and true incremental model integration.
- [ ] Add experimental Live tab/client only after transport acceptance.

## Settings Reset app acceptance follow-up

Implementation and focused browser/native verification are recorded in [completed work](../archive-history/completed/2026-11-26.md#settings-reset-app).

- [ ] Verify the reset action in an actual Tauri WebView with disposable learner data; native tests and Chrome IPC fixtures passed, but real user data was not cleared during development.

## Tauri desktop acceptance and follow-up (2026-11-26)

Implementation of the approved database/assets separation is recorded in [completed work](../archive-history/completed/2026-11-26.md#tauri-frontenddatabase-separation). Build and Chrome probes pass for that scope; full CLI validation retains three independently reproduced pre-existing failures.

- [x] User reports testing the separated application with Tauri on 2026-11-26. Exact flows and installer acceptance were not supplied.
- [ ] Build/install using `cargo make desktop-build`; verify resource/DLL resolution and Practice-history persistence across restarts.
- [ ] Separate follow-up: approve and implement remaining native personal-data adapters and the trusted speech IPC bridge.
- [ ] Separate maintenance: fix pre-existing CLI archive restore/release-name test failures and the Docs validator object-key-order comparison; dependency changes remain unapproved.

## R684 Zipformer cache compile corrections and checkpoint port validation (WIP)

- [x] Fix E0689 floating-point inference in `audio_codec.rs` resampling regression.
- [x] Fix E0308 3D/4D `Tensor::zeros` closure inference in Zipformer cache initialization.
- [x] Require output cache-port dtype, rank and fixed channel dimensions during checkpoint preflight.
- [x] Extend regression tests for 4D cache, cache update rejection and malformed output metadata.
- [ ] Run Windows `cargo fmt`, locked check, unit and parity-runner tests.
- [ ] Export the real 32/128 ONNX decoder and joiner weights, generate reference fixtures and pass full numerical parity.
- [ ] Implement the exact native streaming Zipformer encoder and validate reference Mandarin transcription.

## R683 checkpoint-backed Zipformer head numerical parity (WIP)

- [x] Correct decoder convolution to bias-free construction matching the ONNX initializer inventory.
- [x] Implement an optional ONNX Runtime reference generator using fixed token contexts and 512-wide encoder vectors.
- [x] Implement native Candle parity runner reading converted safetensors and comparing complete decoder/joiner outputs.
- [ ] Run Rust build, tests and strict Clippy checks on Windows.
- [ ] Generate ONNX reference fixture and converted head weights from the pinned 32/128 checkpoint.
- [ ] Compare all decoder outputs and joiner logits; debug any numerical mismatches.
- [ ] Implement actual Zipformer encoder and streaming-state transitions, then verify Mandarin recognition.

## R682 checkpoint-specific native streaming transducer (WIP)

- [x] Record observed chunk-32/left-128 input/cache layout from user-provided Windows checkpoint inventory.
- [x] Add typed 12-group Candle cache allocation and strict encoder/decoder/joiner metadata validation.
- [x] Add checkpoint-namespace Candle decoder and joiner with grouped convolution and pure shape tests.
- [ ] Run Windows compile/tests; correct any Candle API incompatibilities.
- [ ] Convert actual ONNX decoder and joiner initializers to safetensors; validate output against upstream for exact token sequences and logits.
- [ ] Implement checkpoint-compatible streaming Zipformer encoder and frontend and confirm reference parity.

## R680 streaming checkpoint inspection and native integration (WIP)

- [x] Inspect upstream checkpoint repository file layout and model-card licensing signals.
- [x] Implement bounded, dependency-free ONNX protobuf graph/initializer metadata reader with synthetic regression fixtures.
- [x] Add encoder/decoder/joiner/token inventory CLI and validation of missing files, graph shape contracts and vocabulary length.
- [x] Implement strict data-only conversion for embedded ONNX raw initializers into safetensors, without overwriting existing files.
- [ ] Run Windows `cargo fmt --all`, locked speech-crate check and unit tests, then fix compiler or Clippy findings.
- [ ] Pin exact upstream revision and acquire permitted ONNX/token assets; preserve SHA-256 hashes, notices and inventory output.
- [ ] Inspect real checkpoint graph/cache interfaces and resolve incompatibilities discovered by the tools.
- [ ] Implement exact Candle Zipformer2 subsampling and encoder modules, frontend parity and conversion/mapping validation.
- [ ] Run real Mandarin chunked recognition and ONNX-versus-Candle numerical parity before adding transport or Live UI.

## R679 native streaming ASR implementation (WIP)

- [x] Add dependency-free incremental 80-bin log-mel frontend and chunking-invariance tests.
- [x] Add strict token-list reader and raw token-piece lookup without claiming SentencePiece decoding.
- [x] Add streaming pipeline boundary joining bounded PCM, features, abstract encoder and the existing transducer session.
- [x] Document provisional Apache-2.0 streaming Zipformer model reference and its unverified checkpoint/licensing provenance.
- [ ] Run Cargo formatting, locked check, unit tests and Clippy on Windows and fix any failures.
- [ ] Obtain approved model revision and actual weights, token assets, parameter metadata and license notices.
- [ ] Establish frontend numerical parity, including Kaldi frame alignment and final-edge handling.
- [ ] Implement checkpoint-compatible Zipformer2 encoder/stream caches and reference decoder parity.
- [ ] Test real Mandarin streaming recognition before transport or Live integration.

## R678 native streaming ASR implementation (WIP)

- [x] Add separate experimental streaming ASR module without altering production Qwen3-ASR.
- [x] Implement bounded incremental PCM queue, explicit finalization and backpressure.
- [x] Implement persistent transducer decoding context, blank handling, cancellation and token budgets.
- [x] Implement native Candle stateless transducer head and bounded causal attention-cache building block.
- [x] Add checkpoint-free regression tests and describe the unsupported checkpoint boundary.
- [ ] Run locked `cargo fmt --all`, `cargo check --locked -p zipeek-tts-stt`, `cargo test --locked -p zipeek-tts-stt --lib` and Clippy on Windows.
- [ ] Select and license-check a specific Mandarin streaming Zipformer checkpoint before conversion.
- [ ] Port actual Zipformer encoder, Fbank frontend and tokenizer, then establish numerical parity with reference inference.
- [ ] Test streaming Mandarin partial transcripts and bounded session lifecycle before any WebSocket/Live integration.

## R677 Record playback and result acceptance (WIP)

- [x] Prevent stop playback from changing reference waveform panel height.
- [x] Align Heard status icon and add an explanatory accessible hover tooltip.
- [x] Resolve recognized sentence pinyin through the existing batched dictionary query; ignore stale resolution responses.
- [x] Inspect the provided WAV for duration, PCM format, levels and audible speech.
- [ ] Run locked WASM check and web tests in Windows.
- [ ] Test browser layout, tooltip, pinyin loading, context ambiguity, target switches and microphone-to-transcript association in Chrome.

## R675 Record reading and audio acceptance (WIP)

- [x] Author batched character/word pinyin candidate query and conservative aligned presentation with unambiguous readings only.
- [x] Replace Next word text with accessible dice-only control.
- [x] Prepare TTS reference after target selection; debounce custom text and reuse prepared waveform/audio for playback.
- [x] Retain generated reference audio while recording; release blob URLs when replaced or route unmounts.
- [ ] Run Rust formatting, locked WASM build and unit tests on Windows.
- [ ] Verify dictionary query correctness and polyphonic words/long sentences with installed DuckDB.
- [ ] Verify Chrome idle/loading/retry/ready/playback and failure/cancellation, including rapid target changes, recording/replay and responsive pinyin layout.

## R674 Record waveform activation and audio inspection

- [x] Hide the empty speech recognition results card until a transcript is present.
- [x] Make the reference waveform start TTS playback when idle; preserve real-time fill and seek during playback and a stop control.
- [x] Allow a retained recording's waveform to initiate replay and add an explicit browser WAV download for the exact WAV bytes submitted to STT.
- [x] Keep downloads user-initiated, origin-scoped, with short-lived blob URLs and no server persistence or new dependencies.
- [ ] Run cargo fmt --all -- --check, cargo check --locked -p zipeek-web --target wasm32-unknown-unknown, cargo test --locked -p zipeek-web and cargo test --locked -p zipeek-browser on Windows.
- [ ] Verify click, keyboard activation, seeking, recording retention, download byte identity and audio playback in Chrome, including failed STT, cancellation and target changes.
- [ ] Inspect a real exported WAV for sample rate, leading/trailing silence, clipping and truncated capture. Use evidence to tune VAD and microphone processing; do not attribute unexpected transcripts to VAD without a recording.

## R673 Record speech activity auto-stop acceptance

- [x] Implement browser-only automatic stop using adaptive noise-floor speech activity, speech-start confirmation, hysteresis and trailing-silence detection.
- [x] Distinguish HSK-word and custom-sentence silence windows while keeping manual stop, cancellation, user privacy and a hard audio-duration cap.
- [x] Avoid empty-speech STT submissions when the detector never hears sustained speech; add pure detector regression tests for noise, natural pauses and timer stalls.
- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, `cargo test --locked -p zipeek-browser`, and `cargo test --locked -p zipeek-web` on Windows.
- [ ] In Chrome test HSK single words and custom sentences with normal speech, soft speech, background fan noise, short pauses, initial silence, no speech, rapid double stop, manual stop, permissions denied, cancellation, server offline, target changes and maximum-duration completion.
- [ ] Tune thresholds using measured microphone conditions; energy-based browser VAD is not a trained voice classifier and may misclassify strong non-speech noise.

## R672 Record playback progress acceptance

- [x] Render filled/unfilled actual PCM waveform bars based on the active browser audio position for TTS and microphone replay.
- [x] Add focusable, keyboard-operable seek control with bounds validation; reset progress after completion/cancel.
- [x] Highlight approximate reference Hanzi by playback progress without claiming word-level timing; keep updates inside panel components.
- [ ] Run Windows `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, and affected unit tests.
- [ ] Test Chrome reference/replay playback, seek, reset, target switches, accessibility, responsive layout and reduced-motion preference.

## R671 Record recognition comparison acceptance

- [x] Remove redundant comparison disclaimer and large status badge.
- [x] Apply compact warning/success icon and distinctive recognized Hanzi text styling.
- [x] Show dictionary-grounded pinyin for expected HSK words and recognized vocabulary where resolvable; do not fabricate sentence pronunciations.
- [x] Correct the stale R670 reducer test field from `has_audio` to `recording_id`.
- [ ] Run `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, and `cargo test --locked -p zipeek-web` on Windows.
- [ ] Verify matched/mismatched result colour contrast, pinyin display, custom sentences, keyboard and screen-reader state, and local and hosted route behaviour in Chrome.

## R670 Record component architecture acceptance

- [x] Extract Record route state/reducer, controller, audio resources, target selector, reference panel, recording panel, waveform and recognition result into focused modules.
- [x] Replace `has_recording` reducer flag with typed recording identity; derive replay availability from matching owned WAV resource.
- [x] Retain captured WAV and waveform in browser memory only, and replay without copying the WAV buffer.
- [x] Preserve HSK and custom-sentence targets, local speech policy, existing controls and stale-async-response checks.
- [x] Add audio-resource identity test and migrate reducer tests.
- [ ] On Windows run `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, and `cargo test --locked -p zipeek-web`; resolve any findings.
- [ ] Test `/record` in Chrome: word/bank/mode/sentence switches during TTS/STT, waveform rendering, capture, stop, cancel, replay on STT success and error, microphone denial, narrow layout and keyboard semantics.
- [ ] Verify localhost and `/zipeek/` hosted route paths and address any browser lifecycle issues before acceptance.

## R669 Record sentence and actual waveform acceptance

- [x] Add a custom sentence target to the Record reducer; clear stale transcript and recording whenever the target changes.
- [x] Replace static pulse bars with waveform peaks derived from decoded TTS PCM, submitted microphone PCM16 WAV, and the live microphone analyser.
- [x] Make microphone and replay actions icon-only with accessible labels; hide idle status.
- [x] Keep the existing HSK bank selector and enforce the same loopback-only speech policy.
- [ ] Run locked WASM build, Rust formatting, web and browser tests on Windows.
- [ ] Verify reference waveform and playback, microphone waveform, captured waveform/replay, sentence transcription, and cancellation during mode or target switches.
- [ ] Verify keyboard, screen reader names and compact responsive layout in Chrome.

## R667 Record capture playback and reducer acceptance

- [x] Replace independent coupled Record page states with a reducer and explicit workflow transitions.
- [x] Preserve submitted microphone WAV bytes in browser memory and provide replay/stop without another STT call or filesystem persistence.
- [x] Retain WAV on transcription error and clear it on word/bank change or next recording.
- [x] Add reducer tests for replay retention, cancellation and bank reset.
- [ ] Run Windows `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, and `cargo test --locked -p zipeek-web`.
- [ ] Test Chrome speech flow at `localhost:1420/#/record`, including failed STT, replay while service is offline, microphone denial, async cancellation, permission prompts, and playback termination.
- [ ] Verify compact desktop layout and keyboard/screen-reader accessible replay and stop controls.

## R666 compact Record UI acceptance

- [x] Replace Record HSK dropdown with segmented HSK standard and level controls.
- [x] Remove redundant descriptive headings and condense Record interactions.
- [x] Show only a stop icon during active reference audio playback, retaining an accessible name and tooltip.
- [ ] Run Rust formatting, WASM build, and relevant frontend tests on Windows.
- [ ] Verify viewport fit without page-level scrolling at typical desktop heights, narrow responsive layout, and keyboard/screen-reader operation.
- [ ] Verify actual Play/Stop, HSK selection, recording, STT comparison and service-offline UI in Chrome.

## R665 speech result cache acceptance

- [x] Author bounded per-service in-memory TTS WAV LRU with 20 entries and 32 MiB cap, backend-qualified keys, hit recency, successful-only insertion and unit tests.
- [ ] On Windows, run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt` and `cargo test --locked -p zipeek-tts-stt --lib`.
- [ ] Verify real `/v1/tts` repeat requests return identical cached WAV bytes, a new service instance starts empty, and memory is bounded under varying output lengths.
- [ ] Verify the Record route plays repeated audio correctly, and confirm how UI should expose a fresh stochastic rendition versus cached replay.
- [ ] Determine whether any persistent disk cache or STT transcript cache is needed based on measured real usage; neither is currently approved or implemented.

## R664 Qwen3-ASR CUDA attention layout acceptance

- [x] Make batched Q/K score and attention/value operands contiguous at the encoder's CUDA matmul sites.
- [x] Apply the same contiguous-operand correction to the autoregressive text decoder matmul sites.
- [x] Add checkpoint-free tensor-layout regression tests matching the encoder and decoder attention dimensions.
- [ ] On Windows run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt` and `cargo test --locked -p zipeek-tts-stt --lib`.
- [ ] Run a single CUDA STT transcription with `ZIPEEK_STT_DEVICE=cuda` before rerunning 20-request model-backed acceptance; confirm expected transcript and no additional CUDA backend layout failures.
- [ ] Compare warm CUDA STT latency against the user-reported ~43-second CPU baseline and record device/VRAM. Do not claim performance speedup without measurement.
- [ ] Verify R663 Record route remains buildable and works with the CUDA-enabled native speech server.

## R663 Record route acceptance

- [x] Add `/record` route and microphone `Record` drawer navigation.
- [x] Load a randomly selected word from chosen installed HSK bank and present Hanzi, pinyin and meaning.
- [x] Use existing local speech endpoint adapters for reference playback and microphone STT; expose live microphone level, cancellation and retry/error states.
- [x] Exclude TTS developer controls, API event log, and ungrounded confidence scoring.
- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown` and browser tests on Windows.
- [ ] Verify `#/record` under development root `/` and deployed `/zipeek/` routing, including responsive drawer and keyboard.
- [ ] Manually verify microphone permission denial, silence, stop/automatic 30-second completion, TTS playback, switching HSK banks, cancellation, and server-offline errors at trusted localhost:1420.
- [ ] Evaluate transcription accuracy separately from exact-string comparison; consider recorded WAV waveform and playback waveform analysis if a genuine waveform is later required.

## R662 speech runtime reliability and acceptance

- [x] Replace general string-backed speech failures at the service/HTTP boundary with typed model-loading, inference, codec and cancellation errors.
- [x] Resolve typed CPU/CUDA configuration once; construct each speech worker only once at startup.
- [x] Publish non-sensitive engine failure category and stopped lifecycle status; avoid discarding a model for cancellation before inference begins.
- [x] Add explicit server-triggered worker shutdown commands and bounded shutdown timeout.
- [x] Author unit tests for cancellation-before-inference, owned permit lifetime, bounded queue and worker shutdown, plus HTTP failure-body redaction tests.
- [x] Author ignored model-backed 20-request warm TTS/STT and concurrent independent-worker acceptance tests and log latency observations.
- [ ] Rerun repeated STT acceptance with the documented `你好`/`您好` reference-greeting variants; verify identical transcripts over 20 requests and readiness. The Windows concurrent TTS/STT test passed after 16 kHz WAV fixture normalization. Independently evaluate exact transcript accuracy against the original `你好世界` text.
- [ ] Run Windows Rust formatting, locked build, Clippy, unit/HTTP tests and ignored model-backed acceptance; address compiler and Clippy issues.
- [ ] Reproduce `不便` EOS and the Mandarin neural WAV/Opus/MP3 test on R662, and listen to speech output and reference transcription.
- [ ] Validate shutdown with in-flight CUDA inference, cancellation/timeout permit retention, failure-recovery reinitialization, and backpressure through the live HTTP service.
- [ ] Measure and compare warm latency (p50/p95), RTF, VRAM and CUDA sync/allocator profiles against R661 before approving model/KV-cache optimizations.
- [ ] Trace MSVC LNK4098 `LIBCMT`/CRT conflict to its native dependency using `/VERBOSE:LIB`; do not hide warnings by default.
- [ ] Regenerate maintained docs index with the approved `cargo make docs` workflow and recheck the six frozen-history link warnings.

## R661 native speech worker hardening and acceptance

- [x] Move persistent Qwen3-TTS and Qwen3-ASR ownership to independent bounded channel workers; retain a single active inference permit per backend across request timeouts.
- [x] Expose distinct neural engine installation, loading, warming, ready and failure status; align capability booleans with runtime readiness.
- [x] Validate device selection once during native service construction and sanitize unexpected backend/IO errors at the HTTP boundary.
- [x] Replace fixed MP3 decoded-duration expectation with duration measured from the generated WAV, allowing bounded codec delay.
- [ ] Run Windows locked `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, `cargo clippy --locked -p zipeek-tts-stt --all-targets -- -D warnings`, and `cargo test --locked -p zipeek-tts-stt`.
- [ ] Run real-model startup warm-up and repeated TTS/STT inference; verify intelligibility, transcript accuracy, cancellation, timeout permit retention, graceful shutdown and recovery after failed inference.
- [ ] Reassess dedicated OS-thread worker shutdown and bounded-channel backpressure using actual Windows CUDA traces before accepting R661.

## R660 Qwen3-TTS generation and EOS acceptance

- [x] Replace greedy first-codebook and sub-codebook inference selection with the model-published sampling defaults, without new dependencies.
- [x] Apply repetition penalty to generated first-codebook history and bound pathological repeated acoustic tokens.
- [x] Retain reproducible diagnostic traces and update the engine-reuse test for nondeterministic output.
- [ ] Run Rust formatting, locked build and unit tests on Windows.
- [ ] Run ignored CUDA diagnostic for `不便`, confirm EOS is sampled before the 375-frame limit, and listen to the generated audio.
- [ ] Run model-backed synthesis for `你好世界`, `儿科`, `人权`, `伤口` and punctuated Mandarin; compare quality and runtime with R659.
- [ ] Compare prompt construction, tokenizer token IDs, and default sampling with official Qwen3-TTS reference if any model-backed regression remains.

## R658 practice viewport and audio preparation acceptance

- [x] Remove redundant `min-h-dvh` and vertical padding in the nested practice-session shell, preserving the session page's viewport sizing.
- [x] Display only `Preparing...` alongside the progress indicator while synthesizing prompts, retaining assistive progress announcements.
- [x] Keep failed-word and selected-vocabulary diagnostics in the error state.
- [ ] Verify no unnecessary vertical scrollbar at standard desktop viewport heights in audio-preparation and failure states.
- [ ] Run locked WASM build and practice E2E tests on Windows.
- [ ] Investigate and correct Qwen3-TTS acoustic EOS failure for `不便` using checkpoint-backed token traces; this is separate from the viewport fix.

## R656 live microphone flow acceptance

- [x] Measure microphone RMS levels from real captured audio rather than animating decorative bars.
- [x] Keep explicit recording, transcribing, idle and results states with early-stop support.
- [x] Release microphone tracks, Web Audio resources and meter polling on completion and cancellation.
- [x] Update Settings browser E2E source assertions for the new controls.
- [ ] Run WASM compilation and browser acceptance across local and browser speech providers, microphone permission denial, 5-second automatic stop, manual early stop, silence and disconnected service.

## R655 Speech Settings test UX validation

- [x] Replace routine status labels and success narration with user-centered feedback.
- [x] Add icon control states and compact activity bars using Tailwind utilities.
- [x] Change test phrase to `今天天气晴朗。` in local and browser speech.
- [x] Update Speech Settings E2E source assertions.
- [ ] Verify locked WASM build and E2E browser interaction, including microphone and playback cancel behavior.
- [ ] Validate live ASR capture and ensure no false live waveform claim.

## R653 Qwen3-TTS text pretokenization

- [x] Remove the Hanzi-only text restriction and preserve byte-level BPE model weights/merges.
- [x] Add punctuation, spaces, digits and mixed Chinese/Latin unit regressions.
- [x] Add a checkpoint-backed ignored regression for the original Settings test sentence.
- [ ] Execute locked Rust tests and the punctuated Mandarin neural synthesis on Windows CUDA.
- [ ] Compare token IDs with the official Qwen2TokenizerFast on representative punctuation, contractions and Unicode whitespace cases.
- [ ] Verify Settings Test voice playback from the trusted localhost origin.

## R652 WIP: Trailing-dot localhost speech origin

- [x] Accept the exact `http://localhost.:1420` browser origin alongside existing approved development origins in the web transport and native speech CORS.
- [x] Initialize browser speech origin trust from `AppEnvironment.window` in web setup rather than retrieving a global Window inside the browser speech allowlist.
- [x] Add source-level origin and live-server preflight regression checks for the trailing-dot form and reject lookalike origins.
- [ ] Verify locked Rust unit/HTTP tests, wasm32 compilation and browser Settings plus Audio Matching in Chrome at `http://localhost.:1420`.

## R651 speech session preflight verification

- [x] Block audio Matching session creation until the local service reports speech synthesis ready, with a final preflight before persistence.
- [x] Replace Speech Settings native radios with established Tailwind custom controls.
- [ ] Run WASM build and browser tests for provider selection, unavailable TTS, stopped server, and resumed selection.

## R650 Matching visual regression

- [x] Stabilise completion action and wrong-answer accessible feedback node.
- [x] Feather horizontal progress artwork at the source-image boundary.
- [ ] Verify actual Chrome viewport positioning and E2E completion behaviour in both horizontal creature themes.

## R646 eager speech initialization verification

- [x] Eagerly load Qwen3-TTS and Qwen3-ASR during server startup, perform real TTS warm-up, retain warmed engine, and fail startup on installed-but-broken checkpoints.
- [x] Add server startup regression and ignored real-model CUDA verification.
- [ ] Run locked formatting, Clippy, complete tests and real model-backed warm-up tests on Windows.
- [ ] Measure first-request benefit separately from steady-state RTF; add ASR forward warm-up with a representative licensed/approved audio fixture if needed.

## R645 speech build regression

- [x] Repair the specific R644 E0308 and E0600 compiler errors without dependency or inference changes.
- [ ] Run locked Windows Rust compilation, Clippy, codec tests, and ignored model-backed neural TTS tests. Confirm generated MP3 decodes and matches duration.

## R644 neural speech runtime verification

- [x] Add frame-based Shine encoding, independent MP3 validation and alternate rate attempts, preserving strict failure for unusable output.
- [x] Cache Qwen3-TTS rotary tables and common STT resampling FIR phases.
- [x] Implement quality-factor Accept negotiation, runtime readiness flags, typed timeout response and opt-in CUDA STT.
- [x] Add targeted regression tests and warm-inference timing output.
- [ ] Compile and test R644 on Windows with actual model checkpoints, including MP3 round-trip and the optional STT CUDA path.
- [ ] Measure GPU/CPU throughput and VRAM with repeated requests, and validate Mandarin output against labelled native-speaker fixtures.
- [ ] Implement bounded audio chunk streaming only after defining the client wire contract and measuring actual latency requirements.

## R643 neural speech performance validation

- [x] Author TTS greedy token selection on GPU, autoregressive mask bypass, codec mask reuse, and band-limited STT downsampling.
- [x] Add deterministic CPU tests for token selection, codec attention mask and alias rejection.
- [ ] Run Windows Cargo formatting, Clippy, locked tests and actual CUDA Qwen3-TTS acceptance; compare WAV intelligibility and STT recognition before/after.
- [ ] Measure warm-request latency and VRAM; R642 MP3 regression remains unverified until executed.

## R637 native neural Mandarin synthesis acceptance

- [x] Author Zipeek-owned acoustic-code generation and 16-codebook neural waveform decoding under `zipeek-tts-stt/src/candle_tts/qwen/`; no new dependencies.
- [x] Wire the native implementation into the existing `/v1/tts` route and keep a checkpoint-backed playable WAV acceptance gate.
- [ ] **Blocking:** Compile/format and execute `neural_tts` checkpoint-backed tests with the local downloaded model; fix real tensor names, shapes and output errors; verify audible Mandarin and the TTS → STT test. Do not claim neural TTS operational until that evidence is obtained.

## Native speech implementation (R619 and later)

- [x] R634: author real Qwen3-TTS CustomVoice inference boundary and wire model-backed waveform synthesis into the HTTP API with strict non-silent WAV checks.
- [ ] R634: explicitly approve and resolve pinned `talechime-qwen3-tts` dependency into root Cargo.lock, then verify Windows compile, CUDA and true model-backed Mandarin synthesis plus live STT roundtrip. This remains WIP until confirmed.


- [x] R633: author owned in-process speech server lifecycle API, graceful shutdown and direct `zipeek-e2e` integration tests without new dependencies.
- [ ] R633: run Windows `cargo fmt --all -- --check`, locked speech/E2E Rust tests and verify Ctrl+C graceful shutdown.

- [x] R632: add real-server TTS-generated Mandarin WAV save test and strict TTS-to-STT round-trip acceptance test.
- [ ] R632: run both ignored Windows acceptance tests and verify the generated WAV is pronounced correctly before accepting exact STT transcription.

- [x] R631: correct the default model checkpoint locations to `models/speech` while preserving separately packaged eSpeak resources and explicit overrides; add regression coverage for binary-ancestor discovery.
- [ ] R631: verify Windows Rust compile, tests and actual model availability with downloaded checkpoints.

- [x] R630: author pure-Rust Symphonia compressed input and Opus output with strict type/length validation and Rust/HTTP regression coverage.
- [ ] R630: resolve the two approved dependencies into root `Cargo.lock` on Windows, run locked Rust and real-server tests and verify audio interoperability; no Cargo toolchain available in authoring environment.


- [x] R629: remove experimental STT toggle and hardcode deterministic embedded Mandarin WAV fixture plus fixed expected text in Rust/Node tests.
- [ ] R629: verify embedded Mandarin fixture is intelligible and correctly says the reference phrase; run Windows Rust and real checkpoint acceptance tests.

- [x] R628: author Rust `tests/` suite for real-process speech HTTP routing/security, explicit unavailability, real eSpeak WAV output, Qwen checkpoint mapping and real reference-audio transcription; model-backed checks ignored by default.
- [ ] R628 verification: run locked Windows Rust test suite, strict TTS/STT inference tests and actual Mandarin reference transcription; fix any observed compilation or runtime issues.

- [ ] R626: run the actual speech HTTP E2E harness on Windows; require eSpeak WAV success, check STT correctly returns unavailable, and later require real STT decoding with `--require-stt`.


- [x] Approve Candle-native STT/TTS direction: Qwen3-ASR for Mandarin, Kokoro Chinese initial TTS and optional Qwen3-TTS research.
- [x] Align model preflight contracts with published Qwen3-ASR and Chinese Kokoro file layouts and add shared Candle device/tensor smoke tests.
- [x] R624: user-provided locked Windows Rust tests pass: 44 passed, 0 failed, with unused-code and LNK4098 warnings. The actual checkpoint execution is not yet verified.
- [ ] R625: run locked Rust check/tests and ignored full acoustic encoder smoke test against local safetensors; validate tensor names and shapes and compare Mel and encoder numeric outputs to upstream.
- [ ] R627: compile and numerically verify the experimental 28-layer text decoder, prompt/tokenizer integration and actual Mandarin reference audio transcription. The source is implemented but checkpoint execution remains unverified.
- [ ] Complete Qwen3-ASR 128-Mel numerical parity and multi-clip recognition accuracy before default activation.
- [ ] Implement safe Kokoro weight conversion/load, Mandarin phonemization and synthesis with reference audio comparison.
- [ ] Investigate Qwen3-TTS with its speech-token codec after initial native Mandarin synthesis works.
- [ ] Keep pronunciation assessment independent of STT transcription and retain endpoint readiness gating until verified.

## Documentation and Knowledge Points decisions

- [x] Reconcile root R596/R600 notes and historical R310 E2E note into subject owners and one review history.
- [x] Give prospective R608 Knowledge Points and R609 engine architecture stable subject-based owners.
- [ ] Run real browser Docs reader navigation and deep-link/search smoke after the documentation regrouping.
- [x] Consolidate the approved R608 Knowledge Points category, eligibility, immutable catalogue, quota, debt and relapse product rules.

## R613 personal OPFS persistence simulator

- [x] Implement explicit Start/Stop controls in development DuckDB diagnostics for a maximum of ten synthetic saves at five-second intervals.
- [x] Reuse existing personal DuckDB-WASM/OPFS database, worker and Web Lock; isolate synthetic state in a dedicated simulator table.
- [x] Author atomic revision transactions and close/reopen read-back verification; provide a button to verify the saved state after reload.
- [ ] **Blocking:** Run locked Rust/WASM verification and browser acceptance for all ten cycles, reload, Stop, cross-tab lock and storage failure behavior.
- [ ] Implement durable R608 observations/scoring-state transactions and real Practice adapters in follow-on scope; simulator does not constitute integration.

## R612 Comeback and evidence boundary

- [x] Add hidden 100-success Comeback rule for ordinary disqualified days, preserving debt and severe-relapse constraints.
- [x] Author focused Comeback, manifest-validation and atomic batch ledger tests.
- [x] Add pure manifest-aware target/modality/cue validation and an all-or-nothing in-memory ledger with replay detection.
- [ ] **Blocking:** Execute locked Rust tests, rustfmt and WASM checks, then fix compiler and behavioral findings.
- [ ] Integrate fixed-catalogue target authorization, certified semantic context families, durable atomic browser storage and cross-tab coordination. Pure ledger cloning is not durable persistence.
- [ ] Wire the hidden Comeback notification and validated exercise adapters into actual Practice; verify via browser tests.

## R611 Knowledge Points core acceptance and integration

- [x] Replace R609 uniform five-dimension prototype with Pinyin/Hanzi/Vocabulary/Grammar category-specific scoring, immutable catalogue assumptions and balanced polyphonic evidence.
- [x] Author focused pure Rust transition tests and update canonical R608 owner documentation.
- [ ] **Blocking:** Run pinned rustfmt and locked `cargo test -p zipeek-pedagogy`, Clippy and WASM checks; fix compiler or behavioral failures.
- [ ] Implement validated evidence manifest boundary, durable transactional persistence, fixed catalogue ID/readings mapping, policy-reset transaction, exercise adapters and real statistics integration.

## R609 historical candidate acceptance (superseded by R611 core)

- [x] Add a pure Rust knowledge scoring candidate under `zipeek-pedagogy/src/knowledge/`, without changing the currently running legacy projection.
- [x] Add focused transition tests for daily qualification, debt, relapse, context deduplication and epoch fencing.
- [ ] **Blocking:** Run locked Rust/rustfmt/Clippy/WASM acceptance; correct actual compiler errors before accepting the source candidate.
- [x] Approve R608 numerical debt rules, calendar-day basis and same-day ordering. Semantic context-equivalence certification remains a separate unimplemented evidence-validator task.
- [ ] Implement durable validated evidence journal, catalogue identities, reset boundary, AI evaluator schema, modality-aware exercise adapters and real Statistics/dragon integration in approved follow-on scope.

## R607 AI Settings request lifecycle acceptance

- [x] Move the AI page, reducer and request orchestration into directory-owned modules while keeping the saved configuration contract intact.
- [x] Reject rapidly duplicated connection tests before rerender and stale async completions after config edits, dismissal, restart and unmount.
- [x] Add focused Rust tests for request-gate and reducer ticket invariants.
- [ ] **Blocking:** Run rustfmt, locked Rust tests/WASM build and Chrome Settings AI E2E, then fix any failures.
- [ ] Implement transport-level cancellation only after approval; dismissal currently only invalidates UI publication.

## R606 Practice session-store acceptance

- [x] Move the store root to a directory module while preserving consumer-facing imports.
- [x] Separate storage decoding and inventory predicates from browser storage I/O.
- [x] Detect session inventory key-set changes across an enumeration, with focused source-level regression tests.
- [ ] **Blocking:** Run rustfmt, locked Rust/WASM compilation, unit tests and cross-tab browser storage tests.
- [ ] Design real cross-tab write serialization separately; key snapshots do not provide atomic compare-and-swap.

## R605 Generated Practice state-ownership refactor

- [x] Split generated translation and image vocabulary into typed, isolated child components and reduce the root module to validation/dispatch.
- [x] Derive translation completion from the evaluation result and represent image loading with one typed lifecycle enum.
- [x] Preserve deterministic seeded tile ordering, validated payload decode, callbacks, and saved data formats.
- [ ] **Blocking:** Run rustfmt, locked Rust/WASM tests and Chrome generated-exercise E2E, including delayed AI result and image-load failure.
- [ ] Review transport-level cancellation as a separate approved service change.

## R604 Practice local-session refactor

- [x] Separate page composition, restoration, generated-result reconciliation, progress persistence, history effect, and exercise rendering.
- [x] Use route-keyed page mounting to reset route-scoped state and invalidate stale async publication.
- [x] Reload authoritative session on progress revision conflict; preserve existing store validation.
- [ ] **Blocking:** Run rustfmt, Rust unit tests, locked WASM build and browser race/restore/finalization tests.
- [ ] Audit atomic cross-tab persistence and actual cancellation of underlying generation requests before claiming those contracts complete.

## R603 Practice route and session invariants

- [x] Reject stale generated Practice results when the route session ID changes without unmounting.
- [x] Validate publicly constructed exercise instances before pushing or recording completed results, with regression tests.
- [ ] **Blocking:** Execute locked Rust tests, WASM build and Chrome route-change race tests; correct any errors before acceptance.
- [ ] Continue review of Practice restore races, external generation cancellation, concurrent session storage writes and broader R591 debt.

## R602 correctness and ownership acceptance

- [x] Extract exercise-session transitions and tests into directory-owned modules.
- [x] Reject repeated completion and invalid blank Statistics identifiers; add regression tests.
- [ ] **Blocking:** Run locked Rust unit tests, rustfmt, Clippy, WASM compilation and affected browser E2E; correct any compiler diagnostics.
- [ ] Continue the R591 debt register: Practice async lifecycle, canonical Statistics IDs, storage transaction failure paths and AI cancellation.

## R601 ten-target refactor acceptance

- [x] Inspect all ten approved R600 source targets and extract separable query/validation/presentation/domain responsibilities.
- [x] Preserve the existing feature APIs and avoid duplicate reducer or async state ownership.
- [x] Add deterministic and empty-input generated-tile-order regression coverage.
- [ ] **Blocking:** Run pinned Rustfmt, locked native tests/Clippy, locked WASM check and focused Chrome acceptance against the R601 archive, then correct all findings.
- [ ] Follow-up: Further isolate the Practice async controllers, AI connection lifecycle, DB transaction coordination, and remaining large exercise runtime controllers after compiler evidence.

## R600 Settings architecture acceptance

- [x] Move AI configuration reducer, test lifecycle and UI into dedicated `settings/ai.rs`.
- [x] Move Hanzi font/size state and UI into `settings/characters.rs`.
- [x] Move matching audio preference into `settings/practice.rs`.
- [x] Reduce `settings/mod.rs` to module declarations; `settings/page.rs` owns routing/composition.
- [ ] **Blocking:** Run cargo fmt, locked tests, WASM compile and browser regression for all Settings tabs and base paths; correct compiler and behavioral findings before acceptance.

## R600 Learning Settings acceptance

- [x] Replace duplicate recording control and activity inspection with one Settings > Learning recording toggle.
- [x] Separate unavailable pedagogical assessment, personal data management and worker diagnostics into focused feature modules.
- [x] Keep portable backup exclusions and both delete confirmations intact.
- [ ] **Blocking:** Run `cargo fmt --all -- --check`, locked web tests, locked WASM build and Learning Settings E2E in both `/` and `/zipeek/` deployments.
- [ ] Review focus, storage write failure feedback, same-tab/cross-tab preference changes and mobile layout in Chrome.

## R598 evidence fail-closed checkpoint

- [x] Prevent a disabled pedagogy worker or a failed committed-journal analysis from exposing stale cached projections.
- [x] Add focused tests for disable and retry recovery without changing recognition rules or pending-job policy.
- [ ] **Blocking:** Run locked `zipeek-pedagogy` Rust tests and WASM/browser validation; do not claim release acceptance based on Node tests alone.
- [ ] **P0:** Build and approve a consented character-level evidence producer, durable journal, strict readback/provenance, single-tab write authority and HSK target/denominator before lighting any dragon body region.

## R597 header dragon presentation and evidence follow-up

- [x] Replace the tiny recording-dependent dragon SVG with separate detailed illustrative artwork and an explicit unassessed state.
- [x] Remove the centered global header title and restore contextual page headings; adjust source E2E expectations.
- [ ] **Blocking:** Run locked Rust/WASM compilation and keyboard/desktop/mobile browser acceptance at `/` and `/zipeek/`, including visual comparison to the approved dragon mockup.
- [ ] **P0:** Implement an approved, durable qualifying-pedagogy evidence journal and provenance gate, selected HSK target/denominator and readback before enabling an illuminated dragon progress body.
- [ ] **P0:** Verify no false mastery indication when history saving is off, storage is missing, worker is unavailable, or the UI is in preview mode.

## R596 Learning Data settings and dragon acceptance

- [x] Inspect the maintained Learning Data/pedagogy and header-dragon contracts against source and screenshots.
- [x] Expose the saved Practice recording preference and clarify that pedagogical assessment remains unconnected.
- [x] Group history, backup, deletion and worker diagnostics into clearer progressive disclosure sections.
- [ ] **Blocking:** Compile, format, and browser-test R596 in Chrome for both `/` and `/zipeek/` paths, including keyboard and preference change events.
- [ ] **P0:** Implement the approved long detailed dragon using real persisted pedagogical evidence; do not substitute session counts, a static bar or the current small SVG. Confirm visual art and complete the worker/storage integration first.
- [ ] **P0:** Verify personal Practice history persistence, failures, and authoritative pedagogy-read pipeline before indicating live proficiency.

## R595 Statistics data directory-module correction

- [x] Remove the mixed `data.rs`/`data/` layout and use `data/mod.rs`.
- [x] Extract typed records/projections, source orchestration, reducer transitions and tests to dedicated files.
- [x] Preserve `data::...` import contracts and Statistics-scoped re-export visibility.
- [ ] **Blocking:** Run locked Rustfmt, web Rust tests and WASM compilation on the exact corrective archive; correct any errors before accepting R595.

## R595 Statistics collection visibility correction

- [x] Inspect actual R595 source and match Windows E0365/E0603 diagnostics to module visibility.
- [x] Restrict collection newtype visibility to `stats` and its descendants while preserving existing public interfaces and record ownership.
- [ ] **Blocking:** Rerun locked `zipeek-web` Rust tests and WASM compilation against the corrected full-source ZIP; fix any additional compiler errors before acceptance.

## R595 native speech client protocol acceptance

- [x] Validate successful-response media types, WAV framing and chunk arithmetic at the native client boundary.
- [x] Add malformed WAV/media-type/response framing regression tests without dependency changes.
- [ ] **Blocking:** Compile and test `zipeek-tts-stt-client` and `zipeek-tts-stt` with the locked toolchain; run Rustfmt and Clippy.
- [ ] **Blocking:** Test native TTS/STT against local model assets and separately verify browser permission/cancellation workflows.

## R594 native speech API and reliability acceptance

- [x] Inspect and correct the ten approved primary source targets; document findings and open risks.
- [x] Add dependency-free `zipeek-tts-stt-client` with native loopback transport and bounded HTTP protocol parsing.
- [ ] **Blocking:** `cargo fmt --all -- --check`; locked client/server/AI/web/exercises/docs tests; Clippy and WASM check against exact R594 ZIP.
- [ ] **Blocking:** Test native speech against eSpeak NG and multilingual Whisper assets; verify browser microphone, CORS, focus and cancellation behavior.
- [ ] Verify stale generated Practice evaluation, image-service error rendering and Cloud Match replay in Chrome.

## R593 ten-file correctness remediation acceptance

- [x] Inspect ten additional primary source targets and implement bounded, reviewed corrections.
- [x] Add source regression tests for queries, handoff IDs, Word Set names, image URLs and session timestamps.
- [x] Run available Node checks and data/documentation validations.
- [ ] **Blocking:** Run Rustfmt, locked Rust tests, Clippy and locked WASM check against exact R593 ZIP and correct all failures.
- [ ] **Blocking:** Browser-verify OPFS file limits, file-picker listener cleanup, storage race behavior and Practice launch without Sentence Transformation.
- [ ] Review and close remaining R591/R592 technical debt only after real validation evidence.

## R592 ten-file remediation acceptance

- [x] Inspect and change ten approved primary source targets with documented rationale and a shared typed Statistics bulk-selection hook.
- [x] Add focused Rust regression tests for applicable failure cases and preserve data/route/dependency contracts.
- [x] Execute available Node, Hanzi data and documentation checks.
- [ ] **Blocking:** Run locked Rust tests, WASM build, rustfmt, Clippy and browser regression against the exact R592 archive; correct any failures before acceptance.
- [ ] Verify PersonalStorage import/export with localStorage quota failure injection and CLI unpack collision/rollback on Windows.
- [ ] Finish R591's separate remaining semantic audit actions; R592 does not close them.

## R591 complete-source audit follow-up

- [x] Read every file in the R590 source archive and produce per-path digest, content classification and risk-signal evidence.
- [x] Parse all application JSON; verify bundled images and Hanzi Writer combined/individual shape consistency.
- [x] Correct maintained Settings AI documentation drift and regenerate the docs index.
- [x] Preserve frozen historical links as visible validator warnings rather than active-source blockers.
- [ ] **Blocking:** Compile, format, lint and test the exact R591 candidate, including Rust/WASM and Chrome acceptance.
- [ ] Execute the open A02-A14 risk-register investigations in `docs/engineering/codebase-content-audit-2026-11-19.md`; static content review does not close these findings.

## R590 Statistics state-ownership audit and follow-up

- [x] Separate Overview chart lifecycle, typed status and fallback rendering from the ActivityChart page component.
- [x] Remove false positive-height bars for zero-activity weeks; author a pure height calculation test.
- [x] Review Statistics state ownership, async request guards and shared reducer boundaries; record confirmed issues separately from speculative improvements.
- [ ] **Blocking:** Compile R590 with locked WASM target, execute the newly authored Rust unit test and run Rustfmt/Clippy.
- [ ] **Blocking:** Verify chart lifecycle, fallback, breakdown, theme switching and browser focus with Chrome in both base paths.
- [ ] Refactor duplicated Grammar/Vocabulary bulk-selection async request state into one typed hook after a verified compiler baseline; preserve generation, query-change and unmount guards.
- [ ] Audit exact reducer action invariants, canonical identifiers and background inertness before enabling Statistics learning transfers.

## R589 dialog focus acceptance

- [x] Extract shared initial-focus and Tab/Shift+Tab wrap logic for the three Statistics dialogs.
- [x] Preserve Escape dismissal and focus restoration; add pure boundary-rule unit tests.
- [ ] **Blocking:** Run new dialog unit tests and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` against the exact R589 source.
- [ ] **Blocking:** Browser-verify initial/return focus, Tab/Shift+Tab looping, Escape and background interaction on `/` and `/zipeek/`; test screen reader support and mobile.
- [ ] Decide whether the common dialog contract should also manage background inertness and nested modal stacking before broad reuse elsewhere.

## R588 repository technical debt audit and acceptance

- [x] Inventory workspace and audit selected release, browser and Statistics boundaries; retain findings in `docs/engineering/technical-debt-audit-2026-11-19.md`.
- [x] Bound native source unpack expansion and add archive-limit regression tests.
- [x] Correct Overview chart reload flags and unknown-category aggregation; author focused tests.
- [x] Add opt-in compiler-gated `cargo make pack-verified` workflow; fix documented Node test path.
- [x] Run the three available offline Node test suites (16 passes).
- [ ] **Blocking:** Successfully run R588 `cargo make release-check` and Rust/WASM unit tests in the Windows checkout.
- [ ] **Blocking:** Run Chrome UI/keyboard acceptance and Statistics performance scenarios.
- [ ] Review all open priority issues in the audit report: verified selection identities, dialog focus containment, large workflow owners and storage/security boundaries. These are not marked resolved.

## R587 deep Statistics architecture verification

- [x] Introduce typed `WordRecords`, `GrammarRecords`, `SessionRecords` with `Rc<[T]>` shared snapshot storage and convert SectionPayload, simulator and Overview props.
- [ ] Compile, run new snapshot tests and verify query page semantics in the locked Windows WASM build.


- [x] Correct the user-reported `E0277` Overview `ReviewListProps.words` type mismatch in a new WIP checkpoint.
- [ ] Confirm the corrected R587 ZIP compiles in the locked Windows WASM build; no local compiler verification available.


- [x] Convert Statistics root and all seven feature roots to directory modules and dedicated feature pages.
- [x] Extract Settings presets, core metrics and local catalogue filter state, Skills evidence/writing presentation, Insights aggregation/review, History toolbar, Grammar view controls and selection-review resolver/row.
- [x] Separate reducer-owned metrics and filter types without introducing extra reducer contexts or altering coupled transitions.
- [ ] **Blocking:** Perform locked WASM compilation, Rust formatting, web tests and browser regression acceptance against this exact handoff archive; fix compiler errors before acceptance.
- [ ] Complete state action/identity contracts and modal focus review as a separately verified workstream.
- [ ] Review remaining parent query/search/bulk-selection coordination after compiler gate; avoid changes that invalidate selection semantics.

## R587 remaining Statistics component acceptance

- [x] Correct all seven Statistics feature roots to directory-owned `mod.rs` files; remove the conflicting sibling `feature.rs` paths.
- [ ] **Blocking:** Validate corrected R587 directory-module archive with locked Windows WASM compilation and browser acceptance.

- [x] Extract Settings metric catalogue tile, Skills evidence card, Insights recommendation row and Overview chart, weekly aggregation, breakdown and review list into feature-local modules.
- [ ] **Blocking:** Rustfmt, locked native/WASM compile and `zipeek-web` tests against the exact R587 archive; fix reported failures.
- [ ] Browser E2E, focus and keyboard navigation for Settings ordering, Skills/Insights actions and Overview breakdown, including modal focus containment.
- [ ] Audit the Statistics reducer fields/actions as invariants before further restructuring. Do not mechanically split shared state.
- [ ] Refine parent explorer toolbar/query controller boundaries and confirm selection identity contracts separately.

## R586 Statistics explorer architecture and verification

- [x] Extract History day, timeline entry and table row into feature-local components with typed presentation props.
- [x] Extract Grammar detail inspector, card and table row; preserve parent selection/suggestion/query ownership.
- [x] Extract Vocabulary evidence row and consolidate duplicate paging controls.
- [ ] **Blocking:** Run `cargo fmt --all -- --check`, locked web native/WASM builds and test suite against the exact R586 archive; correct any failures before acceptance.
- [ ] Exercise History, Grammar and Vocabulary interactions in Chrome using both keyboard and pointer, and verify responsive breakpoints.
- [ ] Review the remaining parent toolbar/query controller boundaries, `state.rs` reducer scope, and Settings/Skills/Overview component boundaries as separate approved work.

## R585 Statistics usability and composition acceptance

- [ ] Confirm R585 selection-review E0382 ownership correction compiles in the Windows wasm32 build; the previous archive failed on 2026-11-19.
- [ ] Review component boundaries in Settings, Skills, and Overview, including reusable metric controls, evidence cards, chart lifecycle, dialogs, and reducer-versus-local state ownership; get scope approval before architectural changes.


- [x] Source implementation: consolidate History toolbar with icon controls and scoped accuracy bars, with UTC timestamps retained until timezone behavior is approved.
- [x] Source implementation: replace nested 65vh Vocabulary/Grammar scroll viewports with ordinary page scroll, compact Grammar count/view placement, and use Vocabulary as the navigation label.
- [x] Source implementation: make Settings metric tile text clickable, reduce reset copy, and extract a readable preview-only selection review with per-item removal.
- [ ] Verify R585 Rust formatting, tests, WASM build and browser acceptance on Windows in both themes and responsive sizes; pay special attention to session result semantics, pagination positioning, modal focus containment and escape behavior.
- [ ] Complete dialog focus trap, initial/restored focus and keyboard-only handling after actual browser testing; modal visuals and Escape handling alone do not prove accessibility.
- [ ] Agree canonical typed Word/Grammar identity mapping, eligibility and Practice/Word Set setup contracts before enabling selection transfer. Preview IDs must never cross the production boundary.
- [ ] Confirm revised Vocabulary navigation terminology against the complete Hanzi/word identity split and check tab naming expectations in E2E selectors.

## R582 verification and follow-up

- [ ] Compile the new History week navigation and Grammar combobox with pinned WASM toolchain, fix any reported errors, and run relevant Rust/E2E tests.
- [ ] Browser-check keyboard suggestion navigation, mouse selection, dismissal on blur, Arrow Up/Down/Enter/Escape/Tab, and screen reader announcements, including stale suggestion responses.
- [ ] Replace first-page suggestion query with a dedicated bounded/debounced suggestion service after user-approved API contract, rather than issuing a query on each keystroke.
- [ ] Verify History week changes keep activity aggregation independent of the session page, clear a selected date but preserve free-text search, and allow navigating back to the most recent week.
- [ ] Extend real History navigation beyond preview-anchored week dates with locale-aware week start, time zone, month/year navigation and durable session records.

## R578 Statistics density and technical-debt acceptance

- [ ] Compile R578 on Windows with native/WASM locked checks; run Statistics E2E and inspect Chrome light/dark, narrow/laptop and keyboard-only views.
- [ ] Verify full UTC timestamps and duration in History Timeline and Table, chronological sort, activity-category diversity and valid unscored exercise labels.
- [ ] Verify compact collapsed Overview weekly breakdown matches chart totals and the correct Practice/Games categories.
- [ ] Check reduced repetitive policy text in Skills/Insights without changing evidence semantics.
- [ ] Close P0 canonical identity and guarded Practice/Word Set draft handoff before any real data writes.
- [ ] Close P1 production timezone/browser localization, source caching/versioning, selectable date ranges and Chart.js theme lifecycle after explicit approved source contracts.
- [ ] Measure real Chrome interaction time with 5,000/10,000 records and keyboard focus/scroll usability, instead of treating source tests as performance evidence.

## R574 visual and semantic regression acceptance

- [ ] On Windows verify Statistics compilation and native tests after compact checkbox and Skills presentation changes.
- [ ] Verify the four required competency panels never imply an assessment score from observation coverage; Writing is shown separately.
- [ ] Verify the 36px checkbox hit target with an 18px visual indicator, visible keyboard focus and screen-reader checked state for header and rows.
- [ ] Verify Vocabulary and Grammar selected actions stay accessible above long tables, including cross-page selection, Clear and bulk query cancellation.
- [ ] Inspect desktop 1280/1440/1920, laptop 768-1024 and mobile 375/425px, plus light/dark, reduced motion and keyboard-only navigation.
- [ ] Measure 5,000 and 10,000-record sorting, filtering and page-change latency in browser. The existing unit fixture is not a performance result.
- [ ] Continue Stage B vocabulary/grammar details, History/Insights drilldowns, 20-30 tile registry and validated Practice/Word Sets handoff independently of this visual pass.

## R573 Windows compile and async explorer hardening acceptance

- [ ] Confirm History syntax/ownership repairs by compiling R573 with the pinned Windows WASM toolchain; resolve any new errors reported by the compiler.
- [ ] Run the `zipeek-web` unit tests covering exact remote page response contracts and bulk identity uniqueness.
- [ ] Verify Vocabulary, Grammar and History restore the last valid page after backend totals shrink, and show `0-0` on an empty or out-of-range page without a fabricated row.
- [ ] Exercise rejected source pages, Retry, fast searches, overlapping Select all matching requests, Clear while loading and navigation/unmount in Chrome.
- [ ] Keep source-query request identity tied to canonical backend identities when real APIs are approved; measure 5,000/10,000-record behavior before production adoption.
- [ ] Resolve port 1420 ownership locally if the Windows dev server reports an existing binding.

## R572 asynchronous explorer acceptance

- [ ] Confirm R572 native tests and WASM check on the pinned toolchain.
- [ ] Verify independent async loading, retry, empty results, filter changes and stale-response rejection for all three explorers.
- [ ] Confirm select-all queries fetch no more than 100 records per request, cap at 250 identities and do not apply after unmount or query changes.
- [ ] Verify History activity counts stay stable when changing pages and filter correctly for Practice/Games.
- [ ] Check responsive layout, accessible query/error states and root plus `/zipeek/` browser journeys.
- [ ] Retain approved API-backed identity, timezone and real Practice/Word Sets integration as future gated work.

## R571 typed query acceptance

- [ ] Run locked Rust tests, WASM compilation and Statistics browser journeys on Windows.
- [ ] Verify all three query methods return deterministic pages, totals and typed errors against the 10,000-record fixture suite.
- [x] Move Vocabulary, Grammar and History browser explorers from bounded fixture projections to query-controller driven pages without loading entire API datasets (R572 source implementation; browser acceptance pending).
- [ ] Implement stable, canonical identities and request-query fingerprint guards before any remote source or Practice/Word Set transfer.
- [ ] Verify empty, late, failed, retried and rapidly changed query results without stale data flashing on screen.
- [ ] Preserve offline-only fixture mode and distinguish query-contract tests from actual query performance measurements.

## R570 Statistics asynchronous section-source acceptance

- [ ] Run Rust formatting, native `zipeek-web` tests, WASM check and the Statistics browser suite on Windows.
- [ ] Verify all seven tabs load through the section source, with no direct fixture reads in presentation components.
- [ ] Exercise cancellation on navigation and preview close, retry after an injected error, empty responses and cache reuse across tabs.
- [ ] Design and approve canonical query/response identities and server-driven pagination/filtering for 5,000-10,000+ real records before wiring live evidence.
- [ ] Preserve fixture-only isolation and ensure no personal data read, DuckDB schema change or implicit Practice handoff occurs.

## R569 Statistics fixture and interaction acceptance

- [ ] Compile and test the expanded Practice registry-based History fixture and representative game session catalogue in Windows WASM.
- [ ] Confirm unscored Practice exercises never display verified correctness or mastery in History and the exercise registry contract test passes.
- [ ] Verify History Practice/Games filtering, search, timeline expansion, pagination and seven-day activity view in Chrome, both themes and both deployment paths.
- [ ] Confirm theme-integrated selection controls remain keyboard operable and are announced correctly by assistive technology.
- [ ] Complete the 5,000/10,000 record fixture matrix, robust shared select/listbox controls, full tile Settings library, real Practice/Word Sets handoff and actual Chrome chart lifecycle acceptance.

## R583 Statistics Grammar and overlay acceptance

- [ ] Compile the R583 Rust/WASM source, execute the Statistics browser suite, and fix any errors before marking implementation accepted.
- [ ] Verify Grammar table/card toggles on 550 source entries, preserved page and selected IDs, inspector details on desktop/mobile, and zero table displacement when opening detail.
- [ ] Verify selection review and Overview breakdown open without altering surrounding table/card geometry; test close, keyboard focus, Escape, focus restore, and reduced-motion accessibility.
- [ ] Verify Insights ranked ordering and filters with zero/small/large fixture families, and measure interaction latency at 5,000 and 10,000 identities.
- [ ] Implement canonical Practice/Word Set handoffs only after identity/provenance validation and approved setup contracts.

## Statistics 2.0 approved design and phased implementation (R565 documentation)

The [product and interaction contract](../product-ux/pages/learning-statistics-preview.md), [feature architecture](../architecture/modules/statistics-dashboard.md) and [acceptance matrix](../testing-quality/e2e/statistics-dashboard-design-acceptance.md) are approved for documentation and staged UI implementation. R565 changes no application behavior or production data.

- [ ] Stage A: reducer/context, request guards, skeletons, Overview and Chart.js JSON-compatible configuration updated through R567, pending WASM/browser verification. Complete read-only projection source, selection primitives, accessible chart acceptance and paging tests before considering Stage A accepted.
- [ ] Stage B: R567 implements searchable/paginated vocabulary and Grammar previews, temporary multi-selection review, clickable skill gaps, expandable session history and explainable insight links. R568 adds fixture-only card presets, variable page size and bounded filtered selection. Pending: confirmed Word Sets/Practice typed handoffs, canonical grammar mapping, full calendar, a searchable/reorderable 20+ tile library and complete Chrome acceptance.
- [ ] Stage C: 0/1/30/5,000/10,000-record fixture acceptance, responsive/light/dark/system/reduced motion, keyboard/screen-reader and root/hosted Chrome checks, measured performance.
- [ ] Before production data: approve durable ledger acceptance, per-session caps and per-skill validators, Writing assessment role, timezone, data privacy/retention, stable identity and grammar attribution. No mock-to-live promotion without these gates.
- [ ] Record actual Rust/WASM and browser results; documentation validation alone does not satisfy implementation acceptance.

## R564 Statistics design acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Verify Preview dashboard shows an actual two-second loading indicator, then the Demo indicator; leaving the route cancels the timer.
- [ ] Verify Overview fits common desktop heights with three cards and four table rows; full vocabulary and history tables remain paginated.
- [ ] Verify Writing is separate from Sentence use throughout sample evidence tables and Skills coverage.
- [ ] Verify custom switch controls are theme-aware, keyboard operable, accessible and do not persist changes.
- [ ] Verify neither preview activity nor fixture data reaches the personal database or network.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test stats_dashboard -- --nocapture` and visually inspect both themes and narrow viewports.

## R563 Statistics and Grammar preview acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test stats_dashboard -- --nocapture` and representative course mosaic browser tests.
- [ ] Confirm Statistics is directly after Browse, has a dashboard icon, and shows six icon tabs with keyboard access.
- [ ] Verify thirty unique sample words, paginated evidence/history tables, skill coverage and insights with no real data read or write.
- [ ] Verify card configuration is reversible, per-page metric descriptions are accessible, and sample disclaimers remain visible.
- [ ] Verify Grammar example paging stays within selected course, resets on course change, renders available curated Pinyin beneath Hanzi, and remains usable in dark/light and mobile layouts.

## R560 Province Quest region component acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --locked -p zipeek-web` and `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture`.
- [ ] Verify map region click, keyboard Enter/Space activation, focus, selected/correct/hint state and drag-click suppression across Easy, Medium, and Hard modes.
- [ ] Verify zoom/pan remains responsive with prefecture geometry on root and `/zipeek/` deployments, including mouse, trackpad and touch interactions.

## R559 Province Quest interaction acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --locked -p zipeek-web` and the Province Quest browser gameplay suite.
- [ ] Verify wheel zoom with a mouse and a trackpad, in letterboxed and wide viewports, including zoom-to-cursor behavior.
- [ ] Verify that click selects a region, drag pans without selecting, and many small drag movements work correctly.
- [ ] Verify all three difficulty modes, keyboard selection and both base paths, including Chrome touch input.

## R558 Province Quest map and controls acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --locked -p zipeek-web` and `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture`.
- [ ] Verify Hard mode initially shows a national map, province selection zooms into prefecture geometry, and empty provinces are not offered in the selector.
- [ ] Verify accessible Exit and Settings icons with keyboard, focus and both themes.
- [ ] Verify complete Easy/Medium/Hard journeys, mouse wheel zoom, drag-to-pan and mobile layout on `/` and `/zipeek/`.

## R551 recognition visual acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and the recognition E2E browser journey.
- [ ] Visually confirm the Upload/Recognize icons, pencil empty affordance, loading spinner, reduced-motion fallback and icon accessibility in both themes on mobile and desktop.

## R550 recognition layout acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture`.
- [ ] Check canvas/results side-by-side and bounded results scrolling at desktop sizes, including crowded candidates and short viewports.
- [ ] Check that mobile results scroll into view only after a completed recognition, with no nested mobile scrolling.
- [ ] Verify light/dark contrast, accessible icon toolbar, error handling and stale-result invalidation on both root and `/zipeek/` routes.

## R549 recognition UI acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture` with an installed dictionary.
- [ ] Visually review light/dark and mobile/desktop layouts; confirm icon-only Undo and Clear have keyboard focus, tooltips and accessible names.
- [ ] Confirm missing-model and recognition errors remain actionable, the request requires explicit submission, and image data is never logged.

## R546 recognition feature acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` to catch WASM API and component integration errors.
- [ ] Run `cargo test --locked -p zipeek-browser` and `cargo test --locked -p zipeek-web` where supported.
- [ ] Run the recognition E2E journey and accessibility matrix with working generated dictionary assets.
- [ ] Verify direct typed input/error notifications for upload, paste, Undo/Clear and repaint failures; no `data-recognition-*` attribute or `zipeek-recognition-*` event is used for state transfer.
- [ ] Review recording and AI request privacy, stale completion, light/dark rendering, and browser cleanup on `/` and `/zipeek/`.

# Open tasks and verification

## R544 recognition workspace architecture acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Browser-check image upload/paste, Undo/Clear, theme changes, and navigation cleanup in dark and light themes.
- [ ] Verify missing canvas context and unavailable theme report actionable diagnostics without logging image bytes.
- [ ] Verify the named `CanvasPoint` geometry in unit tests and that generic browser event handlers ignore events of an unexpected subtype.

Use the current authoritative product contracts and [acceptance checklist](../testing-quality/acceptance/approved-behavior-acceptance.md). Checkboxes denote unverified work, not historical completion.

## Rust/WASM chart interop verification

- [ ] Compile the R540 `zipeek-browser` Chart.js dynamic import via `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and verify module loading, destroy-on-drop, and root/`/zipeek/` vendor paths in real Chrome. Keep D35 Chart.js packaging unchanged.

## E2E evidence verification

- [ ] Run `cargo check --locked -p zipeek-e2e --tests` and targeted Rust unit tests on a supported toolchain.
- [ ] Run the Word mosaics Chrome browser journey with FFmpeg and FFprobe available, and inspect the actual MP4, timestamped checkpoints, `agent-feedback.md`, `scenario.json`, and suite `review.md`.
- [ ] Run strict evidence capture using `ZIPEEK_E2E_VIDEO_STRICT=1` and `ZIPEEK_E2E_EVIDENCE_STRICT=1`; audit its real MP4, checkpoints and manifests with the offline evidence auditor. Authored offline integrity tests do not substitute for browser verification.
- [ ] Run the larger E2E suite with deterministic fixtures; distinguish browser, assertion, service, and recording failures.
- [ ] Execute the updated Docs reader journeys for all twelve sections, image decoding, search and deep links at `/` and `/zipeek/`. Source checks and route validation are authored but do not constitute a browser pass.
- [ ] Revisit the [E2E requirements matrix](../testing-quality/e2e/e2e-requirements-matrix.md) after each implementation. Do not upgrade a row to verified without actual passing browser evidence.
- [ ] Replace obsolete URL replay and generic feedback assertions after the corresponding approved product behavior is implemented. Never make old assertions pass by inventing application behavior.

## Product features pending implementation or browser acceptance

- [ ] Implement and verify the common Practice exercise component foundation, consistent session navigation, and typed outcome evidence.
- [ ] Verify Practice completion distribution, persistent sessions, Skip and Replace, dragon evidence and profile JSON portability.
- [ ] Implement and test global exact-entry exclusions, vocabulary visitation, and cancellable nonblocking eligibility counting.
- [ ] Implement and verify context-aware Sentence Relay feedback, token dragging, punctuation and setup stages.
- [ ] Implement and verify selected/correct/incorrect Word mosaics tile states, progressive hints and accessible announcements.
- [ ] Add Multiple Choice pinyin and pronunciation controls, including anti-answer-leak rules and browser audio acceptance.
- [ ] Investigate Write Hanzi animation interruptions and stale frame glitches with recorded browser scenarios.
- [ ] Verify the R532 AI Hanzi Recognition route in real Chrome, including vision capability, paste/upload, cancellation, dictionary actions, Word Sets and strict privacy. Source implementation and authored tests are not browser acceptance.
- [ ] Verify original-tab ownership and duplicate-tab full-screen recovery, System theme, database startup blocking and Credits.
- [ ] Review the [open issue library](../issues/OPEN.md), preserving evidence and avoiding unverified closure.

## Engineering and deployment acceptance

- [ ] Verify R530 dictionary-only `db-install` against a real regenerated bundle, including unchanged practice/course/brand assets, old DB backup and restored `white-tiger.webp`.


- [x] Add an offline source/deploy asset integrity gate, covering manifest hashes, generated asset presence and a consistent preload filename, with fixture-based Node tests.
- [ ] Execute the strict deploy preflight against real generated dictionary, Worker and geography assets, then verify Rust/WASM compilation and Chrome deployment paths.

- [ ] Execute `cargo fmt --all -- --check`, `cargo test --workspace --locked`, WASM build and browser fixtures on the actual Windows environment.
- [ ] Verify dictionary and Province Quest asset generation on both local and hosted paths.
- [ ] Verify capability-aware unavailable AI exercises, grammar evidence, course locking and browser/local speech recognition.
- [ ] Review outstanding architecture proposals D28 (modular E2E organization), D29 (scoped state ownership and initial motion profile), and D30 (crate extraction boundaries) against current source. Do not change dependencies or extract crates without explicit approval.

The historical acceptance backlog remains preserved in the [legacy acceptance ledger](../archive-history/LEGACY_ACCEPTANCE_LEDGER.md) and [completed records](../archive-history/completed/). This active list intentionally contains no release-numbered task chronology.

## Documentation and source reconciliation still pending

- [x] Preserve and reconcile historical development-marker references from maintained technical articles. See [editorial disposition and preserved source evidence](editorial-reconciliation.md).
- [x] Audit the architecture, database generator, checked-in manifest and web runtime against maintained documentation. [Source accuracy audit](source-accuracy-audit.md) records corrected facts and open verification gates.
- [ ] Reconcile remaining technical assertions not sampled in the source accuracy audit against implementation and browser results.
- [ ] Verify Credits source-level licensing and generated asset provenance against shipped distributions before release.
- [ ] Compile the Docs reader after the documentation restructuring; check all section headings, deep links and issue images in real Chrome.
- [ ] Implement the approved shared exercise foundation, long detailed dragon and nonblocking vocabulary eligibility service; authored specifications are not implementation evidence.
- [ ] Link the remaining missing E2E scenarios to precise acceptance rows, then run strict video capture on the Windows/Chrome target.

## R529 generator verification follow-up

- [ ] Run the R529 database generator import tests and full regeneration against the pinned inputs; validate output count, logical snapshot, manifest hashes, FTS and read-only reopening.
- [ ] Benchmark the R528 row-wise generator against R529 parameterized 256-row insertion, including generation time, peak memory, file size and browser dictionary lookup latency.
- [ ] Investigate remaining per-row sense display, relation, qualifier and enrichment writes only after baseline measurements; preserve transactional rollback and evidence.

## R531 E2E follow-up

- [ ] Verify R531 preflight with a real installed compressed dictionary and both Root/Pages Trunk outputs.
- [ ] Re-run accessibility_matrix and debug any remaining Province Quest and Docs route failures from browser diagnostics without weakening route assertions.
- [ ] Complete Cargo/Trunk/Chrome gates and review checkpoint artifacts before release acceptance.

## R532 validation gates

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo test --locked -p zipeek-e2e --lib` and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture` and both root and Pages route/accessibility journeys.
- [ ] Check a real image-capable local AI endpoint, including invalid output, unsupported vision model, user cancellation and no uploaded image bytes in diagnostic artifacts.
- [ ] Verify clipboard and touch behavior in Chrome and Firefox, including replacement confirmation, 5 MB image limit, resource cleanup and image quality.
- [ ] Check R532 candidate actions with a real Word Set and actual dictionary pinyin/meaning against both simplified and traditional hits.
- [ ] Reproduce or verify closure evidence for ISS-001, ISS-021 and ISS-028 on Windows; do not close on source-only evidence.

## R533 native speech service follow-up

- [ ] Implement and test native Candle multilingual Mandarin STT on Windows; Whisper.rs is not approved.
- [ ] Exercise eSpeak Mandarin WAV synthesis and multilingual Whisper Mandarin recognition with licensed local models.
- [ ] Design and obtain approval for a trusted desktop bridge from Yew/browser UI to the loopback-only native service, including origin authentication, microphone permissions, cancellation, and provider selection under D56.
- [ ] Add end-to-end voice recording/assessment tests without interpreting transcript equality as pronunciation mastery.

## R534 native speech protocol hardening

- [ ] Run locked compile and native tests for the speech crate on Windows, including browser-Origin rejection and loopback Host validation.
- [ ] Verify eSpeak overflow/timeout handling and that child processes are reaped on failures using licensed local resources.
- [ ] Keep the trusted desktop bridge pending explicit D56 security approval; do not connect a hosted page directly to loopback HTTP.

## R535 speech output correctness and verification

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, and `cargo test --locked -p zipeek-tts-stt` on Windows.
- [ ] Generate speech via eSpeak NG and validate the served WAV with a decoder, sample-rate check, and real browser playback; confirm stdout placeholder lengths are repaired.
- [ ] Verify valid Mandarin speech with the pinned multilingual Whisper model and inspect empty recognition outcomes, request cancellation and saturated-service behavior.
- [ ] Keep the trusted desktop bridge pending the explicit D56 origin-authentication and microphone-consent design decision; no web-provider changes in this checkpoint.

## R536 local speech and Hanzi recognition acceptance

- [ ] Compile and test `zipeek-tts-stt` with `cargo test --locked -p zipeek-tts-stt`.
- [ ] Compile `zipeek-web` for `wasm32-unknown-unknown` and build with Trunk for `/` and `/zipeek/`.
- [ ] Verify exact-origin CORS and blocked-host behavior with the live local process; test hosted origin denial.
- [ ] Install licensed eSpeak/Whisper resources and confirm local Mandarin audio playback and five-second PCM16 microphone transcription in Chrome.
- [ ] Check microphone permission denial, interruption, cancellation, page navigation, and stale responses.
- [ ] Review Recognize Hanzi in browser with keyboard navigation, mobile layout and model responses (raw/fenced/invalid JSON).

## R537 recognition interface acceptance

- [ ] Verify drawing contrast in Light, Dark and System, including a theme change during an active drawing.
- [ ] Verify page height and vertical scrolling at mobile and desktop sizes, including many candidate results.
- [ ] Run recognition E2E with valid, invalid JSON and unavailable vision model; confirm console diagnostics do not contain image data or raw model text.
- [ ] Run locked WASM check, Node workspace tests and both root/Pages route journeys.

## R538 Rust/WASM interop migration acceptance

- [ ] Run `cargo fmt --all` and `cargo fmt --all -- --check` on the pinned toolchain.
- [ ] Run `cargo test --locked -p zipeek-browser` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Verify local speech Settings, shared Listen buttons and Speaking Recorder through native eSpeak/Whisper in Chrome, including consent denial, cancelled requests, interrupted playback and navigation cleanup.
- [ ] Confirm that local speech is restricted to localhost:1420, preserves browser default, and never persists or logs audio, speech text or recordings.
- [ ] Run `cargo make prepare-summary` and `cargo make verify-assets`; confirm the Chart.js-only thin module loads in root and `/zipeek/` deployments.
- [ ] Mount/verify the new Rust chart handle in real Yew summary/course components. Existing wrappers had no live Rust consumers; numeric text and D35 animation/reduced-motion behavior still require browser acceptance.

## R541 Rust/WASM browser service migration acceptance

- [ ] Run locked native `zipeek-browser` unit tests and locked `wasm32-unknown-unknown` check for `zipeek-web`.
- [ ] Confirm AI configuration can save URL/model, preserves other preferences, rejects credentials and invalid endpoints, and does not persist API keys; reload must clear the API key.
- [ ] Run Hanzi Recognition browser journeys with mouse, stylus, touch, paste, upload, undo, clear, light/dark/system appearance, stale image decode and navigation cleanup.
- [ ] Verify restricted image MIME, byte size and dimensions; confirm no image bytes in console logs or persisted settings.
- [ ] Run root `/` and hosted `/zipeek/` navigation, production asset preparation and pinned Chart.js bundle integrity checks.

- [ ] R545 verification: run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and recognition browser E2E to validate the reducer-backed feature split.

- [ ] Verify R552 recognition request-token compatibility against configured vision providers and visually test compact desktop/mobile suggestions with keyboard and screen reader.

- [ ] Run `cargo make install` on Windows with DuckDB v2 native libraries and 7z/7zz available; confirm dictionary, Worker and Province Quest assets all pass `preflight-deploy`.
- [ ] Validate Province Quest root and `/zipeek/` asset URLs, including both JSON files and playable map geometry, after unified install.
- [ ] Review the redistribution rights of upstream AreaCity boundary data before shipping generated Province Quest assets publicly.

## R557 follow-up verification

- [ ] Verify typed Province Quest lifecycle and gesture changes with WASM compilation and Chrome E2E.
- [ ] Cover wheel zoom, pan, pointer cancellation and click suppression in real browser sessions.
- [ ] Profile Hard mode SVG rendering before introducing optimization caches.

### R575 Statistics customization checkpoint

- [x] Author a searchable, categorized 20-item optional metric registry with theme-integrated accessible switches, reducer-owned ordering, reset and presets, on top of four existing core metrics.
- [x] Keep all tile values derived from an asynchronously loaded Overview projection, not from UI fixture globals or assumed assessed evidence.
- [ ] Execute Rust/WASM compilation, Chrome light/dark/mobile keyboard acceptance and the 5,000/10,000-record measured performance gate.
- [ ] Finish core workflow integrations and Stage B explorer detail acceptance, preserving evidence and canonical identity rules.

### R576 Statistics deep exploration checkpoint

- [x] Implement History Timeline/Table mode switching in route-scoped reducer and keyboard-accessible date filtering against the existing async History query contract.
- [x] Keep seven-day activity context independent of selected-date session pages; distinguish scored and unscored outcomes in table rows.
- [x] Distinguish sample skill coverage, review markers, missing evidence and unavailable assessed performance; retain Writing as an additional category.
- [x] Expand explainable Insights into recognition, listening, speaking, sentence-use and review scopes with direct evidence-filter navigation.
- [ ] Run Windows Rust/WASM build and Chrome theme/keyboard/mobile acceptance for both History modes and date filtering.
- [ ] Complete Grammar course/provenance drilldowns, verified Practice/Word Set handoff, and measured browser performance on 5,000 and 10,000 items.

### R577 Statistics Grammar provenance checkpoint

- [x] Expose grammar pattern examples and provenance in keyboard-focusable expandable rows using async loaded page data.
- [x] Keep illustrative grammar/course identifiers separate from canonical production identifiers and qualify mock practice-attempt counts.
- [x] Avoid presenting the independent Practice navigation as a selected-item handoff; maintain no-write demo behavior.
- [ ] Confirm Windows WASM compilation and browser keyboard/aria-expanded behavior in both themes and on narrow viewports.
- [ ] Obtain approved canonical grammar/course ID mapping and assessed-attempt contract before creating a live Practice setup draft with Sentence Transformation selected.
- [ ] Close ST2-017, ST2-018 and measured large-dataset acceptance only after end-to-end validation.

## R579 Statistics interaction validation and remaining debt

- [ ] Verify selection count remains at constant page geometry on the first word or grammar selection, on clearing, when switching pages and after retry; keyboard/screen reader should announce state without focus loss.
- [ ] Verify compact History date navigator shows Practice/Games breakdown, correct filtering and accessible full-date labels in both themes, narrow widths and the hosted base path.
- [ ] Execute 5,000- and 10,000-word simulator/query tests and measure browser behavior; separately profile fixture rebuild overhead and choose a shared source snapshot if warranted.
- [ ] Complete canonical IDs, safe Word Set/Practice handoffs, reusable local-time formatting, date-range navigation and async error acceptance. These remain open and are not implied implemented by R579.

### Statistics R580 follow-through

- [ ] Validate the 550-source-row Grammar explorer with native/WASM and Chrome, including search suggestions, editorial details and bounded selection.
- [ ] Measure column-header sort keyboard and screen-reader behavior, and all control-row responsive breakpoints.
- [ ] Implement source-wide async suggestions without bypassing the data-source interface.
- [ ] Finish guarded Practice and Word Set drafts only after canonical ID contracts are approved.
- [ ] Complete History time-zone and calendar navigation, and the 5,000/10,000-record browser quality gate.


## R581 outstanding Statistics acceptance

- [ ] Compile the Grammar query/filter/sort changes with the locked native and WASM toolchain; run the focused Rust tests and Statistics Chrome E2E.
- [ ] Verify teaching-region filtering and sortable headers in dark/light themes and at narrow widths; test keyboard and screen-reader handling of suggestion list.
- [ ] Implement approved, validated canonical Grammar-to-Practice draft handoff only after identities and task eligibility are established.
- [ ] Add query debouncing and accessible combobox semantics to the independent source-wide Grammar suggestion flow, then verify network-request cancellation and stale-response behavior.

### R584 Statistics UX, simulator and remaining debt

- [x] Replace the hard-coded 30-word Vocabulary preview with an HSK-derived seeded catalogue and 240-item standard scenario; retain canonical grammar teaching source separately.
- [x] Generate History focus terms from the same vocabulary identities and derive per-word encounter/session/day counts from session records.
- [x] Add themed reusable ChoiceSelect for the Grammar collection and Vocabulary/Grammar page-size controls with keyboard and click-away handling.
- [x] Replace vague Grammar region-range labels with teaching-collection partition labels without inventing course or ability classifications.
- [x] Constrain long desktop table/card areas and keep page controls reachable; compact Settings with group filters and optional Overview metrics.
- [ ] P0: R584 Windows WASM compilation reported E0382 in `stats/controls.rs`; the approved trigger/option component extraction removes the inline state-capturing callback, but has not been compiled. Rerun native/WASM checks, then browser flows and light/dark/mobile custom-select keyboard/ARIA/focus tests.
- [ ] P0: Complete verified canonical Word Set/Practice handoff without transporting demo IDs.
- [ ] P1: Promote ChoiceSelect to the shared UI crate after validating use cases and focus/escape/mobile semantics; do not copy local instances across routes.
- [ ] P1: Test desktop internal table scrolling versus natural mobile scroll; measure 5,000/10,000 records, custom-select popup collision, History timezone, and chart updates.
- [ ] P1: Complete empty/small/large scenario injection and pending production source identity/query contracts before any personal-data integration.

## R615 generator and speech follow-ups

- [ ] Compile and benchmark the new bulk sense/enrichment import on identical input files; compare stage timing, memory and output fingerprint.
- [ ] Generate DuckDB with bundled `relationships.csv`; inspect all loaded rows and source/target `resolved`, `missing`, `ambiguous` counts and read-only artifact verification.
- [ ] Validate candidate Pinyin/character gloss completeness and explicitly curate immutable R608 entities separately.
- [x] Add experimental Candle STT audio frontend with bounded PCM processing and numerical/unit tests (R618 source; native compilation and reference comparison pending).
- [ ] Implement Candle STT tokenizer, model load, decoder and Mandarin fixture comparison; remove temporary unavailable status only after verified inference.
- [x] Add initial Mandarin TTS punctuation-aware segmentation and Kokoro asset contract (R618 source; native compilation pending).
- [ ] Implement Candle TTS contextual phonemization and synthesis and remove eSpeak only after quality and protocol acceptance.


## R621 speech inference follow-up

- [x] Replace R620 fixed 3,000-frame Qwen3-ASR Mel inputs with 100-frame-multiple variable length and per-frame attention mask (source authored; compile-unverified).
- [x] Add deterministic 100-Mel-frame / 13-token convolution chunk and 104-token attention window layout (source authored; compile-unverified).
- [ ] Verify the R621 Rust unit tests with the locked native toolchain and compare Mel coefficients against the official Qwen3-ASR feature extractor.
- [x] Author Qwen3-ASR-0.6B Candle Conv2D stem, per-valid-chunk execution and the 7680-to-896 projection using published checkpoint tensor namespaces (R622 WIP; compile and weights unverified).
- [ ] Compile/test the new Candle Conv2D stem against actual safetensors, compare projected tensor values against the upstream implementation and check GPU behavior.
- [x] Apply non-interleaved per-100-Mel-frame CNN positional embeddings after projection; preserve padded convolution context and trim invalid tail tokens (R624 source authored; compile/parity unverified).
- [ ] Add 18 audio Transformer layers, final LayerNorm and output projector; validate checkpoint tensor shapes and attention-window behavior.
- [ ] Implement the Qwen tokenizer and LLM decoder with context-length and KV-cache limits; confirm Mandarin audio-to-text inference before enabling the STT API.
- [ ] Implement and verify Chinese Kokoro checkpoint conversion, phonemization and neural synthesis; keep native eSpeak active until the new engine is validated.

## R639 Qwen3-TTS progress and CUDA layout

- [x] Fix reported codec Transformer CUDA non-contiguous attention matmul and add a shape regression.
- [x] Expose progress callback and trace events in ignored neural WAV acceptance.
- [ ] Verify CUDA checkpoint synthesis on Windows; check audible quality and end-to-end STT round trip.

## R640 neural speech file exports

- [x] Update real Qwen3-TTS test to save `mandarin_nihao_shijie.wav` and validated `mandarin_nihao_shijie.opus` using existing approved codec.
- [ ] Run model-backed Windows test and listen to audio; MP3 encoding requires separate dependency approval.

- [ ] R676 WIP: Run Windows wasm32 check, browser crate tests and Chrome microphone/no-speech and waveform-hover acceptance before signoff.

- [ ] R704: Implement durable, atomic KnowledgeLedger evidence processor with validated target authorization and replay-safe reconciliation; Character Builder construction observations are intentionally not KP-scored.
- [ ] R704: Add exact visible-presentation timestamp instrumentation and authoritative item-level evidence adapters beyond Character Builder.
