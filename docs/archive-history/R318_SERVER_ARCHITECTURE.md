# R318 server architecture checkpoint

Current implementation was verified against the R317 source. Documentation and pending tests do not establish that a browser journey passes.

## Exercise coverage status

- Speaking: target switching, accessible selection, and permission-denial recovery have E2E scenarios. Successful microphone capture, discard, retry, playback, and cleanup are not verified.
- Listening choice: sufficient-vocabulary gating and a configured session-and-restore journey exist; audible playback was not verified here.
- Audio dictation: the browser suite contains playback gating, answering, and restoration; browser execution remains required.
- Image Vocabulary: mode selection has browser coverage; a full image generation, approval, selection, scoring, and completion scenario is still absent.
- Translation and sentence transformation: deterministic fixture and exercise interactions exist; the harness does not yet run the native AI-server process for those scenarios.

## New owned servers

- `zipeek-static-server::StaticServer::run(ServerConfig)` binds and serves a built Trunk dist tree, returns its actual address, and supports graceful stop.
- `zipeek-image-server::ImageServer::run(ImageServerOptions)` binds a loopback endpoint, starts the placeholder or SDXL backend, returns its actual address, and supports graceful stop. Placeholder mode needs no external model.
- `zipeek-e2e` declares dependencies on all three server crates. Static, Image and AI have owned server lifecycle tests, but compilation and runtime acceptance remain pending.

Both APIs return owned handles that initiate graceful shutdown on drop. Explicit `.stop().await` waits for task termination. These tests must be compiled and run on the target Rust toolchain before acceptance.

## Implemented in the continuing R318 checkpoint

- The image server now accepts an explicit list of strictly validated loopback HTTP origins in `ImageServerOptions.allowed_origins`. The legacy local development origins remain accepted. E2E passes its actual dynamically allocated browser origin instead of disabling CORS.
- `Harness::start` launches and owns a placeholder `ImageServer`, passes its URL to `ScenarioContext`, stores image files under suite artifacts, and awaits shutdown in `Harness::finish`.
- A non-ignored browser scenario selects Picture Vocabulary from Advanced Practice, checks the selection state, configures the placeholder image endpoint, and performs real browser-origin health, job submission, job polling, and generated-image retrieval requests. It does **not** complete the Image Vocabulary Yew answer workflow yet.
- `page_objects/audio_probes.rs` verifies that silent-mode speech recorded a Mandarin `speak` event, not merely that a listen button was clicked. Listening-session coverage now invokes the play control and verifies the service event.
- The existing microphone-permission denial fixture counts `getUserMedia` attempts. The Speaking browser test verifies both attempts reached the microphone API. It does not prove successful capture, MediaRecorder stop/discard, or audio fidelity.
- Added service-level Rust regression checks for allowed image origins and invalid origins, plus a deterministic image-server preflight test. Browser tests remain unexecuted until Windows Chrome acceptance.

## Next integration steps

1. Implement a full Image Vocabulary browser scenario with dynamically valid AI scene composition for the selected HSK words, image review and approval, answer submission, scoring, and regeneration. The standalone placeholder image backend and browser CORS boundary are now testable.
2. Add a deterministic `getUserMedia`/`MediaRecorder` fixture that produces a non-empty recording and exposes stream-track cleanup, then verify stop, retry, discard, and recorder unmount. The current denial fixture only proves acquisition attempts and failure handling.
3. Verify actual audible output and microphone capture on a supported device. Silent speech event capture proves service invocation, not sound reproduction.
4. The native `AiServer::run(address)` now returns `RunningAiServer` with graceful stop. Its HTTP health/models/CORS fixture test is written; migrating browser translation scenarios still requires the custom fixture's request-record/reset controls. Run model-backed acceptance only when explicitly configured.
5. Root suites now use owned `StaticServer::run` after `trunk build`. The Pages `/zipeek/` fixture adapter remains unchanged; verify both hosting paths in Chrome.
6. Execute Cargo check, tests and managed Chrome suites; investigate any new errors without interpreting Node source contracts as compiler or browser acceptance.

## Known limitations

There is no local Rust toolchain in the authoring environment. No Rust or Chrome pass is claimed. The image-server dependency includes the existing CUDA-enabled Candle stack and may increase the build burden of the E2E crate. No third-party package versions were modified.

## Continuing R318 static and AI-server lifecycle work

- Root `Harness::start` retains `trunk build` for compiling assets and uses an
  owned `StaticServer` instead of `trunk serve`. Startup allocates a dynamic
  loopback port, uses that origin for the Image Server CORS configuration, and
  reports static-server shutdown errors during `Harness::finish`.
- `server::BuiltWeb` makes the built distribution and adapter URL explicit;
  Pages still use the Node fixture adapter and `/zipeek/` public URL.
- `AiServer::run(address)` returns `RunningAiServer`, an owned fixture-capable
  server with ephemeral-port binding and graceful stop. New HTTP integration
  assertions check health, model listing, approved/disallowed browser origins,
  and port release after stop. Existing translation fixture remains active.
- Added static hosting tests for index, asset, 404, missing build root and
  cleanup. None have been executed under Cargo in this environment.
