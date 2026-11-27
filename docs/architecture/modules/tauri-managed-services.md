# Tauri-managed local services

The desktop shell registers `NativeServices`, a tuple wrapper around a shared `Arc` registry and a native database manager through `.manage()` in `zipeek-app/src/setup.rs`. `zipeek-app/src/commands/mod.rs` registers the IPC handlers through `generate_handlers()`, with thin typed handlers in `commands/services.rs` and `commands/database.rs`. Tauri commands `start_service`, `stop_service`, and `service_status` accept `ai`, `image`, or `speech`. The registry holds owned `Child` handles, refuses to start a new process on an occupied loopback port, and never stops a process it did not spawn. A native thread checks each service's `GET /health` response every five seconds using short loopback HTTP requests. Process health requires HTTP 200 and either a JSON `status` of `ok` for AI/image or a plain `ok` body for speech; it does not invoke inference. Startup waits for readiness for up to 20 seconds (120 seconds for the speech engine cold start), detects early child exit, and terminates owned processes that never become ready. Duplicate launches and concurrent start/stop requests are rejected safely.

Tauri service commands return serializable `ServiceError` variants and native database commands return serializable `DatabaseError` variants. The Yew frontend invokes Tauri from Rust through `js_sys::Reflect`, `Function`, and `JsFuture` in `zipeek-web/src/services/tauri_service.rs`, with structured rejection decoding shared by the service and database IPC bridges. `ServiceProvider` in `zipeek-web/src/contexts/service_management.rs` owns status, pending operations, errors, and five-second polling for all three services. `use_ai_service()`, `use_image_service()` and `use_speech_service()` expose capabilities and actions to Settings. The Settings component builds service-control fragments before its main markup, and does not check `is_tauri` to decide what to render. Browser mode does not expose process controls. Externally owned processes cannot be stopped. The native status contract reports `stopped`, `starting`, `running`, `stopping`, `failed`, or `external`, with an actionable message; browser Settings does not expose process controls. Tauri commands forward to async `NativeServices` methods, which own the `spawn_blocking` boundary. No extra command-error enum is required: the existing typed `ServiceError` is already serializable. Tauri command execution uses blocking workers so readiness waits do not block the command executor. Close and exit requests are coordinated before window destruction as described below. On Windows, managed processes are launched without opening an additional console window.

## Building and staging sidecars

From the workspace root on Windows:

```powershell
cargo make desktop-dev
cargo make desktop-build
```

`desktop-dev` builds the three servers in debug mode, stages their target-triple-suffixed executables, prepares Docs and checks shared desktop vendor assets, then calls `cargo tauri dev` inside `crates/zipeek-app`. `desktop-build` does the same with release binaries and calls `cargo tauri build`. The Tauri build hook invokes `zipeek-cli desktop-frontend --serve` or `--release`, which prepares separate desktop assets and invokes Trunk. It does not depend on npm scripts or browser DuckDB vendor preparation. The desktop output is `crates/zipeek-web/dist-desktop`; development remains at localhost:1421.

To prepare binaries without launching Tauri:

```powershell
cargo make desktop-sidecars
cargo make desktop-sidecars-release
```

The Rust CLI command `zipeek-cli desktop-sidecars [--release]` stages existing compiled binaries into `crates/zipeek-app/binaries/`, with names such as `zipeek-image-server-x86_64-pc-windows-msvc.exe`. It resolves the host triple from `rustc -vV` and rejects missing/empty server executables. This workflow targets **the host triple**, not cross-compilation; the desktop build currently targets NSIS on Windows.

Tauri's `bundle.externalBin` declares all three binary basenames, and bundles them with the installer. Tauri development and installed builds place the runnable sidecars near the app executable under their unsuffixed names. The process registry resolves that sibling path by default; absolute overrides `ZIPEEK_TAURI_AI_SERVER_EXE`, `ZIPEEK_TAURI_IMAGE_SERVER_EXE` and `ZIPEEK_TAURI_SPEECH_SERVER_EXE` remain available for debugging. Build products under `crates/zipeek-app/binaries/` are ignored by Git and excluded from Zipeek source packages.

On Windows, a desktop rebuild can fail in `tauri_build::build()` with `os error 32` when another process has a bundled sidecar executable or DLL open. Stop running Zipeek desktop instances and owned AI/image/speech sidecar processes, then retry `cargo make desktop-dev`. Do not delete the generated dictionary, rebuild it, or terminate unrelated processes to resolve a file lock. The sidecar staging command compares existing files byte-for-byte and skips unchanged binaries and dictionary bytes, reducing unnecessary overwrites; it cannot bypass a lock held by the Tauri build itself or another process.

`cargo tauri` requires the Tauri CLI to be installed (`cargo install tauri-cli --locked`). Web development still requires Trunk and the usual prepared vendor assets. The existing Zipeek favicon is used for the bundled desktop icon files.

## Supported service modes

- **Image:** launches `zipeek-image-server` at `127.0.0.1:1431`, respecting the image backend's model and environment configuration. SDXL model weights and any CUDA runtime dependencies are not embedded in the installer.
- **Speech:** launches `zipeek-tts-stt` at `127.0.0.1:8799`. TTS, STT and Live STT are workers/endpoints in the same process; start/stop controls operate on the process as one unit. `/health` reports process liveness only, while `/v1/capabilities` exposes engine status and failure categories. CPU/CUDA and model asset configuration are inherited from the desktop environment.
- **AI:** launches `zipeek-ai-server --fixture --bind 127.0.0.1:1430`. This executable presently supports fixture responses only; it is not a production LLM service.

The native service controller does not manage services already listening on the configured ports. Inference/model assets are deliberately outside the source package and the Tauri sidecar binary bundle.

## R699 image client boundary

The Tauri image Settings view uses `image_health` and `image_generate` native IPC commands. The native implementation connects only to `127.0.0.1:1431`, using bounded HTTP requests, validates returned job identifiers and paths, and polls image jobs off the UI thread. It does not accept URLs from WebView arguments. The Yew image-generation client automatically uses IPC when the desktop runtime is present. The hosted browser version retains the configured image endpoint and direct HTTP workflow. The image element displays the resulting loopback image URL without a cross-origin `fetch`; native health/generation requests no longer depend on WebView CORS.

The Settings image page displays backend health and supports a user-triggered test-generation preview; Tauri process ownership and Start/Stop remain exclusively in Services. Generated test images are stored by the existing image server under its configured storage, not in personal DuckDB. Process health is distinct from an actual successful test generation.


## Desktop shutdown lifecycle

The approved 2026-11-27 shutdown flow is owned by `zipeek-app/src/shutdown.rs`. A main-window close request (including title-bar X and Alt+F4) or application exit request is prevented until owned service cleanup finishes. One native shutdown state rejects duplicate cleanup requests; a shared closing flag prevents service starts, including launches that were still preparing when shutdown began. Spawn and registry insertion remain under one registry lock so cleanup cannot miss a newly spawned owned child.

The frontend `DesktopLifecycle` sits under the theme provider and outside the service/database providers. It listens through the centralized Tauri bridge, then reads a shutdown snapshot to cover late subscription. State moves only from idle to closing to failure, so a stale idle snapshot cannot hide the closing screen. Closing replaces the interactive application subtree with the existing themed loading panel, spinner, Closing Zipeek label and Stopping local services detail. Focus moves to the status panel; reduced motion uses the existing spinner policy. The main-window capability permits only core event listen/unlisten for this integration.

Cleanup runs off the UI thread. It attempts termination for every registered owned child before waiting for child exits; it does not request a graceful HTTP shutdown. Successful cleanup permits final application exit. Cleanup errors, worker disconnection or a ten-second timeout show a failure screen with a keyboard-operable Close now button. A further native close request after failure also exits, including when the frontend could not initialize. Forced exit does not claim that all services stopped. Externally started services and the development Trunk process are outside this registry and are not stopped by this flow.

The native tests and mocked-IPC Chrome checks establish source and presentation behavior. Actual Tauri/WebView close interaction remains pending in TASKS.


## Approved desktop development model path

On 2026-11-27 the user approved a machine-specific `ZIPEEK_AI_MODEL_PATH` entry under `tasks.desktop-dev.env` in Makefile.toml. Every `cargo make desktop-dev` launch uses the specified existing Qwen3.5-9B Q4_K_M GGUF in the user's LM Studio model directory, and the native child service inherits it. The environment value is scoped to desktop development; it is not a bundled model resource or a change to installed application discovery. Changing the model file location requires updating this explicit task setting. The configured file exists, but model loading and inference remain unverified.
