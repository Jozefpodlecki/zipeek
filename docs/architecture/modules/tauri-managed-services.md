# Tauri-managed local services

The desktop shell registers an `Arc<NativeServices>` and a native database manager through `.manage()` in `zipeek-app/src/setup.rs`. `zipeek-app/src/commands/mod.rs` registers the IPC handlers through `generate_handlers()`, with thin typed handlers in `commands/services.rs` and `commands/database.rs`. Tauri commands `start_service`, `stop_service`, and `service_status` accept only `ai` or `image`. The registry holds owned `Child` handles, refuses to start a new process on an occupied loopback port, and never stops a process it did not spawn. A native thread checks each service's `GET /health` response every five seconds using short loopback HTTP requests. Readiness requires HTTP 200 and a JSON `status` value of `ok`; it does not invoke inference. Startup waits for readiness for up to 20 seconds, detects early child exit, and terminates owned processes that never become ready. Duplicate launches and concurrent start/stop requests are rejected safely.

Tauri service commands return serializable `ServiceError` variants and native database commands return serializable `DatabaseError` variants. The Yew frontend invokes Tauri from Rust through `js_sys::Reflect`, `Function`, and `JsFuture` in `zipeek-web/src/services/tauri_service.rs`, with structured rejection decoding shared by the service and database IPC bridges. `ServiceProvider` in `zipeek-web/src/contexts/service_management.rs` owns status, pending operations, errors, and five-second polling for both services. `use_ai_service()` and `use_image_service()` expose capabilities and actions to Settings. The Settings component builds service-control fragments before its main markup, and does not check `is_tauri` to decide what to render. Browser mode does not expose process controls. Externally owned processes cannot be stopped. The native status contract reports `stopped`, `starting`, `running`, `stopping`, `failed`, or `external`, with an actionable message; browser Settings does not expose process controls. Tauri command execution uses blocking workers so readiness waits do not block the command executor. When Tauri emits its exit event, the registry terminates and reaps remaining owned children. On Windows, managed processes are launched without opening an additional console window.

## Building and staging sidecars

From the workspace root on Windows:

```powershell
cargo make desktop-dev
cargo make desktop-build
```

`desktop-dev` builds the two servers in debug mode, stages their target-triple-suffixed executables, prepares Docs and checks web vendor assets, then calls `cargo tauri dev` inside `crates/zipeek-app`. `desktop-build` does the same with release binaries and calls `cargo tauri build`. The Tauri build hook invokes Trunk; it does not depend on nonexistent npm scripts.

To prepare binaries without launching Tauri:

```powershell
cargo make desktop-sidecars
cargo make desktop-sidecars-release
```

The Rust CLI command `zipeek-cli desktop-sidecars [--release]` stages existing compiled binaries into `crates/zipeek-app/binaries/`, with names such as `zipeek-image-server-x86_64-pc-windows-msvc.exe`. It resolves the host triple from `rustc -vV` and rejects missing/empty server executables. This workflow targets **the host triple**, not cross-compilation; the desktop build currently targets NSIS on Windows.

Tauri's `bundle.externalBin` declares both binary basenames, and bundles them with the installer. Tauri development and installed builds place the runnable sidecars near the app executable under their unsuffixed names. The process registry resolves that sibling path by default; absolute overrides `ZIPEEK_TAURI_AI_SERVER_EXE` and `ZIPEEK_TAURI_IMAGE_SERVER_EXE` remain available for debugging. Build products under `crates/zipeek-app/binaries/` are ignored by Git and excluded from Zipeek source packages.

`cargo tauri` requires the Tauri CLI to be installed (`cargo install tauri-cli --locked`). Web development still requires Trunk and the usual prepared vendor assets. The existing Zipeek favicon is used for the bundled desktop icon files.

## Supported service modes

- **Image:** launches `zipeek-image-server` at `127.0.0.1:1431`, respecting the image backend's model and environment configuration. SDXL model weights and any CUDA runtime dependencies are not embedded in the installer.
- **AI:** launches `zipeek-ai-server --fixture --bind 127.0.0.1:1430`. This executable presently supports fixture responses only; it is not a production LLM service.

The native service controller does not manage services already listening on the configured ports. Inference/model assets are deliberately outside the source package and the Tauri sidecar binary bundle.
